/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::{anyhow, Result};
use linux_cec::device::{AsyncDevice, Envelope, PollResult, PollStatus};
use linux_cec::message::Message;
use linux_cec::operand::{AbortReason, PowerStatus, UiCommand};
use linux_cec::{Error, LogicalAddress};
use nix::errno::Errno;
use num_enum::TryFromPrimitive;
use std::fmt::Display;
use std::mem::drop;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::canonicalize;
use tokio::select;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;
use tokio::task::{spawn, JoinHandle};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use zbus::object_server::{InterfaceRef, SignalEmitter};
use zbus::{fdo, interface, Connection};

use crate::system::{SystemHandle, SystemMessage};
use crate::uinput::UInputDevice;

fn into_fdo_error<T: Display>(val: T) -> fdo::Error {
    fdo::Error::Failed(format!("{val}"))
}

const PATH: &str = "/com/steampowered/CecDaemon1";
const LOG_ADDR_RETRIES: i32 = 10;
const WAKE_TRIES: i32 = 10;
const WAKE_DELAY: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub struct CecDevice {
    device: Arc<Mutex<AsyncDevice>>,
    pub token: CancellationToken,
    poller: Option<JoinHandle<Result<()>>>,
    path: PathBuf,
    cached_phys_addr: u16,
    cached_log_addrs: Vec<u8>,
    cached_vendor_id: i32,
}

