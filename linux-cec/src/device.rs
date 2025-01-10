/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use linux_cec_sys::constants::{
    CEC_CONNECTOR_TYPE_DRM, CEC_CONNECTOR_TYPE_NO_CONNECTOR, CEC_EVENT_LOST_MSGS,
    CEC_EVENT_PIN_5V_HIGH, CEC_EVENT_PIN_5V_LOW, CEC_EVENT_PIN_CEC_HIGH, CEC_EVENT_PIN_CEC_LOW,
    CEC_EVENT_PIN_HPD_HIGH, CEC_EVENT_PIN_HPD_LOW, CEC_EVENT_STATE_CHANGE, CEC_MAX_LOG_ADDRS,
};
use linux_cec_sys::ioctls::{
    adapter_get_capabilities, adapter_get_connector_info, adapter_get_logical_addresses,
    adapter_get_physical_address, adapter_set_logical_addresses, adapter_set_physical_address,
    dequeue_event, get_mode, receive_message, set_mode, transmit_message,
};
use linux_cec_sys::structs::{
    cec_caps, cec_connector_info, cec_drm_connector_info, cec_event, cec_log_addrs, cec_msg,
    CEC_RX_STATUS,
};
use linux_cec_sys::{PhysicalAddress as SysPhysicalAddress, Timestamp, VendorId as SysVendorId};
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::poll::{poll, PollFd, PollFlags};
use num_enum::TryFromPrimitive;
use std::fs::{File, OpenOptions};
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::path::Path;
use std::str::FromStr;
#[cfg(feature = "tracing")]
use tracing::{debug, warn};

pub use linux_cec_sys::structs::CEC_CAP as Capabilities;
pub use nix::poll::PollTimeout;

use crate::ioctls::CecMessageHandlingMode;
use crate::message::Message;
use crate::operand::{BufferOperand, UiCommand, VendorId};
use crate::{
    Error, FollowerMode, InitiatorMode, LogicalAddress, LogicalAddressType, PhysicalAddress, Range,
    Result, Timeout,
};

#[cfg(feature = "async")]
pub use crate::async_support::Device as AsyncDevice;

/// An object for interacting with system CEC devices.
#[derive(Debug)]
pub struct Device {
    file: File,
    tx_logical_address: LogicalAddress,
    internal_log_addrs: cec_log_addrs,
}

/// A representation of a received [`Message`] and its associated metadata.
#[derive(Debug, Clone, Hash)]
pub struct Envelope {
    /// The received message data.
    pub message: Message,
    /// The logical address of the CEC device that sent the message.
    pub initiator: LogicalAddress,
    /// The logical address to which this message was sent. Unless the [`Device`]
    /// has the [`FollowerMode`] set to either [`FollowerMode::Monitor`] or
    /// [`FollowerMode::MonitorAll`], this value will either be the logical address
    /// of the `Device` itself or [`LogicalAddress::BROADCAST`].
    pub destination: LogicalAddress,
    /// The time at which this message was received. This may be different
    /// from the time at which the message was read out of the kernel.
    pub timestamp: Timestamp,
}

/// A physical pin that can be monitored via [`PinEvent`]s.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Pin {
    /// The CEC data pin (13)
    Cec,
    /// The hot plug detect (HPD) pin (19)
    HotPlugDetect,
    /// The +5 V power pin (18)
    Power5V,
}

/// The logic level of a physical pin as reported in a [`PinEvent`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PinState {
    Low,
    High,
}

/// An event representing a logic level change of a physical pin.
///
/// To receive `PinEvent`s from a [`Device`], it must be configured with a
/// [`FollowerMode`] of either [`FollowerMode::MonitorPin`] or
/// [`FollowerMode::MonitorAll`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PinEvent {
    /// Which physical pin generated the event.
    pub pin: Pin,
    /// The logic level of the pin.
    pub state: PinState,
}

