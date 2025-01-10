/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use linux_cec_sys::constants;
use nix::errno::Errno;
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use std::fmt::{self, Debug, Display, Formatter};
use std::io;
use std::ops::Add;
use std::string::ToString;
use std::time::Duration;
use strum::{Display, EnumString};
use thiserror::Error;
use tinyvec::{Array, ArrayVec};

pub mod cdc;
pub mod device;
pub mod ioctls;
pub mod message;
pub mod operand;

#[cfg(feature = "async")]
mod async_support;

pub use linux_cec_sys as sys;

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Hash,
    IntoPrimitive,
    TryFromPrimitive,
    Display,
    EnumString,
)]
#[repr(u8)]
pub enum LogicalAddress {
    #[strum(serialize = "tv", serialize = "0")]
    Tv = constants::CEC_LOG_ADDR_TV,
    #[strum(
        serialize = "recording-device1",
        serialize = "recording-device-1",
        serialize = "recording-device",
        serialize = "1"
    )]
    RecordingDevice1 = constants::CEC_LOG_ADDR_RECORD_1,
    #[strum(
        serialize = "recording-device2",
        serialize = "recording-device-2",
        serialize = "2"
    )]
    RecordingDevice2 = constants::CEC_LOG_ADDR_RECORD_2,
    #[strum(
        serialize = "tuner1",
        serialize = "tuner-1",
        serialize = "tuner",
        serialize = "3"
    )]
    Tuner1 = constants::CEC_LOG_ADDR_TUNER_1,
    #[strum(
        serialize = "playback-device1",
        serialize = "playback-device-1",
        serialize = "playback-device",
        serialize = "4"
    )]
    PlaybackDevice1 = constants::CEC_LOG_ADDR_PLAYBACK_1,
    #[strum(serialize = "audio-system", serialize = "5")]
    AudioSystem = constants::CEC_LOG_ADDR_AUDIOSYSTEM,
    #[strum(serialize = "tuner2", serialize = "tuner-2", serialize = "6")]
    Tuner2 = constants::CEC_LOG_ADDR_TUNER_2,
    #[strum(serialize = "tuner3", serialize = "tuner-3", serialize = "7")]
    Tuner3 = constants::CEC_LOG_ADDR_TUNER_3,
    #[strum(
        serialize = "playback-device2",
        serialize = "playback-device-2",
        serialize = "8"
    )]
    PlaybackDevice2 = constants::CEC_LOG_ADDR_PLAYBACK_2,
    #[strum(
        serialize = "recording-device3",
        serialize = "recording-device-3",
        serialize = "9"
    )]
    RecordingDevice3 = constants::CEC_LOG_ADDR_RECORD_3,
    #[strum(
        serialize = "tuner4",
        serialize = "tuner-4",
        serialize = "10",
        serialize = "a"
    )]
    Tuner4 = constants::CEC_LOG_ADDR_TUNER_4,
    #[strum(
        serialize = "playback-device3",
        serialize = "playback-device-3",
        serialize = "11",
        serialize = "b"
    )]
    PlaybackDevice3 = constants::CEC_LOG_ADDR_PLAYBACK_3,
    #[strum(
        serialize = "backup1",
        serialize = "backup-1",
        serialize = "12",
        serialize = "c"
    )]
    Backup1 = constants::CEC_LOG_ADDR_BACKUP_1,
    #[strum(
        serialize = "backup2",
        serialize = "backup-2",
        serialize = "13",
        serialize = "d"
    )]
    Backup2 = constants::CEC_LOG_ADDR_BACKUP_2,
    #[strum(serialize = "specific", serialize = "14", serialize = "e")]
    Specific = constants::CEC_LOG_ADDR_SPECIFIC,
    #[default]
    #[strum(
        serialize = "unregistered",
        serialize = "broadcast",
        serialize = "15",
        serialize = "f"
    )]
    UnregisteredOrBroadcast = constants::CEC_LOG_ADDR_UNREGISTERED,
}

impl LogicalAddress {
    /** When used as initiator address */
    pub const UNREGISTERED: LogicalAddress = LogicalAddress::UnregisteredOrBroadcast;
    /** When used as destination address */
    pub const BROADCAST: LogicalAddress = LogicalAddress::UnregisteredOrBroadcast;

