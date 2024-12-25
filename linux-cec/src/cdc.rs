/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

#![allow(clippy::len_without_is_empty)]

use bitfield_struct::bitfield;
#[cfg(test)]
use linux_cec_macros::opcode_test;
use linux_cec_macros::{BitfieldSpecifier, MessageEnum, Operand};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::operand::OperandEncodable;
use crate::{constants, operand, PhysicalAddress, Result};
#[cfg(test)]
use crate::{Error, Range};

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[bits = 2]
#[repr(u8)]
pub enum CdcErrorCode {
    NoError = constants::CEC_OP_HPD_ERROR_NONE,
    InitiatorNotCapable = constants::CEC_OP_HPD_ERROR_INITIATOR_NOT_CAPABLE,
    InitiatorIncapableState = constants::CEC_OP_HPD_ERROR_INITIATOR_WRONG_STATE,
    Other = constants::CEC_OP_HPD_ERROR_OTHER,
}

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[bits = 2]
#[repr(u8)]
pub enum FunctionalityState {
    NotSupported = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_NOT_SUPPORTED,
    Inactive = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_INACTIVE,
    Active = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_ACTIVE,
    #[default]
    Invalid(u8),
}

pub type EncFunctionalityState = FunctionalityState;
pub type HostFunctionalityState = FunctionalityState;

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[bits = 2]
#[repr(u8)]
pub enum HecFunctionalityState {
    NotSupported = constants::CEC_OP_HEC_FUNC_STATE_NOT_SUPPORTED,
    Inactive = constants::CEC_OP_HEC_FUNC_STATE_INACTIVE,
    Active = constants::CEC_OP_HEC_FUNC_STATE_ACTIVE,
    ActivationField = constants::CEC_OP_HEC_FUNC_STATE_ACTIVATION_FIELD,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash, Operand)]
pub struct HecState {
    #[bits(2)]
    pub cdc_error: CdcErrorCode,
    #[bits(2)]
    pub enc_functionality: EncFunctionalityState,
    #[bits(2)]
    pub host_functionality: HostFunctionalityState,
    #[bits(2)]
    pub hec_functionality: HecFunctionalityState,
}

#[cfg(test)]
mod test_hec_state {
    use super::*;

    opcode_test! {
        ty: HecState,
        instance: HecState::new()
            .with_cdc_error(CdcErrorCode::NoError)
            .with_enc_functionality(EncFunctionalityState::Inactive)
            .with_host_functionality(HostFunctionalityState::Active)
            .with_hec_functionality(HecFunctionalityState::ActivationField),
        bytes: [0xE4],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            HecState::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(1),
                quantity: "bytes"
            })
        );
    }
}

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[bits = 4]
#[repr(u8)]
pub enum HpdErrorCode {
    NoError = constants::CEC_OP_HPD_ERROR_NONE,
    InitiatorNotCapable = constants::CEC_OP_HPD_ERROR_INITIATOR_NOT_CAPABLE,
    InitiatorIncapableState = constants::CEC_OP_HPD_ERROR_INITIATOR_WRONG_STATE,
    Other = constants::CEC_OP_HPD_ERROR_OTHER,
    NoVideo = constants::CEC_OP_HPD_ERROR_NONE_NO_VIDEO,
    #[default]
    Invalid(u8),
}

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[bits = 4]
#[repr(u8)]
pub enum HpdState {
    CpAndEdidDisable = constants::CEC_OP_HPD_STATE_CP_EDID_DISABLE,
    CpAndEdidEnable = constants::CEC_OP_HPD_STATE_CP_EDID_ENABLE,
    CpAndEdidDisableEnable = constants::CEC_OP_HPD_STATE_CP_EDID_DISABLE_ENABLE,
    EdidDisable = constants::CEC_OP_HPD_STATE_EDID_DISABLE,
    EdidEnable = constants::CEC_OP_HPD_STATE_EDID_ENABLE,
    EdidDisableEnable = constants::CEC_OP_HPD_STATE_EDID_DISABLE_ENABLE,
    #[default]
    Invalid(u8),
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash, Operand)]
pub struct InputPortHpdState {
    #[bits(4)]
    pub state: HpdState,
    #[bits(4)]
    pub input_port: usize,
}

#[cfg(test)]
mod test_input_port_hpd_state {
    use super::*;