/// An object used for polling the status of the event and message queues in the
/// kernel without borrowing the [`Device`], created by [`Device::get_poller`].
#[derive(Debug)]
pub struct DevicePoller {
    fd: OwnedFd,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PollStatus {
    Nothing,
    Destroyed,
    GotEvent,
    GotMessage,
    GotAll,
}

/// A return result of [`Device::handle_status`], containing
/// about a single message or event was returned from the kernel.
#[derive(Debug, Clone, Hash)]
pub enum PollResult {
    /// The device received a [`Message`].
    Message(Envelope),
    /// A monitored pin changed state.
    PinEvent(PinEvent),
    /// The message queue was full and a number of messages were
    /// received and lost. To avoid this, make sure to poll as
    /// frequently as possible.
    LostMessages(u32),
    /// The device state changed. Usually this means the device was configured, unconfigured,
    /// or the physical address changed. If the caller has cached any properties like logical
    /// address or physical address, these values should be refreshed.
    StateChange,
}

#[derive(Debug, Clone, Hash)]
pub enum ConnectorInfo {
    None,
    /// Tells which drm connector is associated with the CEC adapter.
    DrmConnector {
        /// drm card number
        card_no: u32,
        /// drm connector ID
        connector_id: u32,
    },
    Unknown {
        ty: u32,
        data: [u32; 16],
    },
}

impl Device {
    /// Open a CEC device from a given `path`. Generally, this will be of the
    /// form `/dev/cecX`.
    pub fn open(path: impl AsRef<Path>) -> Result<Device> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(path)?;
        Device::try_from(file)
    }

    /// Get a new [`DevicePoller`] object that can be used to poll the status
    /// of the kernel queue without having to borrow the `Device` object itself.
    ///
    /// While the poller can return a [`PollStatus`] to say which kinds of
    /// events or messages are available, it must be passed to
    /// [`handle_status`](Device::handle_status), which will require borrowing
    /// the `Device`, to get the actual data.
    pub fn get_poller(&self) -> Result<DevicePoller> {
        Ok(DevicePoller {
            fd: self.file.as_fd().try_clone_to_owned()?,
        })
    }

    pub fn poll(&mut self, timeout: PollTimeout) -> Result<Vec<PollResult>> {
        let poller = self.get_poller()?;
        let status = poller.poll(timeout)?;
        self.handle_status(status)
    }

    pub fn set_blocking(&self, blocking: bool) -> Result<()> {
        let rawfd = self.file.as_raw_fd();
        let mut flags = OFlag::from_bits_retain(fcntl(rawfd, FcntlArg::F_GETFL)?);
        flags.set(OFlag::O_NONBLOCK, !blocking);
        fcntl(rawfd, FcntlArg::F_SETFL(flags))?;
        Ok(())
    }

    pub fn set_initiator_mode(&self, mode: InitiatorMode) -> Result<()> {
        let mode = self.get_mode()?.with_initiator(mode.into());
        self.set_mode(mode)
    }

    pub fn set_follower_mode(&self, mode: FollowerMode) -> Result<()> {
        let mode = self.get_mode()?.with_follower(mode.into());
        self.set_mode(mode)
    }

    pub fn get_raw_capabilities(&self) -> Result<cec_caps> {
        let mut caps = cec_caps::default();
        unsafe {
            adapter_get_capabilities(self.file.as_raw_fd(), &mut caps)?;
        }
        Ok(caps)
    }

    pub fn get_capabilities(&self) -> Result<Capabilities> {
        self.get_raw_capabilities().map(|caps| caps.capabilities)
    }

    pub fn get_physical_address(&self) -> Result<PhysicalAddress> {
        let mut phys_addr: SysPhysicalAddress = 0;
        unsafe {
            adapter_get_physical_address(self.file.as_raw_fd(), &mut phys_addr)?;
        }
        Ok(PhysicalAddress(phys_addr))
    }

    /// Set the [`PhysicalAddress`] of the device. This function will only work if the capability
    /// [`Capabilities::PHYS_ADDR`] is present.
    pub fn set_physical_address(&self, phys_addr: PhysicalAddress) -> Result<()> {
        unsafe {
            adapter_set_physical_address(self.file.as_raw_fd(), &phys_addr.0)?;
        }
        Ok(())
    }

    pub fn get_logical_addresses(&mut self) -> Result<Vec<LogicalAddress>> {
        unsafe {
            adapter_get_logical_addresses(self.file.as_raw_fd(), &mut self.internal_log_addrs)?;
        }

        let mut vec = Vec::new();
        for index in 0..self.internal_log_addrs.num_log_addrs {
            vec.push(self.internal_log_addrs.log_addr[index as usize].try_into()?);
        }
        Ok(vec)
    }

