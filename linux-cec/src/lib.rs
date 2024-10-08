use bitflags::bitflags;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;

pub(crate) mod ioctls;

pub mod constants;
pub mod device;
pub mod message;
pub mod operand;

pub type LogicalAddress = u8;
pub type PhysicalAddress = u16;

bitflags! {
    #[derive(Debug, Copy, Clone, Default)]
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

#[derive(Error, Debug)]
pub enum Error {
    #[error("Expected at least {required} bytes, got {got} bytes")]
    InsufficientLength { required: usize, got: usize },
    #[error("Expected a length of one of {expected} bytes, got {got} bytes")]
    InvalidLength { expected: String, got: usize },
    #[error("Value {provided} of range {min}..{max}")]
    OutOfRange {
        provided: String,
        min: String,
        max: String,
    },
    #[error("Invalid value {value} for type {ty}")]
    InvalidValueForType { ty: String, value: String },
    #[error("The provided data was not valid")]
    InvalidData,
}

pub(crate) fn check_range<T: ToString + PartialOrd>(val: T, min: T, max: T) -> Result<()> {
    if val < min || val >= max {
        Err(Error::OutOfRange {
            provided: val.to_string(),
            min: min.to_string(),
            max: max.to_string(),
        })
    } else {
        Ok(())
    }
}

impl<T: TryFromPrimitive> From<TryFromPrimitiveError<T>> for Error {
    fn from(val: TryFromPrimitiveError<T>) -> Error {
        Error::InvalidValueForType {
            ty: T::NAME.to_string(),
            value: format!("{:?}", val),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