    opcode_test! {
        ty: InputPortHpdState,
        instance: InputPortHpdState::new()
            .with_input_port(0xA)
            .with_state(HpdState::EdidDisableEnable),
        bytes: [0xA5],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            InputPortHpdState::try_from_bytes(&[]),
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
pub struct HpdStateErrorCode {
    #[bits(4)]
    pub error_code: HpdErrorCode,
    #[bits(4)]
    pub state: HpdState,
}

#[cfg(test)]
mod test_hpd_state_error_code {
    use super::*;

    opcode_test! {
        ty: HpdStateErrorCode,
        instance: HpdStateErrorCode::new()
            .with_state(HpdState::EdidDisableEnable)
            .with_error_code(HpdErrorCode::InitiatorNotCapable),
        bytes: [0x51],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            HpdStateErrorCode::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(1),
                quantity: "bytes"
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct HecField {
    pub input: [bool; 14],
    pub output: bool,
}

impl OperandEncodable for HecField {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        let word = if self.output { 0x4000 } else { 0 };
        let word = self
            .input
            .iter()
            .enumerate()
            .fold(word, |accum, (idx, bit)| accum | (*bit as u16) << idx);
        word.to_bytes(buf);
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        let word = u16::try_from_bytes(bytes)?;
        let mut input = [false; 14];
        for (idx, item) in input.iter_mut().enumerate() {
            *item = (word >> idx) & 1 == 1;
        }
        Ok(HecField {
            input,
            output: (word & 0x4000) != 0,
        })
    }

    fn len(&self) -> usize {
        2
    }
}

#[cfg(test)]
mod test_hec_field {
    use super::*;

    opcode_test! {
        ty: HecField,
        instance: HecField {
            input: [
                 true, false, false, false,
                false,  true, false, false,
                false, false,  true,  true,
                false, false,
            ],
            output: true,
        },
        bytes: [0x4C, 0x21],
        extra: [Overfull],
    }

    #[test]
    fn test_decode_empty() {
        assert_eq!(
            HecField::try_from_bytes(&[]),
            Err(Error::OutOfRange {
                got: 0,
                expected: Range::AtLeast(2),
                quantity: "bytes"
            })
        );
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, MessageEnum)]
#[repr(u8)]
pub enum Message {
    // TODO: Unit tests
    HecInquireState {
        terminating_address1: PhysicalAddress,
        terminating_address2: PhysicalAddress,
    } = constants::CEC_MSG_CDC_HEC_INQUIRE_STATE,
    // TODO: Unit tests
    HecReportState {
        physical_address: PhysicalAddress,
        state: HecState,
        field: Option<HecField>,
    } = constants::CEC_MSG_CDC_HEC_REPORT_STATE,
    // TODO: Unit tests
    HecSetStateAdjacent {
        terminating_address: PhysicalAddress,
        set_state: bool,
    } = constants::CEC_MSG_CDC_HEC_SET_STATE_ADJACENT,
    // TODO: Unit tests
    HecSetState {
        terminating_address1: PhysicalAddress,
        terminating_address2: PhysicalAddress,
        set_state: bool,
        terminating_addresses: operand::BoundedBufferOperand<3, PhysicalAddress>,
    } = constants::CEC_MSG_CDC_HEC_SET_STATE,
    // TODO: Unit tests
    HecRequestDeactivation {
        terminating_address1: PhysicalAddress,
        terminating_address2: PhysicalAddress,
        terminating_address3: PhysicalAddress,
    } = constants::CEC_MSG_CDC_HEC_REQUEST_DEACTIVATION,
    HecNotifyAlive = constants::CEC_MSG_CDC_HEC_NOTIFY_ALIVE,
    HecDiscover = constants::CEC_MSG_CDC_HEC_DISCOVER,
    // TODO: Unit tests
    HpdSetState(InputPortHpdState) = constants::CEC_MSG_CDC_HPD_SET_STATE,
    // TODO: Unit tests
    HpdReportState(HpdStateErrorCode) = constants::CEC_MSG_CDC_HPD_REPORT_STATE,
}

impl Message {
    pub fn opcode(&self) -> Opcode {
        let opcode = unsafe { *<*const _>::from(self).cast::<u8>() };
        Opcode::try_from_primitive(opcode).unwrap()
    }
}

impl OperandEncodable for Message {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        let bytes = Message::to_bytes(self);
        buf.extend(bytes);
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        Message::try_from_bytes(bytes)
    }

    fn len(&self) -> usize {
        Message::len(self)
    }
}