    /// Set the [`LogicalAddress`]es of the device. This function will only work if the capability
    /// [`Capabilities::LOG_ADDRS`] is present.
    pub fn set_logical_addresses(&mut self, log_addrs: &[LogicalAddressType]) -> Result<()> {
        Range::AtMost(CEC_MAX_LOG_ADDRS).check(log_addrs.len(), "logical addresses")?;

        for (index, log_addr) in log_addrs.iter().enumerate() {
            self.internal_log_addrs.log_addr_type[index] = (*log_addr).into();
            if let Some(prim_dev_type) = (*log_addr).primary_device_type() {
                self.internal_log_addrs.primary_device_type[index] = prim_dev_type.into();
            } else {
                self.internal_log_addrs.primary_device_type[index] = 0xFF;
            }
        }

        if !log_addrs.is_empty() && self.internal_log_addrs.num_log_addrs > 0 {
            // Clear old logical addresses first, if present
            self.clear_logical_addresses()?;
        }

        self.internal_log_addrs.num_log_addrs = log_addrs.len().try_into().unwrap();
        unsafe {
            adapter_set_logical_addresses(self.file.as_raw_fd(), &mut self.internal_log_addrs)?;
        }
        if !log_addrs.is_empty() {
            self.tx_logical_address =
                LogicalAddress::try_from_primitive(self.internal_log_addrs.log_addr[0])
                    .unwrap_or(LogicalAddress::UNREGISTERED);
        }
        Ok(())
    }

    /// Set the single [`LogicalAddress`] of the device. This function will only work if the capability
    /// [`Capabilities::LOG_ADDRS`] is present.
    pub fn set_logical_address(&mut self, log_addr: LogicalAddressType) -> Result<()> {
        self.set_logical_addresses(&[log_addr])
    }

    /// Clear the [`LogicalAddress`]es of the device. This function will only work if the capability
    /// [`Capabilities::LOG_ADDRS`] is present.
    pub fn clear_logical_addresses(&mut self) -> Result<()> {
        self.internal_log_addrs.num_log_addrs = 0;
        self.tx_logical_address = LogicalAddress::UNREGISTERED;
        unsafe {
            adapter_set_logical_addresses(self.file.as_raw_fd(), &mut self.internal_log_addrs)?;
        }
        Ok(())
    }

    pub fn get_osd_name(&mut self) -> Result<String> {
        unsafe {
            adapter_get_logical_addresses(self.file.as_raw_fd(), &mut self.internal_log_addrs)?;
        }
        Ok(String::from_utf8_lossy(&self.internal_log_addrs.osd_name).to_string())
    }

    pub fn set_osd_name(&mut self, name: &str) -> Result<()> {
        let name_buffer = BufferOperand::from_str(name)?;
        #[cfg(feature = "tracing")]
        debug!("Setting OSD name to {name}");
        self.internal_log_addrs.osd_name[..14].copy_from_slice(&name_buffer.buffer);
        if self.tx_logical_address != LogicalAddress::UNREGISTERED {
            let message = Message::SetOsdName { name: name_buffer };
            self.tx_message(&message, LogicalAddress::Tv)?;
        }
        Ok(())
    }

    pub fn get_vendor_id(&mut self) -> Result<Option<VendorId>> {
        unsafe {
            adapter_get_logical_addresses(self.file.as_raw_fd(), &mut self.internal_log_addrs)?;
        }
        VendorId::try_from_sys(self.internal_log_addrs.vendor_id)
    }

    pub fn set_vendor_id(&mut self, vendor_id: Option<VendorId>) -> Result<()> {
        if let Some(vendor_id) = vendor_id {
            self.internal_log_addrs.vendor_id = vendor_id.into();
            #[cfg(feature = "tracing")]
            debug!(
                "Setting vendor ID to {:02X}-{:02X}-{:02X}",
                vendor_id.0[0], vendor_id.0[1], vendor_id.0[2]
            );
        } else {
            #[cfg(feature = "tracing")]
            debug!("Clearing vendor ID");
            self.internal_log_addrs.vendor_id = SysVendorId::default();
        }
        Ok(())
    }

    pub fn tx_message(&self, message: &Message, destination: LogicalAddress) -> Result<()> {
        let mut raw_message = cec_msg::new(self.tx_logical_address.into(), destination.into());
        let bytes = message.to_bytes();
        let len = usize::min(bytes.len(), 15) + 1;
        raw_message.len = len.try_into().unwrap();
        raw_message.msg[1..len].copy_from_slice(&bytes[..len - 1]);
        #[cfg(feature = "tracing")]
        debug!(
            "Sending message {message:#?} to {destination} ({:x})",
            destination as u8
        );
        self.tx_raw_message(&mut raw_message)
    }

    pub fn tx_raw_message(&self, message: &mut cec_msg) -> Result<()> {
        unsafe {
            transmit_message(self.file.as_raw_fd(), message)?;
        }
        Ok(())
    }

