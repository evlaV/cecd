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
    FollowerMode, InitiatorMode, LogicalAddress, LogicalAddressType, PhysicalAddress, Result,
    Timeout, VendorId,
};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct ArcDevice;

#[derive(Debug)]
pub struct Device;

#[derive(Debug)]
pub struct DevicePoller;

impl ArcDevice {
    pub async fn open(_path: impl AsRef<Path>) -> anyhow::Result<ArcDevice> {
        todo!();
    }

    pub async fn lock(&self) -> Device {
        Device {}
    }
}

impl Device {
    pub async fn set_blocking(&self, _blocking: bool) -> Result<()> {
        todo!();
    }

    pub async fn get_poller(&self) -> Result<DevicePoller> {
        todo!();
    }

    pub async fn poll(&self, _timeout: PollTimeout) -> Result<Vec<PollResult>> {
        todo!();
    }

    pub async fn set_initiator_mode(&self, _mode: InitiatorMode) -> Result<()> {
        todo!();
    }

    pub async fn set_follower_mode(&self, _mode: FollowerMode) -> Result<()> {
        todo!();
    }

    pub async fn get_capabilities(&self) -> Result<Capabilities> {
        todo!();
    }

    pub async fn get_physical_address(&self) -> Result<PhysicalAddress> {
        todo!();
    }

    pub async fn set_physical_address(&self, _phys_addr: PhysicalAddress) -> Result<()> {
        todo!();
    }

    pub async fn get_logical_addresses(&self) -> Result<Vec<LogicalAddress>> {
        todo!();
    }

    pub async fn set_logical_addresses(&self, _log_addrs: &[LogicalAddressType]) -> Result<()> {
        todo!();
    }

    pub async fn set_logical_address(&self, _log_addr: LogicalAddressType) -> Result<()> {
        todo!();
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
