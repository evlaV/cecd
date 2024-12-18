/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use linux_cec_sys::constants;
use nix::errno::Errno;
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use std::fmt::{self, Display, Formatter};
use std::io;
use std::ops::Add;
use std::string::ToString;
use std::time::Duration;
use strum::{Display, EnumString};
use thiserror::Error;

pub mod cdc;
pub mod device;
pub mod ioctls;
pub mod message;
pub mod operand;

#[cfg(feature = "async")]
mod async_support;

pub use linux_cec_sys as sys;
pub use linux_cec_sys::PhysicalAddress;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, IntoPrimitive, TryFromPrimitive, Display, EnumString,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitiatorMode {
    Disabled,
    Enabled,
    Exclusive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FollowerMode {
    Disabled,
    Enabled,
    Exclusive,
    Monitor,
    MonitorAll,
    // TODO: other modes
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Range<T: PartialOrd + Display> {
    AtMost(T),
    AtLeast(T),
    Exact(T),
    Only(Vec<T>),
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

impl<T: PartialOrd + Display> Display for Range<T> {
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

impl<T: PartialOrd + Display + Add<Output = T> + Copy> Add<T> for Range<T> {
    type Output = Range<T>;

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

#[derive(Error, Debug, PartialEq)]
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
            todo!();
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Timeout(u32);

impl Timeout {
    pub fn as_ms(&self) -> u32 {
        self.0
    }

    pub fn from_ms(millis: u32) -> Timeout {
        Timeout(millis)
    }
}

impl TryFrom<&Duration> for Timeout {
    type Error = Error;

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
