use std::ffi::c_char;

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
