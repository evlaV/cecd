use bitfield_struct::bitfield;
use linux_cec_macros::{BitfieldSpecifier, Operand};

use crate::operand::OperandEncodable;
use crate::{constants, Result};

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[bits = 2]
#[repr(u8)]
pub enum EncFunctionalityState {
    NotSupported = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_NOT_SUPPORTED,
    Inactive = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_INACTIVE,
    Active = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_ACTIVE,
    #[default]
    Invalid(u8),
}

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[bits = 2]
#[repr(u8)]
pub enum HecFunctionalityState {
    NotSupported = constants::CEC_OP_HEC_FUNC_STATE_NOT_SUPPORTED,
    Inactive = constants::CEC_OP_HEC_FUNC_STATE_INACTIVE,
    Active = constants::CEC_OP_HEC_FUNC_STATE_ACTIVE,
    ActivationField = constants::CEC_OP_HEC_FUNC_STATE_ACTIVATION_FIELD,
    #[default]
    Invalid(u8),
}

// TODO: Unit tests
#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash, Operand)]
pub struct HecState {
    #[bits(2)]
    pub hec_functionality: HecFunctionalityState,
    #[bits(2)]
    pub host_functionality: HostFunctionalityState,
    #[bits(2)]
    pub enc_functionality: EncFunctionalityState,
    #[bits(2)]
    _reserved: usize,
}

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[bits = 2]
#[repr(u8)]
pub enum HostFunctionalityState {
    NotSupported = constants::CEC_OP_HOST_FUNC_STATE_NOT_SUPPORTED,
    Inactive = constants::CEC_OP_HOST_FUNC_STATE_INACTIVE,
    Active = constants::CEC_OP_HOST_FUNC_STATE_ACTIVE,
    #[default]
    Invalid(u8),
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

// TODO: Unit tests
#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash, Operand)]
pub struct InputPortHpdState {
    #[bits(4)]
    pub state: HpdState,
    #[bits(4)]
    pub input_port: usize,
}

// TODO: Unit tests
#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash, Operand)]
pub struct HpdStateErrorCode {
    #[bits(4)]
    pub error_code: HpdErrorCode,
    #[bits(4)]
    pub state: HpdState,
}

// TODO: Unit tests
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
        for idx in 0..14 {
            input[idx] = (word >> idx) & 1 == 1;
        }
        Ok(HecField {
            input,
            output: (word & 0x4000) != 0,
        })
    }

    fn len(&self) -> usize {
        1
    }
}

pub type HecSupportField = HecField;
pub type HecActivationField = HecField;
