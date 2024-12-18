/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::{anyhow, Result};
use linux_cec::device::{AsyncDevice, PollResult, PollStatus};
use linux_cec::message::Message;
use linux_cec::operand::{AbortReason, UiCommand};
use linux_cec::{FollowerMode, InitiatorMode, LogicalAddress};
use num_enum::TryFromPrimitive;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::canonicalize;
use tokio::select;
use tokio::sync::Mutex;
use tokio::task::{spawn, JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use zbus::object_server::{InterfaceRef, SignalEmitter};
use zbus::{fdo, interface, Connection};

use crate::system::SystemHandle;
use crate::uinput::UInputDevice;

fn into_fdo_error<T: Display>(val: T) -> fdo::Error {
    fdo::Error::Failed(format!("{val}"))
}

const PATH: &str = "/com/steampowered/CecDaemon1";

#[derive(Debug)]
pub struct CecDevice {
    device: Arc<Mutex<AsyncDevice>>,
    pub token: CancellationToken,
    poller: Option<JoinHandle<Result<()>>>,
    path: PathBuf,
}

struct PollTask {
    device: Arc<Mutex<AsyncDevice>>,
    token: CancellationToken,
    interface: InterfaceRef<CecDevice>,
    uinput: Option<UInputDevice>,
    active_key: Option<UiCommand>,
    connection: Connection,
    path: String,
}

impl CecDevice {
    pub async fn open(path: impl AsRef<Path>, token: CancellationToken) -> Result<CecDevice> {
        let path = canonicalize(path).await?;
        let device = Arc::new(Mutex::new(AsyncDevice::open(&path).await?));
        Ok(CecDevice {
            device,
            token,
            path,
            poller: None,
        })
    }

    pub async fn register(self, connection: Connection, system: SystemHandle) -> Result<()> {
        debug!("Registering CEC device {} on bus", self.path.display());

        let device = self.device.clone();
        let osd_name;
        let log_addr;
        let vendor_id;
        let mappings;
        {
            let system = system.lock().await;
            osd_name = system.osd_name.clone();
            if system.log_addr != LogicalAddress::UNREGISTERED {
                log_addr = system.log_addr;
            } else {
                log_addr = LogicalAddress::PlaybackDevice1;
            }
            vendor_id = system.vendor_id;
            mappings = system.mappings.clone();
        }
        debug!("OSD name: {osd_name}");
        debug!("Logical address: {log_addr:?}");
        debug!("Vendor ID: {vendor_id:?}");

        let uinput = if !mappings.is_empty() {
            let mut uinput_dev = UInputDevice::new()?;
            uinput_dev.set_mappings(mappings)?;
            uinput_dev.set_name(osd_name.clone())?;
            uinput_dev.open()?;
            Some(uinput_dev)
        } else {
            None
        };

        {
            let device = device.lock().await;
            device.set_initiator(InitiatorMode::Enabled).await?;
            device.set_osd_name(&osd_name).await?;
            device.set_vendor_id(vendor_id).await?;
            device.set_logical_address(log_addr).await?;
            device.set_follower(FollowerMode::Enabled).await?;
        }

        let token = self.token.clone();
        let object_server = connection.object_server();
        let path = self.dbus_path()?;
        object_server.at(path.clone(), self).await?;

        let interface = object_server.interface(path.clone()).await?;
        let poll_task = PollTask {
            device,
            token,
            interface: interface.clone(),
            uinput,
            active_key: None,
            connection,
            path: path.clone(),
        };
        interface.get_mut().await.poller = Some(spawn(poll_task.run()));
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
        button: u8,
        initiator: u8,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn user_control_released(
        signal_emitter: &SignalEmitter<'_>,
        initiator: u8,
    ) -> zbus::Result<()>;

    async fn activate_source(&self, text_view: bool) -> fdo::Result<()> {
        self.device
            .lock()
            .await
            .activate_source(text_view)
            .await
            .map_err(into_fdo_error)
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
}

impl PollTask {
    async fn run(mut self) -> Result<()> {
        let poller = self.device.lock().await.get_poller().await?;
        loop {
            select! {
                status = poller.poll(Duration::from_secs(2).try_into().unwrap()) => {
                    let Ok(status) = status else {
                        continue
                    };
                    if status == PollStatus::Destroyed {
                        let path = &self.path;
                        info!("Device {path} disconnected");
                        self.token.cancel();
                        break;
                    }
                    let Ok(results) = self
                        .device
                        .lock()
                        .await
                        .handle_status(status)
                        .await
                        .inspect_err(|e| warn!("Failed to handle status: {e}"))
                    else {
                        continue
                    };
                    for res in results {
                        if let Err(err) = self.handle_poll_result(res).await {
                            error!("Poll handling failed: {err}");
                        }
                    }
                }
                _ = self.token.cancelled() => break,
            }
        }
        let path = self.path;
        let object_server = self.connection.object_server();
        object_server.remove::<CecDevice, String>(path).await?;
        Ok(())
    }

    async fn handle_poll_result(&mut self, result: PollResult) -> Result<()> {
        let PollResult::Message(envelope) = result else {
            return Ok(());
        };

        let initiator: u8 = envelope.initiator.into();
        let destination: u8 = envelope.destination.into();
        debug!(
            "Got message from {initiator} to {destination}: {:?}",
            envelope.message
        );
        self.interface
            .received_message(
                envelope.initiator.into(),
                envelope.destination.into(),
                envelope.timestamp,
                envelope.message.to_bytes().as_ref(),
            )
            .await?;

        let reply = match envelope.message {
            Message::UserControlPressed { ui_command } => {
                self.interface
                    .user_control_pressed(ui_command as u8, envelope.initiator.into())
                    .await?;
                if let Some(uinput) = self.uinput.as_ref() {
                    if let Some(old_key) = self.active_key {
                        uinput.key_up(old_key)?;
                    }
                    uinput.key_down(ui_command)?;
                }
                self.active_key = Some(ui_command);
                None
            }
            Message::UserControlReleased => {
                self.interface
                    .user_control_released(envelope.initiator.into())
                    .await?;
                if let Some(old_key) = self.active_key {
                    if let Some(uinput) = self.uinput.as_ref() {
                        uinput.key_up(old_key)?;
                    }
                    self.active_key = None;
                }
                None
            }
            _ => None,
        };

        if let Some(reply) = reply {
            self.device
                .lock()
                .await
                .tx_message(&reply, envelope.initiator)
                .await?;
        } else if envelope.destination != LogicalAddress::BROADCAST {
            let abort = Message::FeatureAbort {
                opcode: envelope.message.opcode(),
                abort_reason: AbortReason::UnrecognizedOp,
            };
            self.device
                .lock()
                .await
                .tx_message(&abort, envelope.initiator)
                .await?;
        }
        Ok(())
    }
}
