use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use thiserror::Error;

pub(crate) mod ioctls;

pub mod constants;
pub mod device;
pub mod message;
pub mod operand;

pub type LogicalAddress = u8;
pub type PhysicalAddress = u16;

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
