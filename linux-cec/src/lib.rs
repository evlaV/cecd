/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use linux_cec_macros::Operand;
use linux_cec_sys::{constants, VendorId as SysVendorId, CEC_TX_STATUS};
use nix::errno::Errno;
use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use std::fmt::{self, Debug, Display, Formatter};
use std::io;
use std::ops::{Add, Deref, RangeInclusive};
use std::str::FromStr;
use std::string::ToString;
use std::time::Duration;
use strum::{Display, EnumString};
use thiserror::Error;

pub mod cdc;
pub mod device;
pub mod message;
pub mod operand;

#[cfg(feature = "async")]
mod async_support;

pub(crate) mod ioctls;

pub use linux_cec_sys as sys;
pub use linux_cec_sys::Timestamp;

/// A CEC logical address, used for identifying devices
/// attached to the CEC bus.
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
    #![allow(non_upper_case_globals)]
    /** When used as initiator address */
    pub const Unregistered: LogicalAddress = LogicalAddress::UnregisteredOrBroadcast;
    /** When used as destination address */
    pub const Broadcast: LogicalAddress = LogicalAddress::UnregisteredOrBroadcast;

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

/// The type of a CEC logical address, used for determining what type
/// type of device is at the given address and for requesting an address.
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

/// An mode specifying how a given [`Device`](device::Device) should act as
/// an initiator; that is, if the device should be able to send messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InitiatorMode {
    /// Do not act as an initiator.
    Disabled,
    /// Act as an initiator.
    Enabled,
    /// Act as an initiator and disallow other processes
    /// acting as an initiator while the device is open.
    Exclusive,
}

/// A mode specifying how a given [`Device`](device::Device) should act as
/// a follower; that is, how receiving messages should be handled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
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