    #[must_use]
    pub fn primary_device_type(self) -> Option<operand::PrimaryDeviceType> {
        match self {
            LogicalAddress::Tv => Some(operand::PrimaryDeviceType::Tv),
            LogicalAddress::RecordingDevice1
            | LogicalAddress::RecordingDevice2
            | LogicalAddress::RecordingDevice3 => Some(operand::PrimaryDeviceType::Recording),
            LogicalAddress::Tuner1
            | LogicalAddress::Tuner2
            | LogicalAddress::Tuner3
            | LogicalAddress::Tuner4 => Some(operand::PrimaryDeviceType::Tuner),
            LogicalAddress::PlaybackDevice1
            | LogicalAddress::PlaybackDevice2
            | LogicalAddress::PlaybackDevice3 => Some(operand::PrimaryDeviceType::Playback),
            LogicalAddress::AudioSystem => Some(operand::PrimaryDeviceType::Audio),
            LogicalAddress::Backup1 | LogicalAddress::Backup2 => None,
            LogicalAddress::Specific => None,
            LogicalAddress::UnregisteredOrBroadcast => None,
        }
    }

    #[must_use]
    pub fn all_device_types(self) -> operand::AllDeviceTypes {
        match self {
            LogicalAddress::Tv => operand::AllDeviceTypes::TV,
            LogicalAddress::RecordingDevice1
            | LogicalAddress::RecordingDevice2
            | LogicalAddress::RecordingDevice3 => operand::AllDeviceTypes::RECORDING,
            LogicalAddress::Tuner1
            | LogicalAddress::Tuner2
            | LogicalAddress::Tuner3
            | LogicalAddress::Tuner4 => operand::AllDeviceTypes::TUNER,
            LogicalAddress::PlaybackDevice1
            | LogicalAddress::PlaybackDevice2
            | LogicalAddress::PlaybackDevice3 => operand::AllDeviceTypes::PLAYBACK,
            LogicalAddress::AudioSystem => operand::AllDeviceTypes::AUDIOSYSTEM,
            LogicalAddress::Backup1 | LogicalAddress::Backup2 => operand::AllDeviceTypes::empty(),
            LogicalAddress::Specific => operand::AllDeviceTypes::empty(),
            LogicalAddress::UnregisteredOrBroadcast => operand::AllDeviceTypes::empty(),
        }
    }

    #[must_use]
    pub fn ty(self) -> Option<LogicalAddressType> {
        match self {
            LogicalAddress::Tv => Some(LogicalAddressType::Tv),
            LogicalAddress::RecordingDevice1
            | LogicalAddress::RecordingDevice2
            | LogicalAddress::RecordingDevice3 => Some(LogicalAddressType::Record),
            LogicalAddress::Tuner1
            | LogicalAddress::Tuner2
            | LogicalAddress::Tuner3
            | LogicalAddress::Tuner4 => Some(LogicalAddressType::Tuner),
            LogicalAddress::PlaybackDevice1
            | LogicalAddress::PlaybackDevice2
            | LogicalAddress::PlaybackDevice3 => Some(LogicalAddressType::Playback),
            LogicalAddress::AudioSystem => Some(LogicalAddressType::AudioSystem),
            LogicalAddress::Backup1 | LogicalAddress::Backup2 => None,
            LogicalAddress::Specific => Some(LogicalAddressType::Specific),
            LogicalAddress::UnregisteredOrBroadcast => Some(LogicalAddressType::Unregistered),
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    IntoPrimitive,
    TryFromPrimitive,
    Display,
    EnumString,
)]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub enum LogicalAddressType {
    Tv = constants::CEC_LOG_ADDR_TYPE_TV,
    Record = constants::CEC_LOG_ADDR_TYPE_RECORD,
    Tuner = constants::CEC_LOG_ADDR_TYPE_TUNER,
    Playback = constants::CEC_LOG_ADDR_TYPE_PLAYBACK,
    AudioSystem = constants::CEC_LOG_ADDR_TYPE_AUDIOSYSTEM,
    Specific = constants::CEC_LOG_ADDR_TYPE_SPECIFIC,
    #[default]
    Unregistered = constants::CEC_LOG_ADDR_TYPE_UNREGISTERED,
}

impl LogicalAddressType {
    #[must_use]
    pub fn primary_device_type(self) -> Option<operand::PrimaryDeviceType> {
        match self {
            LogicalAddressType::Tv => Some(operand::PrimaryDeviceType::Tv),
            LogicalAddressType::Record => Some(operand::PrimaryDeviceType::Recording),
            LogicalAddressType::Tuner => Some(operand::PrimaryDeviceType::Tuner),
            LogicalAddressType::Playback => Some(operand::PrimaryDeviceType::Playback),
            LogicalAddressType::AudioSystem => Some(operand::PrimaryDeviceType::Audio),
            LogicalAddressType::Specific => None,
            LogicalAddressType::Unregistered => None,
        }
    }

