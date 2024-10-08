use bitflags::bitflags;
use nix::{ioctl_read, ioctl_readwrite, ioctl_write_ptr};
use std::ffi::c_char;

use crate::constants;
use crate::message::Opcode;
use crate::{LogicalAddress, PhysicalAddress};

pub type Timestamp = u64;

bitflags! {
    #[derive(Debug, Copy, Clone, Default)]
    struct Capabilities: u32 {
        /// Userspace has to configure the physical address
        const PHYS_ADDR = constants::CEC_CAP_PHYS_ADDR;
        /// Userspace has to configure the logical addresses
        const LOG_ADDRS = constants::CEC_CAP_LOG_ADDRS;
        /// Userspace can transmit messages (and thus become follower as well)
        const TRANSMIT = constants::CEC_CAP_TRANSMIT;
        /// Passthrough all messages instead of processing them.
        const PASSTHROUGH = constants::CEC_CAP_PASSTHROUGH;
        /// Supports remote control
        const RC = constants::CEC_CAP_RC;
        /// Hardware can monitor all messages, not just directed and broadcast.
        const MONITOR_ALL = constants::CEC_CAP_MONITOR_ALL;
        /// Hardware can use CEC only if the HDMI HPD pin is high.
        const NEEDS_HPD = constants::CEC_CAP_NEEDS_HPD;
        /// Hardware can monitor CEC pin transitions
        const MONITOR_PIN = constants::CEC_CAP_MONITOR_PIN;
        /// CEC_ADAP_G_CONNECTOR_INFO is available
        const CONNECTOR_INFO = constants::CEC_CAP_CONNECTOR_INFO;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, Default)]
    struct LogicalAddressMask: u16 {
        const TV = constants::CEC_LOG_ADDR_MASK_TV;
        const MASK_RECORD = constants::CEC_LOG_ADDR_MASK_RECORD;
        const TUNER = constants::CEC_LOG_ADDR_MASK_TUNER;
        const PLAYBACK = constants::CEC_LOG_ADDR_MASK_PLAYBACK;
        const AUDIOSYSTEM = constants::CEC_LOG_ADDR_MASK_AUDIOSYSTEM;
        const BACKUP = constants::CEC_LOG_ADDR_MASK_BACKUP;
        const SPECIFIC = constants::CEC_LOG_ADDR_MASK_SPECIFIC;
        const UNREGISTERED = constants::CEC_LOG_ADDR_MASK_UNREGISTERED;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    struct TxStatus: u8 {
        const OK = constants::CEC_TX_STATUS_OK;
        const ARB_LOST = constants::CEC_TX_STATUS_ARB_LOST;
        const NACK = constants::CEC_TX_STATUS_NACK;
        const LOW_DRIVE = constants::CEC_TX_STATUS_LOW_DRIVE;
        const ERROR = constants::CEC_TX_STATUS_ERROR;
        const MAX_RETRIES = constants::CEC_TX_STATUS_MAX_RETRIES;
        const ABORTED = constants::CEC_TX_STATUS_ABORTED;
        const TIMEOUT = constants::CEC_TX_STATUS_TIMEOUT;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
    struct RxStatus: u8 {
        const OK = constants::CEC_RX_STATUS_OK;
        const TIMEOUT = constants::CEC_RX_STATUS_TIMEOUT;
        const FEATURE_ABORT = constants::CEC_RX_STATUS_FEATURE_ABORT;
        const ABORTED = constants::CEC_RX_STATUS_ABORTED;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, Default)]
    struct MsgFlags: u32 {
        const REPLY_TO_FOLLOWERS = constants::CEC_MSG_FL_REPLY_TO_FOLLOWERS;
        const RAW = constants::CEC_MSG_FL_RAW;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, Default)]
    struct LogicalAddressesFlags: u32 {
        /// Allow a fallback to unregistered
        const ALLOW_UNREG_FALLBACK = constants::CEC_LOG_ADDRS_FL_ALLOW_UNREG_FALLBACK;
        /// Passthrough RC messages to the input subsystem
        const ALLOW_RC_PASSTHRU = constants::CEC_LOG_ADDRS_FL_ALLOW_RC_PASSTHRU;
        /// CDC-Only device: supports only CDC messages
        const CDC_ONLY = constants::CEC_LOG_ADDRS_FL_CDC_ONLY;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, Default)]
    struct EventFlags: u32 {
        const INITIAL_STATE = constants::CEC_EVENT_FL_INITIAL_STATE;
        const DROPPED_EVENTS = constants::CEC_EVENT_FL_DROPPED_EVENTS;
    }
}

