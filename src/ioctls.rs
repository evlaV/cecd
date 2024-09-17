use nix::{ioctl_read, ioctl_readwrite, ioctl_write_ptr};
use std::ffi::c_char;

use crate::log_addrs::CecLogicalAddresses;
use crate::message::CecMessage;
use crate::{Capabilities, EventFlags, LogicalAddressMask, Timestamp};

/// CEC capabilities structure.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct CecCapabilities {
    /// Name of the CEC device driver.
    driver: [c_char; 32],
    /// Name of the CEC device. @driver + @name must be unique.
    name: [c_char; 32],
    /// Number of available logical addresses.
    available_log_addrs: u32,
    /// Capabilities of the CEC adapter.
    capabilities: Capabilities,
    /// version of the CEC adapter framework.
    version: u32,
}

/// Tells which drm connector is
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct CecDrmConnectorInfo {
    /// drm card number
    card_no: u32,
    /// drm connector ID
    connector_id: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
union CecConnectorInfoUnion {
    /// drm connector info
    drm: CecDrmConnectorInfo,
    /// Array to pad the union
    raw: [u32; 16],
}

/// Tells if and which connector is associated with the CEC adapter.
#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct CecConnectorInfo {
    /// Connector type (if any)
    ty: u32,
    data: CecConnectorInfoUnion,
}

/// Used when the CEC adapter changes state.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct CecEventStateChange {
    /// The current physical address
    phys_addr: u16,
    /// The current logical address mask
    log_addr_mask: LogicalAddressMask,
    /** If non-zero, then HDMI connector information is available.
     *	This field is only valid if CEC_CAP_CONNECTOR_INFO is set. If that
     *	capability is set and @have_conn_info is zero, then that indicates
     *	that the HDMI connector device is not instantiated, either because
     *	the HDMI driver is still configuring the device or because the HDMI
     *	device was unbound.
     */
    have_conn_info: u16,
}

/// Tells you how many messages were lost.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct CecEventLostMsgs {
    /// How many messages were lost.
    lost_msgs: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
union CecEventUnion {
    /// The event payload for CEC_EVENT_STATE_CHANGE.
    state_change: CecEventStateChange,
    /// The event payload for CEC_EVENT_LOST_MSGS.
    lost_msgs: CecEventLostMsgs,
    /// Array to pad the union.
    raw: [u32; 16],
}

/// CEC event structure
#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct CecEvent {
    /// The timestamp of when the event was sent.
    ts: Timestamp,
    /// The event.
    event: u32,
    /// Event flags.
    flags: EventFlags,
    data: CecEventUnion,
}

/* Adapter capabilities */
ioctl_readwrite!(adapter_get_capabilities, b'a', 0, CecCapabilities);

/*
 * phys_addr is either 0 (if this is the CEC root device)
 * or a valid physical address obtained from the sink's EDID
 * as read by this CEC device (if this is a source device)
 * or a physical address obtained and modified from a sink
 * EDID and used for a sink CEC device.
 * If nothing is connected, then phys_addr is 0xffff.
 * See HDMI 1.4b, section 8.7 (Physical Address).
 *
 * The CEC_ADAP_S_PHYS_ADDR ioctl may not be available if that is handled
 * internally.
 */
ioctl_read!(adapter_get_physical_address, b'a', 1, u16);
ioctl_write_ptr!(adapter_set_physical_address, b'a', 2, u16);

/*
 * Configure the CEC adapter. It sets the device type and which
 * logical types it will try to claim. It will return which
 * logical addresses it could actually claim.
 * An error is returned if the adapter is disabled or if there
 * is no physical address assigned.
 */
ioctl_read!(adapter_get_logical_addresses, b'a', 3, CecLogicalAddresses);
ioctl_readwrite!(adapter_set_logical_addresses, b'a', 4, CecLogicalAddresses);

/* Transmit/receive a CEC command */
ioctl_readwrite!(transmit_message, b'a', 5, CecMessage);
ioctl_readwrite!(receive_message, b'a', 6, CecMessage);

/* Dequeue CEC events */
ioctl_readwrite!(dequeue_event, b'a', 7, CecEvent);

/*
 * Get and set the message handling mode for this filehandle.
 */
ioctl_read!(get_mode, b'a', 8, u32);
ioctl_write_ptr!(set_mode, b'a', 9, u32);

/* Get the connector info */
ioctl_read!(adapter_get_connector_info, b'a', 10, CecConnectorInfo);
