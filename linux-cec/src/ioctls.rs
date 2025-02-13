/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use bitfield_struct::bitfield;
use linux_cec_macros::BitfieldSpecifier;
use linux_cec_sys::constants;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{Error, FollowerMode, InitiatorMode, Result};

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub(crate) enum CecEventType {
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

impl TryFrom<CecInitiatorModes> for InitiatorMode {
    type Error = Error;

    fn try_from(mode: CecInitiatorModes) -> Result<InitiatorMode> {
        match mode {
            CecInitiatorModes::NoInitiator => Ok(InitiatorMode::Disabled),
            CecInitiatorModes::Initiator => Ok(InitiatorMode::Enabled),
            CecInitiatorModes::ExclusiveInitiator => Ok(InitiatorMode::Exclusive),
            CecInitiatorModes::Invalid(x) => Err(Error::InvalidValueForType {
                ty: "CecInitiatorModes",
                value: x.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod test_initiator_mode {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(
            CecInitiatorModes::NoInitiator,
            InitiatorMode::Disabled.into()
        );
        assert_eq!(CecInitiatorModes::Initiator, InitiatorMode::Enabled.into());
        assert_eq!(
            CecInitiatorModes::ExclusiveInitiator,
            InitiatorMode::Exclusive.into()
        );
    }

    #[test]
    fn test_try_into() {
        assert_eq!(
            Ok(InitiatorMode::Disabled),
            CecInitiatorModes::NoInitiator.try_into()
        );
        assert_eq!(
            Ok(InitiatorMode::Enabled),
            CecInitiatorModes::Initiator.try_into()
        );
        assert_eq!(
            Ok(InitiatorMode::Exclusive),
            CecInitiatorModes::ExclusiveInitiator.try_into()
        );
        assert_eq!(
            <_ as TryInto<InitiatorMode>>::try_into(CecInitiatorModes::Invalid(256)),
            Err(Error::InvalidValueForType {
                ty: "CecInitiatorModes",
                value: String::from("256"),
            })
        );
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
            FollowerMode::ExclusivePassthru => CecFollowerModes::ExclusiveFollowerPassthru,
            FollowerMode::MonitorPin => CecFollowerModes::MonitorPin,
            FollowerMode::Monitor => CecFollowerModes::Monitor,
            FollowerMode::MonitorAll => CecFollowerModes::MonitorAll,
        }
    }
}

impl TryFrom<CecFollowerModes> for FollowerMode {
    type Error = Error;

    fn try_from(mode: CecFollowerModes) -> Result<FollowerMode> {
        match mode {
            CecFollowerModes::NoFollower => Ok(FollowerMode::Disabled),
            CecFollowerModes::Follower => Ok(FollowerMode::Enabled),
            CecFollowerModes::ExclusiveFollower => Ok(FollowerMode::Exclusive),
            CecFollowerModes::ExclusiveFollowerPassthru => Ok(FollowerMode::ExclusivePassthru),
            CecFollowerModes::MonitorPin => Ok(FollowerMode::MonitorPin),
            CecFollowerModes::Monitor => Ok(FollowerMode::Monitor),
            CecFollowerModes::MonitorAll => Ok(FollowerMode::MonitorAll),
            CecFollowerModes::Invalid(x) => Err(Error::InvalidValueForType {
                ty: "CecFollowerModes",
                value: x.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod test_follower_mode {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(CecFollowerModes::NoFollower, FollowerMode::Disabled.into());
        assert_eq!(CecFollowerModes::Follower, FollowerMode::Enabled.into());
        assert_eq!(
            CecFollowerModes::ExclusiveFollower,
            FollowerMode::Exclusive.into()
        );
        assert_eq!(
            CecFollowerModes::ExclusiveFollowerPassthru,
            FollowerMode::ExclusivePassthru.into()
        );
        assert_eq!(
            CecFollowerModes::MonitorPin,
            FollowerMode::MonitorPin.into()
        );
        assert_eq!(CecFollowerModes::Monitor, FollowerMode::Monitor.into());
        assert_eq!(
            CecFollowerModes::MonitorAll,
            FollowerMode::MonitorAll.into()
        );
    }

    #[test]
    fn test_try_into() {
        assert_eq!(
            Ok(FollowerMode::Disabled),
            CecFollowerModes::NoFollower.try_into()
        );
        assert_eq!(
            Ok(FollowerMode::Enabled),
            CecFollowerModes::Follower.try_into()
        );
        assert_eq!(
            Ok(FollowerMode::Exclusive),
            CecFollowerModes::ExclusiveFollower.try_into()
        );
        assert_eq!(
            Ok(FollowerMode::ExclusivePassthru),
            CecFollowerModes::ExclusiveFollowerPassthru.try_into()
        );
        assert_eq!(
            Ok(FollowerMode::MonitorPin),
            CecFollowerModes::MonitorPin.try_into()
        );
        assert_eq!(
            Ok(FollowerMode::Monitor),
            CecFollowerModes::Monitor.try_into()
        );
        assert_eq!(
            Ok(FollowerMode::MonitorAll),
            CecFollowerModes::MonitorAll.try_into()
        );
        assert_eq!(
            <_ as TryInto<FollowerMode>>::try_into(CecFollowerModes::Invalid(256)),
            Err(Error::InvalidValueForType {
                ty: "CecFollowerModes",
                value: String::from("256"),
            })
        );
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
