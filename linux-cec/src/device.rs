use num_enum::TryFromPrimitive;
use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::Path;

use crate::constants::{
    CEC_CONNECTOR_TYPE_DRM, CEC_CONNECTOR_TYPE_NO_CONNECTOR, CEC_MAX_LOG_ADDRS,
};
use crate::ioctls::{
    adapter_get_capabilities, adapter_get_connector_info, adapter_get_logical_addresses,
    adapter_get_physical_address, adapter_set_logical_addresses, adapter_set_physical_address,
    dequeue_event, get_mode, receive_message, set_mode, transmit_message, CecCapabilities,
    CecConnectorInfo, CecDrmConnectorInfo, CecEvent, CecLogicalAddresses, CecMessage,
    CecMessageHandlingMode,
};
use crate::message::Message;
use crate::{Error, FollowerMode, InitiatorMode, LogicalAddress, PhysicalAddress, Range, Result};

#[cfg(feature = "async")]
pub use crate::async_support::Device as AsyncDevice;

pub struct Device {
    file: File,
    tx_logical_address: LogicalAddress,
    internal_log_addrs: CecLogicalAddresses,
}

#[derive(Debug)]
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
    pub fn open(path: impl AsRef<Path>) -> Result<Device> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(path)?;
        Device::try_from(file)
    }

    pub fn set_initiator(&self, mode: InitiatorMode) -> Result<()> {
        let mode = self.get_mode()?.with_initiator(mode.into());
        self.set_mode(mode)
    }

    pub fn set_follower(&self, mode: FollowerMode) -> Result<()> {
        let mode = self.get_mode()?.with_follower(mode.into());
        self.set_mode(mode)
    }

    pub(crate) fn get_capabilities(&self) -> Result<CecCapabilities> {
        let mut caps = CecCapabilities::default();
        unsafe {
            adapter_get_capabilities(self.file.as_raw_fd(), &mut caps)?;
        }
        Ok(caps)
    }

    pub fn get_physical_address(&self) -> Result<PhysicalAddress> {
        let mut phys_addr: PhysicalAddress = 0;
        unsafe {
            adapter_get_physical_address(self.file.as_raw_fd(), &mut phys_addr)?;
        }
        Ok(phys_addr)
    }

    pub fn set_physical_address(&self, phys_addr: PhysicalAddress) -> Result<()> {
        unsafe {
            adapter_set_physical_address(self.file.as_raw_fd(), &phys_addr)?;
        }
        Ok(())
    }

    pub fn get_logical_addresses(&self) -> Result<Vec<LogicalAddress>> {
        let mut log_addrs = CecLogicalAddresses::default();
        unsafe {
            adapter_get_logical_addresses(self.file.as_raw_fd(), &mut log_addrs)?;
        }

        let mut vec = Vec::new();
        for index in 0..log_addrs.num_log_addrs {
            vec.push(log_addrs.log_addr[index as usize].try_into()?);
        }
        Ok(vec)
    }

    pub fn set_logical_addresses(&mut self, log_addrs: &[LogicalAddress]) -> Result<()> {
        Range::AtMost(CEC_MAX_LOG_ADDRS).check(log_addrs.len(), "logical addresses")?;

        for (index, log_addr) in log_addrs.iter().enumerate() {
            self.internal_log_addrs.log_addr[index] = (*log_addr).into();
        }
        self.internal_log_addrs.num_log_addrs = log_addrs.len().try_into().unwrap();
        unsafe {
            adapter_set_logical_addresses(self.file.as_raw_fd(), &mut self.internal_log_addrs)?;
        }
        Ok(())
    }

    pub fn set_logical_address(&mut self, log_addr: LogicalAddress) -> Result<()> {
        self.set_logical_addresses(&[log_addr])
    }

    pub fn tx_message(&self, message: &Message, destination: LogicalAddress) -> Result<()> {
        let mut raw_message =
            CecMessage::new(self.tx_logical_address, destination).with_message(message);
        self.tx_raw_message(&mut raw_message)
    }

    pub(crate) fn tx_raw_message(&self, message: &mut CecMessage) -> Result<()> {
        unsafe {
            transmit_message(self.file.as_raw_fd(), message)?;
        }
        Ok(())
    }

    pub(crate) fn rx_raw_message(&self, timeout_ms: u32) -> Result<CecMessage> {
        let mut message = CecMessage::from_timeout(timeout_ms);
        unsafe {
            receive_message(self.file.as_raw_fd(), &mut message)?;
        }
        Ok(message)
    }

    pub(crate) fn dequeue_event(&self) -> Result<CecEvent> {
        let mut event = CecEvent::default();
        unsafe {
            dequeue_event(self.file.as_raw_fd(), &mut event)?;
        }
        Ok(event)
    }

    pub(crate) fn get_mode(&self) -> Result<CecMessageHandlingMode> {
        let mut mode = CecMessageHandlingMode::default();
        unsafe {
            get_mode(self.file.as_raw_fd(), &mut mode)?;
        }
        Ok(mode)
    }

    pub(crate) fn set_mode(&self, mode: CecMessageHandlingMode) -> Result<()> {
        unsafe {
            set_mode(self.file.as_raw_fd(), &mode)?;
        }
        Ok(())
    }

    pub fn get_connector_info(&self) -> Result<ConnectorInfo> {
        let mut conn_info = CecConnectorInfo::default();
        unsafe {
            adapter_get_connector_info(self.file.as_raw_fd(), &mut conn_info)?;
        }
        match conn_info.ty {
            CEC_CONNECTOR_TYPE_NO_CONNECTOR => Ok(ConnectorInfo::None),
            CEC_CONNECTOR_TYPE_DRM => {
                let CecDrmConnectorInfo {
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
}

impl TryFrom<File> for Device {
    type Error = Error;

    fn try_from(file: File) -> Result<Device> {
        let mut internal_log_addrs = CecLogicalAddresses::default();
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
