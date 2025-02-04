/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

//! Capability Discovery and Control support

#![allow(clippy::len_without_is_empty)]

use bitfield_struct::bitfield;
#[cfg(test)]
use linux_cec_macros::{message_test, opcode_test};
use linux_cec_macros::{BitfieldSpecifier, MessageEnum, Operand};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::operand::OperandEncodable;
#[cfg(test)]
use crate::Error;
use crate::{constants, operand, PhysicalAddress, Range, Result};

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
            .fold(word, |accum, (idx, bit)| accum | (u16::from(*bit) << idx));
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

    fn expected_len() -> Range<usize> {
        Range::AtLeast(2)
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
    HecInquireState {
        terminating_address1: PhysicalAddress,
        terminating_address2: PhysicalAddress,
    } = constants::CEC_MSG_CDC_HEC_INQUIRE_STATE,
    HecReportState {
        physical_address: PhysicalAddress,
        state: HecState,
        field: Option<HecField>,
    } = constants::CEC_MSG_CDC_HEC_REPORT_STATE,
    HecSetStateAdjacent {
        terminating_address: PhysicalAddress,
        state: bool,
    } = constants::CEC_MSG_CDC_HEC_SET_STATE_ADJACENT,
    HecSetState {
        terminating_address1: PhysicalAddress,
        terminating_address2: PhysicalAddress,
        state: bool,
        terminating_addresses: operand::BoundedBufferOperand<3, PhysicalAddress>,
    } = constants::CEC_MSG_CDC_HEC_SET_STATE,
    HecRequestDeactivation {
        terminating_address1: PhysicalAddress,
        terminating_address2: PhysicalAddress,
        terminating_address3: PhysicalAddress,
    } = constants::CEC_MSG_CDC_HEC_REQUEST_DEACTIVATION,
    HecNotifyAlive = constants::CEC_MSG_CDC_HEC_NOTIFY_ALIVE,
    HecDiscover = constants::CEC_MSG_CDC_HEC_DISCOVER,
    HpdSetState(InputPortHpdState) = constants::CEC_MSG_CDC_HPD_SET_STATE,
    HpdReportState(HpdStateErrorCode) = constants::CEC_MSG_CDC_HPD_REPORT_STATE,
}

impl Message {
    #[must_use]
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

    fn expected_len() -> Range<usize> {
        Range::AtLeast(1)
    }
}

#[cfg(test)]
mod test_hec_inquire_state {
    use super::*;

    message_test! {
        ty: HecInquireState,
        instance: Message::HecInquireState {
            terminating_address1: PhysicalAddress(0x1234),
            terminating_address2: PhysicalAddress(0x5678),
        },
        bytes: [0x12, 0x34, 0x56, 0x78],
        extra: [Overfull, Empty],
    }

    #[test]
    fn test_decoding_missing_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecInquireState as u8, 0x12, 0x34, 0x56]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(5),
                got: 4,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecInquireState as u8, 0x12, 0x34]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(5),
                got: 3,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand_and_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecInquireState as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(5),
                got: 2,
                quantity: "bytes",
            })
        );
    }
}

#[cfg(test)]
mod test_hec_report_state {
    use super::*;

    message_test! {
        name: _field,
        ty: HecReportState,
        instance: Message::HecReportState {
            physical_address: PhysicalAddress(0x1234),
            state: HecState::new()
                .with_cdc_error(CdcErrorCode::NoError)
                .with_enc_functionality(EncFunctionalityState::Inactive)
                .with_host_functionality(HostFunctionalityState::Active)
                .with_hec_functionality(HecFunctionalityState::ActivationField),
            field: Some(HecField {
                input: [
                     true, false, false, false,
                    false,  true, false, false,
                    false, false,  true,  true,
                    false, false,
                ],
                output: true,
            }),
        },
        bytes: [0x12, 0x34, 0xE4, 0x4C, 0x21],
        extra: [Overfull],
    }

    message_test! {
        name: _no_field,
        ty: HecReportState,
        instance: Message::HecReportState {
            physical_address: PhysicalAddress(0x1234),
            state: HecState::new()
                .with_cdc_error(CdcErrorCode::NoError)
                .with_enc_functionality(EncFunctionalityState::Inactive)
                .with_host_functionality(HostFunctionalityState::Active)
                .with_hec_functionality(HecFunctionalityState::ActivationField),
            field: None,
        },
        bytes: [0x12, 0x34, 0xE4],
    }

    #[test]
    fn test_decoding_missing_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecReportState as u8, 0x12, 0x34, 0xE4, 0x4C]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(6),
                got: 5,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecReportState as u8, 0x12, 0x34]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(4),
                got: 3,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand_and_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecReportState as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(4),
                got: 2,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operands() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecReportState as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(4),
                got: 1,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_opcode() {
        assert_eq!(
            Message::HecReportState {
                physical_address: PhysicalAddress(0x1234),
                state: HecState::new()
                    .with_cdc_error(CdcErrorCode::NoError)
                    .with_enc_functionality(EncFunctionalityState::Inactive)
                    .with_host_functionality(HostFunctionalityState::Active)
                    .with_hec_functionality(HecFunctionalityState::ActivationField),
                field: None,
            }
            .opcode(),
            Opcode::HecReportState
        );
    }
}

#[cfg(test)]
mod test_hec_set_state_adjacent {
    use super::*;