/// This enum encodes the diferent ways in which a given message may be
/// allowed to be addressed.
///
/// Hypothetically all messages can be addressed either directly or broadcast;
/// however, the specification states that each message type only has specific
/// ways it can be addressed. This enum encodes the various ways a message may
/// be allowed to be addressed, per the specification, and can be queried on
/// either a [`Message`](crate::message::Message) or an
/// [`Opcode`](crate::message::Opcode).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AddressingType {
    /// The given message can only be addressed
    /// directly to a specific logical address.
    Direct,
    /// The given message can only be broadcast to all logical addresses.
    Broadcast,
    /// The given message can be either directly addressed or broadcast.
    Either,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Range<T: PartialOrd + Clone + Display + Default + Debug> {
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

impl<T: PartialOrd + Clone + Display + Default + Debug + Eq> From<RangeInclusive<T>> for Range<T> {
    fn from(val: RangeInclusive<T>) -> Range<T> {
        Range::Interval {
            min: val.start().clone(),
            max: val.end().clone(),
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

#[cfg(test)]
mod test_range {
    use super::Range;

    #[test]
    fn test_display_at_most() {
        assert_eq!(Range::AtMost(10).to_string(), "at most 10");
    }

    #[test]
    fn test_display_at_least() {
        assert_eq!(Range::AtLeast(10).to_string(), "at least 10");
    }

    #[test]
    fn test_display_exact() {
        assert_eq!(Range::Exact(10).to_string(), "10");
    }

    #[test]
    fn test_display_only_1() {
        assert_eq!(Range::Only(vec![10]).to_string(), "one of 10");
    }

    #[test]
    fn test_display_only_2() {
        assert_eq!(Range::Only(vec![10, 11]).to_string(), "one of 10, 11");
    }

    #[test]
    fn test_display_only_3() {
        assert_eq!(
            Range::Only(vec![10, 11, 12]).to_string(),
            "one of 10, 11, 12"
        );
    }

    #[test]
    fn test_display_interval() {
        assert_eq!(
            Range::Interval { min: 1, max: 10 }.to_string(),
            "between 1 and 10"
        );
    }

    #[test]
    fn test_add_at_most() {
        assert_eq!(Range::AtMost(10) + 1, Range::AtMost(11));
    }

    #[test]
    fn test_add_at_least() {
        assert_eq!(Range::AtLeast(10) + 1, Range::AtLeast(11));
    }

    #[test]
    fn test_add_exact() {
        assert_eq!(Range::Exact(10) + 1, Range::Exact(11));
    }

    #[test]
    fn test_add_only() {
        assert_eq!(
            Range::Only(vec![10, 11, 12]) + 1,
            Range::Only(vec![11, 12, 13])
        );
    }

    #[test]
    fn test_add_interval() {
        assert_eq!(
            Range::Interval { min: 1, max: 10 } + 1,
            Range::Interval { min: 2, max: 11 }
        );
    }
}

/// A set of common errors.
#[derive(Error, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// A value of a given `quantity` was outside of the `expected` range.
    #[error("Expected {expected} {quantity}, got {got} {quantity}")]
    OutOfRange {
        expected: Range<usize>,
        got: usize,
        quantity: &'static str,
    },
    /// Got a `value` for a given type that was invalid for that `ty`.
    #[error("Invalid value {value} for type {ty}")]
    InvalidValueForType { ty: &'static str, value: String },
    /// Got generic invalid data.
    #[error("The provided data was not valid")]
    InvalidData,
    /// A timeout occurred.
    #[error("A timeout occurred")]
    Timeout,
    /// A request was aborted.
    #[error("The request was aborted")]
    Abort,
    /// The request failed because the device doesn't have a logical address.
    #[error("The device does not have a logical address")]
    NoLogicalAddress,
    /// The device was disconnected from the system.
    #[error("The device was disconnected")]
    Disconnected,
    /// A generic system error occurred.
    #[error("Got unexpected result from system: {0}")]
    SystemError(Errno),
    /// Got an error while transmitting a [`Message`](crate::message::Message)
    /// that did not correspond to one of the other error types.
    #[error("{0}")]
    TxError(#[from] TxError),
    /// Got an error while receiving a [`Message`](crate::message::Message)
    /// that did not correspond to one of the other error types.
    #[error("{0}")]
    RxError(#[from] RxError),
    /// Got an error that does not map to any of the other error types.
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

/// A set of error codes that correspond to [`CEC_TX_STATUS`].
#[derive(Error, Clone, Debug, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum TxError {
    #[error("Arbitration was lost")]
    ArbLost = constants::CEC_TX_STATUS_ARB_LOST,
    #[error("No acknowledgement")]
    Nack = constants::CEC_TX_STATUS_NACK,
    #[error("Low drive on bus")]
    LowDrive = constants::CEC_TX_STATUS_LOW_DRIVE,
    #[error("An unknown error occurred")]
    UnknownError = constants::CEC_TX_STATUS_ERROR,
    #[error("Maximum retries hit")]
    MaxRetries = constants::CEC_TX_STATUS_MAX_RETRIES,
    #[error("The request was aborted")]
    Aborted = constants::CEC_TX_STATUS_ABORTED,
    #[error("The request timed out")]
    Timeout = constants::CEC_TX_STATUS_TIMEOUT,
}

/// A set of error codes that correspond to [`CEC_RX_STATUS`](sys::CEC_RX_STATUS).
#[derive(Error, Clone, Debug, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum RxError {
    #[error("The request timed out")]
    Timeout = constants::CEC_RX_STATUS_TIMEOUT,
    #[error("The requested feature was not present")]
    FeatureAbort = constants::CEC_RX_STATUS_FEATURE_ABORT,
    #[error("The request was aborted")]
    Aborted = constants::CEC_RX_STATUS_ABORTED,
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

impl From<Errno> for Error {
    fn from(val: Errno) -> Error {
        match val {
            Errno::EINVAL => Error::InvalidData,
            Errno::ETIMEDOUT => Error::Timeout,
            Errno::ENODEV => Error::Disconnected,
            Errno::ENONET => Error::NoLogicalAddress,
            x => Error::SystemError(x),
        }
    }
}

impl From<CEC_TX_STATUS> for Error {
    fn from(tx_status: CEC_TX_STATUS) -> Error {
        if tx_status.contains(CEC_TX_STATUS::TIMEOUT) {
            return Error::Timeout;
        }
        if tx_status.contains(CEC_TX_STATUS::ABORTED) {
            return Error::Abort;
        }
        if tx_status.contains(CEC_TX_STATUS::ARB_LOST) {
            return TxError::ArbLost.into();
        }
        if tx_status.contains(CEC_TX_STATUS::NACK) {
            return TxError::Nack.into();
        }
        if tx_status.contains(CEC_TX_STATUS::LOW_DRIVE) {
            return TxError::LowDrive.into();
        }
        if tx_status.contains(CEC_TX_STATUS::MAX_RETRIES) {
            return TxError::MaxRetries.into();
        }
        Error::UnknownError(format!("{tx_status:?}"))
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

/// A unique 16-bit value that refers to a single
/// device in the topology of the CDC network.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PhysicalAddress(pub(crate) u16);

impl PhysicalAddress {
    /// The physical address reserved for the root device of the CEC hierarchy
    pub const ROOT: PhysicalAddress = PhysicalAddress(0x0000);

    /// The physical address reserved to denote an invalid address
    pub const INVALID: PhysicalAddress = PhysicalAddress(0xFFFF);

    /// Check whether or not the physical address is valid, i.e. not `f.f.f.f`.
    #[must_use]
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.0 != 0xFFFF
    }

    /// Check whether or not the physical address is the root device, i.e. `0.0.0.0`.
    #[must_use]
    #[inline]
    pub fn is_root(&self) -> bool {
        self.0 == 0x0000
    }
}

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

impl FromStr for PhysicalAddress {
    type Err = Error;

    fn from_str(val: &str) -> Result<PhysicalAddress> {
        if let Some(val) = val.strip_prefix("0x") {
            if val.len() == 4 {
                if let Ok(addr) = u16::from_str_radix(val, 16) {
                    return Ok(PhysicalAddress(addr));
                }
            }
        }
        if val.len() == 4 {
            if let Ok(addr) = u16::from_str_radix(val, 16) {
                return Ok(PhysicalAddress(addr));
            }
        }
        if val.len() == 7 {
            let split: Vec<&str> = val.split('.').collect();
            if split.len() != 4 {
                return Err(Error::InvalidData);
            }
            if !split.iter().all(|place| place.len() == 1) {
                return Err(Error::InvalidData);
            }
            let Ok(a) = u16::from_str_radix(split[0], 16) else {
                return Err(Error::InvalidData);
            };
            let Ok(b) = u16::from_str_radix(split[1], 16) else {
                return Err(Error::InvalidData);
            };
            let Ok(c) = u16::from_str_radix(split[2], 16) else {
                return Err(Error::InvalidData);
            };
            let Ok(d) = u16::from_str_radix(split[3], 16) else {
                return Err(Error::InvalidData);
            };
            return Ok(PhysicalAddress((a << 12) | (b << 8) | (c << 4) | d));
        }
        Err(Error::InvalidData)
    }
}

#[cfg(test)]
mod test_physical_address {
    use super::*;

    #[test]
    fn test_fmt() {
        assert_eq!(format!("{}", PhysicalAddress::from(0x12AB)), "1.2.a.b");
    }

    #[test]
    fn test_from() {
        assert_eq!(PhysicalAddress::from(0x12AB), PhysicalAddress(0x12AB));
    }

    #[test]
    fn test_into() {
        assert_eq!(<_ as Into<u16>>::into(PhysicalAddress(0x12AB)), 0x12AB);
    }

    #[test]
    fn test_from_str_hex_lower() {
        assert_eq!(
            PhysicalAddress::from_str("12ab").unwrap(),
            PhysicalAddress(0x12AB)
        );
    }

    #[test]
    fn test_from_str_hex_upper() {
        assert_eq!(
            PhysicalAddress::from_str("12AB").unwrap(),
            PhysicalAddress(0x12AB)
        );
    }

    #[test]
    fn test_from_str_hex_prefix() {
        assert_eq!(
            PhysicalAddress::from_str("0x12AB").unwrap(),
            PhysicalAddress(0x12AB)
        );
    }

    #[test]
    fn test_from_str_dotted_lower() {
        assert_eq!(
            PhysicalAddress::from_str("1.2.a.b").unwrap(),
            PhysicalAddress(0x12AB)
        );
    }

    #[test]
    fn test_from_str_dotted_upper() {
        assert_eq!(
            PhysicalAddress::from_str("1.2.A.B").unwrap(),
            PhysicalAddress(0x12AB)
        );
    }

    #[test]
    fn test_from_str_too_short() {
        assert!(PhysicalAddress::from_str("12a").is_err(),);
    }

    #[test]
    fn test_from_str_too_long() {
        assert!(PhysicalAddress::from_str("12abcd").is_err(),);
    }

    #[test]
    fn test_from_str_dotted_too_short() {
        assert!(PhysicalAddress::from_str("1.2.a").is_err(),);
    }

    #[test]
    fn test_from_str_dotted_too_long() {
        assert!(PhysicalAddress::from_str("1.2.a.b.c").is_err(),);
    }

    #[test]
    fn test_from_str_dotted_missing() {
        assert!(PhysicalAddress::from_str("1.2.ab").is_err(),);
    }

    #[test]
    fn test_from_str_dotted_too_long_group() {
        assert!(PhysicalAddress::from_str("1.2.a.bc").is_err(),);
    }

    #[test]
    fn test_from_str_dotted_too_short_group() {
        assert!(PhysicalAddress::from_str("1.2.a.").is_err(),);
    }

    #[test]
    fn test_from_str_dotted_extra_group_before() {
        assert!(PhysicalAddress::from_str(".1.2.a.b").is_err(),);
    }

    #[test]
    fn test_from_str_dotted_extra_group_mid() {
        assert!(PhysicalAddress::from_str("1..2.a.b").is_err(),);
    }

    #[test]
    fn test_from_str_dotted_extra_group_after() {
        assert!(PhysicalAddress::from_str("1.2.a.b.").is_err(),);
    }
}

/// A 24-bit [MA-L/OUI](https://en.wikipedia.org/wiki/Organizationally_unique_identifier)
/// identifying a device's vendor or manufacturer.
///
/// These IDs are obtained from the IEEE, and a current list of OUIs can be queried from
/// [their website](https://regauth.standards.ieee.org/standards-ra-web/pub/view.html#registries).
/// A full list is also available as [plain text](https://standards-oui.ieee.org/oui/oui.txt) or
/// [CSV](https://standards-oui.ieee.org/oui/oui.csv).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Operand)]
pub struct VendorId(pub [u8; 3]);

impl Deref for VendorId {
    type Target = [u8; 3];

    fn deref(&self) -> &[u8; 3] {
        &self.0
    }
}

impl From<VendorId> for SysVendorId {
    #[inline]
    fn from(val: VendorId) -> SysVendorId {
        SysVendorId::try_from(u32::from(val)).unwrap()
    }
}

impl From<VendorId> for i32 {
    #[inline]
    fn from(val: VendorId) -> i32 {
        (i32::from(val.0[0]) << 16) | (i32::from(val.0[1]) << 8) | i32::from(val.0[2])
    }
}

impl From<VendorId> for u32 {
    #[inline]
    fn from(val: VendorId) -> u32 {
        (u32::from(val.0[0]) << 16) | (u32::from(val.0[1]) << 8) | u32::from(val.0[2])
    }
}

impl FromStr for VendorId {
    type Err = Error;

    fn from_str(val: &str) -> Result<VendorId> {
        let parts: Vec<&str> = val.split('-').collect();
        if parts.len() != 3 {
            return Err(Error::InvalidData);
        }

        let mut id = [0; 3];
        for (idx, part) in parts.into_iter().enumerate() {
            if part.len() != 2 {
                return Err(Error::InvalidData);
            }
            id[idx] = u8::from_str_radix(part, 16).map_err(|_| Error::InvalidData)?;
        }
        Ok(VendorId(id))
    }
}

impl Display for VendorId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}-{:02x}-{:02x}", self.0[0], self.0[1], self.0[2])
    }
}

impl VendorId {
    /// Convert a [`linux_cec_sys::VendorId`] into a `VendorId`. Since `linux_cec_sys::VendorId` is just
    /// a convenience type around `u32`, not all values are valid, so this conversion can fail: the
    /// reserved value 0xFFFFFFFF is treated as `Ok(None)`, and all over values outside of the valid range
    /// return [`Error::InvalidData`].
    pub fn try_from_sys(vendor_id: SysVendorId) -> Result<Option<VendorId>> {
        match vendor_id {
            x if x.is_none() => Ok(None),
            x if x.is_valid() => {
                let val: u32 = x.into();
                Ok(Some(VendorId([
                    ((val >> 16) & 0xFF).try_into().unwrap(),
                    ((val >> 8) & 0xFF).try_into().unwrap(),
                    (val & 0xFF).try_into().unwrap(),
                ])))
            }
            _ => Err(Error::InvalidData),
        }
    }
}

#[cfg(test)]
mod test_vendor_id {
    use super::*;

    #[test]
    fn test_parsing() {
        assert_eq!(
            VendorId::from_str("01-ab-2c"),
            Ok(VendorId([0x01, 0xAB, 0x2C]))
        );
        assert_eq!(
            VendorId::from_str("01-23-45"),
            Ok(VendorId([0x01, 0x23, 0x45]))
        );
    }

    #[test]
    fn test_invalid_parsing() {
        assert_eq!(VendorId::from_str("01-ab-2g"), Err(Error::InvalidData));
        assert_eq!(VendorId::from_str("01-ab"), Err(Error::InvalidData));
        assert_eq!(VendorId::from_str("01ab2c"), Err(Error::InvalidData));
        assert_eq!(VendorId::from_str("01:ab:2c"), Err(Error::InvalidData));
    }

    #[test]
    fn test_fmt() {
        assert_eq!(format!("{}", VendorId([0x01, 0xAB, 0x2C])), "01-ab-2c");
    }

    #[test]
    fn test_from_sys() {
        assert_eq!(
            VendorId::try_from_sys(SysVendorId::try_from(0x01ab2c).unwrap()),
            Ok(Some(VendorId([0x01, 0xAB, 0x2C])))
        );

        assert_eq!(
            VendorId::try_from_sys(SysVendorId::try_from(0xFFFFFFFF).unwrap()),
            Ok(None)
        );
    }

    #[test]
    fn test_into_i32() {
        assert_eq!(
            <_ as Into<i32>>::into(VendorId([0x01, 0xAB, 0x2C])),
            0x01ab2ci32
        );
    }

    #[test]
    fn test_into_u32() {
        assert_eq!(
            <_ as Into<u32>>::into(VendorId([0x01, 0xAB, 0x2C])),
            0x01ab2cu32
        );
    }

    #[test]
    fn test_into_sys() {
        assert_eq!(
            <_ as Into<SysVendorId>>::into(VendorId([0x01, 0xAB, 0x2C])),
            SysVendorId::try_from(0x01ab2c).unwrap()
        );
    }
}

/// A convenience type for an unsigned 32-bit millisecond-granularity
/// timeout, as used in the kernel interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Timeout(u32);

impl Timeout {
    /// A timeout that never expires.
    pub const NONE: Timeout = Timeout(0);

    /// The maximum timeout allowed by CEC.
    pub const MAX: Timeout = Timeout(1000);

    /// Return the number of milliseconds for this timeout.
    #[must_use]
    #[inline]
    pub fn as_ms(&self) -> u32 {
        self.0
    }

    /// Create a timeout for a number of milliseconds.
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

impl From<Timeout> for Duration {
    fn from(val: Timeout) -> Duration {
        Duration::from_millis(val.as_ms().into())
    }
}

#[cfg(test)]
mod test_timeout {
    use super::*;

    #[test]
    fn test_from_duration() {
        assert_eq!(
            Timeout::try_from(&Duration::from_secs(2)),
            Ok(Timeout::from_ms(2000))
        );
        assert_eq!(
            Timeout::try_from(&Duration::from_millis(0x1_0000_0000)),
            Err(Error::OutOfRange {
                expected: Range::AtMost(0xFFFFFFFF),
                got: 0x1_0000_0000,
                quantity: "milliseconds",
            })
        );
    }
}

pub type Result<T> = std::result::Result<T, Error>;