    #[must_use]
    pub fn all_device_types(self) -> operand::AllDeviceTypes {
        match self {
            LogicalAddressType::Tv => operand::AllDeviceTypes::TV,
            LogicalAddressType::Record => operand::AllDeviceTypes::RECORDING,
            LogicalAddressType::Tuner => operand::AllDeviceTypes::TUNER,
            LogicalAddressType::Playback => operand::AllDeviceTypes::PLAYBACK,
            LogicalAddressType::AudioSystem => operand::AllDeviceTypes::AUDIOSYSTEM,
            LogicalAddressType::Specific | LogicalAddressType::Unregistered => {
                operand::AllDeviceTypes::empty()
            }
        }
    }
}

/// An initiator mode specifies how a given [`Device`](device::Device) should handle
/// acting an initiator; that is, if the device should be able to send messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InitiatorMode {
    /// Do not act as an initiator.
    Disabled,
    /// Act as an initiator.
    Enabled,
    /// Act as an initiator and disallow other processes
    /// acting as an initiator while the device is open.
    Exclusive,
}

/// A follower mode specifies how a given [`Device`](device::Device) should
/// handle acting a follower; that is, how receiving messages should be handled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FollowerMode {
    /// Do not act as a follower.
    Disabled,
    /// Act as a follower.
    Enabled,
    /// Act as a follower and disallow other processes
    /// acting as an follower while the device is open.
    Exclusive,
    /// Act as a follower and pass through all messages, bypassing
    /// any in-kernel processing that would normally be done.
    ExclusivePassthru,
    /// Enable monitoring of applicable [`Pin`](device::Pin)s. This mode requires
    /// [`Capabilities::MONITOR_PIN`](device::Capabilities::MONITOR_PIN) to be
    /// present on the device.
    MonitorPin,
    /// Enable monitoring of all messages on the CEC bus, not just messages
    /// addressed to this device and broadcast messages. This requires
    /// [`Capabilities::MONITOR_ALL`](device::Capabilities::MONITOR_ALL) to be
    /// present on the device.
    Monitor,
    /// Enable monitoring of applicable [`Pin`](device::Pin)s and all messages on the
    /// CEC bus, not just messages addressed to this device and broadcast messages.
    /// This requires [`Capabilities::MONITOR_PIN`](device::Capabilities::MONITOR_PIN)
    /// and [`Capabilities::MONITOR_ALL`](device::Capabilities::MONITOR_ALL) to be
    /// present on the device.
    MonitorAll,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Range<T: PartialOrd + Clone + Display + Default + Debug, const S: usize = 4>
where
    [T; S]: Array,
    <[T; S] as Array>::Item: Clone + Debug + Eq,
{
    AtMost(T),
    AtLeast(T),
    Exact(T),
    Only(ArrayVec<[T; S]>),
    Interval { min: T, max: T },
}

impl Range<usize> {
    pub fn check(self, val: impl Into<usize>, quantity: &'static str) -> Result<()> {
        let val: usize = val.into();
        match self {
            Range::AtMost(max) if val <= max => Ok(()),
            Range::AtLeast(min) if val >= min => Ok(()),
            Range::Exact(x) if val == x => Ok(()),
            Range::Only(list) if list.contains(&val) => Ok(()),
            Range::Interval { min, max } if val >= min && val <= max => Ok(()),
            _ => Err(Error::OutOfRange {
                got: val,
                expected: self,
                quantity,
            }),
        }
    }
}

