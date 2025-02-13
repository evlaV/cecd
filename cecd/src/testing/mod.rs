/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use linux_cec::device::{
    Capabilities, ConnectorInfo, Envelope, PollResult, PollStatus, PollTimeout,
};
use linux_cec::message::{Message, Opcode};
use linux_cec::operand::UiCommand;
use linux_cec::{
    Error, FollowerMode, InitiatorMode, LogicalAddress, LogicalAddressType, PhysicalAddress,
    Result, Timeout, VendorId,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use zbus::connection::Builder;
use zbus::proxy;

mod dbus;

use crate::system::{System, SystemHandle};
pub(crate) use crate::testing::dbus::MockDBus;

#[derive(Clone, Debug)]
struct DeviceState {
    follower: FollowerMode,
    initiator: InitiatorMode,
    phys_addr: PhysicalAddress,
    log_addrs: Vec<LogicalAddress>,
}

#[derive(Debug)]
pub struct AsyncDevice {
    _path: PathBuf,
    caps: Capabilities,
    state: RwLock<DeviceState>,
}

#[derive(Debug)]
pub struct DevicePoller;

impl AsyncDevice {
    pub async fn open(path: impl AsRef<Path>) -> Result<AsyncDevice> {
        Ok(AsyncDevice {
            _path: path.as_ref().to_path_buf(),
            caps: Capabilities::empty(),
            state: RwLock::new(DeviceState {
                follower: FollowerMode::Disabled,
                initiator: InitiatorMode::Disabled,
                phys_addr: PhysicalAddress::default(),
                log_addrs: Vec::new(),
            }),
        })
    }

    pub(crate) fn _set_caps(&mut self, caps: Capabilities) {
        self.caps = caps;
    }

    pub(crate) async fn _set_phys_addr(&mut self, phys_addr: PhysicalAddress) {
        self.state.write().await.phys_addr = phys_addr;
    }

    pub async fn set_blocking(&self, _blocking: bool) -> Result<()> {
        todo!();
    }

    pub async fn get_poller(&self) -> Result<DevicePoller> {
        todo!();
    }

    pub async fn poll(&self, _timeout: PollTimeout) -> Result<Vec<PollResult>> {
        todo!();
    }

    pub async fn set_initiator_mode(&self, mode: InitiatorMode) -> Result<()> {
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

    pub async fn set_logical_addresses(&self, _log_addrs: &[LogicalAddressType]) -> Result<()> {
        todo!();
    }

    pub async fn set_logical_address(&self, log_addr: LogicalAddressType) -> Result<()> {
        self.set_logical_addresses(&[log_addr]).await
    }

    pub async fn clear_logical_addresses(&self) -> Result<()> {
        todo!();
    }

    pub async fn get_osd_name(&self) -> Result<String> {
        todo!();
    }

    pub async fn set_osd_name(&self, _name: &str) -> Result<()> {
        todo!();
    }

    pub async fn get_vendor_id(&self) -> Result<Option<VendorId>> {
        todo!();
    }

    pub async fn set_vendor_id(&self, _vendor_id: Option<VendorId>) -> Result<()> {
        todo!();
    }

    pub async fn tx_message(
        &self,
        _message: &Message,
        _destination: LogicalAddress,
    ) -> Result<u32> {
        todo!();
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
        todo!();
    }

    pub async fn handle_status(&self, _status: PollStatus) -> Result<Vec<PollResult>> {
        todo!();
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

    pub async fn standby(&self, _target: LogicalAddress) -> Result<()> {
        todo!();
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
        todo!();
    }
}

impl DevicePoller {
    pub async fn poll(&self, _timeout: PollTimeout) -> Result<PollStatus> {
        todo!();
    }
}

#[proxy(
    interface = "com.steampowered.CecDaemon1.CecDevice1",
    default_service = "com.steampowered.CecDaemon1"
)]
trait CecDevice {
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
    fn set_physical_address(&self, address: u16) -> zbus::Result<()>;

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

#[tokio::test]
async fn test_no_caps() {
    tracing_subscriber::fmt::init();
    let dbus = MockDBus::new().await.unwrap();
    let builder = Builder::address(dbus.address())
        .unwrap()
        .name("com.steampowered.CecDaemon1")
        .unwrap();
    let connection = Builder::address(dbus.address())
        .unwrap()
        .build()
        .await
        .unwrap();

    let token = CancellationToken::new();
    let system = SystemHandle(Arc::new(Mutex::new(
        System::new(token.clone(), builder, connection.clone())
            .await
            .unwrap(),
    )));

    let _dev = system.find_dev("/dev/null").await.unwrap();
    let proxy = CecDeviceProxy::new(&connection, "/com/steampowered/CecDaemon1/Null")
        .await
        .unwrap();
    assert_eq!(proxy.physical_address().await.unwrap(), 0xFFFF);
    assert!(proxy.set_physical_address(0xF000).await.is_err());
}
