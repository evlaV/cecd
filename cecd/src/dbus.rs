/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::{anyhow, Result};
use linux_cec::device::AsyncDevice;
use linux_cec::message::{Message, Opcode};
use linux_cec::operand::UiCommand;
use linux_cec::{LogicalAddress, PhysicalAddress, Timeout};
use num_enum::TryFromPrimitive;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::canonicalize;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio::task::{spawn, JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use zbus::object_server::SignalEmitter;
use zbus::{fdo, interface, Connection};

use crate::device::DeviceTask;
use crate::system::{SystemHandle, SystemMessage};

fn into_fdo_error<T: Display>(val: T) -> fdo::Error {
    fdo::Error::Failed(format!("{val}"))
}

const PATH: &str = "/com/steampowered/CecDaemon1";

#[derive(Debug)]
pub struct CecDevice {
    pub device: Arc<Mutex<AsyncDevice>>,
    pub token: CancellationToken,
    task: Option<JoinHandle<Result<()>>>,
    channel: Option<UnboundedSender<SystemMessage>>,
    path: PathBuf,
    pub cached_phys_addr: u16,
    pub cached_log_addrs: Vec<u8>,
    pub cached_vendor_id: i32,
}

impl CecDevice {
    pub async fn open(path: impl AsRef<Path>, token: CancellationToken) -> Result<CecDevice> {
        let path = canonicalize(path).await?;
        let device = Arc::new(Mutex::new(AsyncDevice::open(&path).await?));
        Ok(CecDevice {
            device,
            token,
            path,
            task: None,
            channel: None,
            cached_phys_addr: 0xFFFF,
            cached_log_addrs: Vec::new(),
            cached_vendor_id: -1,
        })
    }

    pub async fn register(
        self,
        connection: Connection,
        system: SystemHandle,
        tx: UnboundedSender<SystemMessage>,
        rx: UnboundedReceiver<SystemMessage>,
    ) -> Result<()> {
        debug!("Registering CEC device {} on bus", self.path.display());

        let object_server = connection.object_server();
        let path = self.dbus_path()?;
        object_server.at(path.clone(), self).await?;

        let interface = object_server.interface(path.clone()).await?;
        let task = DeviceTask::new(interface.clone(), system, rx, connection).await?;
        let mut interface = interface.get_mut().await;
        interface.task = Some(spawn(task.run()));
        interface.channel = Some(tx);
        info!("Device {path} registered");
        Ok(())
    }

    pub fn dbus_path(&self) -> Result<String> {
        let path = self.path.to_str().ok_or(anyhow!("Invalid path supplied"))?;
        let path = path.strip_prefix("/dev").unwrap_or(path);
        let path = path
            .split('/')
            .filter_map(|node| {
                // Capitalize the first letter of all path elements, if present
                let mut chars = node.chars();
                chars
                    .next()
                    .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
            })
            .collect::<String>();
        Ok(format!("{PATH}/{path}"))
    }
}

#[interface(name = "com.steampowered.CecDaemon1.CecDevice1")]
impl CecDevice {
    #[zbus(signal)]
    async fn received_message(
        signal_emitter: &SignalEmitter<'_>,
        initiator: u8,
        destination: u8,
        timestamp: u64,
        message: &[u8],
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn user_control_pressed(
        signal_emitter: &SignalEmitter<'_>,
        button: &[u8],
        initiator: u8,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn user_control_released(
        signal_emitter: &SignalEmitter<'_>,
        initiator: u8,
    ) -> zbus::Result<()>;

    #[zbus(property)]
    async fn logical_addresses(&self) -> Vec<u8> {
        self.cached_log_addrs.clone()
    }

    #[zbus(property)]
    async fn physical_address(&self) -> u16 {
        self.cached_phys_addr
    }

    #[zbus(property)]
    async fn vendor_id(&self) -> i32 {
        self.cached_vendor_id
    }

    async fn set_osd_name(&self, name: &str) -> fdo::Result<()> {
        self.device
            .lock()
            .await
            .set_osd_name(name)
            .await
            .map_err(into_fdo_error)
    }

    async fn set_active_source(&self, phys_addr: i32) -> fdo::Result<()> {
        let phys_addr = match <_ as TryInto<u16>>::try_into(phys_addr) {
            Ok(phys_addr) => Some(PhysicalAddress::from(phys_addr)),
            Err(_) => None,
        };
        self.device
            .lock()
            .await
            .set_active_source(phys_addr)
            .await
            .map_err(into_fdo_error)
    }

    async fn wake(&self) -> fdo::Result<()> {
        let Some(ref tx) = self.channel else {
            return Err(fdo::Error::Failed(String::from(
                "Device task has not started",
            )));
        };
        tx.send(SystemMessage::Wake).map_err(into_fdo_error)
    }

    async fn standby(&self, target: u8) -> fdo::Result<()> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        self.device
            .lock()
            .await
            .standby(target)
            .await
            .map_err(into_fdo_error)
    }

    async fn volume_up(&self, target: u8) -> fdo::Result<()> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let device = self.device.lock().await;
        device
            .press_user_control(UiCommand::VolumeUp, target)
            .await
            .map_err(into_fdo_error)?;
        device
            .release_user_control(target)
            .await
            .map_err(into_fdo_error)
    }

    async fn volume_down(&self, target: u8) -> fdo::Result<()> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let device = self.device.lock().await;
        device
            .press_user_control(UiCommand::VolumeDown, target)
            .await
            .map_err(into_fdo_error)?;
        device
            .release_user_control(target)
            .await
            .map_err(into_fdo_error)
    }

    async fn mute(&self, target: u8) -> fdo::Result<()> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let device = self.device.lock().await;
        device
            .press_user_control(UiCommand::Mute, target)
            .await
            .map_err(into_fdo_error)?;
        device
            .release_user_control(target)
            .await
            .map_err(into_fdo_error)
    }

    async fn send_raw_message(&self, raw_message: &[u8], target: u8) -> fdo::Result<u32> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let raw_message = Message::try_from_bytes(raw_message).map_err(into_fdo_error)?;
        self.device
            .lock()
            .await
            .tx_message(&raw_message, target)
            .await
            .map_err(into_fdo_error)
    }

    async fn send_receive_raw_message(
        &self,
        raw_message: &[u8],
        target: u8,
        opcode: u8,
        timeout: u16,
    ) -> fdo::Result<Vec<u8>> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let raw_message = Message::try_from_bytes(raw_message).map_err(into_fdo_error)?;
        let reply = self
            .device
            .lock()
            .await
            .tx_rx_message(
                &raw_message,
                target,
                Opcode::try_from_primitive(opcode).map_err(into_fdo_error)?,
                Timeout::from_ms(timeout.into()),
            )
            .await
            .map_err(into_fdo_error)?;
        Ok(reply.message.to_bytes())
    }
}