struct PollTask {
    device: Arc<Mutex<AsyncDevice>>,
    system: SystemHandle,
    token: CancellationToken,
    interface: InterfaceRef<CecDevice>,
    uinput: Option<UInputDevice>,
    active_key: Option<UiCommand>,
    channel: UnboundedReceiver<SystemMessage>,
    connection: Connection,
    path: String,
    log_addr_try: i32,
    awaiting_wake: bool,
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
            cached_phys_addr: 0xFFFF,
            cached_log_addrs: Vec::new(),
            cached_vendor_id: -1,
        })
    }

    pub async fn register(
        self,
        connection: Connection,
        system: SystemHandle,
        channel: UnboundedReceiver<SystemMessage>,
    ) -> Result<()> {
        debug!("Registering CEC device {} on bus", self.path.display());

        let device = self.device.clone();
        let uinput = system.lock().await.configure_dev(device.clone()).await?;
        let token = self.token.clone();
        let object_server = connection.object_server();
        let path = self.dbus_path()?;
        object_server.at(path.clone(), self).await?;

        let interface = object_server.interface(path.clone()).await?;
        let poll_task = PollTask {
            device,
            system,
            token,
            interface: interface.clone(),
            uinput,
            active_key: None,
            channel,
            connection,
            path: path.clone(),
            log_addr_try: LOG_ADDR_RETRIES,
            awaiting_wake: false,
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

    async fn send_raw_message(&self, raw_message: &[u8], target: u8) -> fdo::Result<()> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let raw_message = Message::try_from_bytes(raw_message).map_err(into_fdo_error)?;
        self.device
            .lock()
            .await
            .tx_message(&raw_message, target)
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
                message = self.channel.recv() => {
                    let Some(message) = message else {
                        break;
                    };
                    if let Err(err) = self.handle_system_message(message).await {
                        error!("Message handling failed: {err}");
                    }
                }
                _ = self.token.cancelled() => break,
            }
        }
        let path = self.path;
        info!("Deregistering path {path}");
        let object_server = self.connection.object_server();
        object_server.remove::<CecDevice, String>(path).await?;
        Ok(())
    }

    async fn handle_poll_result(&mut self, result: PollResult) -> Result<()> {
        match result {
            PollResult::Message(envelope) => self.handle_message(envelope).await?,
            PollResult::PinEvent(_) => (),
            PollResult::LostMessages(n) => warn!("Lost {n} messages!"),
            PollResult::StateChange => {
                let device = self.device.lock().await;
                let phys_addr = device
                    .get_physical_address()
                    .await
                    .unwrap_or_default()
                    .into();
                let log_addrs = device
                    .get_logical_addresses()
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|v| v.into())
                    .collect();
                let vendor_id = device
                    .get_vendor_id()
                    .await
                    .unwrap_or_default()
                    .map(|v| ((v.0[0] as i32) << 16) | ((v.0[1] as i32) << 8) | (v.0[2] as i32))
                    .unwrap_or(-1);

                let emitter = self.interface.signal_emitter();
                let mut iface = self.interface.get_mut().await;
                if iface.cached_phys_addr != phys_addr {
                    info!(
                        "Physical address changed from {:?} to {phys_addr:?}",
                        iface.cached_phys_addr
                    );
                    iface.cached_phys_addr = phys_addr;
                    iface.physical_address_changed(emitter).await?;
                }
                if iface.cached_vendor_id != vendor_id {
                    info!(
                        "Vendor ID changed from {:?} to {vendor_id:?}",
                        iface.cached_vendor_id
                    );
                    iface.cached_vendor_id = vendor_id;
                    iface.vendor_id_changed(emitter).await?;
                }
                if iface.cached_log_addrs != log_addrs {
                    info!(
                        "Logical addresses changed from {:?} to {log_addrs:?}",
                        iface.cached_log_addrs
                    );
                    iface.cached_log_addrs = log_addrs;
                    if iface.cached_log_addrs.is_empty() {
                        self.log_addr_try = LOG_ADDR_RETRIES;
                    }
                    iface.logical_addresses_changed(emitter).await?;
                } else if log_addrs.is_empty() && phys_addr != 0xFFFF {
                    if self.log_addr_try > 0 {
                        info!("Did not get logical address, retrying registration");
                        self.log_addr_try -= 1;
                        drop(device);
                        self.system
                            .lock()
                            .await
                            .configure_dev(self.device.clone())
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_message(&mut self, envelope: Envelope) -> Result<()> {
        let initiator = envelope.initiator;
        let destination = envelope.destination;
        debug!(
            "Got message from {initiator} ({:x}) to {destination} ({:x}): {:?}",
            initiator as u8, destination as u8, envelope.message
        );
        self.interface
            .received_message(
                initiator.into(),
                destination.into(),
                envelope.timestamp,
                envelope.message.to_bytes().as_ref(),
            )
            .await?;

        let reply = match envelope.message {
            Message::GiveDevicePowerStatus => Some(Message::ReportPowerStatus {
                status: PowerStatus::On,
            }),
            Message::UserControlPressed { ui_command } => {
                self.interface
                    .user_control_pressed(ui_command as u8, initiator as u8)
                    .await?;
                if let Some(uinput) = self.uinput.as_ref() {
                    if let Some(old_key) = self.active_key {
                        uinput.key_up(old_key)?;
                    }
                    uinput.key_down(ui_command)?;
                }
                self.active_key = Some(ui_command);
                return Ok(());
            }
            Message::UserControlReleased => {
                self.interface
                    .user_control_released(initiator as u8)
                    .await?;
                if let Some(old_key) = self.active_key {
                    if let Some(uinput) = self.uinput.as_ref() {
                        uinput.key_up(old_key)?;
                    }
                    self.active_key = None;
                }
                return Ok(());
            }
            Message::SetStreamPath { address } => {
                let this_address = self.device.lock().await.get_physical_address().await?;
                if address == this_address {
                    Some(Message::ActiveSource {
                        address: this_address,
                    })
                } else {
                    None
                }
            }
            _ if envelope.destination != LogicalAddress::BROADCAST => Some(Message::FeatureAbort {
                opcode: envelope.message.opcode(),
                abort_reason: AbortReason::UnrecognizedOp,
            }),
            Message::RoutingChange { new_address, .. } => {
                let this_address = self.device.lock().await.get_physical_address().await?;
                if new_address == this_address {
                    self.awaiting_wake = false;
                }
                None
            }
            Message::RequestActiveSource if self.awaiting_wake => {
                let address = self.device.lock().await.get_physical_address().await?;
                Some(Message::ActiveSource { address })
            }
            _ => None,
        };

        if let Some(reply) = reply {
            self.device
                .lock()
                .await
                .tx_message(&reply, envelope.initiator)
                .await?;
        }
        Ok(())
    }

    async fn wake(&mut self) -> Result<()> {
        self.awaiting_wake = true;
        for _ in 0..WAKE_TRIES {
            let result = self.device.lock().await.activate_source(false).await;
            match result {
                Ok(()) => {
                    sleep(WAKE_DELAY).await;
                    if !self.awaiting_wake {
                        return Ok(());
                    }
                    continue;
                }
                Err(Error::Errno(Errno::ENONET)) => {
                    debug!("Lost logical address. Retrying configuring.");
                    let Err(err) = self
                        .system
                        .lock()
                        .await
                        .configure_dev(self.device.clone())
                        .await
                    else {
                        continue;
                    };
                    if matches!(err.downcast::<Error>(), Ok(Error::Errno(Errno::ENODEV))) {
                        self.awaiting_wake = false;
                        debug!("Device was disconnected.");
                        return Err(Error::Errno(Errno::ENODEV).into());
                    }
                }
                Err(Error::Errno(Errno::ENODEV)) => {
                    self.awaiting_wake = false;
                    result?;
                }
                Err(e) => warn!("Failed to activate source: {e}"),
            };
            sleep(WAKE_DELAY).await;
        }
        warn!("Failed to wake TV");
        Ok(())
    }

    async fn handle_system_message(&mut self, message: SystemMessage) -> Result<()> {
        match message {
            SystemMessage::Wake => self.wake().await,
            SystemMessage::Standby => {
                self.device.lock().await.standby(LogicalAddress::Tv).await?;
                Ok(())
            }
            SystemMessage::ReloadConfig => {
                self.uinput = None; // Drop old UInputDevice before opening a new one
                self.uinput = self
                    .system
                    .lock()
                    .await
                    .configure_dev(self.device.clone())
                    .await?;
                Ok(())
            }
        }
    }
}
