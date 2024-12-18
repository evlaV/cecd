/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use bitfield_struct::bitfield;
use linux_cec_macros::BitfieldSpecifier;
use linux_cec_sys::constants;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{FollowerMode, InitiatorMode};

pub use linux_cec_sys::Timestamp;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum CecEventType {
    /// Event that occurs when the adapter state changes
    StateChange = constants::CEC_EVENT_STATE_CHANGE,
    /**
     * This event is sent when messages are lost because the application
     * didn't empty the message queue in time
     */
    LostMessages = constants::CEC_EVENT_LOST_MSGS,
    PinCecLow = constants::CEC_EVENT_PIN_CEC_LOW,
    PinCecHigh = constants::CEC_EVENT_PIN_CEC_HIGH,
    PinHpdLow = constants::CEC_EVENT_PIN_HPD_LOW,
    PinHpdHigh = constants::CEC_EVENT_PIN_HPD_HIGH,
    Pin5VLow = constants::CEC_EVENT_PIN_5V_LOW,
    Pin5VHigh = constants::CEC_EVENT_PIN_5V_HIGH,
}

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq)]
#[bits = 4]
#[repr(u32)]
pub(crate) enum CecInitiatorModes {
    NoInitiator = constants::CEC_MODE_NO_INITIATOR,
    Initiator = constants::CEC_MODE_INITIATOR,
    ExclusiveInitiator = constants::CEC_MODE_EXCL_INITIATOR,
    #[default]
    Invalid(u32),
}

impl From<InitiatorMode> for CecInitiatorModes {
    fn from(mode: InitiatorMode) -> CecInitiatorModes {
        match mode {
            InitiatorMode::Disabled => CecInitiatorModes::NoInitiator,
            InitiatorMode::Enabled => CecInitiatorModes::Initiator,
            InitiatorMode::Exclusive => CecInitiatorModes::ExclusiveInitiator,
        }
    }
}

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq)]
#[bits = 4]
#[repr(u32)]
pub(crate) enum CecFollowerModes {
    NoFollower = constants::CEC_MODE_NO_FOLLOWER >> 4,
    Follower = constants::CEC_MODE_FOLLOWER >> 4,
    ExclusiveFollower = constants::CEC_MODE_EXCL_FOLLOWER >> 4,
    ExclusiveFollowerPassthru = constants::CEC_MODE_EXCL_FOLLOWER_PASSTHRU >> 4,
    MonitorPin = constants::CEC_MODE_MONITOR_PIN >> 4,
    Monitor = constants::CEC_MODE_MONITOR >> 4,
    MonitorAll = constants::CEC_MODE_MONITOR_ALL >> 4,
    #[default]
    Invalid(u32),
}

impl From<FollowerMode> for CecFollowerModes {
    fn from(mode: FollowerMode) -> CecFollowerModes {
        match mode {
            FollowerMode::Disabled => CecFollowerModes::NoFollower,
            FollowerMode::Enabled => CecFollowerModes::Follower,
            FollowerMode::Exclusive => CecFollowerModes::ExclusiveFollower,
        }
    }
}

#[bitfield(u32)]
#[derive(PartialEq, Eq)]
pub(crate) struct CecMessageHandlingMode {
    #[bits(4)]
    pub initiator: CecInitiatorModes,
    #[bits(4)]
    pub follower: CecFollowerModes,
    #[bits(24)]
    __: usize,
}
