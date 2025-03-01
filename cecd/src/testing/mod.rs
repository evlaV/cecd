/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use linux_cec::device::{
    Capabilities, ConnectorInfo, Envelope, PollResult, PollStatus, PollTimeout,
};
use linux_cec::message::{Message, Opcode};
use linux_cec::operand::{BufferOperand, UiCommand};
use linux_cec::{
    Error, FollowerMode, InitiatorMode, LogicalAddress, LogicalAddressType, PhysicalAddress,
    Result, Timeout, VendorId,
};
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::dispatcher::DefaultGuard;
use tracing::{debug, subscriber};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};
use zbus::connection::Builder;
use zbus::names::OwnedErrorName;
use zbus::{proxy, Connection};

pub(crate) mod dbus;

use crate::config::Config;
use crate::system::{System, SystemHandle};
pub(crate) use crate::testing::dbus::MockDBus;
use crate::ArcDevice;

#[derive(Clone, Debug)]
struct DeviceState {
    pollers: Vec<Sender<PollStatus>>,
    follower: FollowerMode,
    initiator: InitiatorMode,
    phys_addr: PhysicalAddress,
    log_addrs: Vec<LogicalAddress>,
    osd_name: BufferOperand,
    vendor_id: Option<VendorId>,
    tx_queue: VecDeque<(Message, LogicalAddress)>,
    rx_queue: VecDeque<Envelope>,
    sequence: u32,
}

#[derive(Debug)]
pub struct AsyncDevice {
    _path: PathBuf,
    caps: Capabilities,
    token: CancellationToken,
    force_unregistered: bool,
    state: RwLock<DeviceState>,
}

#[derive(Debug)]
pub struct AsyncDevicePoller {
    token: CancellationToken,
    channel: UnsafeCell<Receiver<PollStatus>>,
}

unsafe impl Sync for AsyncDevicePoller {}

impl AsyncDevice {
    pub async fn open(path: impl AsRef<Path>) -> Result<AsyncDevice> {
        Ok(AsyncDevice {
            _path: path.as_ref().to_path_buf(),
            caps: Capabilities::empty(),
            token: CancellationToken::new(),
            force_unregistered: false,
            state: RwLock::new(DeviceState {
                pollers: Vec::new(),
                follower: FollowerMode::Disabled,
                initiator: InitiatorMode::Disabled,
                phys_addr: PhysicalAddress::default(),
                log_addrs: Vec::new(),
                osd_name: BufferOperand::default(),
                vendor_id: None,
                tx_queue: VecDeque::new(),
                rx_queue: VecDeque::new(),
                sequence: 1,
            }),
        })
    }

    pub(crate) fn set_caps(&mut self, caps: Capabilities) {
        self.caps = caps;
    }

    pub(crate) async fn set_phys_addr(&mut self, phys_addr: PhysicalAddress) {
        self.state.write().await.phys_addr = phys_addr;
    }

    pub(crate) async fn queue_rx_message(&self, message: Envelope) {
        let mut state = self.state.write().await;
        state.rx_queue.push_back(message);
        for poller in state.pollers.iter() {
            poller.send(PollStatus::GotMessage).await.unwrap();
        }
    }

    pub(crate) async fn dequeue_tx_message(&self) -> Option<(Message, LogicalAddress)> {
        self.state.write().await.tx_queue.pop_front()
    }

    pub async fn set_blocking(&self, _blocking: bool) -> Result<()> {
        todo!();
    }

    pub async fn get_poller(&self) -> Result<AsyncDevicePoller> {
        let (tx, rx) = channel(1);
        self.state.write().await.pollers.push(tx);
        Ok(AsyncDevicePoller {
            channel: UnsafeCell::new(rx),
            token: self.token.clone(),
        })
    }

    pub async fn poll(&self, _timeout: PollTimeout) -> Result<Vec<PollResult>> {
        todo!();
    }

    pub async fn set_initiator_mode(&self, mode: InitiatorMode) -> Result<()> {
        if !self.caps.contains(Capabilities::TRANSMIT) {
            return Err(Error::InvalidData);
        }
        self.state.write().await.initiator = mode;
        Ok(())
    }

    pub async fn set_follower_mode(&self, mode: FollowerMode) -> Result<()> {
        self.state.write().await.follower = mode;
        Ok(())
    }