/// CEC capabilities structure.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct CecCapabilities {
    /// Name of the CEC device driver.
    driver: [c_char; 32],
    /// Name of the CEC device. `driver` + `name` must be unique.
    name: [c_char; 32],
    /// Number of available logical addresses.
    available_log_addrs: u32,
    /// Capabilities of the CEC adapter.
    capabilities: Capabilities,
    /// version of the CEC adapter framework.
    version: u32,
}

/// Tells which drm connector is associated with the CEC adapter.
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
    phys_addr: PhysicalAddress,
    /// The current logical address mask
    log_addr_mask: LogicalAddressMask,
    /** If non-zero, then HDMI connector information is available.
     *  This field is only valid if CEC_CAP_CONNECTOR_INFO is set. If that
     *  capability is set and @have_conn_info is zero, then that indicates
     *  that the HDMI connector device is not instantiated, either because
     *  the HDMI driver is still configuring the device or because the HDMI
     *  device was unbound.
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

/// CEC message structure.
#[repr(C)]
pub(crate) struct CecMessage {
    /// Timestamp in nanoseconds using `CLOCK_MONOTONIC`. Set by the
    /// driver when the message transmission has finished.
    tx_ts: Timestamp,
    /// Timestamp in nanoseconds using `CLOCK_MONOTONIC`. Set by the
    /// driver when the message was received.
    rx_ts: Timestamp,
    /// Length in bytes of the message.
    len: u32,
    /**
     * The timeout (in ms) that is used to timeout `CEC_RECEIVE`.
     * Set to 0 if you want to wait forever. This timeout can also be
     * used with `CEC_TRANSMIT` as the timeout for waiting for a reply.
     * If 0, then it will use a 1 second timeout instead of waiting
     * forever as is done with `CEC_RECEIVE`.
     */
    timeout: u32,
    /// The framework assigns a sequence number to messages that are
    /// sent. This can be used to track replies to previously sent messages.
    sequence: u32,
    /// Set to 0.
    flags: MsgFlags,
    /// The message payload.
    msg: [u8; constants::CEC_MAX_MSG_SIZE],
    /**
     * This field is ignored with `CEC_RECEIVE` and is only used by
     * `CEC_TRANSMIT`. If non-zero, then wait for a reply with this
     * opcode. Set to `CEC_MSG_FEATURE_ABORT` if you want to wait for
     * a possible `ABORT` reply. If there was an error when sending the
     * msg or `FeatureAbort` was returned, then reply is set to 0.
     * If reply is non-zero upon return, then len/msg are set to
     * the received message.
     * If reply is zero upon return and status has the
     * `CEC_TX_STATUS_FEATURE_ABORT` bit set, then len/msg are set to
     * the received feature abort message.
     * If reply is zero upon return and status has the
     * `CEC_TX_STATUS_MAX_RETRIES` bit set, then no reply was seen at
     * all. If reply is non-zero for `CEC_TRANSMIT` and the message is a
     * broadcast, then `-EINVAL` is returned.
     * if reply is non-zero, then timeout is set to 1000 (the required
     * maximum response time).
     */
    reply: u8,
    /// The message receive status bits. Set by the driver.
    rx_status: RxStatus,
    /// The message transmit status bits. Set by the driver.
    tx_status: TxStatus,
    /// The number of 'Arbitration Lost' events. Set by the driver.
    tx_arb_lost_cnt: u8,
    /// The number of 'Not Acknowledged' events. Set by the driver.
    tx_nack_cnt: u8,
    /// The number of 'Low Drive Detected' events. Set by the driver.
    tx_low_drive_cnt: u8,
    /// The number of 'Error' events. Set by the driver.
    tx_error_cnt: u8,
}

