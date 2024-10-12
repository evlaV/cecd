use nix::errno::Errno;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;

pub(crate) mod ioctls;

pub mod constants;
pub mod device;
pub mod message;
pub mod operand;

pub type LogicalAddress = u8;
pub type PhysicalAddress = u16;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Expected at least {required} bytes, got {got} bytes")]
    InsufficientLength { required: usize, got: usize },
    #[error(
        "Expected a length of one of {} bytes, got {got} bytes",
        .expected.iter().map(ToString::to_string).fold(String::new(), |a, b| {
            if a.is_empty() {
                b
            } else {
                format!("{a}, {b}")
            }
        })
    )]
    InvalidLength { expected: Vec<usize>, got: usize },
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
    #[error("Errno {0}")]
    Errno(#[from] Errno),
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

pub(crate) fn add_error_offset(offset: usize) -> impl FnOnce(Error) -> Error {
    move |error| match error {
        Error::InsufficientLength { required, got } => Error::InsufficientLength {
            required: required + offset,
            got: got + offset,
        },
        Error::InvalidLength { expected, got } => Error::InvalidLength {
            expected: expected.into_iter().map(|val| val + offset).collect(),
            got: got + offset,
        },
        err => err,
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