    pub async fn get_capabilities(&self) -> Result<Capabilities> {
        Ok(self.caps.clone())
    }

    pub async fn get_physical_address(&self) -> Result<PhysicalAddress> {
        Ok(self.state.read().await.phys_addr)
    }

    pub async fn set_physical_address(&self, phys_addr: PhysicalAddress) -> Result<()> {
        if !self.caps.contains(Capabilities::PHYS_ADDR) {
            return Err(Error::InvalidData);
        }
        self.state.write().await.phys_addr = phys_addr;
        Ok(())
    }

    pub async fn get_logical_addresses(&self) -> Result<Vec<LogicalAddress>> {
        Ok(self.state.read().await.log_addrs.clone())
    }

    pub async fn set_logical_addresses(&self, log_addrs: &[LogicalAddressType]) -> Result<()> {
        if !self.caps.contains(Capabilities::LOG_ADDRS) {
            return Err(Error::InvalidData);
        }
        {
            let state = self.state.read().await;
            if state.initiator == InitiatorMode::Disabled {
                return Err(Error::InvalidData);
            }
            if !state.log_addrs.is_empty() {
                return Err(Error::InvalidData);
            }
            if !state.phys_addr.is_valid() {
                return Err(Error::InvalidData);
            }
        }
        let mut state = self.state.write().await;
        if self.force_unregistered {
            state.log_addrs = vec![LogicalAddress::Unregistered];
        } else {
            state.log_addrs = log_addrs
                .into_iter()
                .map(|ty| match *ty {
                    LogicalAddressType::Tv => LogicalAddress::Tv,
                    LogicalAddressType::Record => LogicalAddress::RecordingDevice1,
                    LogicalAddressType::Tuner => LogicalAddress::Tuner1,
                    LogicalAddressType::Playback => LogicalAddress::PlaybackDevice1,
                    LogicalAddressType::AudioSystem => LogicalAddress::AudioSystem,
                    LogicalAddressType::Specific => LogicalAddress::Specific,
                    LogicalAddressType::Unregistered => LogicalAddress::Unregistered,
                })
                .collect();
        }
        for poller in state.pollers.iter() {
            poller.send(PollStatus::GotEvent).await.unwrap();
        }
        Ok(())
    }

    pub async fn set_logical_address(&self, log_addr: LogicalAddressType) -> Result<()> {
        self.set_logical_addresses(&[log_addr]).await
    }

    pub async fn clear_logical_addresses(&self) -> Result<()> {
        if !self.caps.contains(Capabilities::LOG_ADDRS) {
            return Err(Error::InvalidData);
        }
        self.state.write().await.log_addrs.clear();
        Ok(())
    }

    pub async fn get_osd_name(&self) -> Result<String> {
        Ok(String::from_utf8_lossy(self.state.read().await.osd_name.as_bytes()).to_string())
    }

    pub async fn set_osd_name(&self, name: &str) -> Result<()> {
        self.state.write().await.osd_name = BufferOperand::from_str(name)?;
        Ok(())
    }

    pub async fn get_vendor_id(&self) -> Result<Option<VendorId>> {
        Ok(self.state.read().await.vendor_id)
    }

    pub async fn set_vendor_id(&self, vendor_id: Option<VendorId>) -> Result<()> {
        self.state.write().await.vendor_id = vendor_id;
        Ok(())
    }

    pub async fn tx_message(&self, message: &Message, destination: LogicalAddress) -> Result<u32> {
        let mut state = self.state.write().await;
        if state.log_addrs.is_empty() {
            return Err(Error::NoLogicalAddress);
        }
        if !state
            .log_addrs
            .iter()
            .any(|addr| *addr != LogicalAddress::Unregistered)
        {
            return Err(Error::InvalidData);
        }
        if state.initiator == InitiatorMode::Disabled {
            return Err(Error::InvalidData);
        }
        state.tx_queue.push_front((message.clone(), destination));
        let seq = state.sequence;
        state.sequence += 1;
        Ok(seq)
    }

    pub async fn tx_rx_message(
        &self,
        _message: &Message,
        _destination: LogicalAddress,
        _opcode: Opcode,
        _timeout: Timeout,
    ) -> Result<Envelope> {
        todo!();
    }