    pub fn rx_message(&self, timeout: Timeout) -> Result<Envelope> {
        let message = self.rx_raw_message(timeout.as_ms())?;
        if message.rx_status.contains(CEC_RX_STATUS::TIMEOUT) {
            return Err(Error::Timeout);
        }
        if message.len > 15 {
            return Err(Error::InvalidData);
        }
        let bytes = &message.msg[1..message.len as usize];
        let initiator = LogicalAddress::try_from_primitive(message.msg[0] >> 4)?;
        let destination = LogicalAddress::try_from_primitive(message.msg[0] & 0xF)?;
        let timestamp = message.rx_ts;

        #[cfg(feature = "tracing")]
        let message = Message::try_from_bytes(bytes).inspect_err(|e| {
            warn!("Failed to parse incoming message {bytes:?}: {e}");
        })?;
        #[cfg(not(feature = "tracing"))]
        let message = Message::try_from_bytes(bytes)?;

        let envelope = Envelope {
            message,
            initiator,
            destination,
            timestamp,
        };
        #[cfg(feature = "tracing")]
        debug!("Got message {envelope:#?}");
        Ok(envelope)
    }

    pub fn rx_raw_message(&self, timeout_ms: u32) -> Result<cec_msg> {
        let mut message = cec_msg::from_timeout(timeout_ms);
        unsafe {
            receive_message(self.file.as_raw_fd(), &mut message)?;
        }
        Ok(message)
    }

    pub(crate) fn dequeue_event(&self) -> Result<cec_event> {
        let mut event = cec_event::default();
        unsafe {
            dequeue_event(self.file.as_raw_fd(), &mut event)?;
        }
        Ok(event)
    }

    pub(crate) fn get_mode(&self) -> Result<CecMessageHandlingMode> {
        let mut mode = 0u32;
        unsafe {
            get_mode(self.file.as_raw_fd(), &mut mode)?;
        }
        Ok(mode.into())
    }

    pub(crate) fn set_mode(&self, mode: CecMessageHandlingMode) -> Result<()> {
        unsafe {
            set_mode(self.file.as_raw_fd(), &mode.into())?;
        }
        Ok(())
    }

    /// Handle the [`PollStatus`] returned from [`DevicePoller::poll`]. This
    /// will dequeue any indicated messages or events, handle any internal
    /// processing, and return information about the events or [`Envelope`]s
    /// containing [`Message`]s.
    pub fn handle_status(&mut self, status: PollStatus) -> Result<Vec<PollResult>> {
        let mut results = Vec::new();
        if status.got_event() {
            let ev = self.dequeue_event()?;
            match ev.event {
                CEC_EVENT_STATE_CHANGE => {
                    unsafe {
                        adapter_get_logical_addresses(
                            self.file.as_raw_fd(),
                            &mut self.internal_log_addrs,
                        )?;
                    }
                    if self.internal_log_addrs.num_log_addrs > 0 {
                        self.tx_logical_address =
                            LogicalAddress::try_from_primitive(self.internal_log_addrs.log_addr[0])
                                .unwrap_or(LogicalAddress::UNREGISTERED);
                    } else {
                        self.tx_logical_address = LogicalAddress::UNREGISTERED;
                    }
                    results.push(PollResult::StateChange);
                }
                CEC_EVENT_LOST_MSGS => results.push(PollResult::LostMessages(unsafe {
                    ev.data.lost_msgs.lost_msgs
                })),
                CEC_EVENT_PIN_CEC_LOW => results.push(PollResult::PinEvent(PinEvent {
                    pin: Pin::Cec,
                    state: PinState::Low,
                })),
                CEC_EVENT_PIN_CEC_HIGH => results.push(PollResult::PinEvent(PinEvent {
                    pin: Pin::Cec,
                    state: PinState::High,
                })),
                CEC_EVENT_PIN_HPD_LOW => results.push(PollResult::PinEvent(PinEvent {
                    pin: Pin::HotPlugDetect,
                    state: PinState::Low,
                })),
                CEC_EVENT_PIN_HPD_HIGH => results.push(PollResult::PinEvent(PinEvent {
                    pin: Pin::HotPlugDetect,
                    state: PinState::High,
                })),
                CEC_EVENT_PIN_5V_LOW => results.push(PollResult::PinEvent(PinEvent {
                    pin: Pin::Power5V,
                    state: PinState::Low,
                })),
                CEC_EVENT_PIN_5V_HIGH => results.push(PollResult::PinEvent(PinEvent {
                    pin: Pin::Power5V,
                    state: PinState::High,
                })),
                _ => return Err(Error::SystemError),
            }
        }

        if status.got_message() {
            results.push(PollResult::Message(self.rx_message(Timeout::from_ms(1))?));
        }

        Ok(results)
    }

