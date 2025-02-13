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

#[tokio::test]
async fn test() {
    tracing_subscriber::fmt::init();
    let dbus = MockDBus::new().await.unwrap();
    let builder = Builder::address(dbus.address()).unwrap();
    let connection = Builder::address(dbus.address()).unwrap().build().await.unwrap();

    let token = CancellationToken::new();
    let system = SystemHandle(Arc::new(Mutex::new(
        System::new(token.clone(), builder, connection).await.unwrap(),
    )));

    let _dev = system.find_dev("/dev/null").await.unwrap();
}