    pub async fn rx_message(&self, _timeout: Timeout) -> Result<Envelope> {
        let mut state = self.state.write().await;
        if state.log_addrs.is_empty() {
            return Err(Error::NoLogicalAddress);
        }
        if state.follower == FollowerMode::Disabled {
            return Err(Error::InvalidData);
        }
        if let Some(message) = state.rx_queue.pop_front() {
            Ok(message)
        } else {
            Err(Error::Timeout)
        }
    }

    pub async fn handle_status(&self, status: PollStatus) -> Result<Vec<PollResult>> {
        let mut results = Vec::new();
        if status.got_event() {
            results.push(PollResult::StateChange);
        }

        if status.got_message() {
            if let Some(message) = self.state.write().await.rx_queue.pop_front() {
                results.push(PollResult::Message(message));
            }
        }
        Ok(results)
    }

    pub async fn get_connector_info(&self) -> Result<ConnectorInfo> {
        todo!();
    }

    pub async fn set_active_source(&self, _address: Option<PhysicalAddress>) -> Result<()> {
        todo!();
    }

    pub async fn wake(&self, _set_active: bool, _text_view: bool) -> Result<()> {
        todo!();
    }

    pub async fn standby(&self, target: LogicalAddress) -> Result<()> {
        let standby = Message::Standby {};
        self.tx_message(&standby, target).await?;
        Ok(())
    }

    pub async fn press_user_control(
        &self,
        _ui_command: UiCommand,
        _target: LogicalAddress,
    ) -> Result<()> {
        todo!();
    }

    pub async fn release_user_control(&self, _target: LogicalAddress) -> Result<()> {
        todo!();
    }

    pub async fn close(self) -> Result<()> {
        self.token.cancel();
        Ok(())
    }
}

impl AsyncDevicePoller {
    pub async fn poll(&self, _timeout: PollTimeout) -> Result<PollStatus> {
        loop {
            let channel = unsafe { &mut *self.channel.get() };
            select! {
                _ = self.token.cancelled() => return Ok(PollStatus::Destroyed),
                status = channel.recv() => if let Some(status) = status {
                    debug!("Got message {status:?}");
                    return Ok(status);
                } else {
                    return Ok(PollStatus::Destroyed);
                },
            };
        }
    }
}

#[proxy(
    interface = "com.steampowered.CecDaemon1.CecDevice1",
    default_service = "com.steampowered.CecDaemon1"
)]
pub(crate) trait CecDevice {
    #[zbus(signal)]
    fn received_message(
        initiator: u8,
        destination: u8,
        timestamp: u64,
        message: &[u8],
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    fn user_control_pressed(button: &[u8], initiator: u8) -> zbus::Result<()>;

    #[zbus(signal)]
    fn user_control_released(initiator: u8) -> zbus::Result<()>;

    #[zbus(property)]
    fn logical_addresses(&self) -> zbus::Result<Vec<u8>>;

    #[zbus(property)]
    fn physical_address(&self) -> zbus::Result<u16>;

    #[zbus(property)]
    fn vendor_id(&self) -> zbus::Result<i32>;

    fn set_osd_name(&self, name: &str) -> zbus::Result<()>;

    fn set_active_source(&self, phys_addr: i32) -> zbus::Result<()>;

    fn wake(&self) -> zbus::Result<()>;

    fn standby(&self, target: u8) -> zbus::Result<()>;

    fn press_user_control(&mut self, button: &[u8], target: u8) -> zbus::Result<()>;

    fn release_user_control(&mut self, target: u8) -> zbus::Result<()>;

    fn press_once_user_control(&mut self, button: &[u8], target: u8) -> zbus::Result<()>;

    fn volume_up(&self, target: u8) -> zbus::Result<()>;

    fn volume_down(&self, target: u8) -> zbus::Result<()>;

    fn mute(&self, target: u8) -> zbus::Result<()>;

    fn send_raw_message(&self, raw_message: &[u8], target: u8) -> zbus::Result<u32>;

    fn send_receive_raw_message(
        &self,
        raw_message: &[u8],
        target: u8,
        opcode: u8,
        timeout: u16,
    ) -> zbus::Result<Vec<u8>>;
}

pub(crate) struct DBusTest<'a> {
    pub dev: ArcDevice,
    pub proxy: CecDeviceProxy<'a>,
    pub connection: Connection,
    _guard: DefaultGuard,
}

