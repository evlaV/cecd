use bitflags::bitflags;

pub mod constants;
pub mod message;

bitflags! {
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
    pub struct LogicalAddressMask: u16 {
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