    message_test! {
        ty: HecSetStateAdjacent,
        instance: Message::HecSetStateAdjacent {
            terminating_address: PhysicalAddress(0x1234),
            state: true,
        },
        bytes: [0x12, 0x34, 0x01],
        extra: [Overfull, Empty],
    }

    #[test]
    fn test_decoding_missing_operand() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecSetStateAdjacent as u8, 0x12, 0x34]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(4),
                got: 3,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand_and_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecSetStateAdjacent as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(4),
                got: 2,
                quantity: "bytes",
            })
        );
    }
}

#[cfg(test)]
mod test_hec_set_state {
    use super::*;

    message_test! {
        name: _empty,
        ty: HecSetState,
        instance: Message::HecSetState {
            terminating_address1: PhysicalAddress(0x1234),
            terminating_address2: PhysicalAddress(0x5678),
            state: true,
            terminating_addresses: operand::BoundedBufferOperand::default(),
        },
        bytes: [0x12, 0x34, 0x56, 0x78, 0x01],
    }

    message_test! {
        name: _1_extra,
        ty: HecSetState,
        instance: Message::HecSetState {
            terminating_address1: PhysicalAddress(0x1234),
            terminating_address2: PhysicalAddress(0x5678),
            state: true,
            terminating_addresses: operand::BoundedBufferOperand::try_from(
                [PhysicalAddress(0xABCD)].as_ref()).unwrap(),
        },
        bytes: [0x12, 0x34, 0x56, 0x78, 0x01, 0xAB, 0xCD],
    }

    message_test! {
        name: _2_extra,
        ty: HecSetState,
        instance: Message::HecSetState {
            terminating_address1: PhysicalAddress(0x1234),
            terminating_address2: PhysicalAddress(0x5678),
            state: true,
            terminating_addresses: operand::BoundedBufferOperand::try_from([
                PhysicalAddress(0xABCD),
                PhysicalAddress(0xEF01)
            ].as_ref()).unwrap(),
        },
        bytes: [0x12, 0x34, 0x56, 0x78, 0x01, 0xAB, 0xCD, 0xEF, 0x01],
    }

    message_test! {
        name: _full,
        ty: HecSetState,
        instance: Message::HecSetState {
            terminating_address1: PhysicalAddress(0x1234),
            terminating_address2: PhysicalAddress(0x5678),
            state: true,
            terminating_addresses: operand::BoundedBufferOperand::try_from([
                PhysicalAddress(0xABCD),
                PhysicalAddress(0xEF01),
                PhysicalAddress(0x2345)
            ].as_ref()).unwrap(),
        },
        bytes: [0x12, 0x34, 0x56, 0x78, 0x01, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45],
        extra: [Overfull],
    }

    #[test]
    fn test_opcode() {
        assert_eq!(
            Message::HecSetState {
                terminating_address1: PhysicalAddress(0x1234),
                terminating_address2: PhysicalAddress(0x5678),
                state: true,
                terminating_addresses: operand::BoundedBufferOperand::default(),
            }
            .opcode(),
            Opcode::HecSetState
        );
    }

    #[test]
    fn test_decoding_missing_operand_1() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecSetState as u8, 0x12, 0x34, 0x56, 0x78]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(6),
                got: 5,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand_1_and_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecSetState as u8, 0x12, 0x34, 0x56]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(6),
                got: 4,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand_2() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecSetState as u8, 0x12, 0x34]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(6),
                got: 3,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand_2_and_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecSetState as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(6),
                got: 2,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operands() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecSetState as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(6),
                got: 1,
                quantity: "bytes",
            })
        );
    }
}

#[cfg(test)]
mod test_hec_request_deactivation {
    use super::*;

    message_test! {
        ty: HecRequestDeactivation,
        instance: Message::HecRequestDeactivation {
            terminating_address1: PhysicalAddress(0x1234),
            terminating_address2: PhysicalAddress(0x5678),
            terminating_address3: PhysicalAddress(0x9ABC),
        },
        bytes: [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC],
        extra: [Overfull, Empty],
    }

    #[test]
    fn test_decoding_missing_byte() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::HecRequestDeactivation as u8,
                0x12,
                0x34,
                0x56,
                0x78,
                0x9A
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(7),
                got: 6,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand_1() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::HecRequestDeactivation as u8,
                0x12,
                0x34,
                0x56,
                0x78
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(7),
                got: 5,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand_1_and_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecRequestDeactivation as u8, 0x12, 0x34, 0x56]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(7),
                got: 4,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand_2() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecRequestDeactivation as u8, 0x12, 0x34]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(7),
                got: 3,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand_2_and_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::HecRequestDeactivation as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(7),
                got: 2,
                quantity: "bytes",
            })
        );
    }
}

#[cfg(test)]
mod test_hpd_set_state {
    use super::*;

    message_test! {
        ty: HpdSetState,
        instance: Message::HpdSetState(InputPortHpdState::new()
            .with_input_port(0xA)
            .with_state(HpdState::EdidDisableEnable)),
        bytes: [0xA5],
        extra: [Overfull, Empty],
    }
}

#[cfg(test)]
mod test_hpd_report_state {
    use super::*;

    message_test! {
        ty: HpdReportState,
        instance: Message::HpdReportState(HpdStateErrorCode::new()
            .with_state(HpdState::EdidDisableEnable)
            .with_error_code(HpdErrorCode::InitiatorNotCapable)),
        bytes: [0x51],
        extra: [Overfull, Empty],
    }
}