    pub fn get_connector_info(&self) -> Result<ConnectorInfo> {
        let mut conn_info = cec_connector_info::default();
        unsafe {
            adapter_get_connector_info(self.file.as_raw_fd(), &mut conn_info)?;
        }
        match conn_info.ty {
            CEC_CONNECTOR_TYPE_NO_CONNECTOR => Ok(ConnectorInfo::None),
            CEC_CONNECTOR_TYPE_DRM => {
                let cec_drm_connector_info {
                    card_no,
                    connector_id,
                } = unsafe { conn_info.data.drm };
                Ok(ConnectorInfo::DrmConnector {
                    card_no,
                    connector_id,
                })
            }
            ty => Ok(ConnectorInfo::Unknown {
                ty,
                data: unsafe { conn_info.data.raw },
            }),
        }
    }

    pub fn activate_source(&self, text_view: bool) -> Result<()> {
        let address = self.get_physical_address()?;
        let active_source = Message::ActiveSource { address };
        if text_view {
            let text_view_on = Message::TextViewOn {};
            self.tx_message(&text_view_on, LogicalAddress::Tv)?;
        } else {
            let image_view_on = Message::ImageViewOn {};
            self.tx_message(&image_view_on, LogicalAddress::Tv)?;
        }
        self.tx_message(&active_source, LogicalAddress::BROADCAST)
    }

    pub fn standby(&self, target: LogicalAddress) -> Result<()> {
        let standby = Message::Standby {};
        self.tx_message(&standby, target)
    }

    pub fn press_user_control(&self, ui_command: UiCommand, target: LogicalAddress) -> Result<()> {
        let user_control = Message::UserControlPressed { ui_command };
        self.tx_message(&user_control, target)
    }

    pub fn release_user_control(&self, target: LogicalAddress) -> Result<()> {
        let user_control = Message::UserControlReleased {};
        self.tx_message(&user_control, target)
    }
}

impl TryFrom<File> for Device {
    type Error = Error;

    fn try_from(file: File) -> Result<Device> {
        let mut internal_log_addrs = cec_log_addrs::default();
        unsafe {
            adapter_get_logical_addresses(file.as_raw_fd(), &mut internal_log_addrs)?;
        }
        let tx_logical_address = if internal_log_addrs.num_log_addrs > 0 {
            LogicalAddress::try_from_primitive(internal_log_addrs.log_addr[0]).unwrap_or_default()
        } else {
            LogicalAddress::UNREGISTERED
        };

        Ok(Device {
            file,
            tx_logical_address,
            internal_log_addrs,
        })
    }
}

impl DevicePoller {
    /// Poll the kernel queues for the [`Device`]. The returned [`PollStatus`]
    /// must be passed to [`Device::handle_status`] to dequeue the events or
    /// messages that the status may indicate are present.
    pub fn poll(&self, timeout: PollTimeout) -> Result<PollStatus> {
        let mut pollfd = [PollFd::new(
            self.fd.as_fd(),
            PollFlags::POLLPRI | PollFlags::POLLIN,
        )];
        let done = poll(&mut pollfd, timeout)?;

        if done == 0 {
            return Err(Error::Timeout);
        }

        match pollfd[0].revents() {
            None => Ok(PollStatus::Nothing),
            Some(flags) if flags.contains(PollFlags::POLLHUP) => Ok(PollStatus::Destroyed),
            Some(flags) if flags.contains(PollFlags::POLLIN | PollFlags::POLLPRI) => {
                Ok(PollStatus::GotAll)
            }
            Some(flags) if flags.contains(PollFlags::POLLIN) => Ok(PollStatus::GotMessage),
            Some(flags) if flags.contains(PollFlags::POLLPRI) => Ok(PollStatus::GotEvent),
            Some(_) => Err(Error::UnknownError(String::from(
                "Polling error encountered",
            ))),
        }
    }
}

impl PollStatus {
    #[must_use]
    pub fn got_message(&self) -> bool {
        matches!(self, PollStatus::GotMessage | PollStatus::GotAll)
    }

    #[must_use]
    pub fn got_event(&self) -> bool {
        matches!(self, PollStatus::GotEvent | PollStatus::GotAll)
    }
}
