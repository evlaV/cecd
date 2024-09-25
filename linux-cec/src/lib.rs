use bitflags::bitflags;

pub(crate) mod ioctls;
pub(crate) mod log_addrs;
pub(crate) mod message;
pub(crate) mod operand;

pub mod constants;
pub mod device;

type LogicalAddress = u8;
type PhysicalAddress = u16;
type Timestamp = u64;

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct Capabilities: u32 {
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
    #[derive(Debug, Copy, Clone)]
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
    #[derive(Debug, Copy, Clone)]
    struct MsgFlags: u32 {
        const REPLY_TO_FOLLOWERS = constants::CEC_MSG_FL_REPLY_TO_FOLLOWERS;
        const RAW = constants::CEC_MSG_FL_RAW;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
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
    #[derive(Debug, Copy, Clone)]
    struct EventFlags: u32 {
        const INITIAL_STATE = constants::CEC_EVENT_FL_INITIAL_STATE;
        const DROPPED_EVENTS = constants::CEC_EVENT_FL_DROPPED_EVENTS;
    }
}