pub(crate) async fn setup_dbus_test<F, Fut>(
    setup_dev: F,
    config: Option<Config>,
) -> anyhow::Result<DBusTest<'static>>
where
    F: FnOnce(ArcDevice) -> Fut,
    Fut: Future<Output = anyhow::Result<()>>,
{
    let guard = subscriber::set_default(
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(EnvFilter::from_default_env()),
    );

    debug!("Setting up DBus test");
    let dbus = MockDBus::new().await?;
    let builder = Builder::address(dbus.address())?;
    let connection = Builder::address(dbus.address())?.build().await?;
    debug!("Got DBus connection");

    let token = CancellationToken::new();
    let system = SystemHandle(Arc::new(Mutex::new(
        System::new(token.clone(), builder, connection.clone()).await?,
    )));
    let config = config.unwrap_or_else(|| {
        let mut config = Config::default();
        config.disable_uinput = true;
        config
    });
    system.set_config(config).await?;
    debug!("System created");

    let dev;
    let rx;
    let tx;
    let connection;
    {
        let mut system = system.lock().await;
        (dev, tx, rx) = system.find_dev("/dev/null").await?;
        connection = system.connection.clone();
    }
    debug!("Device created");
    let arc_dev = dev.device.clone();
    setup_dev(arc_dev.clone()).await?;
    dev.register(connection.clone(), system.clone(), tx, rx)
        .await?;
    debug!("Device registered");

    Ok(DBusTest {
        dev: arc_dev,
        proxy: CecDeviceProxy::new(&connection, "/com/steampowered/CecDaemon1/Null").await?,
        connection,
        _guard: guard,
    })
}

#[tokio::test]
async fn test_no_caps() {
    async fn cb(_dev: ArcDevice) -> anyhow::Result<()> {
        Ok(())
    }
    assert!(setup_dbus_test(cb, None).await.is_err());
}

#[tokio::test]
async fn test_caps_transmit_only() {
    async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
        let mut dev = dev.lock().await;
        dev.set_caps(Capabilities::TRANSMIT);
        Ok(())
    }
    let test = setup_dbus_test(cb, None).await.unwrap();
    assert_eq!(test.proxy.physical_address().await.unwrap(), 0xFFFF);
    assert!(test.proxy.logical_addresses().await.unwrap().is_empty());
}

#[tokio::test]
async fn test_caps_log_addr() {
    async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
        let mut dev = dev.lock().await;
        dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
        dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
        Ok(())
    }
    let test = setup_dbus_test(cb, None).await.unwrap();
    assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
    assert_eq!(
        test.proxy.logical_addresses().await.unwrap(),
        &[u8::from(LogicalAddress::PlaybackDevice1)]
    );
}

#[tokio::test]
async fn test_tx_no_log_addr() {
    async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
        let mut dev = dev.lock().await;
        dev.set_caps(Capabilities::TRANSMIT);
        Ok(())
    }
    let test = setup_dbus_test(cb, None).await.unwrap();
    assert!(test.proxy.logical_addresses().await.unwrap().is_empty());
    let err = test.proxy.standby(0).await.unwrap_err();
    let zbus::Error::MethodError(name, text, _) = err else {
        panic!();
    };
    assert_eq!(
        OwnedErrorName::try_from("org.freedesktop.DBus.Error.Failed").unwrap(),
        name
    );
    assert_eq!(Some(Error::NoLogicalAddress.to_string()), text);
}

#[tokio::test]
async fn test_tx_invalid_log_addr() {
    async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
        let mut dev = dev.lock().await;
        dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
        dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
        dev.force_unregistered = true;
        Ok(())
    }
    let test = setup_dbus_test(cb, None).await.unwrap();
    assert_eq!(test.proxy.physical_address().await.unwrap(), 0x1000);
    assert_eq!(
        test.proxy.logical_addresses().await.unwrap(),
        &[u8::from(LogicalAddress::Unregistered)]
    );

    let err = test
        .proxy
        .standby(LogicalAddress::Tv.into())
        .await
        .unwrap_err();
    let zbus::Error::MethodError(name, text, _) = err else {
        panic!();
    };
    assert_eq!(
        OwnedErrorName::try_from("org.freedesktop.DBus.Error.Failed").unwrap(),
        name
    );
    assert_eq!(Some(Error::InvalidData.to_string()), text);
}
