use std::ffi::c_char;

use crate::constants::{
    CEC_LOG_ADDR_BACKUP_1, CEC_LOG_ADDR_SPECIFIC, CEC_LOG_ADDR_UNREGISTERED, CEC_MAX_LOG_ADDRS,
    CEC_OP_PRIM_DEVTYPE_PROCESSOR, CEC_OP_PRIM_DEVTYPE_SWITCH, CEC_OP_PRIM_DEVTYPE_TV,
};
use crate::{LogicalAddress, LogicalAddressMask, LogicalAddressesFlags};

/// CEC logical addresses structure
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct CecLogicalAddresses {
    /// The claimed logical addresses. Set by the driver.
    log_addr: [LogicalAddress; CEC_MAX_LOG_ADDRS],
    /// Current logical address mask. Set by the driver.
    log_addr_mask: LogicalAddressMask,
    /// The CEC version that the adapter should implement. Set by the caller.
    cec_version: u8,
    /// How many logical addresses should be claimed. Set by the caller.
    num_log_addrs: u8,
    /// The vendor ID of the device. Set by the caller.
    vendor_id: u32,
    /// Flags.
    flags: LogicalAddressesFlags,
    /// The OSD name of the device. Set by the caller.
    osd_name: [c_char; 15],
    /// The primary device type for each logical address. Set by the caller.
    primary_device_type: [u8; CEC_MAX_LOG_ADDRS],
    /// The logical address types. Set by the caller.
    log_addr_type: [u8; CEC_MAX_LOG_ADDRS],

    /* CEC 2.0 */
    /// CEC 2.0: all device types represented by the logical address. Set by the caller.
    all_device_types: [u8; CEC_MAX_LOG_ADDRS],
    /// CEC 2.0: The logical address features. Set by the caller.
    features: [[u8; 12]; CEC_MAX_LOG_ADDRS],
}

impl CecLogicalAddresses {
    /* Helper functions to identify the 'special' CEC devices */

    fn is_2nd_tv(&self) -> bool {
        /*
         * It is a second TV if the logical address is 14 or 15 and the
         * primary device type is a TV.
         */
        self.num_log_addrs != 0
            && self.log_addr[0] >= CEC_LOG_ADDR_SPECIFIC
            && self.primary_device_type[0] == CEC_OP_PRIM_DEVTYPE_TV
    }

    fn is_processor(&self) -> bool {
        /*
         * It is a processor if the logical address is 12-15 and the
         * primary device type is a Processor.
         */
        self.num_log_addrs != 0
            && self.log_addr[0] >= CEC_LOG_ADDR_BACKUP_1
            && self.primary_device_type[0] == CEC_OP_PRIM_DEVTYPE_PROCESSOR
    }

    fn is_switch(&self) -> bool {
        /*
         * It is a switch if the logical address is 15 and the
         * primary device type is a Switch and the CDC-Only flag is not set.
         */
        self.num_log_addrs == 1
            && self.log_addr[0] == CEC_LOG_ADDR_UNREGISTERED
            && self.primary_device_type[0] == CEC_OP_PRIM_DEVTYPE_SWITCH
            && !self.flags.contains(LogicalAddressesFlags::CDC_ONLY)
    }

    fn is_cdc_only(&self) -> bool {
        /*
         * It is a CDC-only device if the logical address is 15 and the
         * primary device type is a Switch and the CDC-Only flag is set.
         */
        self.num_log_addrs == 1
            && self.log_addr[0] == CEC_LOG_ADDR_UNREGISTERED
            && self.primary_device_type[0] == CEC_OP_PRIM_DEVTYPE_SWITCH
            && self.flags.contains(LogicalAddressesFlags::CDC_ONLY)
    }
}
