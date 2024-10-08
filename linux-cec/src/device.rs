use nix::errno::Errno;
use std::fs::File;
use std::os::fd::AsRawFd;

use crate::ioctls::{
    adapter_get_logical_addresses, adapter_get_physical_address, receive_message, transmit_message,
    CecLogicalAddresses, CecMessage,
};
use crate::{LogicalAddress, PhysicalAddress};

pub struct Device {
    file: File,
}

impl Device {
    pub fn tx_raw_message(&self, message: &mut CecMessage) -> Result<(), Errno> {
        unsafe {
            transmit_message(self.file.as_raw_fd(), message)?;
        }
        Ok(())
    }

    pub fn rx_raw_message(&self, timeout_ms: u32) -> Result<CecMessage, Errno> {
        let mut message = CecMessage::with_timeout(timeout_ms);
        unsafe {
            receive_message(self.file.as_raw_fd(), &mut message)?;
        }
        Ok(message)
    }

    pub fn get_logical_addresses(&self) -> Result<Vec<LogicalAddress>, Errno> {
        let mut log_addrs = CecLogicalAddresses::default();
        unsafe {
            adapter_get_logical_addresses(self.file.as_raw_fd(), &mut log_addrs)?;
        }
        Ok((0..log_addrs.num_log_addrs)
            .map(|index| log_addrs.log_addr[index as usize])
            .collect())
    }

    pub fn get_physical_address(&self) -> Result<PhysicalAddress, Errno> {
        let mut phys_addr: PhysicalAddress = 0;
        unsafe {
            adapter_get_physical_address(self.file.as_raw_fd(), &mut phys_addr)?;
        }
        Ok(phys_addr)
    }
}
