use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::Path;

use crate::constants::{CEC_CONNECTOR_TYPE_DRM, CEC_CONNECTOR_TYPE_NO_CONNECTOR};
use crate::ioctls::{
    adapter_get_capabilities, adapter_get_connector_info, adapter_get_logical_addresses,
    adapter_get_physical_address, adapter_set_physical_address, dequeue_event, get_mode,
    receive_message, set_mode, transmit_message, CecCapabilities, CecConnectorInfo,
    CecDrmConnectorInfo, CecEvent, CecLogicalAddresses, CecMessage, CecMessageHandlingMode,
};
use crate::{LogicalAddress, PhysicalAddress, Result};

pub struct Device {
    file: File,
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
        Ok(Device {
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .create(false)
                .open(path)?,
        })
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
        Ok((0..log_addrs.num_log_addrs)
            .map(|index| log_addrs.log_addr[index as usize])
            .collect())
    }

    pub(crate) fn tx_raw_message(&self, message: &mut CecMessage) -> Result<()> {
        unsafe {
            transmit_message(self.file.as_raw_fd(), message)?;
        }
        Ok(())
    }

    pub(crate) fn rx_raw_message(&self, timeout_ms: u32) -> Result<CecMessage> {
        let mut message = CecMessage::with_timeout(timeout_ms);
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
