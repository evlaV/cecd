use nix::errno::Errno;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use std::fmt::{self, Display, Formatter};
use std::io;
use std::ops::Add;
use std::string::ToString;
use thiserror::Error;

pub(crate) mod ioctls;

pub mod constants;
pub mod device;
pub mod message;
pub mod operand;

pub type LogicalAddress = u8;
pub type PhysicalAddress = u16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Range<T: PartialOrd + Display> {
    AtMost(T),
    AtLeast(T),
    Exact(T),
    Only(Vec<T>),
    Interval { min: T, max: T },
}

impl Range<usize> {
    pub fn check(self, val: impl Into<usize>, quantity: &(impl ToString + ?Sized)) -> Result<()> {
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
                quantity: quantity.to_string(),
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
    #[error("Expected a {expected} {quantity}, got {got} {quantity}")]
    OutOfRange {
        expected: Range<usize>,
        got: usize,
        quantity: String,
    },
    #[error("Invalid value {value} for type {ty}")]
    InvalidValueForType { ty: String, value: String },
    #[error("The provided data was not valid")]
    InvalidData,
    #[error("Errno {0}")]
    Errno(#[from] Errno),
}

impl Error {
    pub(crate) fn add_offset(offset: usize) -> impl FnOnce(Error) -> Error {
        move |error| match error {
            Error::OutOfRange {
                got,
                expected,
                quantity,
            } => Error::OutOfRange {
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
            ty: T::NAME.to_string(),
            value: format!("{:?}", val.number),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