/// CEC logical addresses structure
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct CecLogicalAddresses {
    /// The claimed logical addresses. Set by the driver.
    pub log_addr: [LogicalAddress; constants::CEC_MAX_LOG_ADDRS],
    /// Current logical address mask. Set by the driver.
    pub log_addr_mask: LogicalAddressMask,
    /// The CEC version that the adapter should implement. Set by the caller.
    pub cec_version: u8,
    /// How many logical addresses should be claimed. Set by the caller.
    pub num_log_addrs: u8,
    /// The vendor ID of the device. Set by the caller.
    pub vendor_id: u32,
    /// Flags.
    pub flags: LogicalAddressesFlags,
    /// The OSD name of the device. Set by the caller.
    pub osd_name: [c_char; 15],
    /// The primary device type for each logical address. Set by the caller.
    pub primary_device_type: [u8; constants::CEC_MAX_LOG_ADDRS],
    /// The logical address types. Set by the caller.
    pub log_addr_type: [u8; constants::CEC_MAX_LOG_ADDRS],

    /* CEC 2.0 */
    /// CEC 2.0: all device types represented by the logical address. Set by the caller.
    pub all_device_types: [u8; constants::CEC_MAX_LOG_ADDRS],
    /// CEC 2.0: The logical address features. Set by the caller.
    pub features: [[u8; 12]; constants::CEC_MAX_LOG_ADDRS],
}

impl CecLogicalAddresses {
    /* Helper functions to identify the 'special' CEC devices */

    fn is_2nd_tv(&self) -> bool {
        /*
         * It is a second TV if the logical address is 14 or 15 and the
         * primary device type is a TV.
         */
        self.num_log_addrs != 0
            && self.log_addr[0] >= constants::CEC_LOG_ADDR_SPECIFIC
            && self.primary_device_type[0] == constants::CEC_OP_PRIM_DEVTYPE_TV
    }

    fn is_processor(&self) -> bool {
        /*
         * It is a processor if the logical address is 12-15 and the
         * primary device type is a Processor.
         */
        self.num_log_addrs != 0
            && self.log_addr[0] >= constants::CEC_LOG_ADDR_BACKUP_1
            && self.primary_device_type[0] == constants::CEC_OP_PRIM_DEVTYPE_PROCESSOR
    }

    fn is_switch(&self) -> bool {
        /*
         * It is a switch if the logical address is 15 and the
         * primary device type is a Switch and the CDC-Only flag is not set.
         */
        self.num_log_addrs == 1
            && self.log_addr[0] == constants::CEC_LOG_ADDR_UNREGISTERED
            && self.primary_device_type[0] == constants::CEC_OP_PRIM_DEVTYPE_SWITCH
            && !self.flags.contains(LogicalAddressesFlags::CDC_ONLY)
    }

    fn is_cdc_only(&self) -> bool {
        /*
         * It is a CDC-only device if the logical address is 15 and the
         * primary device type is a Switch and the CDC-Only flag is set.
         */
        self.num_log_addrs == 1
            && self.log_addr[0] == constants::CEC_LOG_ADDR_UNREGISTERED
            && self.primary_device_type[0] == constants::CEC_OP_PRIM_DEVTYPE_SWITCH
            && self.flags.contains(LogicalAddressesFlags::CDC_ONLY)
    }
}