impl<T: PartialOrd + Clone + Display + Default + Debug + Eq> Display for Range<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Range::AtMost(max) => f.write_fmt(format_args!("at most {max}")),
            Range::AtLeast(min) => f.write_fmt(format_args!("at least {min}")),
            Range::Exact(x) => f.write_fmt(format_args!("{x}")),
            Range::Only(list) => {
                let list = list
                    .iter()
                    .map(ToString::to_string)
                    .fold(String::new(), |a, b| {
                        if a.is_empty() {
                            b
                        } else {
                            format!("{a}, {b}")
                        }
                    });
                f.write_fmt(format_args!("one of {list}"))
            }
            Range::Interval { min, max } => f.write_fmt(format_args!("between {min} and {max}")),
        }
    }
}

impl<T: PartialOrd + Clone + Display + Default + Debug + Eq + Add<Output = T> + Copy> Add<T>
    for Range<T>
{
    type Output = Range<T>;

    #[must_use]
    fn add(self, rhs: T) -> Self::Output {
        match self {
            Range::AtMost(max) => Range::AtMost(max + rhs),
            Range::AtLeast(min) => Range::AtLeast(min + rhs),
            Range::Exact(x) => Range::Exact(x + rhs),
            Range::Only(list) => Range::Only(list.into_iter().map(|val| val + rhs).collect()),
            Range::Interval { min, max } => Range::Interval {
                min: min + rhs,
                max: max + rhs,
            },
        }
    }
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum Error {
    #[error("Expected {expected} {quantity}, got {got} {quantity}")]
    OutOfRange {
        expected: Range<usize>,
        got: usize,
        quantity: &'static str,
    },
    #[error("Invalid value {value} for type {ty}")]
    InvalidValueForType { ty: &'static str, value: String },
    #[error("The provided data was not valid")]
    InvalidData,
    #[error("A timeout occurred")]
    Timeout,
    #[error("Got unexpected result from system")]
    SystemError,
    #[error("Errno {0}")]
    Errno(#[from] Errno),
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl Error {
    pub(crate) fn add_offset(offset: usize) -> impl FnOnce(Error) -> Error {
        move |error| match error {
            Error::OutOfRange {
                got,
                expected,
                quantity,
            } if quantity == "bytes" => Error::OutOfRange {
                expected: expected + offset,
                got: got + offset,
                quantity,
            },
            _ => error,
        }
    }
}

impl From<io::Error> for Error {
    fn from(val: io::Error) -> Error {
        if let Some(raw) = val.raw_os_error() {
            Errno::from_raw(raw).into()
        } else {
            Error::UnknownError(format!("{val}"))
        }
    }
}

impl<T: TryFromPrimitive> From<TryFromPrimitiveError<T>> for Error {
    fn from(val: TryFromPrimitiveError<T>) -> Error {
        Error::InvalidValueForType {
            ty: T::NAME,
            value: format!("{:?}", val.number),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PhysicalAddress(pub(crate) u16);

impl Display for PhysicalAddress {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{:x}.{:x}.{:x}.{:x}",
            self.0 >> 12,
            (self.0 >> 8) & 0xF,
            (self.0 >> 4) & 0xF,
            self.0 & 0xF
        ))
    }
}

impl Default for PhysicalAddress {
    #[inline]
    fn default() -> PhysicalAddress {
        PhysicalAddress(0xFFFF)
    }
}

impl From<u16> for PhysicalAddress {
    #[inline]
    fn from(val: u16) -> PhysicalAddress {
        PhysicalAddress(val)
    }
}

impl From<PhysicalAddress> for u16 {
    #[inline]
    fn from(val: PhysicalAddress) -> u16 {
        val.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Timeout(u32);

impl Timeout {
    #[must_use]
    #[inline]
    pub fn as_ms(&self) -> u32 {
        self.0
    }

    #[must_use]
    #[inline]
    pub fn from_ms(millis: u32) -> Timeout {
        Timeout(millis)
    }
}

impl TryFrom<&Duration> for Timeout {
    type Error = Error;

    #[inline]
    fn try_from(duration: &Duration) -> Result<Timeout> {
        let millis = duration.as_millis();
        if let Ok(millis) = u32::try_from(millis) {
            Ok(Timeout(millis))
        } else {
            Err(Error::OutOfRange {
                expected: Range::AtMost(0xFFFFFFFF),
                got: if let Ok(millis) = usize::try_from(millis) {
                    millis
                } else {
                    usize::MAX
                },
                quantity: "milliseconds",
            })
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
