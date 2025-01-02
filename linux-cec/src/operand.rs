/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

#![allow(clippy::enum_variant_names)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::len_zero)]

use bitfield_struct::bitfield;
use bitflags::bitflags;
#[cfg(test)]
use linux_cec_macros::opcode_test;
use linux_cec_macros::{BitfieldSpecifier, Operand};
use linux_cec_sys::VendorId as SysVendorId;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use strum::{Display, EnumString};
use tinyvec::array_vec;

use crate::{constants, Error, PhysicalAddress, Range, Result};

pub type AnalogueFrequency = u16; // TODO: Limit range
pub type DurationHours = BcdByte;
pub type Hour = BcdByte<0, 23>;
pub type Minute = BcdByte<0, 59>;
pub type ShortAudioDescriptor = [u8; 3];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Operand)]
pub struct VendorId(pub [u8; 3]);

pub trait OperandEncodable: Sized {
    fn to_bytes(&self, buf: &mut impl Extend<u8>);
    fn try_from_bytes(bytes: &[u8]) -> Result<Self>;
    fn len(&self) -> usize;
    fn expected_len() -> Range<usize>;
}

impl From<VendorId> for SysVendorId {
    fn from(val: VendorId) -> SysVendorId {
        SysVendorId::try_from(
            ((val.0[0] as u32) << 16) | ((val.0[1] as u32) << 8) | (val.0[2] as u32),
        )
        .unwrap()
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
            id[idx] = u8::from_str_radix(part, 16).map_err(|_| Error::InvalidData)?
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Delay(u8);

impl From<Delay> for u8 {
    fn from(val: Delay) -> u8 {
        val.0
    }
}

impl Delay {
    const MIN: u8 = 1;
    const MAX: u8 = 251;

    pub fn is_valid(&self) -> bool {
        Range::Interval {
            min: Delay::MIN as usize,
            max: Delay::MAX as usize,
        }
        .check(self.0, "value")
        .is_ok()
    }
}

impl TryFrom<u8> for Delay {
    type Error = Error;

    fn try_from(val: u8) -> Result<Delay> {
        Range::Interval {
            min: Delay::MIN as usize,
            max: Delay::MAX as usize,
        }
        .check(val, "value")?;
        Ok(Delay(val))
    }
}

impl OperandEncodable for Delay {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        buf.extend([self.0]);
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 1 {
            Err(crate::Error::OutOfRange {
                expected: crate::Range::AtLeast(1),
                got: bytes.len(),
                quantity: "bytes",
            })
        } else {
            Delay::try_from(bytes[0])
        }
    }

    fn len(&self) -> usize {
        1
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(1)
    }
}

#[cfg(test)]
mod test_delay {
    use super::*;

    opcode_test! {
        name: _min,
        ty: Delay,
        instance: Delay(1),
        bytes: [1],
        extra: [Overfull],
    }

    opcode_test! {
        name: _max,
        ty: Delay,
        instance: Delay(251),
        bytes: [251],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_out_of_range() {
        assert_eq!(
            <Delay as OperandEncodable>::try_from_bytes(&[0]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::Interval { min: 1, max: 251 },
                quantity: "value"
            })
        );

        assert_eq!(
            <Delay as OperandEncodable>::try_from_bytes(&[252]),
            Err(Error::OutOfRange {
                got: 252,
                expected: Range::Interval { min: 1, max: 251 },
                quantity: "value"
            })
        );
    }

    #[test]
    fn test_encode_out_of_range() {
        let mut buf = Vec::new();
        Delay(0).to_bytes(&mut buf);
        assert_eq!(&buf, &[0]);

        let mut buf = Vec::new();
        Delay(255).to_bytes(&mut buf);
        assert_eq!(&buf, &[255]);
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            <Delay as OperandEncodable>::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(1),
                quantity: "bytes"
            })
        );
    }
}

impl OperandEncodable for u8 {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        buf.extend([*self]);
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 1 {
            Err(crate::Error::OutOfRange {
                expected: crate::Range::AtLeast(1),
                got: bytes.len(),
                quantity: "bytes",
            })
        } else {
            Ok(bytes[0])
        }
    }

    fn len(&self) -> usize {
        1
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(1)
    }
}

#[cfg(test)]
mod test_u8 {
    use super::*;

    opcode_test! {
        ty: u8,
        instance: 0x56,
        bytes: [0x56],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            <u8 as OperandEncodable>::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(1),
                quantity: "bytes"
            })
        );
    }
}

impl<T: OperandEncodable> OperandEncodable for Option<T> {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        if let Some(data) = self {
            data.to_bytes(buf);
        }
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 1 {
            Ok(None)
        } else {
            Ok(Some(T::try_from_bytes(bytes)?))
        }
    }

    fn len(&self) -> usize {
        if let Some(ref data) = self {
            data.len()
        } else {
            0
        }
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(0)
    }
}

#[cfg(test)]
mod test_option {
    use super::*;

    opcode_test! {
        name: _none,
        ty: Option<u8>,
        instance: None,
        bytes: [],
    }

    opcode_test! {
        name: _some,
        ty: Option<u8>,
        instance: Some(0x56),
        bytes: [0x56],
        extra: [Overfull],
    }
}

impl<const S: usize> OperandEncodable for [u8; S] {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        buf.extend(*self);
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < S {
            return Err(crate::Error::OutOfRange {
                expected: crate::Range::AtLeast(S),
                got: bytes.len(),
                quantity: "bytes",
            });
        }
        Ok(bytes[..S].try_into().unwrap())
    }

    fn len(&self) -> usize {
        S
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(S)
    }
}

#[cfg(test)]
mod test_array {
    use super::*;

    opcode_test! {
        name: _1,
        ty: [u8; 1],
        instance: [0x56],
        bytes: [0x56],
        extra: [Overfull],
    }

    opcode_test! {
        name: _2,
        ty: [u8; 2],
        instance: [0x56, 0x78],
        bytes: [0x56, 0x78],
        extra: [Overfull],
    }

    opcode_test! {
        name: _3,
        ty: [u8; 3],
        instance: [0x56, 0x78, 0x9A],
        bytes: [0x56, 0x78, 0x9A],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_missing_byte() {
        assert_eq!(
            <[u8; 2] as OperandEncodable>::try_from_bytes(&[0x12]),
            Err(Error::OutOfRange {
                got: 1,
                expected: Range::AtLeast(2),
                quantity: "bytes"
            })
        );
    }

    #[test]
    fn test_decode_missing_bytes() {
        assert_eq!(
            <[u8; 3] as OperandEncodable>::try_from_bytes(&[0x12]),
            Err(Error::OutOfRange {
                got: 1,
                expected: Range::AtLeast(3),
                quantity: "bytes"
            })
        );
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            <[u8; 1] as OperandEncodable>::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(1),
                quantity: "bytes"
            })
        );
    }
}

impl OperandEncodable for u16 {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        buf.extend([
            u8::try_from(*self >> 8).unwrap(),
            u8::try_from(*self & 0xFF).unwrap(),
        ]);
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 2 {
            Err(crate::Error::OutOfRange {
                expected: crate::Range::AtLeast(2),
                got: bytes.len(),
                quantity: "bytes",
            })
        } else {
            Ok((u16::from(bytes[0]) << 8) | u16::from(bytes[1]))
        }
    }

    fn len(&self) -> usize {
        2
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(2)
    }
}

#[cfg(test)]
mod test_u16 {
    use super::*;

    opcode_test! {
        ty: u16,
        instance: 0x5678,
        bytes: [0x56, 0x78],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_underfull() {
        assert_eq!(
            <u16 as OperandEncodable>::try_from_bytes(&[0x56]),
            Err(Error::OutOfRange {
                got: 1,
                expected: Range::AtLeast(2),
                quantity: "bytes"
            })
        );
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            <u16 as OperandEncodable>::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(2),
                quantity: "bytes"
            })
        );
    }
}

impl OperandEncodable for bool {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        buf.extend([if *self { 1 } else { 0 }]);
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 1 {
            Err(crate::Error::OutOfRange {
                expected: crate::Range::AtLeast(1),
                got: bytes.len(),
                quantity: "bytes",
            })
        } else {
            Ok(bytes[0] != 0)
        }
    }

    fn len(&self) -> usize {
        1
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(1)
    }
}

#[cfg(test)]
mod test_bool {
    use super::*;

    opcode_test! {
        name: _true,
        ty: bool,
        instance: true,
        bytes: [0x01],
        extra: [Overfull],
    }

    opcode_test! {
        name: _false,
        ty: bool,
        instance: false,
        bytes: [0x00],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_nb() {
        assert_eq!(
            <bool as OperandEncodable>::try_from_bytes(&[0x02]),
            Ok(true)
        );
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            <bool as OperandEncodable>::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(1),
                quantity: "bytes"
            })
        );
    }
}

pub trait TaggedLengthBuffer: Sized {
    type FixedParam: Into<u8> + TryFrom<u8> + Copy;

    fn try_new(first: Self::FixedParam, extra: &[u8]) -> Result<Self>;

    fn fixed_param(&self) -> Self::FixedParam;

    fn extra_params(&self) -> &[u8] {
        &[] as &[u8; 0]
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(1)
    }
}

impl<T: TryFrom<u8> + Into<u8> + Copy, U: TaggedLengthBuffer<FixedParam = T>> OperandEncodable for U
where
    Error: From<<T as TryFrom<u8>>::Error>,
{
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        let head: u8 = self.fixed_param().into();
        let extra_params = self.extra_params();
        if !extra_params.is_empty() {
            buf.extend([head | 0x80]);
            buf.extend(
                extra_params
                    .iter()
                    .take(extra_params.len() - 1)
                    .map(|b| b | 0x80),
            );
            buf.extend([extra_params.last().unwrap() & 0x7F]);
        } else {
            buf.extend([head & 0x7F]);
        }
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        Range::AtLeast(1).check(bytes.len(), "bytes")?;
        let first = T::try_from(bytes[0] & 0x7F)?;
        let mut extra = Vec::new();
        if bytes[0] & 0x80 == 0x80 {
            let mut offset = 1;
            while offset < bytes.len() {
                let byte = bytes[offset];
                extra.push(byte & 0x7F);
                if (byte & 0x80) == 0 {
                    break;
                }
                offset += 1;
            }
        }
        Self::try_new(first, &extra)
    }

    fn len(&self) -> usize {
        1 + self.extra_params().len()
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(1)
    }
}

#[cfg(test)]
mod test_tagged_length_buffer {
    use super::*;

    #[derive(PartialEq, Debug, Copy, Clone)]
    #[repr(transparent)]
    struct U8(pub u8);

    impl Into<u8> for U8 {
        fn into(self) -> u8 {
            self.0
        }
    }

    impl TryFrom<u8> for U8 {
        type Error = Error;
        fn try_from(val: u8) -> Result<U8> {
            Range::AtMost(0x7F).check(val, "value")?;
            Ok(U8(val))
        }
    }

    #[derive(PartialEq, Debug, Copy, Clone)]
    struct U8Buffer {
        first: U8,
        rest: [u8; 13],
        len: usize,
    }

    impl TaggedLengthBuffer for U8Buffer {
        type FixedParam = U8;

        fn try_new(first: Self::FixedParam, extra: &[u8]) -> Result<Self> {
            let len = extra.len();
            Range::AtMost(13).check(len, "bytes")?;
            let mut rest = [0; 13];
            rest[..len].copy_from_slice(&extra[..len]);
            Ok(U8Buffer { first, rest, len })
        }

        fn fixed_param(&self) -> Self::FixedParam {
            self.first
        }

        fn extra_params(&self) -> &[u8] {
            &self.rest[..self.len]
        }
    }

    #[test]
    fn test_u8_fixed_param() {
        assert_eq!(
            U8Buffer {
                first: U8(0x56),
                rest: [0; 13],
                len: 0,
            }
            .fixed_param(),
            U8(0x56)
        );
    }

    #[test]
    fn test_u8_extra_params() {
        assert_eq!(
            U8Buffer {
                first: U8(0),
                rest: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
                len: 2,
            }
            .extra_params(),
            &[1, 2]
        );
    }