impl CecMessage {
    /// Return the initiator's logical address.
    pub fn initiator(&self) -> u8 {
        self.msg[0] >> 4
    }

    /// Return the destination's logical address.
    pub fn destination(&self) -> u8 {
        self.msg[0] & 0xf
    }

    /// Return the opcode of the message, None for poll
    pub fn raw_opcode(&self) -> Option<u8> {
        if self.len > 1 {
            Some(self.msg[1])
        } else {
            None
        }
    }

    /// Return true if this is a broadcast message.
    pub fn is_broadcast(&self) -> bool {
        (self.msg[0] & 0xf) == 0xf
    }

    /**
     * Initialize the message structure.
     * `initiator` is the logical address of the initiator and
     * `destination` the logical address of the destination (`0xf` for broadcast).
     *
     * The whole structure is zeroed, the len field is set to 1 (i.e. a poll
     * message) and the initiator and destination are filled in.
     */
    pub fn new(initiator: LogicalAddress, destination: LogicalAddress) -> CecMessage {
        let mut msg = CecMessage {
            tx_ts: 0,
            rx_ts: 0,
            len: 1,
            timeout: 0,
            sequence: 0,
            flags: MsgFlags::empty(),
            msg: [0; 16],
            reply: 0,
            rx_status: RxStatus::empty(),
            tx_status: TxStatus::empty(),
            tx_arb_lost_cnt: 0,
            tx_nack_cnt: 0,
            tx_low_drive_cnt: 0,
            tx_error_cnt: 0,
        };
        msg.msg[0] = (initiator << 4) | destination;

        msg
    }

    pub fn with_timeout(timeout_ms: u32) -> CecMessage {
        CecMessage {
            tx_ts: 0,
            rx_ts: 0,
            len: 0,
            timeout: timeout_ms,
            sequence: 0,
            flags: MsgFlags::empty(),
            msg: [0; 16],
            reply: 0,
            rx_status: RxStatus::empty(),
            tx_status: TxStatus::empty(),
            tx_arb_lost_cnt: 0,
            tx_nack_cnt: 0,
            tx_low_drive_cnt: 0,
            tx_error_cnt: 0,
        }
    }

    /**
     * Fill in destination/initiator in a reply message.
     *
     * Set the msg destination to the orig initiator and the msg initiator to the
     * orig destination. Note that msg and orig may be the same pointer, in which
     * case the change is done in place.
     */
    pub fn set_reply_to(&mut self, orig: &CecMessage) {
        /* The destination becomes the initiator and vice versa */
        self.msg[0] = (orig.destination() << 4) | orig.initiator();
        self.reply = 0;
        self.timeout = 0;
    }

    /// Return true if this message contains the result of an earlier non-blocking transmit
    pub fn recv_is_tx_result(&self) -> bool {
        self.sequence != 0 && !self.tx_status.is_empty() && self.rx_status.is_empty()
    }

    /// Return true if this message contains the reply of an earlier non-blocking transmit
    pub fn recv_is_rx_result(&self) -> bool {
        self.sequence != 0 && self.tx_status.is_empty() && !self.rx_status.is_empty()
    }

    pub fn status_is_ok(&self) -> bool {
        if !self.tx_status.is_empty() && !self.tx_status.contains(TxStatus::OK) {
            return false;
        }
        if !self.rx_status.is_empty() && !self.rx_status.contains(RxStatus::OK) {
            return false;
        }
        if self.tx_status.is_empty() && self.rx_status.is_empty() {
            return false;
        }
        !self.rx_status.contains(RxStatus::FEATURE_ABORT)
    }

    pub fn opcode(&self) -> Option<Opcode> {
        let raw = self.raw_opcode()?;
        Opcode::try_from(raw).ok()
    }
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
ioctl_read!(adapter_get_physical_address, b'a', 1, PhysicalAddress);
ioctl_write_ptr!(adapter_set_physical_address, b'a', 2, PhysicalAddress);

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
