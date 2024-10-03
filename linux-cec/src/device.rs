use nix::errno::Errno;
use std::fs::File;
use std::os::fd::AsRawFd;

use crate::ioctls::{receive_message, transmit_message, CecMessage};

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
}