    #[test]
    fn test_u8_try_new() {
        assert_eq!(
            Ok(U8Buffer {
                first: U8(0x56),
                rest: [0x78, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                len: 1,
            }),
            U8Buffer::try_new(U8(0x56), &[0x78])
        );
    }

    opcode_test! {
        name: _fixed_only,
        ty: U8Buffer,
        instance: U8Buffer {
            first: U8(0x12),
            rest: [0; 13],
            len: 0,
        },
        bytes: [0x12],
    }

    opcode_test! {
        name: _fixed_and_one_byte,
        ty: U8Buffer,
        instance: U8Buffer {
            first: U8(0x12),
            rest: [0x34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            len: 1,
        },
        bytes: [0x92, 0x34],
    }

    opcode_test! {
        name: _fixed_and_two_bytes,
        ty: U8Buffer,
        instance: U8Buffer {
            first: U8(0x12),
            rest: [0x34, 0x56, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            len: 2,
        },
        bytes: [0x92, 0xB4, 0x56],
    }

    #[test]
    fn test_decode_zero_bytes() {
        assert_eq!(
            U8Buffer::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(1),
                got: 0,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decode_one_byte_junk() {
        assert_eq!(
            U8Buffer::try_from_bytes(&[0x12, 0x34]),
            Ok(U8Buffer {
                first: U8(0x12),
                rest: [0; 13],
                len: 0,
            })
        );
    }

    #[test]
    fn test_decode_two_bytes_junk() {
        assert_eq!(
            U8Buffer::try_from_bytes(&[0x92, 0x34, 0x56]),
            Ok(U8Buffer {
                first: U8(0x12),
                rest: [0x34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                len: 1,
            })
        );
    }

    #[test]
    fn test_decode_missing_byte() {
        assert_eq!(
            U8Buffer::try_from_bytes(&[0x92]),
            Ok(U8Buffer {
                first: U8(0x12),
                rest: [0; 13],
                len: 0,
            })
        );
    }

    #[test]
    fn test_decode_missing_byte_2() {
        assert_eq!(
            U8Buffer::try_from_bytes(&[0x92, 0xB4]),
            Ok(U8Buffer {
                first: U8(0x12),
                rest: [0x34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                len: 1,
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BoundedBufferOperand<const S: usize, T: OperandEncodable + Default + Copy> {
    pub(crate) buffer: [T; S],
    pub(crate) len: usize,
}

impl<const S: usize, T: OperandEncodable + Default + Copy> OperandEncodable
    for BoundedBufferOperand<S, T>
{
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        for elem in &self.buffer[..self.len] {
            elem.to_bytes(buf);
        }
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut buf = Vec::new();
        let mut offset = 0;
        let mut len = 0;
        while offset < S * size_of::<T>() && offset + size_of::<T>() <= bytes.len() {
            buf.push(
                T::try_from_bytes(&bytes[offset..]).map_err(crate::Error::add_offset(offset))?,
            );
            offset += size_of::<T>();
            len += 1;
        }
        buf.resize(S, T::default());
        Ok(Self {
            buffer: *buf.first_chunk().unwrap(),
            len,
        })
    }

    fn len(&self) -> usize {
        usize::min(self.len, S) * size_of::<T>()
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(0)
    }
}

impl<const S: usize, T: OperandEncodable + Copy + Default> Default for BoundedBufferOperand<S, T> {
    fn default() -> BoundedBufferOperand<S, T> {
        BoundedBufferOperand {
            buffer: [T::default(); S],
            len: 0,
        }
    }
}

impl<const S: usize> FromStr for BoundedBufferOperand<S, u8> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let bytes = s.as_bytes();
        Range::AtMost(S).check(bytes.len(), "bytes")?;
        let mut buffer = [0; S];
        buffer[..bytes.len()].copy_from_slice(bytes);
        Ok(BoundedBufferOperand::<S, u8> {
            buffer,
            len: bytes.len(),
        })
    }
}

impl<const S: usize, T: OperandEncodable + Default + Copy> TryFrom<&[T]>
    for BoundedBufferOperand<S, T>
{
    type Error = Error;

    fn try_from(arr: &[T]) -> Result<Self> {
        Range::AtMost(S).check(arr.len(), "elements")?;
        let mut buffer = [T::default(); S];
        buffer[..arr.len()].copy_from_slice(arr);
        Ok(BoundedBufferOperand::<S, T> {
            buffer,
            len: arr.len(),
        })
    }
}

impl<const S: usize, T: OperandEncodable + Default + Copy> BoundedBufferOperand<S, T> {
    pub fn new() -> BoundedBufferOperand<S, T> {
        BoundedBufferOperand::default()
    }
}

pub type BufferOperand = BoundedBufferOperand<14, u8>;

#[cfg(test)]
mod test_buffer_operand {
    use super::*;

    opcode_test! {
        name: _empty,
        ty: BoundedBufferOperand::<2, u8>,
        instance: BoundedBufferOperand::<2, u8> {
            buffer: [0; 2],
            len: 0,
        },
        bytes: [],
    }

    opcode_test! {
        name: _underfull,
        ty: BoundedBufferOperand::<2, u8>,
        instance: BoundedBufferOperand::<2, u8> {
            buffer: [0x12, 0],
            len: 1,
        },
        bytes: [0x12],
    }

    opcode_test! {
        name: _full,
        ty: BoundedBufferOperand::<2, u8>,
        instance: BoundedBufferOperand::<2, u8> {
            buffer: [0x12, 0x34],
            len: 2,
        },
        bytes: [0x12, 0x34],
    }

    opcode_test! {
        name: _u16_empty,
        ty: BoundedBufferOperand::<2, u16>,
        instance: BoundedBufferOperand::<2, u16> {
            buffer: [0; 2],
            len: 0,
        },
        bytes: [],
    }

    opcode_test! {
        name: _u16_underfull,
        ty: BoundedBufferOperand::<2, u16>,
        instance: BoundedBufferOperand::<2, u16> {
            buffer: [0x1234, 0],
            len: 1,
        },
        bytes: [0x12, 0x34],
    }

    opcode_test! {
        name: _u16_full,
        ty: BoundedBufferOperand::<2, u16>,
        instance: BoundedBufferOperand::<2, u16> {
            buffer: [0x1234, 0x5678],
            len: 2,
        },
        bytes: [0x12, 0x34, 0x56, 0x78],
    }

    #[test]
    fn test_encode_underfull_with_junk() {
        let mut buf = Vec::new();

        BoundedBufferOperand::<2, u8> {
            buffer: [0x12, 0x34],
            len: 1,
        }
        .to_bytes(&mut buf);
        assert_eq!(&buf, &[0x12]);
    }

    #[test]
    fn test_decode_overfull() {
        assert_eq!(
            BoundedBufferOperand::<2, u8>::try_from_bytes(&[0x12, 0x34, 0x56]).unwrap(),
            BoundedBufferOperand::<2, u8> {
                buffer: [0x12, 0x34],
                len: 2,
            }
        );
    }

    #[test]
    fn test_decode_u16_misaligned() {
        assert_eq!(
            BoundedBufferOperand::<2, u16>::try_from_bytes(&[0x12, 0x34, 0x56]).unwrap(),
            BoundedBufferOperand::<2, u16> {
                buffer: [0x1234, 0],
                len: 1,
            }
        );
    }

    #[test]
    fn test_from_string_fit() {
        let s = "abc";
        let buffer = BoundedBufferOperand::<3, u8>::from_str(s).unwrap();
        assert_eq!(buffer.len, 3);
        assert_eq!(&buffer.buffer, s.as_bytes());
    }

    #[test]
    fn test_from_string_underfull() {
        let s = "abc";
        let buffer = BoundedBufferOperand::<4, u8>::from_str(s).unwrap();
        assert_eq!(buffer.len, 3);
        assert_ne!(&buffer.buffer, s.as_bytes());
        assert_eq!(&buffer.buffer, &['a' as u8, 'b' as u8, 'c' as u8, 0]);
    }

    #[test]
    fn test_from_string_overfull() {
        let s = "abc";
        let buffer = BoundedBufferOperand::<2, u8>::from_str(s);
        assert_eq!(
            buffer,
            Err(Error::OutOfRange {
                expected: Range::AtMost(2),
                got: 3,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_from_string_overfull_utf8() {
        let s = "💩";
        let buffer = BoundedBufferOperand::<2, u8>::from_str(s);
        assert_eq!(
            buffer,
            Err(Error::OutOfRange {
                expected: Range::AtMost(2),
                got: 4,
                quantity: "bytes",
            })
        );
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum AbortReason {
    /// Unrecognized opcode
    UnrecognizedOp = constants::CEC_OP_ABORT_UNRECOGNIZED_OP,
    /// Not in correct mode to respond
    IncorrectMode = constants::CEC_OP_ABORT_INCORRECT_MODE,
    /// Cannot provide source
    NoSource = constants::CEC_OP_ABORT_NO_SOURCE,
    /// Invalid operand
    InvalidOp = constants::CEC_OP_ABORT_INVALID_OP,
    /// Refused
    Refused = constants::CEC_OP_ABORT_REFUSED,
    /// Unable to determine
    Undetermined = constants::CEC_OP_ABORT_UNDETERMINED,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum AnalogueBroadcastType {
    Cable = constants::CEC_OP_ANA_BCAST_TYPE_CABLE,
    Satellite = constants::CEC_OP_ANA_BCAST_TYPE_SATELLITE,
    Terrestrial = constants::CEC_OP_ANA_BCAST_TYPE_TERRESTRIAL,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum AudioRate {
    /// Rate control off
    Off = constants::CEC_OP_AUD_RATE_OFF,
    /// Standard rate, wide range control (IEEE 1394 compatible)
    WideStandard = constants::CEC_OP_AUD_RATE_WIDE_STD,
    /// Fast rate, wide range control (IEEE 1394 compatible)
    WideFast = constants::CEC_OP_AUD_RATE_WIDE_FAST,
    /// Slow rate, wide range control (IEEE 1394 compatible)
    WideSlow = constants::CEC_OP_AUD_RATE_WIDE_SLOW,
    /// Standard rate, narrow range control (HDMI transparent)
    NarrowStandard = constants::CEC_OP_AUD_RATE_NARROW_STD,
    /// Fast rate, narrow range control (HDMI transparent)
    NarrowFast = constants::CEC_OP_AUD_RATE_NARROW_FAST,
    /// Slow rate, narrow range control (HDMI transparent)
    NarrowSlow = constants::CEC_OP_AUD_RATE_NARROW_SLOW,
}

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[bits = 2]
#[repr(u8)]
pub enum AudioFormatId {
    CEA861 = constants::CEC_OP_AUD_FMT_ID_CEA861,
    CEA861Cxt = constants::CEC_OP_AUD_FMT_ID_CEA861_CXT,
    #[default]
    Invalid(u8),
}

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[bits = 2]
#[repr(u8)]
pub enum AudioOutputCompensated {
    NotApplicable = constants::CEC_OP_AUD_OUT_COMPENSATED_NA,
    Delay = constants::CEC_OP_AUD_OUT_COMPENSATED_DELAY,
    NoDelay = constants::CEC_OP_AUD_OUT_COMPENSATED_NO_DELAY,
    PartialDelay = constants::CEC_OP_AUD_OUT_COMPENSATED_PARTIAL_DELAY,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum BroadcastSystem {
    PalBG = constants::CEC_OP_BCAST_SYSTEM_PAL_BG,
    SecamLq = constants::CEC_OP_BCAST_SYSTEM_SECAM_LQ, /* SECAM L' */
    PalM = constants::CEC_OP_BCAST_SYSTEM_PAL_M,
    NtscM = constants::CEC_OP_BCAST_SYSTEM_NTSC_M,
    PalI = constants::CEC_OP_BCAST_SYSTEM_PAL_I,
    SecamDK = constants::CEC_OP_BCAST_SYSTEM_SECAM_DK,
    SecamBG = constants::CEC_OP_BCAST_SYSTEM_SECAM_BG,
    SecamL = constants::CEC_OP_BCAST_SYSTEM_SECAM_L,
    PalDK = constants::CEC_OP_BCAST_SYSTEM_PAL_DK,
    Other = constants::CEC_OP_BCAST_SYSTEM_OTHER,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum CdcErrorCode {
    None = constants::CEC_OP_CDC_ERROR_CODE_NONE,
    CapUnsupported = constants::CEC_OP_CDC_ERROR_CODE_CAP_UNSUPPORTED,
    WrongState = constants::CEC_OP_CDC_ERROR_CODE_WRONG_STATE,
    Other = constants::CEC_OP_CDC_ERROR_CODE_OTHER,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
enum ChannelNumberFormat {
    Fmt1Part = constants::CEC_OP_CHANNEL_NUMBER_FMT_1_PART,
    Fmt2Part = constants::CEC_OP_CHANNEL_NUMBER_FMT_2_PART,
}

#[repr(u8)]
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    IntoPrimitive,
    TryFromPrimitive,
    Operand,
)]
pub enum DayOfMonth {
    Day1 = 1,
    Day2 = 2,
    Day3 = 3,
    Day4 = 4,
    Day5 = 5,
    Day6 = 6,
    Day7 = 7,
    Day8 = 8,
    Day9 = 9,
    Day10 = 10,
    Day11 = 11,
    Day12 = 12,
    Day13 = 13,
    Day14 = 14,
    Day15 = 15,
    Day16 = 16,
    Day17 = 17,
    Day18 = 18,
    Day19 = 19,
    Day20 = 20,
    Day21 = 21,
    Day22 = 22,
    Day23 = 23,
    Day24 = 24,
    Day25 = 25,
    Day26 = 26,
    Day27 = 27,
    Day28 = 28,
    Day29 = 29,
    Day30 = 30,
    Day31 = 31,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum DeckControlMode {
    SkipForward = constants::CEC_OP_DECK_CTL_MODE_SKIP_FWD,
    SkipReverse = constants::CEC_OP_DECK_CTL_MODE_SKIP_REV,
    Stop = constants::CEC_OP_DECK_CTL_MODE_STOP,
    Eject = constants::CEC_OP_DECK_CTL_MODE_EJECT,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum DeckInfo {
    Play = constants::CEC_OP_DECK_INFO_PLAY,
    Record = constants::CEC_OP_DECK_INFO_RECORD,
    PlayReverse = constants::CEC_OP_DECK_INFO_PLAY_REV,
    Still = constants::CEC_OP_DECK_INFO_STILL,
    Slow = constants::CEC_OP_DECK_INFO_SLOW,
    SlowReverse = constants::CEC_OP_DECK_INFO_SLOW_REV,
    FastForward = constants::CEC_OP_DECK_INFO_FAST_FWD,
    FastReverse = constants::CEC_OP_DECK_INFO_FAST_REV,
    NoMedia = constants::CEC_OP_DECK_INFO_NO_MEDIA,
    Stop = constants::CEC_OP_DECK_INFO_STOP,
    SkipForward = constants::CEC_OP_DECK_INFO_SKIP_FWD,
    SkipReverse = constants::CEC_OP_DECK_INFO_SKIP_REV,
    IndexSearchForward = constants::CEC_OP_DECK_INFO_INDEX_SEARCH_FWD,
    IndexSearchReverse = constants::CEC_OP_DECK_INFO_INDEX_SEARCH_REV,
    Other = constants::CEC_OP_DECK_INFO_OTHER,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum DigitalServiceBroadcastSystem {
    AribGeneric = constants::CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ARIB_GEN,
    AtscGeneric = constants::CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ATSC_GEN,
    DvbGeneric = constants::CEC_OP_DIG_SERVICE_BCAST_SYSTEM_DVB_GEN,
    AribBs = constants::CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ARIB_BS,
    AribCs = constants::CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ARIB_CS,
    AribT = constants::CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ARIB_T,
    AtscCable = constants::CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ATSC_CABLE,
    AtscSatellite = constants::CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ATSC_SAT,
    AtscTerrestrial = constants::CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ATSC_T,
    DvbC = constants::CEC_OP_DIG_SERVICE_BCAST_SYSTEM_DVB_C,
    DvbS = constants::CEC_OP_DIG_SERVICE_BCAST_SYSTEM_DVB_S,
    DvbS2 = constants::CEC_OP_DIG_SERVICE_BCAST_SYSTEM_DVB_S2,
    DvbT = constants::CEC_OP_DIG_SERVICE_BCAST_SYSTEM_DVB_T,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum DisplayControl {
    Default = constants::CEC_OP_DISP_CTL_DEFAULT,
    UntilCleared = constants::CEC_OP_DISP_CTL_UNTIL_CLEARED,
    Clear = constants::CEC_OP_DISP_CTL_CLEAR,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum EncFunctionalityState {
    ExtConNotSupported = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_NOT_SUPPORTED,
    ExtConInactive = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_INACTIVE,
    ExtConActive = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_ACTIVE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum ExternalSourceSpecifier {
    ExternalPlug = constants::CEC_OP_EXT_SRC_PLUG,
    ExternalPhysicalAddress = constants::CEC_OP_EXT_SRC_PHYS_ADDR,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum HecFunctionalityState {
    NotSupported = constants::CEC_OP_HEC_FUNC_STATE_NOT_SUPPORTED,
    Inactive = constants::CEC_OP_HEC_FUNC_STATE_INACTIVE,
    Active = constants::CEC_OP_HEC_FUNC_STATE_ACTIVE,
    ActivationField = constants::CEC_OP_HEC_FUNC_STATE_ACTIVATION_FIELD,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum HostFunctionalityState {
    NotSupported = constants::CEC_OP_HOST_FUNC_STATE_NOT_SUPPORTED,
    Inactive = constants::CEC_OP_HOST_FUNC_STATE_INACTIVE,
    Active = constants::CEC_OP_HOST_FUNC_STATE_ACTIVE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum HpdErrorCode {
    None = constants::CEC_OP_HPD_ERROR_NONE,
    InitiatorNotCapable = constants::CEC_OP_HPD_ERROR_INITIATOR_NOT_CAPABLE,
    InitiatorWrongState = constants::CEC_OP_HPD_ERROR_INITIATOR_WRONG_STATE,
    Other = constants::CEC_OP_HPD_ERROR_OTHER,
    NoneNoVideo = constants::CEC_OP_HPD_ERROR_NONE_NO_VIDEO,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum HpdStateState {
    CpEdidDisable = constants::CEC_OP_HPD_STATE_CP_EDID_DISABLE,
    CpEdidEnable = constants::CEC_OP_HPD_STATE_CP_EDID_ENABLE,
    CpEdidDisableEnable = constants::CEC_OP_HPD_STATE_CP_EDID_DISABLE_ENABLE,
    EdidDisable = constants::CEC_OP_HPD_STATE_EDID_DISABLE,
    EdidEnable = constants::CEC_OP_HPD_STATE_EDID_ENABLE,
    EdidDisableEnable = constants::CEC_OP_HPD_STATE_EDID_DISABLE_ENABLE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum MediaInfo {
    UnprotectedMedia = constants::CEC_OP_MEDIA_INFO_UNPROT_MEDIA,
    ProtectedMedia = constants::CEC_OP_MEDIA_INFO_PROT_MEDIA,
    NoMedia = constants::CEC_OP_MEDIA_INFO_NO_MEDIA,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum MenuRequestType {
    Activate = constants::CEC_OP_MENU_REQUEST_ACTIVATE,
    Deactivate = constants::CEC_OP_MENU_REQUEST_DEACTIVATE,
    Query = constants::CEC_OP_MENU_REQUEST_QUERY,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum MenuState {
    Activated = constants::CEC_OP_MENU_STATE_ACTIVATED,
    Deactivated = constants::CEC_OP_MENU_STATE_DEACTIVATED,
}

#[repr(u8)]
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    IntoPrimitive,
    TryFromPrimitive,
    Operand,
)]
pub enum MonthOfYear {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum PlayMode {
    Forward = constants::CEC_OP_PLAY_MODE_PLAY_FWD,
    Reverse = constants::CEC_OP_PLAY_MODE_PLAY_REV,
    Still = constants::CEC_OP_PLAY_MODE_PLAY_STILL,
    FastForwardMinimum = constants::CEC_OP_PLAY_MODE_PLAY_FAST_FWD_MIN,
    FastForwardMedium = constants::CEC_OP_PLAY_MODE_PLAY_FAST_FWD_MED,
    FastForwardMaximum = constants::CEC_OP_PLAY_MODE_PLAY_FAST_FWD_MAX,
    FastReverseMinimum = constants::CEC_OP_PLAY_MODE_PLAY_FAST_REV_MIN,
    FastReverseMedium = constants::CEC_OP_PLAY_MODE_PLAY_FAST_REV_MED,
    FastReverseMaximum = constants::CEC_OP_PLAY_MODE_PLAY_FAST_REV_MAX,
    SlowForwardMinimum = constants::CEC_OP_PLAY_MODE_PLAY_SLOW_FWD_MIN,
    SlowForwardMedium = constants::CEC_OP_PLAY_MODE_PLAY_SLOW_FWD_MED,
    SlowForwardMaximum = constants::CEC_OP_PLAY_MODE_PLAY_SLOW_FWD_MAX,
    SlowReverseMinimum = constants::CEC_OP_PLAY_MODE_PLAY_SLOW_REV_MIN,
    SlowReverseMedium = constants::CEC_OP_PLAY_MODE_PLAY_SLOW_REV_MED,
    SlowReverseMaximum = constants::CEC_OP_PLAY_MODE_PLAY_SLOW_REV_MAX,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum PowerStatus {
    /// On
    On = constants::CEC_OP_POWER_STATUS_ON,
    /// Standby
    Standby = constants::CEC_OP_POWER_STATUS_STANDBY,
    /// In transition from Standby to On
    ToOn = constants::CEC_OP_POWER_STATUS_TO_ON,
    /// In transition from On to Standby
    ToStandby = constants::CEC_OP_POWER_STATUS_TO_STANDBY,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum PrimaryDeviceType {
    Tv = constants::CEC_OP_PRIM_DEVTYPE_TV,
    Recording = constants::CEC_OP_PRIM_DEVTYPE_RECORD,
    Tuner = constants::CEC_OP_PRIM_DEVTYPE_TUNER,
    Playback = constants::CEC_OP_PRIM_DEVTYPE_PLAYBACK,
    Audio = constants::CEC_OP_PRIM_DEVTYPE_AUDIOSYSTEM,
    Switch = constants::CEC_OP_PRIM_DEVTYPE_SWITCH,
    Processor = constants::CEC_OP_PRIM_DEVTYPE_PROCESSOR,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum RecordSourceType {
    Own = constants::CEC_OP_RECORD_SRC_OWN,
    Digital = constants::CEC_OP_RECORD_SRC_DIGITAL,
    Analogue = constants::CEC_OP_RECORD_SRC_ANALOG,
    ExternalPlug = constants::CEC_OP_RECORD_SRC_EXT_PLUG,
    ExternalPhysicalAddress = constants::CEC_OP_RECORD_SRC_EXT_PHYS_ADDR,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum RecordStatusInfo {
    CurrentSource = constants::CEC_OP_RECORD_STATUS_CUR_SRC,
    DigitalService = constants::CEC_OP_RECORD_STATUS_DIG_SERVICE,
    AnalogueService = constants::CEC_OP_RECORD_STATUS_ANA_SERVICE,
    ExternalInput = constants::CEC_OP_RECORD_STATUS_EXT_INPUT,
    NoDigitalService = constants::CEC_OP_RECORD_STATUS_NO_DIG_SERVICE,
    NoAnalogueService = constants::CEC_OP_RECORD_STATUS_NO_ANA_SERVICE,
    NoService = constants::CEC_OP_RECORD_STATUS_NO_SERVICE,
    InvalidExternalPlug = constants::CEC_OP_RECORD_STATUS_INVALID_EXT_PLUG,
    InvalidExternalPhysicalAddress = constants::CEC_OP_RECORD_STATUS_INVALID_EXT_PHYS_ADDR,
    CaUnsupported = constants::CEC_OP_RECORD_STATUS_UNSUP_CA,
    InsufficientCaEntitlements = constants::CEC_OP_RECORD_STATUS_NO_CA_ENTITLEMENTS,
    DisallowedCopySource = constants::CEC_OP_RECORD_STATUS_CANT_COPY_SRC,
    NoFurtherCopies = constants::CEC_OP_RECORD_STATUS_NO_MORE_COPIES,
    NoMedia = constants::CEC_OP_RECORD_STATUS_NO_MEDIA,
    Playing = constants::CEC_OP_RECORD_STATUS_PLAYING,
    AlreadyRecording = constants::CEC_OP_RECORD_STATUS_ALREADY_RECORDING,
    MediaProtected = constants::CEC_OP_RECORD_STATUS_MEDIA_PROT,
    NoSignal = constants::CEC_OP_RECORD_STATUS_NO_SIGNAL,
    MediaProblem = constants::CEC_OP_RECORD_STATUS_MEDIA_PROBLEM,
    NotEnoughSpace = constants::CEC_OP_RECORD_STATUS_NO_SPACE,
    ParentalLock = constants::CEC_OP_RECORD_STATUS_PARENTAL_LOCK,
    TerminatedOk = constants::CEC_OP_RECORD_STATUS_TERMINATED_OK,
    AlreadyTerminated = constants::CEC_OP_RECORD_STATUS_ALREADY_TERM,
    Other = constants::CEC_OP_RECORD_STATUS_OTHER,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum RcProfileId {
    ProfileNone = constants::CEC_OP_FEAT_RC_TV_PROFILE_NONE,
    Profile1 = constants::CEC_OP_FEAT_RC_TV_PROFILE_1,
    Profile2 = constants::CEC_OP_FEAT_RC_TV_PROFILE_2,
    Profile3 = constants::CEC_OP_FEAT_RC_TV_PROFILE_3,
    Profile4 = constants::CEC_OP_FEAT_RC_TV_PROFILE_4,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
enum ServiceIdMethod {
    ByDigitalId = constants::CEC_OP_SERVICE_ID_METHOD_BY_DIG_ID,
    ByChannel = constants::CEC_OP_SERVICE_ID_METHOD_BY_CHANNEL,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ServiceId {
    Digital(DigitalServiceId),
    Analogue(AnalogueServiceId),
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum StatusRequest {
    On = constants::CEC_OP_STATUS_REQ_ON,
    Off = constants::CEC_OP_STATUS_REQ_OFF,
    Once = constants::CEC_OP_STATUS_REQ_ONCE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum TimerClearedStatusData {
    Recording = constants::CEC_OP_TIMER_CLR_STAT_RECORDING,
    NoMatching = constants::CEC_OP_TIMER_CLR_STAT_NO_MATCHING,
    NoInfo = constants::CEC_OP_TIMER_CLR_STAT_NO_INFO,
    Cleared = constants::CEC_OP_TIMER_CLR_STAT_CLEARED,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum TunerDisplayInfo {
    Digital = constants::CEC_OP_TUNER_DISPLAY_INFO_DIGITAL,
    None = constants::CEC_OP_TUNER_DISPLAY_INFO_NONE,
    Analogue = constants::CEC_OP_TUNER_DISPLAY_INFO_ANALOGUE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum UiBroadcastType {
    ToggleAll = constants::CEC_OP_UI_BCAST_TYPE_TOGGLE_ALL,
    ToggleDigitalAnalogue = constants::CEC_OP_UI_BCAST_TYPE_TOGGLE_DIG_ANA,
    Analogue = constants::CEC_OP_UI_BCAST_TYPE_ANALOGUE,
    AnalogueTerrestrial = constants::CEC_OP_UI_BCAST_TYPE_ANALOGUE_T,
    AnalogueCable = constants::CEC_OP_UI_BCAST_TYPE_ANALOGUE_CABLE,
    AnalogueSatellite = constants::CEC_OP_UI_BCAST_TYPE_ANALOGUE_SAT,
    Digital = constants::CEC_OP_UI_BCAST_TYPE_DIGITAL,
    DigitalTerrestrial = constants::CEC_OP_UI_BCAST_TYPE_DIGITAL_T,
    DigitalCable = constants::CEC_OP_UI_BCAST_TYPE_DIGITAL_CABLE,
    DigitalSatellite = constants::CEC_OP_UI_BCAST_TYPE_DIGITAL_SAT,
    DigitalCommsSatellite = constants::CEC_OP_UI_BCAST_TYPE_DIGITAL_COM_SAT,
    DigitalCommsSatellite2 = constants::CEC_OP_UI_BCAST_TYPE_DIGITAL_COM_SAT2,
    Ip = constants::CEC_OP_UI_BCAST_TYPE_IP,
}

#[repr(u8)]
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    IntoPrimitive,
    TryFromPrimitive,
    Operand,
    Display,
    EnumString,
)]
#[strum(serialize_all = "kebab-case")]
pub enum UiCommand {
    Select = constants::CEC_OP_UI_CMD_SELECT,
    Up = constants::CEC_OP_UI_CMD_UP,
    Down = constants::CEC_OP_UI_CMD_DOWN,
    Left = constants::CEC_OP_UI_CMD_LEFT,
    Right = constants::CEC_OP_UI_CMD_RIGHT,
    RightUp = constants::CEC_OP_UI_CMD_RIGHT_UP,
    RightDown = constants::CEC_OP_UI_CMD_RIGHT_DOWN,
    LeftUp = constants::CEC_OP_UI_CMD_LEFT_UP,
    LeftDown = constants::CEC_OP_UI_CMD_LEFT_DOWN,
    DeviceRootMenu = constants::CEC_OP_UI_CMD_DEVICE_ROOT_MENU,
    DeviceSetupMenu = constants::CEC_OP_UI_CMD_DEVICE_SETUP_MENU,
    ContentsMenu = constants::CEC_OP_UI_CMD_CONTENTS_MENU,
    FavoriteMenu = constants::CEC_OP_UI_CMD_FAVORITE_MENU,
    Back = constants::CEC_OP_UI_CMD_BACK,
    MediaTopMenu = constants::CEC_OP_UI_CMD_MEDIA_TOP_MENU,
    MediaContextSensitiveMenu = constants::CEC_OP_UI_CMD_MEDIA_CONTEXT_SENSITIVE_MENU,
    NumberEntryMode = constants::CEC_OP_UI_CMD_NUMBER_ENTRY_MODE,
    #[strum(serialize = "11")]
    Number11 = constants::CEC_OP_UI_CMD_NUMBER_11,
    #[strum(serialize = "12")]
    Number12 = constants::CEC_OP_UI_CMD_NUMBER_12,
    #[strum(serialize = "0", serialize = "10")]
    Number0OrNumber10 = constants::CEC_OP_UI_CMD_NUMBER_0_OR_NUMBER_10,
    #[strum(serialize = "1")]
    Number1 = constants::CEC_OP_UI_CMD_NUMBER_1,
    #[strum(serialize = "2")]
    Number2 = constants::CEC_OP_UI_CMD_NUMBER_2,
    #[strum(serialize = "3")]
    Number3 = constants::CEC_OP_UI_CMD_NUMBER_3,
    #[strum(serialize = "4")]
    Number4 = constants::CEC_OP_UI_CMD_NUMBER_4,
    #[strum(serialize = "5")]
    Number5 = constants::CEC_OP_UI_CMD_NUMBER_5,
    #[strum(serialize = "6")]
    Number6 = constants::CEC_OP_UI_CMD_NUMBER_6,
    #[strum(serialize = "7")]
    Number7 = constants::CEC_OP_UI_CMD_NUMBER_7,
    #[strum(serialize = "8")]
    Number8 = constants::CEC_OP_UI_CMD_NUMBER_8,
    #[strum(serialize = "9")]
    Number9 = constants::CEC_OP_UI_CMD_NUMBER_9,
    Dot = constants::CEC_OP_UI_CMD_DOT,
    Enter = constants::CEC_OP_UI_CMD_ENTER,
    Clear = constants::CEC_OP_UI_CMD_CLEAR,
    NextFavorite = constants::CEC_OP_UI_CMD_NEXT_FAVORITE,
    ChannelUp = constants::CEC_OP_UI_CMD_CHANNEL_UP,
    ChannelDown = constants::CEC_OP_UI_CMD_CHANNEL_DOWN,
    PreviousChannel = constants::CEC_OP_UI_CMD_PREVIOUS_CHANNEL,
    SoundSelect = constants::CEC_OP_UI_CMD_SOUND_SELECT,
    InputSelect = constants::CEC_OP_UI_CMD_INPUT_SELECT,
    DisplayInformation = constants::CEC_OP_UI_CMD_DISPLAY_INFORMATION,
    Help = constants::CEC_OP_UI_CMD_HELP,
    PageUp = constants::CEC_OP_UI_CMD_PAGE_UP,
    PageDown = constants::CEC_OP_UI_CMD_PAGE_DOWN,
    Power = constants::CEC_OP_UI_CMD_POWER,
    VolumeUp = constants::CEC_OP_UI_CMD_VOLUME_UP,
    VolumeDown = constants::CEC_OP_UI_CMD_VOLUME_DOWN,
    Mute = constants::CEC_OP_UI_CMD_MUTE,
    Play = constants::CEC_OP_UI_CMD_PLAY,
    Stop = constants::CEC_OP_UI_CMD_STOP,
    Pause = constants::CEC_OP_UI_CMD_PAUSE,
    Record = constants::CEC_OP_UI_CMD_RECORD,
    Rewind = constants::CEC_OP_UI_CMD_REWIND,
    FastForward = constants::CEC_OP_UI_CMD_FAST_FORWARD,
    Eject = constants::CEC_OP_UI_CMD_EJECT,
    SkipForward = constants::CEC_OP_UI_CMD_SKIP_FORWARD,
    SkipBackward = constants::CEC_OP_UI_CMD_SKIP_BACKWARD,
    StopRecord = constants::CEC_OP_UI_CMD_STOP_RECORD,
    PauseRecord = constants::CEC_OP_UI_CMD_PAUSE_RECORD,
    Angle = constants::CEC_OP_UI_CMD_ANGLE,
    SubPicture = constants::CEC_OP_UI_CMD_SUB_PICTURE,
    VideoOnDemand = constants::CEC_OP_UI_CMD_VIDEO_ON_DEMAND,
    ElectronicProgramGuide = constants::CEC_OP_UI_CMD_ELECTRONIC_PROGRAM_GUIDE,
    TimerProgramming = constants::CEC_OP_UI_CMD_TIMER_PROGRAMMING,
    InitialConfiguration = constants::CEC_OP_UI_CMD_INITIAL_CONFIGURATION,
    SelectBroadcastType = constants::CEC_OP_UI_CMD_SELECT_BROADCAST_TYPE,
    SelectSoundPresentation = constants::CEC_OP_UI_CMD_SELECT_SOUND_PRESENTATION,
    AudioDescription = constants::CEC_OP_UI_CMD_AUDIO_DESCRIPTION,
    Internet = constants::CEC_OP_UI_CMD_INTERNET,
    ThreeDMode = constants::CEC_OP_UI_CMD_3D_MODE,
    PlayFunction = constants::CEC_OP_UI_CMD_PLAY_FUNCTION,
    PausePlayFunction = constants::CEC_OP_UI_CMD_PAUSE_PLAY_FUNCTION,
    RecordFunction = constants::CEC_OP_UI_CMD_RECORD_FUNCTION,
    PauseRecordFunction = constants::CEC_OP_UI_CMD_PAUSE_RECORD_FUNCTION,
    StopFunction = constants::CEC_OP_UI_CMD_STOP_FUNCTION,
    MuteFunction = constants::CEC_OP_UI_CMD_MUTE_FUNCTION,
    RestoreVolumeFunction = constants::CEC_OP_UI_CMD_RESTORE_VOLUME_FUNCTION,
    TuneFunction = constants::CEC_OP_UI_CMD_TUNE_FUNCTION,
    SelectMediaFunction = constants::CEC_OP_UI_CMD_SELECT_MEDIA_FUNCTION,
    AvInputFunction = constants::CEC_OP_UI_CMD_SELECT_AV_INPUT_FUNCTION,
    AudioInputFunction = constants::CEC_OP_UI_CMD_SELECT_AUDIO_INPUT_FUNCTION,
    PowerToggleFunction = constants::CEC_OP_UI_CMD_POWER_TOGGLE_FUNCTION,
    PowerOffFunction = constants::CEC_OP_UI_CMD_POWER_OFF_FUNCTION,
    PowerOnFunction = constants::CEC_OP_UI_CMD_POWER_ON_FUNCTION,
    #[strum(serialize = "f1", serialize = "blue")]
    F1Blue = constants::CEC_OP_UI_CMD_F1_BLUE,
    #[strum(serialize = "f2", serialize = "red")]
    F2Red = constants::CEC_OP_UI_CMD_F2_RED,
    #[strum(serialize = "f3", serialize = "green")]
    F3Green = constants::CEC_OP_UI_CMD_F3_GREEN,
    #[strum(serialize = "f4", serialize = "yellow")]
    F4Yellow = constants::CEC_OP_UI_CMD_F4_YELLOW,
    F5 = constants::CEC_OP_UI_CMD_F5,
    Data = constants::CEC_OP_UI_CMD_DATA,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum Version {
    // These first few versions predate CEC specification and are
    // theoretically invalid, but we should probably recognize anyway
    V1_1 = 0,
    V1_2 = 1,
    V1_2a = 2,
    V1_3 = 3,
    V1_3a = constants::CEC_OP_CEC_VERSION_1_3A,
    V1_4 = constants::CEC_OP_CEC_VERSION_1_4,
    V2_0 = constants::CEC_OP_CEC_VERSION_2_0,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum UiSoundPresentationControl {
    DualMono = constants::CEC_OP_UI_SND_PRES_CTL_DUAL_MONO,
    Karaoke = constants::CEC_OP_UI_SND_PRES_CTL_KARAOKE,
    Downmix = constants::CEC_OP_UI_SND_PRES_CTL_DOWNMIX,
    Reverb = constants::CEC_OP_UI_SND_PRES_CTL_REVERB,
    Equalizer = constants::CEC_OP_UI_SND_PRES_CTL_EQUALIZER,
    BassUp = constants::CEC_OP_UI_SND_PRES_CTL_BASS_UP,
    BassNeutral = constants::CEC_OP_UI_SND_PRES_CTL_BASS_NEUTRAL,
    BassDown = constants::CEC_OP_UI_SND_PRES_CTL_BASS_DOWN,
    TrebleUp = constants::CEC_OP_UI_SND_PRES_CTL_TREBLE_UP,
    TrebleNeutral = constants::CEC_OP_UI_SND_PRES_CTL_TREBLE_NEUTRAL,
    TrebleDown = constants::CEC_OP_UI_SND_PRES_CTL_TREBLE_DOWN,
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Operand)]
    pub struct AllDeviceTypes: u8 {
        const TV = constants::CEC_OP_ALL_DEVTYPE_TV;
        const RECORDING = constants::CEC_OP_ALL_DEVTYPE_RECORD;
        const TUNER = constants::CEC_OP_ALL_DEVTYPE_TUNER;
        const PLAYBACK = constants::CEC_OP_ALL_DEVTYPE_PLAYBACK;
        const AUDIOSYSTEM = constants::CEC_OP_ALL_DEVTYPE_AUDIOSYSTEM;
        const SWITCH = constants::CEC_OP_ALL_DEVTYPE_SWITCH;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Operand)]
    pub struct DeviceFeatures1: u8 {
        const HAS_RECORD_TV_SCREEN = constants::CEC_OP_FEAT_DEV_HAS_RECORD_TV_SCREEN;
        const HAS_SET_OSD_STRING = constants::CEC_OP_FEAT_DEV_HAS_SET_OSD_STRING;
        const HAS_DECK_CONTROL = constants::CEC_OP_FEAT_DEV_HAS_DECK_CONTROL;
        const HAS_SET_AUDIO_RATE = constants::CEC_OP_FEAT_DEV_HAS_SET_AUDIO_RATE;
        const SINK_HAS_ARC_TX = constants::CEC_OP_FEAT_DEV_SINK_HAS_ARC_TX;
        const SOURCE_HAS_ARC_RX = constants::CEC_OP_FEAT_DEV_SOURCE_HAS_ARC_RX;
        const HAS_SET_AUDIO_VOLUME_LEVEL = constants::CEC_OP_FEAT_DEV_HAS_SET_AUDIO_VOLUME_LEVEL;
    }
}

impl From<DeviceFeatures1> for u8 {
    fn from(flags: DeviceFeatures1) -> u8 {
        flags.bits()
    }
}

impl TryFrom<u8> for DeviceFeatures1 {
    type Error = Error;

    fn try_from(flags: u8) -> Result<DeviceFeatures1> {
        Ok(DeviceFeatures1::from_bits_retain(flags))
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Operand)]
    pub struct RcProfileSource: u8 {
        const HAS_DEV_ROOT_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_DEV_ROOT_MENU;
        const HAS_DEV_SETUP_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_DEV_SETUP_MENU;
        const HAS_CONTENTS_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_CONTENTS_MENU;
        const HAS_MEDIA_TOP_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_MEDIA_TOP_MENU;
        const HAS_MEDIA_CONTEXT_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_MEDIA_CONTEXT_MENU;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Operand)]
    pub struct RecordingSequence: u8 {
        const SUNDAY = constants::CEC_OP_REC_SEQ_SUNDAY;
        const MONDAY = constants::CEC_OP_REC_SEQ_MONDAY;
        const TUESDAY = constants::CEC_OP_REC_SEQ_TUESDAY;
        const WEDNESDAY = constants::CEC_OP_REC_SEQ_WEDNESDAY;
        const THURSDAY = constants::CEC_OP_REC_SEQ_THURSDAY;
        const FRIDAY = constants::CEC_OP_REC_SEQ_FRIDAY;
        const SATURDAY = constants::CEC_OP_REC_SEQ_SATURDAY;
    }
}

impl RecordingSequence {
    pub const ONCE_ONLY: RecordingSequence = RecordingSequence::empty();

    pub fn is_once_only(&self) -> bool {
        self.is_empty()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Operand)]
pub struct AnalogueServiceId {
    pub broadcast_type: AnalogueBroadcastType,
    pub frequency: AnalogueFrequency,
    pub broadcast_system: BroadcastSystem,
}

#[cfg(test)]
mod test_analogue_service_id {
    use super::*;

    opcode_test! {
        ty: AnalogueServiceId,
        instance: AnalogueServiceId {
            broadcast_type: AnalogueBroadcastType::Terrestrial,
            frequency: 0x1234,
            broadcast_system: BroadcastSystem::PalBG,
        },
        bytes: [
            AnalogueBroadcastType::Terrestrial as u8,
            0x12,
            0x34,
            BroadcastSystem::PalBG as u8
        ],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_missing_opcodes_1() {
        assert_eq!(
            AnalogueServiceId::try_from_bytes(&[
                AnalogueBroadcastType::Terrestrial as u8,
                0x12,
                0x34
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(4),
                got: 3,
                quantity: "bytes"
            })
        );
    }

    #[test]
    fn test_decode_missing_opcodes_1_and_byte() {
        assert_eq!(
            AnalogueServiceId::try_from_bytes(&[AnalogueBroadcastType::Terrestrial as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(4),
                got: 2,
                quantity: "bytes"
            })
        );
    }

    #[test]
    fn test_decode_missing_opcodes_2() {
        assert_eq!(
            AnalogueServiceId::try_from_bytes(&[AnalogueBroadcastType::Terrestrial as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(4),
                got: 1,
                quantity: "bytes"
            })
        );
    }

    #[test]
    fn test_decode_missing_opcodes_3() {
        assert_eq!(
            AnalogueServiceId::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(4),
                got: 0,
                quantity: "bytes"
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DigitalServiceId {
    AribGeneric(AribData),
    AtscGeneric(AtscData),
    DvbGeneric(DvbData),
    AribBs(AribData),
    AribCs(AribData),
    AribT(AribData),
    AtscCable(AtscData),
    AtscSatellite(AtscData),
    AtscTerrestrial(AtscData),
    DvbC(DvbData),
    DvbS(DvbData),
    DvbS2(DvbData),
    DvbT(DvbData),
    Channel {
        broadcast_system: DigitalServiceBroadcastSystem,
        channel_id: ChannelId,
        reserved: u16,
    },
}

impl OperandEncodable for DigitalServiceId {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        use DigitalServiceBroadcastSystem as System;
        use DigitalServiceId as Id;

        let (broadcast_system, service_id_method) = match self {
            Id::AribGeneric(_) => (System::AribGeneric, ServiceIdMethod::ByDigitalId),
            Id::AtscGeneric(_) => (System::AtscGeneric, ServiceIdMethod::ByDigitalId),
            Id::DvbGeneric(_) => (System::DvbGeneric, ServiceIdMethod::ByDigitalId),
            Id::AribBs(_) => (System::AribBs, ServiceIdMethod::ByDigitalId),
            Id::AribCs(_) => (System::AribCs, ServiceIdMethod::ByDigitalId),
            Id::AribT(_) => (System::AribT, ServiceIdMethod::ByDigitalId),
            Id::AtscCable(_) => (System::AtscCable, ServiceIdMethod::ByDigitalId),
            Id::AtscSatellite(_) => (System::AtscSatellite, ServiceIdMethod::ByDigitalId),
            Id::AtscTerrestrial(_) => (System::AtscTerrestrial, ServiceIdMethod::ByDigitalId),
            Id::DvbC(_) => (System::DvbC, ServiceIdMethod::ByDigitalId),
            Id::DvbS(_) => (System::DvbS, ServiceIdMethod::ByDigitalId),
            Id::DvbS2(_) => (System::DvbS2, ServiceIdMethod::ByDigitalId),
            Id::DvbT(_) => (System::DvbT, ServiceIdMethod::ByDigitalId),
            Id::Channel {
                broadcast_system, ..
            } => (*broadcast_system, ServiceIdMethod::ByChannel),
        };
        buf.extend([broadcast_system as u8 | ((service_id_method as u8) << 7)]);
        match self {
            Id::AribGeneric(data) | Id::AribBs(data) | Id::AribCs(data) | Id::AribT(data) => {
                data.to_bytes(buf);
            }
            Id::AtscGeneric(data)
            | Id::AtscCable(data)
            | Id::AtscSatellite(data)
            | Id::AtscTerrestrial(data) => {
                data.to_bytes(buf);
            }
            Id::DvbGeneric(data)
            | Id::DvbC(data)
            | Id::DvbS(data)
            | Id::DvbS2(data)
            | Id::DvbT(data) => {
                data.to_bytes(buf);
            }
            Id::Channel {
                channel_id,
                reserved,
                ..
            } => {
                channel_id.to_bytes(buf);
                <u16 as OperandEncodable>::to_bytes(reserved, buf);
            }
        }
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        use DigitalServiceBroadcastSystem as System;
        use DigitalServiceId as Id;

        if bytes.len() < 7 {
            return Err(crate::Error::OutOfRange {
                expected: crate::Range::AtLeast(7),
                got: bytes.len(),
                quantity: "bytes",
            });
        }
        let head = bytes[0];
        let service_id_method = ServiceIdMethod::try_from_primitive(head >> 7)?;
        let broadcast_system = System::try_from_primitive(head & 0x7F)?;
        if service_id_method == ServiceIdMethod::ByChannel {
            let channel_id = <ChannelId as OperandEncodable>::try_from_bytes(&bytes[1..])
                .map_err(Error::add_offset(1))?;
            let reserved = <u16 as OperandEncodable>::try_from_bytes(&bytes[5..])
                .map_err(Error::add_offset(5))?;
            Ok(Id::Channel {
                broadcast_system,
                channel_id,
                reserved,
            })
        } else {
            Ok(match broadcast_system {
                System::AribGeneric => Id::AribGeneric(
                    OperandEncodable::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
                ),
                System::AtscGeneric => Id::AtscGeneric(
                    OperandEncodable::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
                ),
                System::DvbGeneric => Id::DvbGeneric(
                    OperandEncodable::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
                ),
                System::AribCs => Id::AribCs(
                    OperandEncodable::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
                ),
                System::AribBs => Id::AribBs(
                    OperandEncodable::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
                ),
                System::AribT => Id::AribT(
                    OperandEncodable::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
                ),
                System::AtscCable => Id::AtscCable(
                    OperandEncodable::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
                ),
                System::AtscSatellite => Id::AtscSatellite(
                    OperandEncodable::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
                ),
                System::AtscTerrestrial => Id::AtscTerrestrial(
                    OperandEncodable::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
                ),
                System::DvbC => Id::DvbC(
                    OperandEncodable::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
                ),
                System::DvbS => Id::DvbS(
                    OperandEncodable::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
                ),
                System::DvbS2 => Id::DvbS2(
                    OperandEncodable::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
                ),
                System::DvbT => Id::DvbT(
                    OperandEncodable::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
                ),
            })
        }
    }

    fn len(&self) -> usize {
        7
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(7)
    }
}

#[cfg(test)]
mod test_digital_service_id {
    use super::*;

    opcode_test! {
        name: _arib_generic,
        ty: DigitalServiceId,
        instance: DigitalServiceId::AribGeneric(AribData {
            transport_stream_id: 0x1234,
            service_id: 0x5678,
            original_network_id: 0xABCD,
        }),
        bytes: [
            DigitalServiceBroadcastSystem::AribGeneric as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0xAB,
            0xCD,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _atsc_generic,
        ty: DigitalServiceId,
        instance: DigitalServiceId::AtscGeneric(AtscData {
            transport_stream_id: 0x1234,
            program_number: 0x5678,
        }),
        bytes: [
            DigitalServiceBroadcastSystem::AtscGeneric as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0x00,
            0x00,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _dvb_generic,
        ty: DigitalServiceId,
        instance: DigitalServiceId::DvbGeneric(DvbData {
            transport_stream_id: 0x1234,
            service_id: 0x5678,
            original_network_id: 0xABCD,
        }),
        bytes: [
            DigitalServiceBroadcastSystem::DvbGeneric as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0xAB,
            0xCD,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _arib_bs,
        ty: DigitalServiceId,
        instance: DigitalServiceId::AribBs(AribData {
            transport_stream_id: 0x1234,
            service_id: 0x5678,
            original_network_id: 0xABCD,
        }),
        bytes: [
            DigitalServiceBroadcastSystem::AribBs as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0xAB,
            0xCD,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _arib_cs,
        ty: DigitalServiceId,
        instance: DigitalServiceId::AribCs(AribData {
            transport_stream_id: 0x1234,
            service_id: 0x5678,
            original_network_id: 0xABCD,
        }),
        bytes: [
            DigitalServiceBroadcastSystem::AribCs as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0xAB,
            0xCD,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _arib_t,
        ty: DigitalServiceId,
        instance: DigitalServiceId::AribT(AribData {
            transport_stream_id: 0x1234,
            service_id: 0x5678,
            original_network_id: 0xABCD,
        }),
        bytes: [
            DigitalServiceBroadcastSystem::AribT as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0xAB,
            0xCD,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _atsc_cable,
        ty: DigitalServiceId,
        instance: DigitalServiceId::AtscCable(AtscData {
            transport_stream_id: 0x1234,
            program_number: 0x5678,
        }),
        bytes: [
            DigitalServiceBroadcastSystem::AtscCable as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0x00,
            0x00,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _atsc_satellite,
        ty: DigitalServiceId,
        instance: DigitalServiceId::AtscSatellite(AtscData {
            transport_stream_id: 0x1234,
            program_number: 0x5678,
        }),
        bytes: [
            DigitalServiceBroadcastSystem::AtscSatellite as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0x00,
            0x00,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _atsc_terrestrial,
        ty: DigitalServiceId,
        instance: DigitalServiceId::AtscTerrestrial(AtscData {
            transport_stream_id: 0x1234,
            program_number: 0x5678,
        }),
        bytes: [
            DigitalServiceBroadcastSystem::AtscTerrestrial as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0x00,
            0x00,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _dvb_c,
        ty: DigitalServiceId,
        instance: DigitalServiceId::DvbC(DvbData {
            transport_stream_id: 0x1234,
            service_id: 0x5678,
            original_network_id: 0xABCD,
        }),
        bytes: [
            DigitalServiceBroadcastSystem::DvbC as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0xAB,
            0xCD,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _dvb_s,
        ty: DigitalServiceId,
        instance: DigitalServiceId::DvbS(DvbData {
            transport_stream_id: 0x1234,
            service_id: 0x5678,
            original_network_id: 0xABCD,
        }),
        bytes: [
            DigitalServiceBroadcastSystem::DvbS as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0xAB,
            0xCD,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _dvb_s2,
        ty: DigitalServiceId,
        instance: DigitalServiceId::DvbS2(DvbData {
            transport_stream_id: 0x1234,
            service_id: 0x5678,
            original_network_id: 0xABCD,
        }),
        bytes: [
            DigitalServiceBroadcastSystem::DvbS2 as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0xAB,
            0xCD,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _dvb_t,
        ty: DigitalServiceId,
        instance: DigitalServiceId::DvbT(DvbData {
            transport_stream_id: 0x1234,
            service_id: 0x5678,
            original_network_id: 0xABCD,
        }),
        bytes: [
            DigitalServiceBroadcastSystem::DvbT as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0xAB,
            0xCD,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _channel,
        ty: DigitalServiceId,
        instance: DigitalServiceId::Channel {
            broadcast_system: DigitalServiceBroadcastSystem::DvbGeneric,
            channel_id: ChannelId::TwoPart(0x0234, 0x5678),
            reserved: 0xABCD,
        },
        bytes: [
            DigitalServiceBroadcastSystem::DvbGeneric as u8 | 0x80,
            0x0A,
            0x34,
            0x56,
            0x78,
            0xAB,
            0xCD,
        ],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            DigitalServiceId::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(7),
                quantity: "bytes"
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ProgrammedInfo {
    EnoughSpace,
    NotEnoughSpace {
        duration_available: Option<Duration>,
    },
    MayNotBeEnoughSpace {
        duration_available: Option<Duration>,
    },
    NoneAvailable,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum NotProgrammedErrorInfo {
    NoFreeTimer,
    DateOutOfRange,
    RecordingSequenceError,
    InvalidExternalPlug,
    InvalidExternalPhysicalAddress,
    CaUnsupported,
    InsufficientCaEntitlements,
    ResolutionUnsupported,
    ParentalLock,
    ClockFailure,
    Duplicate {
        duration_available: Option<Duration>,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TimerProgrammedInfo {
    Programmed(ProgrammedInfo),
    NotProgrammed(NotProgrammedErrorInfo),
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash, Operand)]
pub struct AudioFormatIdAndCode {
    #[bits(6)]
    pub code: usize,
    #[bits(2)]
    pub id: AudioFormatId,
}

#[cfg(test)]
mod test_audio_format_id_and_code {
    use super::*;

    opcode_test! {
        ty: AudioFormatIdAndCode,
        instance: AudioFormatIdAndCode::new()
            .with_code(0x05)
            .with_id(AudioFormatId::CEA861Cxt),
        bytes: [0x45],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            AudioFormatIdAndCode::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(1),
                quantity: "bytes"
            })
        );
    }
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash, Operand)]
pub struct AudioStatus {
    #[bits(7)]
    pub volume: usize,
    pub mute: bool,
}

#[cfg(test)]
mod test_audio_status {
    use super::*;

    opcode_test! {
        ty: AudioStatus,
        instance: AudioStatus::new().with_volume(0x09).with_mute(true),
        bytes: [0x89],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            AudioStatus::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(1),
                quantity: "bytes"
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct BcdByte<const MIN: u8 = 0, const MAX: u8 = 99>(u8);

impl<const MIN: u8, const MAX: u8> OperandEncodable for BcdByte<MIN, MAX> {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        buf.extend([self.0]);
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<BcdByte<MIN, MAX>> {
        if bytes.len() < 1 {
            return Err(crate::Error::OutOfRange {
                expected: crate::Range::AtLeast(1),
                got: bytes.len(),
                quantity: "bytes",
            });
        }
        let byte = bytes[0];
        Range::Interval { min: 0, max: 9 }.check(byte & 0xF, "low bits")?;
        Range::Interval { min: 0, max: 9 }.check(byte >> 4, "high bits")?;
        Range::Interval {
            min: MIN as usize,
            max: MAX as usize,
        }
        .check((byte >> 4) * 10 + (byte & 0xF), "value")?;
        Ok(BcdByte(byte))
    }

    fn len(&self) -> usize {
        1
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(1)
    }
}

impl<const MIN: u8, const MAX: u8> From<BcdByte<MIN, MAX>> for u8 {
    fn from(bcd: BcdByte<MIN, MAX>) -> u8 {
        (bcd.0 >> 4) * 10 + (bcd.0 & 0xF)
    }
}

impl<const MIN: u8, const MAX: u8> TryFrom<u8> for BcdByte<MIN, MAX> {
    type Error = Error;

    fn try_from(byte: u8) -> Result<BcdByte<MIN, MAX>> {
        Range::Interval {
            min: MIN as usize,
            max: MAX as usize,
        }
        .check(byte, "value")?;
        Ok(BcdByte(((byte / 10) << 4) + (byte % 10)))
    }
}

#[cfg(test)]
mod test_bcd_byte {
    use super::*;

    opcode_test! {
        ty: BcdByte::<0, 99>,
        instance: BcdByte::<0, 99>::try_from(12).unwrap(),
        bytes: [0x12],
        extra: [Overfull],
    }

    #[test]
    fn test_create_range() {
        assert_eq!(
            BcdByte::<10, 20>::try_from(0),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 10, max: 20 },
                got: 0,
                quantity: "value",
            })
        );

        assert_eq!(
            BcdByte::<10, 20>::try_from(9),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 10, max: 20 },
                got: 9,
                quantity: "value",
            })
        );

        assert!(BcdByte::<10, 20>::try_from(10).is_ok());

        assert!(BcdByte::<10, 20>::try_from(20).is_ok());

        assert_eq!(
            BcdByte::<10, 20>::try_from(21),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 10, max: 20 },
                got: 21,
                quantity: "value",
            })
        );

        assert_eq!(
            BcdByte::<10, 20>::try_from(30),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 10, max: 20 },
                got: 30,
                quantity: "value",
            })
        );
    }

    #[test]
    fn test_decode_range() {
        assert_eq!(
            BcdByte::<10, 20>::try_from_bytes(&[0]),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 10, max: 20 },
                got: 0,
                quantity: "value",
            })
        );

        assert_eq!(
            BcdByte::<10, 20>::try_from_bytes(&[9]),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 10, max: 20 },
                got: 9,
                quantity: "value",
            })
        );

        assert_eq!(
            BcdByte::<10, 20>::try_from_bytes(&[0xA]),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 0, max: 9 },
                got: 10,
                quantity: "low bits",
            })
        );

        assert_eq!(
            BcdByte::<10, 20>::try_from_bytes(&[0xA0]),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 0, max: 9 },
                got: 10,
                quantity: "high bits",
            })
        );

        assert!(BcdByte::<10, 20>::try_from_bytes(&[0x10]).is_ok());

        assert!(BcdByte::<10, 20>::try_from_bytes(&[0x20]).is_ok());

        assert_eq!(
            BcdByte::<10, 20>::try_from_bytes(&[0x21]),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 10, max: 20 },
                got: 21,
                quantity: "value",
            })
        );

        assert_eq!(
            BcdByte::<10, 20>::try_from_bytes(&[0x30]),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 10, max: 20 },
                got: 30,
                quantity: "value",
            })
        );
    }

    #[test]
    fn test_into_u8() {
        assert_eq!(u8::from(BcdByte::<0, 99>(0x12)), 12u8);
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            BcdByte::<0, 99>::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(1),
                quantity: "bytes"
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Operand)]
pub struct AribData {
    pub transport_stream_id: u16,
    pub service_id: u16,
    pub original_network_id: u16,
}

#[cfg(test)]
mod test_arib_data {
    use super::*;

    opcode_test! {
        ty: AribData,
        instance: AribData {
            transport_stream_id: 0x1234,
            service_id: 0x5678,
            original_network_id: 0xABCD,
        },
        bytes: [0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            AribData::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(6),
                quantity: "bytes"
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AtscData {
    pub transport_stream_id: u16,
    pub program_number: u16,
}

impl OperandEncodable for AtscData {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        self.transport_stream_id.to_bytes(buf);
        self.program_number.to_bytes(buf);
        <u16 as OperandEncodable>::to_bytes(&0, buf);
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<AtscData> {
        if bytes.len() < 6 {
            return Err(crate::Error::OutOfRange {
                expected: crate::Range::AtLeast(6),
                got: bytes.len(),
                quantity: "bytes",
            });
        }
        let transport_stream_id = u16::try_from_bytes(bytes)?;
        let program_number = u16::try_from_bytes(&bytes[2..]).map_err(Error::add_offset(2))?;
        if u16::try_from_bytes(&bytes[4..]).map_err(Error::add_offset(4))? != 0 {
            return Err(Error::InvalidData);
        }
        Ok(AtscData {
            transport_stream_id,
            program_number,
        })
    }

    fn len(&self) -> usize {
        6
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(6)
    }
}

#[cfg(test)]
mod test_atsc_data {
    use super::*;

    opcode_test! {
        ty: AtscData,
        instance: AtscData {
            transport_stream_id: 0x1234,
            program_number: 0x5678,
        },
        bytes: [0x12, 0x34, 0x56, 0x78, 0x00, 0x00],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_junk() {
        assert_eq!(
            AtscData::try_from_bytes(&[0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD]),
            Err(Error::InvalidData)
        );
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            AtscData::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(6),
                quantity: "bytes"
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ChannelId {
    OnePart(u16),
    TwoPart(u16, u16),
}

impl OperandEncodable for ChannelId {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        let number_format;
        let high;
        let low;
        match self {
            ChannelId::OnePart(part) => {
                number_format = ChannelNumberFormat::Fmt1Part;
                high = 0;
                low = *part;
            }
            ChannelId::TwoPart(major, minor) => {
                number_format = ChannelNumberFormat::Fmt2Part;
                high = *major;
                low = *minor;
            }
        }
        let number_format = (u8::from(number_format) as u16) << 10;
        let high: u16 = number_format | high;
        high.to_bytes(buf);
        low.to_bytes(buf);
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 4 {
            return Err(crate::Error::OutOfRange {
                expected: crate::Range::AtLeast(4),
                got: bytes.len(),
                quantity: "bytes",
            });
        }
        let high = u16::try_from_bytes(bytes)?;
        let low = u16::try_from_bytes(&bytes[2..]).map_err(Error::add_offset(2))?;
        let number_format = u8::try_from(high >> 10).unwrap();
        let number_format = ChannelNumberFormat::try_from_primitive(number_format)?;
        match number_format {
            ChannelNumberFormat::Fmt1Part => Ok(ChannelId::OnePart(low)),
            ChannelNumberFormat::Fmt2Part => Ok(ChannelId::TwoPart(high & 0x3FF, low)),
        }
    }

    fn len(&self) -> usize {
        4
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(4)
    }
}

#[cfg(test)]
mod test_channel_id {
    use super::*;

    opcode_test! {
        name: _1_part,
        ty: ChannelId,
        instance: ChannelId::OnePart(0x1234),
        bytes: [0x04, 0x00, 0x12, 0x34],
        extra: [Overfull],
    }

    opcode_test! {
        name: _2_part,
        ty: ChannelId,
        instance: ChannelId::TwoPart(0x0123, 0x4567),
        bytes: [0x09, 0x23, 0x45, 0x67],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_invalid_format() {
        assert_eq!(
            ChannelId::try_from_bytes(&[0x00, 0x00, 0x12, 0x34]),
            Err(Error::InvalidValueForType {
                ty: "ChannelNumberFormat",
                value: String::from("0")
            })
        )
    }

    #[test]
    fn test_decode_ignored_bytes() {
        assert_eq!(
            ChannelId::try_from_bytes(&[
                (ChannelNumberFormat::Fmt1Part as u8) << 2,
                0x56,
                0x12,
                0x34
            ]),
            Ok(ChannelId::OnePart(0x1234))
        )
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            ChannelId::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(4),
                quantity: "bytes"
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DeviceFeatures {
    pub device_features_1: DeviceFeatures1,
    pub device_features_n: BoundedBufferOperand<14, u8>,
}

impl DeviceFeatures {
    pub fn new(device_features_1: DeviceFeatures1) -> DeviceFeatures {
        DeviceFeatures {
            device_features_1,
            device_features_n: BoundedBufferOperand::default(),
        }
    }
}

impl TaggedLengthBuffer for DeviceFeatures {
    type FixedParam = DeviceFeatures1;

    fn try_new(first: DeviceFeatures1, extra_params: &[u8]) -> Result<DeviceFeatures> {
        Ok(DeviceFeatures {
            device_features_1: first,
            device_features_n: BoundedBufferOperand::<14, u8>::try_from_bytes(extra_params)?,
        })
    }

    fn fixed_param(&self) -> DeviceFeatures1 {
        self.device_features_1
    }

    fn extra_params(&self) -> &[u8] {
        &self.device_features_n.buffer[..self.device_features_n.len]
    }
}

#[cfg(test)]
mod test_device_features {
    use super::*;

    opcode_test! {
        name: _1_only,
        ty: DeviceFeatures,
        instance: DeviceFeatures {
            device_features_1: DeviceFeatures1::HAS_RECORD_TV_SCREEN |
                DeviceFeatures1::HAS_SET_OSD_STRING |
                DeviceFeatures1::HAS_SET_AUDIO_VOLUME_LEVEL,
            device_features_n: BoundedBufferOperand::default(),
        },
        bytes: [0x61],
    }

    opcode_test! {
        name: _n,
        ty: DeviceFeatures,
        instance: DeviceFeatures {
            device_features_1: DeviceFeatures1::HAS_RECORD_TV_SCREEN |
                DeviceFeatures1::HAS_SET_OSD_STRING |
                DeviceFeatures1::HAS_SET_AUDIO_VOLUME_LEVEL,
            device_features_n: BoundedBufferOperand::try_from_bytes(&[
                0x40,
                0x20,
                0x10,
                0x08,
                0x04,
                0x02,
                0x01,
                0x00
            ]).unwrap(),
        },
        bytes: [0xE1, 0xC0, 0xA0, 0x90, 0x88, 0x84, 0x82, 0x81, 0x00],
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Operand)]
pub struct Duration {
    pub hours: DurationHours,
    pub minutes: Minute,
}

#[cfg(test)]
mod test_duration {
    use super::*;

    opcode_test! {
        ty: Duration,
        instance: Duration {
            hours: DurationHours::try_from(99u8).unwrap(),
            minutes: Minute::try_from(20u8).unwrap(),
        },
        bytes: [0x99, 0x20],
        extra: [Overfull],
    }

    #[test]
    fn test_invalid_minute() {
        assert_eq!(
            Duration::try_from_bytes(&[0x04, 0x69]),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 0, max: 59 },
                got: 69,
                quantity: "value",
            })
        );
    }

    #[test]
    fn test_invalid_bcd_hour() {
        assert_eq!(
            Duration::try_from_bytes(&[0x0A, 0x20]),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 0, max: 9 },
                got: 10,
                quantity: "low bits",
            })
        );
    }

    #[test]
    fn test_invalid_bcd_minute() {
        assert_eq!(
            Duration::try_from_bytes(&[0x04, 0x1A]),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 0, max: 9 },
                got: 10,
                quantity: "low bits",
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Operand)]
pub struct DvbData {
    pub transport_stream_id: u16,
    pub service_id: u16,
    pub original_network_id: u16,
}

#[cfg(test)]
mod test_dvb_data {
    use super::*;

    opcode_test! {
        ty: DvbData,
        instance: DvbData {
            transport_stream_id: 0x1234,
            service_id: 0x5678,
            original_network_id: 0xABCD,
        },
        bytes: [0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            DvbData::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(6),
                quantity: "bytes"
            })
        );
    }
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash, Operand)]
pub struct LatencyFlags {
    #[bits(2)]
    pub audio_out_compensated: AudioOutputCompensated,
    pub low_latency_mode: bool,
    #[bits(5)]
    _reserved: usize,
}

#[cfg(test)]
mod test_latency_flags {
    use super::*;

    opcode_test! {
        ty: LatencyFlags,
        instance: LatencyFlags::new()
            .with_audio_out_compensated(AudioOutputCompensated::PartialDelay)
            .with_low_latency_mode(true),
        bytes: [0x07],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            LatencyFlags::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(1),
                quantity: "bytes"
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct RcProfile {
    pub rc_profile_1: RcProfile1,
    pub rc_profile_n: BoundedBufferOperand<14, u8>,
}

impl RcProfile {
    pub fn new(rc_profile_1: RcProfile1) -> RcProfile {
        RcProfile {
            rc_profile_1,
            rc_profile_n: BoundedBufferOperand::default(),
        }
    }
}

impl TaggedLengthBuffer for RcProfile {
    type FixedParam = RcProfile1;

    fn try_new(first: RcProfile1, extra_params: &[u8]) -> Result<RcProfile> {
        Ok(RcProfile {
            rc_profile_1: first,
            rc_profile_n: BoundedBufferOperand::<14, u8>::try_from_bytes(extra_params)?,
        })
    }

    fn fixed_param(&self) -> RcProfile1 {
        self.rc_profile_1
    }

    fn extra_params(&self) -> &[u8] {
        &self.rc_profile_n.buffer[..self.rc_profile_n.len]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RcProfile1 {
    Source(RcProfileSource),
    Tv { id: RcProfileId },
}

impl From<RcProfile1> for u8 {
    fn from(profile: RcProfile1) -> u8 {
        match profile {
            RcProfile1::Source(profile_source) => profile_source.bits(),
            RcProfile1::Tv { id: profile_id } => profile_id.into(),
        }
    }
}

impl TryFrom<u8> for RcProfile1 {
    type Error = Error;

    fn try_from(flags: u8) -> Result<RcProfile1> {
        let flags = flags & 0x7F;
        if (flags & 0x40) == 0x40 {
            Ok(RcProfile1::Source(RcProfileSource::from_bits_retain(flags)))
        } else {
            Ok(RcProfile1::Tv {
                id: RcProfileId::try_from_primitive(flags & 0xF)?,
            })
        }
    }
}

#[cfg(test)]
mod test_rc_profile {
    use super::*;

    #[test]
    fn test_rc_profile_1_tv() {
        assert_eq!(
            RcProfile1::try_from(0x02),
            Ok(RcProfile1::Tv {
                id: RcProfileId::Profile1
            })
        );
    }

    #[test]
    fn test_rc_profile_1_source() {
        assert_eq!(
            RcProfile1::try_from(0x5F),
            Ok(RcProfile1::Source(RcProfileSource::all()))
        );
    }

    opcode_test! {
        name: _tv,
        ty: RcProfile,
        instance: RcProfile {
            rc_profile_1: RcProfile1::Tv { id: RcProfileId::Profile1 },
            rc_profile_n: BoundedBufferOperand::default(),
        },
        bytes: [0x02],
        extra: [Overfull],
    }

    opcode_test! {
        name: _source,
        ty: RcProfile,
        instance: RcProfile {
            rc_profile_1: RcProfile1::Source(RcProfileSource::HAS_DEV_ROOT_MENU),
            rc_profile_n: BoundedBufferOperand::default(),
        },
        bytes: [0x40 | RcProfileSource::HAS_DEV_ROOT_MENU.bits()],
        extra: [Overfull],
    }

    opcode_test! {
        name: _extra_bytes,
        ty: RcProfile,
        instance: RcProfile {
            rc_profile_1: RcProfile1::Tv { id: RcProfileId::Profile1 },
            rc_profile_n: BoundedBufferOperand::try_from_bytes(&[0x40]).unwrap(),
        },
        bytes: [0x82, 0x40],
        extra: [Overfull],
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Operand)]
pub struct Time {
    pub hour: Hour,
    pub minute: Minute,
}

#[cfg(test)]
mod test_time {
    use super::*;

    opcode_test! {
        ty: Time,
        instance: Time {
            hour: Hour::try_from(4u8).unwrap(),
            minute: Minute::try_from(20u8).unwrap(),
        },
        bytes: [0x04, 0x20],
        extra: [Overfull],
    }

    #[test]
    fn test_invalid_hour() {
        assert_eq!(
            Time::try_from_bytes(&[0x24, 0x30]),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 0, max: 23 },
                got: 24,
                quantity: "value",
            })
        );
    }

    #[test]
    fn test_invalid_minute() {
        assert_eq!(
            Time::try_from_bytes(&[0x04, 0x69]),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 0, max: 59 },
                got: 69,
                quantity: "value",
            })
        );
    }

    #[test]
    fn test_invalid_bcd_hour() {
        assert_eq!(
            Time::try_from_bytes(&[0x0A, 0x20]),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 0, max: 9 },
                got: 10,
                quantity: "low bits",
            })
        );
    }

    #[test]
    fn test_invalid_bcd_minute() {
        assert_eq!(
            Time::try_from_bytes(&[0x04, 0x1A]),
            Err(Error::OutOfRange {
                expected: Range::Interval { min: 0, max: 9 },
                got: 10,
                quantity: "low bits",
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TimerStatusData {
    pub overlap_warning: bool,
    pub media_info: MediaInfo,
    pub programmed_info: TimerProgrammedInfo,
}

impl OperandEncodable for TimerStatusData {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        let mut byte: u8 = 0;
        if self.overlap_warning {
            byte |= 0x80;
        }
        byte |= <_ as Into<u8>>::into(self.media_info) << 5;
        let mut duration = None;

        match self.programmed_info {
            TimerProgrammedInfo::Programmed(programmed) => {
                byte |= 0x10;
                byte |= match programmed {
                    ProgrammedInfo::EnoughSpace => constants::CEC_OP_PROG_INFO_ENOUGH_SPACE,
                    ProgrammedInfo::NotEnoughSpace { duration_available } => {
                        duration = duration_available;
                        constants::CEC_OP_PROG_INFO_NOT_ENOUGH_SPACE
                    }
                    ProgrammedInfo::MayNotBeEnoughSpace { duration_available } => {
                        duration = duration_available;
                        constants::CEC_OP_PROG_INFO_MIGHT_NOT_BE_ENOUGH_SPACE
                    }
                    ProgrammedInfo::NoneAvailable => constants::CEC_OP_PROG_INFO_NONE_AVAILABLE,
                };
            }
            TimerProgrammedInfo::NotProgrammed(not_programmed) => {
                byte |= match not_programmed {
                    NotProgrammedErrorInfo::NoFreeTimer => {
                        constants::CEC_OP_PROG_ERROR_NO_FREE_TIMER
                    }
                    NotProgrammedErrorInfo::DateOutOfRange => {
                        constants::CEC_OP_PROG_ERROR_DATE_OUT_OF_RANGE
                    }
                    NotProgrammedErrorInfo::RecordingSequenceError => {
                        constants::CEC_OP_PROG_ERROR_REC_SEQ_ERROR
                    }
                    NotProgrammedErrorInfo::InvalidExternalPlug => {
                        constants::CEC_OP_PROG_ERROR_INV_EXT_PLUG
                    }
                    NotProgrammedErrorInfo::InvalidExternalPhysicalAddress => {
                        constants::CEC_OP_PROG_ERROR_INV_EXT_PHYS_ADDR
                    }
                    NotProgrammedErrorInfo::CaUnsupported => constants::CEC_OP_PROG_ERROR_CA_UNSUPP,
                    NotProgrammedErrorInfo::InsufficientCaEntitlements => {
                        constants::CEC_OP_PROG_ERROR_INSUF_CA_ENTITLEMENTS
                    }
                    NotProgrammedErrorInfo::ResolutionUnsupported => {
                        constants::CEC_OP_PROG_ERROR_RESOLUTION_UNSUPP
                    }
                    NotProgrammedErrorInfo::ParentalLock => {
                        constants::CEC_OP_PROG_ERROR_PARENTAL_LOCK
                    }
                    NotProgrammedErrorInfo::ClockFailure => {
                        constants::CEC_OP_PROG_ERROR_CLOCK_FAILURE
                    }
                    NotProgrammedErrorInfo::Duplicate { duration_available } => {
                        duration = duration_available;
                        constants::CEC_OP_PROG_ERROR_DUPLICATE
                    }
                };
            }
        }

        buf.extend([byte]);
        duration.to_bytes(buf);
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<TimerStatusData> {
        if bytes.len() < 1 {
            return Err(crate::Error::OutOfRange {
                expected: crate::Range::AtLeast(1),
                got: bytes.len(),
                quantity: "bytes",
            });
        }
        let byte = bytes[0];
        let overlap_warning = (byte & 0x80) == 0x80;
        let media_info = MediaInfo::try_from_primitive((byte >> 5) & 3)?;
        let programmed_info = if (byte & 0x10) == 0x10 {
            TimerProgrammedInfo::Programmed(match byte & 0xF {
                constants::CEC_OP_PROG_INFO_ENOUGH_SPACE => ProgrammedInfo::EnoughSpace,
                constants::CEC_OP_PROG_INFO_NOT_ENOUGH_SPACE => {
                    let duration_available = if bytes.len() >= 3 {
                        Some(
                            Duration::try_from_bytes(&bytes[1..])
                                .map_err(crate::Error::add_offset(1))?,
                        )
                    } else {
                        None
                    };
                    ProgrammedInfo::NotEnoughSpace { duration_available }
                }
                constants::CEC_OP_PROG_INFO_MIGHT_NOT_BE_ENOUGH_SPACE => {
                    let duration_available = if bytes.len() >= 3 {
                        Some(
                            Duration::try_from_bytes(&bytes[1..])
                                .map_err(crate::Error::add_offset(1))?,
                        )
                    } else {
                        None
                    };
                    ProgrammedInfo::MayNotBeEnoughSpace { duration_available }
                }
                constants::CEC_OP_PROG_INFO_NONE_AVAILABLE => ProgrammedInfo::NoneAvailable,
                v => {
                    return Err(Error::InvalidValueForType {
                        ty: "ProgrammedInfo",
                        value: v.to_string(),
                    })
                }
            })
        } else {
            TimerProgrammedInfo::NotProgrammed(match byte & 0xF {
                constants::CEC_OP_PROG_ERROR_NO_FREE_TIMER => NotProgrammedErrorInfo::NoFreeTimer,
                constants::CEC_OP_PROG_ERROR_DATE_OUT_OF_RANGE => {
                    NotProgrammedErrorInfo::DateOutOfRange
                }
                constants::CEC_OP_PROG_ERROR_REC_SEQ_ERROR => {
                    NotProgrammedErrorInfo::RecordingSequenceError
                }
                constants::CEC_OP_PROG_ERROR_INV_EXT_PLUG => {
                    NotProgrammedErrorInfo::InvalidExternalPlug
                }
                constants::CEC_OP_PROG_ERROR_INV_EXT_PHYS_ADDR => {
                    NotProgrammedErrorInfo::InvalidExternalPhysicalAddress
                }
                constants::CEC_OP_PROG_ERROR_CA_UNSUPP => NotProgrammedErrorInfo::CaUnsupported,
                constants::CEC_OP_PROG_ERROR_INSUF_CA_ENTITLEMENTS => {
                    NotProgrammedErrorInfo::InsufficientCaEntitlements
                }
                constants::CEC_OP_PROG_ERROR_RESOLUTION_UNSUPP => {
                    NotProgrammedErrorInfo::ResolutionUnsupported
                }
                constants::CEC_OP_PROG_ERROR_PARENTAL_LOCK => NotProgrammedErrorInfo::ParentalLock,
                constants::CEC_OP_PROG_ERROR_CLOCK_FAILURE => NotProgrammedErrorInfo::ClockFailure,
                constants::CEC_OP_PROG_ERROR_DUPLICATE => {
                    let duration_available = if bytes.len() >= 3 {
                        Some(
                            Duration::try_from_bytes(&bytes[1..])
                                .map_err(crate::Error::add_offset(1))?,
                        )
                    } else {
                        None
                    };
                    NotProgrammedErrorInfo::Duplicate { duration_available }
                }
                v => {
                    return Err(Error::InvalidValueForType {
                        ty: "NotProgrammedErrorInfo",
                        value: v.to_string(),
                    })
                }
            })
        };
        Ok(TimerStatusData {
            overlap_warning,
            media_info,
            programmed_info,
        })
    }

    fn len(&self) -> usize {
        match self.programmed_info {
            TimerProgrammedInfo::Programmed(programmed) => match programmed {
                ProgrammedInfo::NotEnoughSpace { duration_available }
                | ProgrammedInfo::MayNotBeEnoughSpace { duration_available } => {
                    if duration_available.is_some() {
                        3
                    } else {
                        1
                    }
                }
                _ => 1,
            },
            TimerProgrammedInfo::NotProgrammed(not_programmed) => match not_programmed {
                NotProgrammedErrorInfo::Duplicate { duration_available } => {
                    if duration_available.is_some() {
                        3
                    } else {
                        1
                    }
                }
                _ => 1,
            },
        }
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(1)
    }
}

#[cfg(test)]
mod test_timer_status_data {
    use super::*;

    opcode_test! {
        name: _not_programmed_no_free_timer,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::NoFreeTimer),
        },
        bytes: [0x01],
        extra: [Overfull],
    }

    opcode_test! {
        name: _overlap_warning,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: true,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::NoFreeTimer),
        },
        bytes: [0x81],
        extra: [Overfull],
    }

    opcode_test! {
        name: _protected_media,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::ProtectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::NoFreeTimer),
        },
        bytes: [0x21],
        extra: [Overfull],
    }

    opcode_test! {
        name: _no_media,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::NoMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::NoFreeTimer),
        },
        bytes: [0x41],
        extra: [Overfull],
    }

    opcode_test! {
        name: _not_programmed_date_out_of_range,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::DateOutOfRange),
        },
        bytes: [0x02],
        extra: [Overfull],
    }

    opcode_test! {
        name: _not_programmed_recording_sequence_error,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::RecordingSequenceError),
        },
        bytes: [0x03],
        extra: [Overfull],
    }

    opcode_test! {
        name: _not_programmed_invalid_ext_plug,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::InvalidExternalPlug),
        },
        bytes: [0x04],
        extra: [Overfull],
    }

    opcode_test! {
        name: _not_programmed_invalid_ext_phys_addr,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::InvalidExternalPhysicalAddress),
        },
        bytes: [0x05],
        extra: [Overfull],
    }

    opcode_test! {
        name: _not_programmed_ca_unsupported,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::CaUnsupported),
        },
        bytes: [0x06],
        extra: [Overfull],
    }

    opcode_test! {
        name: _not_programmed_insufficient_ca_entitlements,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::InsufficientCaEntitlements),
        },
        bytes: [0x07],
        extra: [Overfull],
    }

    opcode_test! {
        name: _not_programmed_resolution_unsupported,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::ResolutionUnsupported),
        },
        bytes: [0x08],
        extra: [Overfull],
    }

    opcode_test! {
        name: _not_programmed_parental_lock,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::ParentalLock),
        },
        bytes: [0x09],
        extra: [Overfull],
    }

    opcode_test! {
        name: _not_programmed_clock_failure,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::ClockFailure),
        },
        bytes: [0x0A],
        extra: [Overfull],
    }

    opcode_test! {
        name: _not_programmed_duplicate,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::Duplicate {
                duration_available: None
            }),
        },
        bytes: [0x0E],
    }

    opcode_test! {
        name: _not_programmed_duplicate_duration,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::NotProgrammed(NotProgrammedErrorInfo::Duplicate {
                duration_available: Some(Duration {
                    hours: DurationHours::try_from(23).unwrap(),
                    minutes: Minute::try_from(59).unwrap(),
                })
            }),
        },
        bytes: [0x0E, 0x23, 0x59],
        extra: [Overfull],
    }

    opcode_test! {
        name: _programmed_enough_space,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::Programmed(ProgrammedInfo::EnoughSpace),
        },
        bytes: [0x18],
        extra: [Overfull],
    }

    opcode_test! {
        name: _programmed_not_enough_space,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::Programmed(ProgrammedInfo::NotEnoughSpace {
                duration_available: None
            }),
        },
        bytes: [0x19],
    }

    opcode_test! {
        name: _programmed_not_enough_space_duration,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::Programmed(ProgrammedInfo::NotEnoughSpace {
                duration_available: Some(Duration {
                    hours: DurationHours::try_from(23).unwrap(),
                    minutes: Minute::try_from(59).unwrap(),
                })
            }),
        },
        bytes: [0x19, 0x23, 0x59],
        extra: [Overfull],
    }

    opcode_test! {
        name: _programmed_no_media,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::Programmed(ProgrammedInfo::NoneAvailable),
        },
        bytes: [0x1A],
        extra: [Overfull],
    }

    opcode_test! {
        name: _programmed_maybe_enough_space,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::Programmed(ProgrammedInfo::MayNotBeEnoughSpace {
                duration_available: None
            }),
        },
        bytes: [0x1B],
    }

    opcode_test! {
        name: _programmed_maybe_enough_space_duration,
        ty: TimerStatusData,
        instance: TimerStatusData {
            overlap_warning: false,
            media_info: MediaInfo::UnprotectedMedia,
            programmed_info: TimerProgrammedInfo::Programmed(ProgrammedInfo::MayNotBeEnoughSpace {
                duration_available: Some(Duration {
                    hours: DurationHours::try_from(23).unwrap(),
                    minutes: Minute::try_from(59).unwrap(),
                })
            }),
        },
        bytes: [0x1B, 0x23, 0x59],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            TimerStatusData::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(1),
                quantity: "bytes"
            })
        );
    }

    #[test]
    fn test_decode_reserved_not_programmed_error_info() {
        assert_eq!(
            TimerStatusData::try_from_bytes(&[0x00]),
            Err(Error::InvalidValueForType {
                ty: "NotProgrammedErrorInfo",
                value: String::from("0")
            })
        );
    }

    #[test]
    fn test_decode_reserved_programmed_info() {
        assert_eq!(
            TimerStatusData::try_from_bytes(&[0x10]),
            Err(Error::InvalidValueForType {
                ty: "ProgrammedInfo",
                value: String::from("0")
            })
        );
    }

    #[test]
    fn test_decode_reserved_duration_underfull() {
        assert_eq!(
            TimerStatusData::try_from_bytes(&[0x0E, 0x01]),
            Ok(TimerStatusData {
                overlap_warning: false,
                media_info: MediaInfo::UnprotectedMedia,
                programmed_info: TimerProgrammedInfo::NotProgrammed(
                    NotProgrammedErrorInfo::Duplicate {
                        duration_available: None
                    }
                ),
            })
        );
    }

    #[test]
    fn test_decode_reserved_media_info() {
        assert_eq!(
            TimerStatusData::try_from_bytes(&[0x60]),
            Err(Error::InvalidValueForType {
                ty: "MediaInfo",
                value: String::from("3")
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TunerDeviceInfo {
    pub recording: bool,
    pub tuner_display_info: TunerDisplayInfo,
    pub service_id: ServiceId,
}

impl OperandEncodable for TunerDeviceInfo {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        match self.service_id {
            ServiceId::Analogue(service_id) => {
                let recording = if self.recording { 0x80 } else { 0 };
                let display_info = u8::from(self.tuner_display_info);
                <u8 as OperandEncodable>::to_bytes(&(recording | display_info), buf);
                service_id.to_bytes(buf);
            }
            ServiceId::Digital(service_id) => {
                let recording = if self.recording { 0x80 } else { 0 };
                let display_info = u8::from(self.tuner_display_info);
                <u8 as OperandEncodable>::to_bytes(&(recording | display_info), buf);
                service_id.to_bytes(buf);
            }
        }
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 5 {
            return Err(crate::Error::OutOfRange {
                expected: Range::Only(array_vec![5, 8]),
                got: bytes.len(),
                quantity: "bytes",
            });
        }
        let head = bytes[0];
        let recording = (head & 0x80) == 0x80;
        let tuner_display_info = TunerDisplayInfo::try_from_primitive(head & 0x7F)?;
        let service_id = match bytes.len() {
            5 => ServiceId::Analogue(AnalogueServiceId::try_from_bytes(&bytes[1..])?),
            8 => ServiceId::Digital(DigitalServiceId::try_from_bytes(&bytes[1..])?),
            l => {
                return Err(Error::OutOfRange {
                    got: l,
                    expected: Range::Only(array_vec![5, 8]),
                    quantity: "bytes",
                })
            }
        };
        Ok(TunerDeviceInfo {
            recording,
            tuner_display_info,
            service_id,
        })
    }

    fn len(&self) -> usize {
        match self.service_id {
            ServiceId::Analogue(_) => 5,
            ServiceId::Digital(_) => 8,
        }
    }

    fn expected_len() -> Range<usize> {
        Range::Only(array_vec![5, 8])
    }
}

#[cfg(test)]
mod test_tuner_device_info {
    use super::*;

    opcode_test! {
        name: _analogue,
        ty: TunerDeviceInfo,
        instance: TunerDeviceInfo {
            recording: true,
            tuner_display_info: TunerDisplayInfo::Analogue,
            service_id: ServiceId::Analogue(AnalogueServiceId {
                broadcast_type: AnalogueBroadcastType::Terrestrial,
                frequency: 0x1234,
                broadcast_system: BroadcastSystem::PalBG,
            })
        },
        bytes: [
            0x82,
            AnalogueBroadcastType::Terrestrial as u8,
            0x12,
            0x34,
            BroadcastSystem::PalBG as u8
        ],
    }

    opcode_test! {
        name: _digital,
        ty: TunerDeviceInfo,
        instance: TunerDeviceInfo {
            recording: true,
            tuner_display_info: TunerDisplayInfo::Analogue,
            service_id: ServiceId::Digital(DigitalServiceId::DvbGeneric(DvbData {
                transport_stream_id: 0x1234,
                service_id: 0x5678,
                original_network_id: 0xABCD,
            }))
        },
        bytes: [
            0x82,
            DigitalServiceBroadcastSystem::DvbGeneric as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0xAB,
            0xCD
        ],
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            TunerDeviceInfo::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::Only(array_vec![5, 8]),
                quantity: "bytes"
            })
        );
    }

    #[test]
    fn test_decode_midrange_6() {
        assert_eq!(
            TunerDeviceInfo::try_from_bytes(&[0x01, 0x23, 0x45, 0x67, 0x89, 0xAB]),
            Err(Error::OutOfRange {
                got: 6,
                expected: Range::Only(array_vec![5, 8]),
                quantity: "bytes"
            })
        );
    }

    #[test]
    fn test_decode_midrange_7() {
        assert_eq!(
            TunerDeviceInfo::try_from_bytes(&[0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD]),
            Err(Error::OutOfRange {
                got: 7,
                expected: Range::Only(array_vec![5, 8]),
                quantity: "bytes"
            })
        );
    }

    #[test]
    fn test_decode_overfull() {
        assert_eq!(
            TunerDeviceInfo::try_from_bytes(&[
                0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01
            ]),
            Err(Error::OutOfRange {
                got: 9,
                expected: Range::Only(array_vec![5, 8]),
                quantity: "bytes"
            })
        );
    }

    #[test]
    fn test_decode_invalid_tuner_display_info() {
        assert_eq!(
            TunerDeviceInfo::try_from_bytes(&[0x09, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]),
            Err(Error::InvalidValueForType {
                ty: "TunerDisplayInfo",
                value: String::from("9"),
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExternalSource {
    Plug(u8),
    PhysicalAddress(PhysicalAddress),
}

impl OperandEncodable for ExternalSource {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        match self {
            ExternalSource::Plug(plug) => buf.extend([*plug]),
            ExternalSource::PhysicalAddress(phys_addr) => phys_addr.to_bytes(buf),
        }
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        match bytes.len() {
            1 => Ok(ExternalSource::Plug(bytes[0])),
            2 => Ok(ExternalSource::PhysicalAddress(
                <PhysicalAddress as OperandEncodable>::try_from_bytes(bytes)?,
            )),
            l => Err(Error::OutOfRange {
                got: l,
                expected: crate::Range::Only(array_vec![1, 2]),
                quantity: "bytes",
            }),
        }
    }

    fn len(&self) -> usize {
        match self {
            ExternalSource::Plug(_) => 1,
            ExternalSource::PhysicalAddress(_) => 2,
        }
    }

    fn expected_len() -> Range<usize> {
        Range::Only(array_vec![1, 2])
    }
}

#[cfg(test)]
mod test_external_source {
    use super::*;

    opcode_test! {
        name: _plug,
        ty: ExternalSource,
        instance: ExternalSource::Plug(0x56),
        bytes: [0x56],
    }

    opcode_test! {
        name: _phys_addr,
        ty: ExternalSource,
        instance: ExternalSource::PhysicalAddress(0x5678),
        bytes: [0x56, 0x78],
    }

    #[test]
    fn test_decode_overfull() {
        assert_eq!(
            ExternalSource::try_from_bytes(&[0x12, 0x34, 0x56]),
            Err(Error::OutOfRange {
                got: 3,
                expected: Range::Only(array_vec![1, 2]),
                quantity: "bytes"
            })
        );
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            ExternalSource::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::Only(array_vec![1, 2]),
                quantity: "bytes"
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RecordSource {
    Own,
    DigitalService(DigitalServiceId),
    AnalogueService(AnalogueServiceId),
    External(ExternalSource),
}

impl OperandEncodable for RecordSource {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        match self {
            RecordSource::Own => RecordSourceType::Own.to_bytes(buf),
            RecordSource::DigitalService(ref service_id) => {
                RecordSourceType::Digital.to_bytes(buf);
                service_id.to_bytes(buf);
            }
            RecordSource::AnalogueService(ref service_id) => {
                RecordSourceType::Analogue.to_bytes(buf);
                service_id.to_bytes(buf);
            }
            RecordSource::External(ref source) => {
                match source {
                    ExternalSource::Plug(_) => RecordSourceType::ExternalPlug.to_bytes(buf),
                    ExternalSource::PhysicalAddress(_) => {
                        RecordSourceType::ExternalPhysicalAddress.to_bytes(buf)
                    }
                }
                source.to_bytes(buf);
            }
        }
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        let record_source_type = RecordSourceType::try_from_bytes(bytes)?;
        match record_source_type {
            RecordSourceType::Own => Ok(RecordSource::Own),
            RecordSourceType::Digital => Ok(RecordSource::DigitalService(
                DigitalServiceId::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
            )),
            RecordSourceType::Analogue => Ok(RecordSource::AnalogueService(
                AnalogueServiceId::try_from_bytes(&bytes[1..]).map_err(Error::add_offset(1))?,
            )),
            RecordSourceType::ExternalPlug => {
                if bytes.len() < 2 {
                    Err(crate::Error::OutOfRange {
                        expected: crate::Range::AtLeast(2),
                        got: bytes.len(),
                        quantity: "bytes",
                    })
                } else {
                    Ok(RecordSource::External(ExternalSource::Plug(bytes[1])))
                }
            }
            RecordSourceType::ExternalPhysicalAddress => {
                Ok(RecordSource::External(ExternalSource::PhysicalAddress(
                    <PhysicalAddress as OperandEncodable>::try_from_bytes(&bytes[1..])
                        .map_err(Error::add_offset(1))?,
                )))
            }
        }
    }

    fn len(&self) -> usize {
        let len = match self {
            RecordSource::Own => 0,
            RecordSource::DigitalService(ref service_id) => service_id.len(),
            RecordSource::AnalogueService(ref service_id) => service_id.len(),
            RecordSource::External(ref source) => source.len(),
        };
        len + 1
    }

    fn expected_len() -> Range<usize> {
        Range::AtLeast(1)
    }
}

#[cfg(test)]
mod test_record_source {
    use super::*;

    opcode_test! {
        name: _own,
        ty: RecordSource,
        instance: RecordSource::Own,
        bytes: [RecordSourceType::Own as u8],
        extra: [Overfull],
    }

    opcode_test! {
        name: _digital,
        ty: RecordSource,
        instance: RecordSource::DigitalService(
            DigitalServiceId::AribGeneric(AribData {
                transport_stream_id: 0x1234,
                service_id: 0x5678,
                original_network_id: 0x9ABC,
            })
        ),
        bytes: [
            RecordSourceType::Digital as u8,
            DigitalServiceBroadcastSystem::AribGeneric as u8,
            0x12,
            0x34,
            0x56,
            0x78,
            0x9A,
            0xBC
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _analogue,
        ty: RecordSource,
        instance: RecordSource::AnalogueService(AnalogueServiceId {
            broadcast_type: AnalogueBroadcastType::Satellite,
            frequency: 0x1234,
            broadcast_system: BroadcastSystem::SecamL,
        }),
        bytes: [
            RecordSourceType::Analogue as u8,
            AnalogueBroadcastType::Satellite as u8,
            0x12,
            0x34,
            BroadcastSystem::SecamL as u8
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _external_plug,
        ty: RecordSource,
        instance: RecordSource::External(ExternalSource::Plug(0x56)),
        bytes: [
            RecordSourceType::ExternalPlug as u8,
            0x56,
        ],
        extra: [Overfull],
    }

    opcode_test! {
        name: _external_phys_addr,
        ty: RecordSource,
        instance: RecordSource::External(ExternalSource::PhysicalAddress(0x1234)),
        bytes: [
            RecordSourceType::ExternalPhysicalAddress as u8,
            0x12,
            0x34
        ],
        extra: [Overfull],
    }

    #[test]
    fn test_digital_decoding_missing_bytes_1() {
        assert_eq!(
            RecordSource::try_from_bytes(&[
                RecordSourceType::Digital as u8,
                DigitalServiceBroadcastSystem::AribGeneric as u8,
                0x12,
                0x34,
                0x56,
                0x78,
                0x9A
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(8),
                got: 7,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_digital_decoding_missing_bytes_2() {
        assert_eq!(
            RecordSource::try_from_bytes(&[
                RecordSourceType::Digital as u8,
                DigitalServiceBroadcastSystem::AribGeneric as u8,
                0x12,
                0x34,
                0x56,
                0x78
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(8),
                got: 6,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_digital_decoding_missing_bytes_3() {
        assert_eq!(
            RecordSource::try_from_bytes(&[
                RecordSourceType::Digital as u8,
                DigitalServiceBroadcastSystem::AribGeneric as u8,
                0x12,
                0x34,
                0x56
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(8),
                got: 5,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_digital_decoding_missing_bytes_4() {
        assert_eq!(
            RecordSource::try_from_bytes(&[
                RecordSourceType::Digital as u8,
                DigitalServiceBroadcastSystem::AribGeneric as u8,
                0x12,
                0x34
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(8),
                got: 4,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_digital_decoding_missing_bytes_5() {
        assert_eq!(
            RecordSource::try_from_bytes(&[
                RecordSourceType::Digital as u8,
                DigitalServiceBroadcastSystem::AribGeneric as u8,
                0x12,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(8),
                got: 3,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_digital_decoding_missing_operand_1() {
        assert_eq!(
            RecordSource::try_from_bytes(&[
                RecordSourceType::Digital as u8,
                DigitalServiceBroadcastSystem::AribGeneric as u8,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(8),
                got: 2,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_digital_decoding_missing_operand_2() {
        assert_eq!(
            RecordSource::try_from_bytes(&[RecordSourceType::Digital as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(8),
                got: 1,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_analogue_decoding_missing_operands_1() {
        assert_eq!(
            RecordSource::try_from_bytes(&[
                RecordSourceType::Analogue as u8,
                AnalogueBroadcastType::Satellite as u8,
                0x12,
                0x34
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(5),
                got: 4,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_analogue_decoding_missing_operands_1_and_byte() {
        assert_eq!(
            RecordSource::try_from_bytes(&[
                RecordSourceType::Analogue as u8,
                AnalogueBroadcastType::Satellite as u8,
                0x12,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(5),
                got: 3,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_analogue_decoding_missing_operands_2() {
        assert_eq!(
            RecordSource::try_from_bytes(&[
                RecordSourceType::Analogue as u8,
                AnalogueBroadcastType::Satellite as u8,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(5),
                got: 2,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_analogue_decoding_missing_operands_3() {
        assert_eq!(
            RecordSource::try_from_bytes(&[RecordSourceType::Analogue as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(5),
                got: 1,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_external_plug_decoding_missing_operand() {
        assert_eq!(
            RecordSource::try_from_bytes(&[RecordSourceType::ExternalPlug as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(2),
                got: 1,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_external_phys_addr_decoding_missing_byte() {
        assert_eq!(
            RecordSource::try_from_bytes(&[RecordSourceType::ExternalPhysicalAddress as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 2,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_external_phys_addr_decoding_missing_operand() {
        assert_eq!(
            RecordSource::try_from_bytes(&[RecordSourceType::ExternalPhysicalAddress as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 1,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_invalid_operand() {
        assert_eq!(
            RecordSource::try_from_bytes(&[0xFE]),
            Err(Error::InvalidValueForType {
                ty: "RecordSourceType",
                value: String::from("254"),
            })
        );
    }
}
