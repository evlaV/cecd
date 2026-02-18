/*
 * Copyright © 2025 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::Result;
use linux_cec::device::{Envelope, MessageData, PollResult, PollStatus};
use linux_cec::message::{Message, Opcode};
use linux_cec::operand::{AbortReason, OperandEncodable, PowerStatus, UiCommand};
use linux_cec::{Error, LogicalAddress};
#[cfg(not(test))]
use std::future::Future;
use std::mem::drop;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use zbus::object_server::InterfaceRef;
use zbus::zvariant::OwnedObjectPath;
use zbus::Connection;

use crate::dbus::{CecDevice, CecDeviceSignals};
use crate::system::{SystemHandle, SystemMessage};
use crate::uinput::UInputDevice;
use crate::{ArcDevice, AsyncDevicePoller};

const LOG_ADDR_RETRIES: i32 = 20;
const WAKE_TRIES: i32 = 2;
const WAKE_DELAY: Duration = Duration::from_millis(1000);

pub struct DeviceTask {
    device: ArcDevice,
    system: SystemHandle,
    token: CancellationToken,
    interface: InterfaceRef<CecDevice>,
    active_key: Option<UiCommand>,
    channel: Receiver<SystemMessage>,
    connection: Connection,
    path: OwnedObjectPath,
    log_addr_try: i32,
    awaiting_wake: bool,
    poller: AsyncDevicePoller,
    active: bool,
}

#[derive(Debug)]
pub struct KeyRepeat {
    pub device: ArcDevice,
    pub token: CancellationToken,
    pub log_addr: LogicalAddress,
    pub key: UiCommand,
}

impl DeviceTask {
    pub const STATIC_HANDLERS: &[Opcode] = &[
        Opcode::GiveDevicePowerStatus,
        Opcode::UserControlPressed,
        Opcode::UserControlReleased,
        Opcode::SetStreamPath,
        Opcode::Standby,
        Opcode::RoutingChange,
        Opcode::RequestActiveSource,
    ];

    pub async fn new(
        iface: InterfaceRef<CecDevice>,
        system: SystemHandle,
        channel: Receiver<SystemMessage>,
        connection: Connection,
    ) -> Result<DeviceTask> {
        let interface = iface.clone();
        let device;
        let token;
        let path;
        {
            let dbus_obj = iface.get().await;
            device = dbus_obj.device.clone();
            token = dbus_obj.token.clone();
            path = OwnedObjectPath::from(dbus_obj.dbus_path().clone());
        }
        let poller = device.lock().await.get_poller().await?;
        system.lock().await.configure_dev(device.clone()).await?;
        let mut device_task = DeviceTask {
            device,
            system,
            token,
            interface,
            active_key: None,
            channel,
            connection,
            path,
            log_addr_try: LOG_ADDR_RETRIES,
            awaiting_wake: false,
            active: false,
            poller,
        };
        device_task.configure_uinput().await?;
        Ok(device_task)
    }

    pub async fn run(mut self) -> Result<()> {
        loop {
            select! {
                status = self.poller.poll(Duration::from_secs(2).try_into().unwrap()) => {
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
                    let Ok(message) = message else {
                        break;
                    };
                    if let Err(err) = self.handle_system_message(message).await {
                        error!("Message handling failed: {err}");
                    }
                }
                () = self.token.cancelled() => break,
            }
        }
        let path = self.path;
        info!("Deregistering path {path}");
        let object_server = self.connection.object_server();
        object_server.remove::<CecDevice, _>(path).await?;
        Ok(())
    }

    async fn handle_poll_result(&mut self, result: PollResult) -> Result<()> {
        match result {
            PollResult::Message(envelope) => self.handle_message(envelope).await?,
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
                    .map(Into::into)
                    .collect();
                let vendor_id = device
                    .get_vendor_id()
                    .await
                    .unwrap_or_default()
                    .map_or(-1, Into::<i32>::into);

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
                } else if log_addrs.is_empty() && phys_addr != 0xFFFF && self.log_addr_try > 0 {
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
            _ => (),
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
            MessageData::Valid(Message::GiveDevicePowerStatus) => Some((
                Message::ReportPowerStatus {
                    status: PowerStatus::On,
                },
                initiator,
            )),
            MessageData::Valid(Message::UserControlPressed { ui_command }) => {
                let mut buf = Vec::new();
                ui_command.to_bytes(&mut buf);
                self.interface
                    .user_control_pressed(buf.as_ref(), initiator as u8)
                    .await?;
                if let Some(uinput) = self.interface.get_mut().await.uinput.as_mut() {
                    if let Some(old_key) = self.active_key {
                        uinput.key_up(old_key)?;
                    }
                    uinput.key_down(ui_command)?;
                }
                self.active_key = Some(ui_command);
                None
            }
            MessageData::Valid(Message::UserControlReleased) => {
                self.interface
                    .user_control_released(initiator as u8)
                    .await?;
                if let Some(old_key) = self.active_key {
                    if let Some(uinput) = self.interface.get_mut().await.uinput.as_mut() {
                        uinput.key_up(old_key)?;
                    }
                    self.active_key = None;
                }
                None
            }
            MessageData::Valid(Message::SetStreamPath { address }) => {
                let this_address = self.device.lock().await.get_physical_address().await?;
                if address == this_address {
                    Some((
                        Message::ActiveSource {
                            address: this_address,
                        },
                        LogicalAddress::Broadcast,
                    ))
                } else {
                    None
                }
            }
            MessageData::Valid(Message::Standby)
                if self.system.lock().await.config.allow_standby =>
            {
                if let Err(e) = self.system.suspend().await {
                    error!("Failed to standby: {e}");
                    Some((
                        Message::FeatureAbort {
                            opcode: envelope.message.opcode(),
                            abort_reason: AbortReason::IncorrectMode,
                        },
                        initiator,
                    ))
                } else {
                    None
                }
            }
            MessageData::Valid(Message::RoutingChange { new_address, .. }) => {
                let this_address = self.device.lock().await.get_physical_address().await?;
                if new_address == this_address {
                    self.awaiting_wake = false;
                    self.active = true;
                } else {
                    self.active = false;
                }
                None
            }
            MessageData::Valid(Message::RequestActiveSource)
                if self.awaiting_wake || self.active =>
            {
                let address = self.device.lock().await.get_physical_address().await?;
                Some((Message::ActiveSource { address }, LogicalAddress::Broadcast))
            }
            _ if envelope.destination != LogicalAddress::Broadcast => {
                let opcode = envelope.message.opcode();
                if let Some(handler) = self.system.get_message_handler(opcode).await {
                    handler.handle(&self.path, opcode, &envelope).await
                } else {
                    Some((
                        Message::FeatureAbort {
                            opcode: envelope.message.opcode(),
                            abort_reason: AbortReason::UnrecognizedOp,
                        },
                        initiator,
                    ))
                }
            }
            _ => None,
        };

        if let Some((reply, address)) = reply {
            self.device.lock().await.tx_message(&reply, address).await?;
        }
        Ok(())
    }

    async fn wake(&mut self) -> Result<()> {
        self.device.lock().await.wake(false, false).await?;
        self.awaiting_wake = true;
        for _ in 0..WAKE_TRIES {
            let result = self.device.lock().await.set_active_source(None).await;
            match result {
                Ok(()) => {
                    if !self.awaiting_wake {
                        return Ok(());
                    }
                }
                Err(Error::NoLogicalAddress) => {
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
                    if matches!(err.downcast::<Error>(), Ok(Error::Disconnected)) {
                        self.awaiting_wake = false;
                        debug!("Device was disconnected.");
                        return Err(Error::Disconnected.into());
                    }
                }
                Err(Error::Disconnected) => {
                    self.awaiting_wake = false;
                    result?;
                }
                Err(e) => warn!("Failed to activate source: {e}"),
            }
            sleep(WAKE_DELAY).await;
        }
        info!("TV did not respond to wake immediately");
        Ok(())
    }

    async fn handle_system_message(&mut self, message: SystemMessage) -> Result<()> {
        match message {
            SystemMessage::Wake => self.wake().await,
            SystemMessage::Standby { standby_tv } => {
                let device = self.device.lock().await;
                let address = device.get_physical_address().await?;
                device
                    .tx_message(&Message::InactiveSource { address }, LogicalAddress::Tv)
                    .await?;
                if standby_tv {
                    device.standby(LogicalAddress::Tv).await?;
                }
                Ok(())
            }
            SystemMessage::ReloadConfig => {
                self.system
                    .lock()
                    .await
                    .configure_dev(self.device.clone())
                    .await?;
                self.configure_uinput().await?;
                Ok(())
            }
        }
    }

    async fn configure_uinput(&mut self) -> Result<()> {
        let mut interface = self.interface.get_mut().await;
        let system = self.system.lock().await;
        interface.uinput = None; // Drop old UInputDevice before opening a new one
        if system.config.mappings.is_empty() || !system.config.uinput {
            return Ok(());
        }

        let mappings = system.config.mappings.clone();
        drop(system);

        let adapter_name = self.device.lock().await.get_adapter_name().await?;
        let adapter_name = adapter_name.to_string_lossy();
        let mut uinput =
            UInputDevice::new().inspect_err(|e| warn!("Failed to open uinput device: {e}"))?;
        uinput.set_mappings(mappings)?;
        uinput.set_name(format!("cecd {adapter_name}"))?;
        uinput.open()?;
        interface.uinput = Some(uinput);
        Ok(())
    }
}

impl KeyRepeat {
    #[cfg(not(test))]
    fn delay(&self) -> impl Future<Output = ()> {
        // Recommended interval of 450ms is per H14b CEC 13.13.3,
        // starting at the beginning of message transmission
        sleep(Duration::from_millis(450))
    }

    #[cfg(test)]
    async fn delay(&self) {
        let key_repeat = self.device.lock().await.key_repeat.clone();
        key_repeat.notified().await
    }

    pub async fn run(self) -> Result<()> {
        loop {
            let delay = self.delay();
            self.device
                .lock()
                .await
                .press_user_control(self.key, self.log_addr)
                .await?;
            select! {
                () = self.token.cancelled() => break,
                () = delay => continue,
            }
        }
        Ok(self
            .device
            .lock()
            .await
            .release_user_control(self.log_addr)
            .await?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use input_linux::{EventKind, Key, KeyEvent, KeyState};
    use linux_cec::device::Capabilities;
    use linux_cec::message::Opcode;
    use linux_cec::{LogicalAddressType, PhysicalAddress};
    use std::time::Duration;

    use crate::config::Config;
    use crate::testing::setup_dbus_test;

    async fn rx_message(dev: &ArcDevice) -> Option<(Message, LogicalAddress)> {
        for _ in 0..100 {
            let Some(message) = dev.lock().await.dequeue_tx_message().await else {
                sleep(Duration::from_millis(1)).await;
                continue;
            };
            return Some(message);
        }
        None
    }

    #[tokio::test]
    async fn test_tx_basic() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = false;
        config.logical_address = LogicalAddressType::Playback;
        let test = setup_dbus_test(cb, Some(config)).await.unwrap();
        assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
        assert_eq!(
            test.proxy.logical_addresses().await.unwrap(),
            &[u8::from(LogicalAddress::PlaybackDevice1)]
        );

        test.proxy.standby(LogicalAddress::Tv.into()).await.unwrap();
        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((Message::Standby {}, LogicalAddress::Tv))
        );
    }

    #[tokio::test]
    async fn test_system_message_standby() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = false;
        config.logical_address = LogicalAddressType::Playback;
        let test = setup_dbus_test(cb, Some(config)).await.unwrap();
        assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
        assert_eq!(
            test.proxy.logical_addresses().await.unwrap(),
            &[u8::from(LogicalAddress::PlaybackDevice1)]
        );

        let interface: InterfaceRef<CecDevice> = test
            .connection
            .object_server()
            .interface("/com/steampowered/CecDaemon1/Devices/Null")
            .await
            .unwrap();
        {
            let dev = interface.get_mut().await;
            dev.send_system_message(SystemMessage::Standby { standby_tv: true })
                .await
                .unwrap();
        }
        assert_eq!(
            rx_message(&test.dev).await.unwrap(),
            (
                Message::InactiveSource {
                    address: PhysicalAddress::from(0x1000)
                },
                LogicalAddress::Tv
            )
        );
        assert_eq!(
            rx_message(&test.dev).await.unwrap(),
            (Message::Standby {}, LogicalAddress::Tv)
        );
    }

    #[tokio::test]
    async fn test_system_message_standby_no_sleep() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = false;
        config.logical_address = LogicalAddressType::Playback;
        let test = setup_dbus_test(cb, Some(config)).await.unwrap();
        assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
        assert_eq!(
            test.proxy.logical_addresses().await.unwrap(),
            &[u8::from(LogicalAddress::PlaybackDevice1)]
        );

        let interface: InterfaceRef<CecDevice> = test
            .connection
            .object_server()
            .interface("/com/steampowered/CecDaemon1/Devices/Null")
            .await
            .unwrap();
        {
            let dev = interface.get_mut().await;
            dev.send_system_message(SystemMessage::Standby { standby_tv: false })
                .await
                .unwrap();
        }
        assert_eq!(
            rx_message(&test.dev).await.unwrap(),
            (
                Message::InactiveSource {
                    address: PhysicalAddress::from(0x1000)
                },
                LogicalAddress::Tv
            )
        );
        assert!(rx_message(&test.dev).await.is_none(),);
    }

    #[tokio::test]
    async fn test_rx_abort() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = false;
        config.logical_address = LogicalAddressType::Playback;
        let test = setup_dbus_test(cb, Some(config)).await.unwrap();
        assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
        assert_eq!(
            test.proxy.logical_addresses().await.unwrap(),
            &[u8::from(LogicalAddress::PlaybackDevice1)]
        );

        test.dev
            .lock()
            .await
            .queue_rx_message(Envelope {
                message: MessageData::Valid(Message::RecordOff {}),
                initiator: LogicalAddress::Tv,
                destination: LogicalAddress::PlaybackDevice1,
                timestamp: 0,
                sequence: 1,
            })
            .await;

        assert_eq!(
            rx_message(&test.dev).await.unwrap(),
            (
                Message::FeatureAbort {
                    opcode: Opcode::RecordOff as u8,
                    abort_reason: AbortReason::UnrecognizedOp,
                },
                LogicalAddress::Tv
            )
        );
    }

    #[tokio::test]
    async fn test_give_device_power_status() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = false;
        config.logical_address = LogicalAddressType::Playback;
        let test = setup_dbus_test(cb, Some(config)).await.unwrap();
        assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
        assert_eq!(
            test.proxy.logical_addresses().await.unwrap(),
            &[u8::from(LogicalAddress::PlaybackDevice1)]
        );

        test.dev
            .lock()
            .await
            .queue_rx_message(Envelope {
                message: MessageData::Valid(Message::GiveDevicePowerStatus {}),
                initiator: LogicalAddress::Tv,
                destination: LogicalAddress::PlaybackDevice1,
                timestamp: 0,
                sequence: 1,
            })
            .await;

        assert_eq!(
            rx_message(&test.dev).await.unwrap(),
            (
                Message::ReportPowerStatus {
                    status: PowerStatus::On,
                },
                LogicalAddress::Tv
            )
        );
    }

    #[tokio::test]
    async fn test_key_repeat_none() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = false;
        config.logical_address = LogicalAddressType::Playback;
        let mut test = setup_dbus_test(cb, Some(config)).await.unwrap();
        assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
        assert_eq!(
            test.proxy.logical_addresses().await.unwrap(),
            &[u8::from(LogicalAddress::PlaybackDevice1)]
        );

        let mut buf = Vec::new();
        UiCommand::Select.to_bytes(&mut buf);
        test.proxy
            .press_user_control(&buf, LogicalAddress::Tv.into())
            .await
            .unwrap();
        test.proxy
            .release_user_control(LogicalAddress::Tv.into())
            .await
            .unwrap();

        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((
                Message::UserControlPressed {
                    ui_command: UiCommand::Select
                },
                LogicalAddress::Tv
            ))
        );
        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((Message::UserControlReleased {}, LogicalAddress::Tv))
        );
        assert!(test.dev.lock().await.dequeue_tx_message().await.is_none());
    }

    #[tokio::test]
    async fn test_key_press_once() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = false;
        config.logical_address = LogicalAddressType::Playback;
        let mut test = setup_dbus_test(cb, Some(config)).await.unwrap();
        assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
        assert_eq!(
            test.proxy.logical_addresses().await.unwrap(),
            &[u8::from(LogicalAddress::PlaybackDevice1)]
        );

        let mut buf = Vec::new();
        UiCommand::Select.to_bytes(&mut buf);
        test.proxy
            .press_once_user_control(&buf, LogicalAddress::Tv.into())
            .await
            .unwrap();
        test.dev.lock().await.key_repeat.notify_one();

        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((
                Message::UserControlPressed {
                    ui_command: UiCommand::Select
                },
                LogicalAddress::Tv
            ))
        );
        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((Message::UserControlReleased {}, LogicalAddress::Tv))
        );
        assert!(test.dev.lock().await.dequeue_tx_message().await.is_none());
    }

    #[tokio::test]
    async fn test_key_repeat() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = false;
        config.logical_address = LogicalAddressType::Playback;
        let mut test = setup_dbus_test(cb, Some(config)).await.unwrap();
        assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
        assert_eq!(
            test.proxy.logical_addresses().await.unwrap(),
            &[u8::from(LogicalAddress::PlaybackDevice1)]
        );

        let mut buf = Vec::new();
        UiCommand::Select.to_bytes(&mut buf);
        test.proxy
            .press_user_control(&buf, LogicalAddress::Tv.into())
            .await
            .unwrap();
        test.dev.lock().await.key_repeat.notify_one();
        test.proxy
            .release_user_control(LogicalAddress::Tv.into())
            .await
            .unwrap();

        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((
                Message::UserControlPressed {
                    ui_command: UiCommand::Select
                },
                LogicalAddress::Tv
            ))
        );
        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((
                Message::UserControlPressed {
                    ui_command: UiCommand::Select
                },
                LogicalAddress::Tv
            ))
        );
        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((Message::UserControlReleased {}, LogicalAddress::Tv))
        );
        assert!(test.dev.lock().await.dequeue_tx_message().await.is_none());
    }

    #[tokio::test]
    async fn test_key_double_press() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = false;
        config.logical_address = LogicalAddressType::Playback;
        let mut test = setup_dbus_test(cb, Some(config)).await.unwrap();
        assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
        assert_eq!(
            test.proxy.logical_addresses().await.unwrap(),
            &[u8::from(LogicalAddress::PlaybackDevice1)]
        );

        let mut buf = Vec::new();
        UiCommand::Select.to_bytes(&mut buf);
        test.proxy
            .press_user_control(&buf, LogicalAddress::Tv.into())
            .await
            .unwrap();
        test.proxy
            .press_user_control(&buf, LogicalAddress::Tv.into())
            .await
            .unwrap();
        test.proxy
            .release_user_control(LogicalAddress::Tv.into())
            .await
            .unwrap();

        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((
                Message::UserControlPressed {
                    ui_command: UiCommand::Select
                },
                LogicalAddress::Tv
            ))
        );
        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((Message::UserControlReleased {}, LogicalAddress::Tv))
        );
        assert!(test.dev.lock().await.dequeue_tx_message().await.is_none());
    }

    #[tokio::test]
    async fn test_key_change() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = false;
        config.logical_address = LogicalAddressType::Playback;
        let mut test = setup_dbus_test(cb, Some(config)).await.unwrap();
        assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
        assert_eq!(
            test.proxy.logical_addresses().await.unwrap(),
            &[u8::from(LogicalAddress::PlaybackDevice1)]
        );

        let mut buf = Vec::new();
        UiCommand::Select.to_bytes(&mut buf);
        test.proxy
            .press_user_control(&buf, LogicalAddress::Tv.into())
            .await
            .unwrap();
        let mut buf = Vec::new();
        UiCommand::Back.to_bytes(&mut buf);
        test.proxy
            .press_user_control(&buf, LogicalAddress::Tv.into())
            .await
            .unwrap();
        test.proxy
            .release_user_control(LogicalAddress::Tv.into())
            .await
            .unwrap();

        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((
                Message::UserControlPressed {
                    ui_command: UiCommand::Select
                },
                LogicalAddress::Tv
            ))
        );
        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((Message::UserControlReleased {}, LogicalAddress::Tv))
        );
        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((
                Message::UserControlPressed {
                    ui_command: UiCommand::Back
                },
                LogicalAddress::Tv
            ))
        );
        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((Message::UserControlReleased {}, LogicalAddress::Tv))
        );
        assert!(test.dev.lock().await.dequeue_tx_message().await.is_none());
    }

    #[tokio::test]
    async fn test_key_change_once() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = false;
        config.logical_address = LogicalAddressType::Playback;
        let mut test = setup_dbus_test(cb, Some(config)).await.unwrap();
        assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
        assert_eq!(
            test.proxy.logical_addresses().await.unwrap(),
            &[u8::from(LogicalAddress::PlaybackDevice1)]
        );

        let mut buf = Vec::new();
        UiCommand::Select.to_bytes(&mut buf);
        test.proxy
            .press_user_control(&buf, LogicalAddress::Tv.into())
            .await
            .unwrap();
        let mut buf = Vec::new();
        UiCommand::Back.to_bytes(&mut buf);
        test.proxy
            .press_once_user_control(&buf, LogicalAddress::Tv.into())
            .await
            .unwrap();

        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((
                Message::UserControlPressed {
                    ui_command: UiCommand::Select
                },
                LogicalAddress::Tv
            ))
        );
        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((Message::UserControlReleased {}, LogicalAddress::Tv))
        );
        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((
                Message::UserControlPressed {
                    ui_command: UiCommand::Back
                },
                LogicalAddress::Tv
            ))
        );
        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((Message::UserControlReleased {}, LogicalAddress::Tv))
        );
        assert!(test.dev.lock().await.dequeue_tx_message().await.is_none());
    }

    #[tokio::test]
    async fn test_key_not_changed_once() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = false;
        config.logical_address = LogicalAddressType::Playback;
        let mut test = setup_dbus_test(cb, Some(config)).await.unwrap();
        assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
        assert_eq!(
            test.proxy.logical_addresses().await.unwrap(),
            &[u8::from(LogicalAddress::PlaybackDevice1)]
        );

        let mut buf = Vec::new();
        UiCommand::Select.to_bytes(&mut buf);
        test.proxy
            .press_user_control(&buf, LogicalAddress::Tv.into())
            .await
            .unwrap();
        test.proxy
            .press_once_user_control(&buf, LogicalAddress::Tv.into())
            .await
            .unwrap();

        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((
                Message::UserControlPressed {
                    ui_command: UiCommand::Select
                },
                LogicalAddress::Tv
            ))
        );
        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((Message::UserControlReleased {}, LogicalAddress::Tv))
        );
        assert!(test.dev.lock().await.dequeue_tx_message().await.is_none());
    }

    #[tokio::test]
    async fn test_key_release_unmatched() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = false;
        config.logical_address = LogicalAddressType::Playback;
        let mut test = setup_dbus_test(cb, Some(config)).await.unwrap();
        assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
        assert_eq!(
            test.proxy.logical_addresses().await.unwrap(),
            &[u8::from(LogicalAddress::PlaybackDevice1)]
        );

        test.proxy
            .release_user_control(LogicalAddress::Tv.into())
            .await
            .unwrap();

        assert_eq!(
            test.dev.lock().await.dequeue_tx_message().await,
            Some((Message::UserControlReleased {}, LogicalAddress::Tv))
        );
        assert!(test.dev.lock().await.dequeue_tx_message().await.is_none());
    }

    #[tokio::test]
    async fn test_mapped_keys() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = true;
        config.mappings = [(UiCommand::Enter, Key::Enter)].into();
        config.logical_address = LogicalAddressType::Playback;
        let test = setup_dbus_test(cb, Some(config)).await.unwrap();

        test.dev
            .lock()
            .await
            .queue_rx_message(Envelope {
                message: MessageData::Valid(Message::UserControlPressed {
                    ui_command: UiCommand::Enter,
                }),
                initiator: LogicalAddress::Tv,
                destination: LogicalAddress::PlaybackDevice1,
                timestamp: 0,
                sequence: 1,
            })
            .await;
        test.dev
            .lock()
            .await
            .queue_rx_message(Envelope {
                message: MessageData::Valid(Message::UserControlReleased {}),
                initiator: LogicalAddress::Tv,
                destination: LogicalAddress::PlaybackDevice1,
                timestamp: 0,
                sequence: 1,
            })
            .await;
        let notify = test.dev.lock().await.rx_queue_empty().await.unwrap();
        notify.notified().await;

        let interface: InterfaceRef<CecDevice> = test
            .connection
            .object_server()
            .interface("/com/steampowered/CecDaemon1/Devices/Null")
            .await
            .unwrap();
        let mut dbus_obj = interface.get_mut().await;
        let uinput = dbus_obj.uinput.as_mut().unwrap();
        let event = uinput.get_next_event().unwrap();
        let key = KeyEvent::try_from(event).unwrap();
        assert_eq!(key.key, Key::Enter);
        assert_eq!(key.value, KeyState::PRESSED);

        assert_eq!(
            uinput.get_next_event().unwrap().kind,
            EventKind::Synchronize
        );

        let event = uinput.get_next_event().unwrap();
        let key = KeyEvent::try_from(event).unwrap();
        assert_eq!(key.key, Key::Enter);
        assert_eq!(key.value, KeyState::RELEASED);

        assert_eq!(
            uinput.get_next_event().unwrap().kind,
            EventKind::Synchronize
        );
    }

    #[tokio::test]
    async fn test_unmapped_keys() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = true;
        config.mappings = [(UiCommand::Enter, Key::Enter)].into();
        config.logical_address = LogicalAddressType::Playback;
        let test = setup_dbus_test(cb, Some(config)).await.unwrap();

        test.dev
            .lock()
            .await
            .queue_rx_message(Envelope {
                message: MessageData::Valid(Message::UserControlPressed {
                    ui_command: UiCommand::Back,
                }),
                initiator: LogicalAddress::Tv,
                destination: LogicalAddress::PlaybackDevice1,
                timestamp: 0,
                sequence: 1,
            })
            .await;
        test.dev
            .lock()
            .await
            .queue_rx_message(Envelope {
                message: MessageData::Valid(Message::UserControlReleased {}),
                initiator: LogicalAddress::Tv,
                destination: LogicalAddress::PlaybackDevice1,
                timestamp: 0,
                sequence: 1,
            })
            .await;
        let notify = test.dev.lock().await.rx_queue_empty().await.unwrap();
        notify.notified().await;

        let interface: InterfaceRef<CecDevice> = test
            .connection
            .object_server()
            .interface("/com/steampowered/CecDaemon1/Devices/Null")
            .await
            .unwrap();
        let mut dbus_obj = interface.get_mut().await;
        let uinput = dbus_obj.uinput.as_mut().unwrap();
        assert!(uinput.get_next_event().is_none());
    }

    #[tokio::test]
    async fn test_mapped_keys_change() {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let mut config = Config::default();
        config.uinput = true;
        config.mappings = [(UiCommand::Enter, Key::Enter), (UiCommand::Back, Key::Exit)].into();
        config.logical_address = LogicalAddressType::Playback;
        let test = setup_dbus_test(cb, Some(config)).await.unwrap();

        test.dev
            .lock()
            .await
            .queue_rx_message(Envelope {
                message: MessageData::Valid(Message::UserControlPressed {
                    ui_command: UiCommand::Enter,
                }),
                initiator: LogicalAddress::Tv,
                destination: LogicalAddress::PlaybackDevice1,
                timestamp: 0,
                sequence: 1,
            })
            .await;
        test.dev
            .lock()
            .await
            .queue_rx_message(Envelope {
                message: MessageData::Valid(Message::UserControlPressed {
                    ui_command: UiCommand::Back,
                }),
                initiator: LogicalAddress::Tv,
                destination: LogicalAddress::PlaybackDevice1,
                timestamp: 0,
                sequence: 1,
            })
            .await;
        test.dev
            .lock()
            .await
            .queue_rx_message(Envelope {
                message: MessageData::Valid(Message::UserControlReleased {}),
                initiator: LogicalAddress::Tv,
                destination: LogicalAddress::PlaybackDevice1,
                timestamp: 0,
                sequence: 1,
            })
            .await;
        let notify = test.dev.lock().await.rx_queue_empty().await.unwrap();
        notify.notified().await;

        let interface: InterfaceRef<CecDevice> = test
            .connection
            .object_server()
            .interface("/com/steampowered/CecDaemon1/Devices/Null")
            .await
            .unwrap();
        let mut dbus_obj = interface.get_mut().await;
        let uinput = dbus_obj.uinput.as_mut().unwrap();
        let event = uinput.get_next_event().unwrap();
        let key = KeyEvent::try_from(event).unwrap();
        assert_eq!(key.key, Key::Enter);
        assert_eq!(key.value, KeyState::PRESSED);

        assert_eq!(
            uinput.get_next_event().unwrap().kind,
            EventKind::Synchronize
        );

        let event = uinput.get_next_event().unwrap();
        let key = KeyEvent::try_from(event).unwrap();
        assert_eq!(key.key, Key::Enter);
        assert_eq!(key.value, KeyState::RELEASED);

        assert_eq!(
            uinput.get_next_event().unwrap().kind,
            EventKind::Synchronize
        );

        let event = uinput.get_next_event().unwrap();
        let key = KeyEvent::try_from(event).unwrap();
        assert_eq!(key.key, Key::Exit);
        assert_eq!(key.value, KeyState::PRESSED);

        assert_eq!(
            uinput.get_next_event().unwrap().kind,
            EventKind::Synchronize
        );

        let event = uinput.get_next_event().unwrap();
        let key = KeyEvent::try_from(event).unwrap();
        assert_eq!(key.key, Key::Exit);
        assert_eq!(key.value, KeyState::RELEASED);

        assert_eq!(
            uinput.get_next_event().unwrap().kind,
            EventKind::Synchronize
        );
    }
}
