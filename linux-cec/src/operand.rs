#![allow(clippy::enum_variant_names)]
#![allow(clippy::len_without_is_empty)]

use bitfield_struct::bitfield;
use bitflags::bitflags;
use linux_cec_macros::{BitfieldSpecifier, Operand};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::convert::TryFrom;

use crate::{add_error_offset, check_range, constants, Error, PhysicalAddress, Result};

pub type AnalogueFrequency = u16; // TODO: Limit range
pub type Delay = u8; // TODO: Limit range
pub type DurationHours = BcdByte;
pub type Hour = BcdByte; // TODO: Limit range
pub type Minute = BcdByte; // TODO: Limit range
pub type ShortAudioDescriptor = [u8; 3];
pub type VendorId = [u8; 3];

pub trait OperandEncodable: Sized {
    fn to_bytes(&self, buf: &mut impl Extend<u8>);
    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self>;
    fn len(&self) -> usize;
}

impl OperandEncodable for u8 {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        buf.extend([*self]);
    }

    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self> {
        if bytes.len() < offset + 1 {
            Err(Error::InsufficientLength {
                required: 1,
                got: bytes.len() - offset,
            })
        } else {
            Ok(bytes[offset])
        }
    }

    fn len(&self) -> usize {
        1
    }
}

impl<T: OperandEncodable> OperandEncodable for Option<T> {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        if let Some(data) = self {
            data.to_bytes(buf);
        }
    }

    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self> {
        if bytes.len() < offset + 1 {
            Ok(None)
        } else {
            Ok(Some(T::from_bytes(bytes, offset)?))
        }
    }

    fn len(&self) -> usize {
        if let Some(ref data) = self {
            data.len()
        } else {
            0
        }
    }
}

impl OperandEncodable for [u8; 3] {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        buf.extend(*self);
    }

    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self> {
        match bytes[offset..=offset + 2].try_into() {
            Ok(array) => Ok(array),
            Err(_) => Err(Error::InsufficientLength {
                required: 3,
                got: bytes.len() - offset,
            }),
        }
    }

    fn len(&self) -> usize {
        3
    }
}

impl OperandEncodable for u16 {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        buf.extend([
            u8::try_from(*self >> 8).unwrap(),
            u8::try_from(*self & 0xFF).unwrap(),
        ]);
    }

    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self> {
        if bytes.len() < offset + 2 {
            Err(Error::InsufficientLength {
                required: 2,
                got: bytes.len() - offset,
            })
        } else {
            Ok((u16::from(bytes[offset]) << 8) | u16::from(bytes[offset + 1]))
        }
    }

    fn len(&self) -> usize {
        2
    }
}

impl OperandEncodable for bool {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        buf.extend([if *self { 1 } else { 0 }]);
    }

    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self> {
        if bytes.len() < offset + 1 {
            Err(Error::InsufficientLength {
                required: 1,
                got: bytes.len() - offset,
            })
        } else {
            Ok(bytes[offset] != 0)
        }
    }

    fn len(&self) -> usize {
        1
    }
}

pub trait TaggedLengthBuffer: Sized {
    type FixedParam: Into<u8> + TryFrom<u8> + Copy;

    fn try_new(first: Self::FixedParam, extra: &[u8]) -> Result<Self>;

    fn fixed_param(&self) -> Self::FixedParam;

    fn extra_params(&self) -> &[u8] {
        &[] as &[u8; 0]
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

    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self> {
        let first = T::try_from(bytes[offset])?;
        let mut extra = Vec::new();
        let mut offset = offset;
        while offset < bytes.len() {
            let byte = bytes[offset];
            extra.push(byte & 0x7F);
            if (byte & 0x80) == 0 {
                break;
            }
            offset += 1;
        }
        Self::try_new(first, &extra)
    }

    fn len(&self) -> usize {
        size_of::<T>() + self.extra_params().len()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BoundedBufferOperand<const S: usize, T: OperandEncodable + Default + Copy> {
    buffer: [T; S],
    len: usize,
}

impl<const S: usize, T: OperandEncodable + Default + Copy> OperandEncodable
    for BoundedBufferOperand<S, T>
{
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        for elem in &self.buffer[..self.len] {
            elem.to_bytes(buf);
        }
    }

    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self> {
        let mut buf = Vec::new();
        let mut offset = offset;
        while offset < S * size_of::<T>() && offset + size_of::<T>() <= bytes.len() {
            buf.push(T::from_bytes(bytes, offset)?);
            offset += size_of::<T>();
        }
        buf.resize(S, T::default());
        Ok(Self {
            buffer: *buf.first_chunk().unwrap(),
            len: buf.len(),
        })
    }

    fn len(&self) -> usize {
        usize::min(self.len, S) * size_of::<T>()
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

pub type BufferOperand = BoundedBufferOperand<14, u8>;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum AbortReason {
    UnrecognizedOp = constants::CEC_OP_ABORT_UNRECOGNIZED_OP,
    IncorrectMode = constants::CEC_OP_ABORT_INCORRECT_MODE,
    NoSource = constants::CEC_OP_ABORT_NO_SOURCE,
    InvalidOp = constants::CEC_OP_ABORT_INVALID_OP,
    Refused = constants::CEC_OP_ABORT_REFUSED,
    Undetermined = constants::CEC_OP_ABORT_UNDETERMINED,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum AnalogueBroadcastType {
    Cable = constants::CEC_OP_ANA_BCAST_TYPE_CABLE,
    Satellite = constants::CEC_OP_ANA_BCAST_TYPE_SATELLITE,
    Terrestrial = constants::CEC_OP_ANA_BCAST_TYPE_TERRESTRIAL,
}

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq)]
#[bits = 2]
#[repr(u8)]
pub enum AudioOutCompensated {
    NotApplicable = constants::CEC_OP_AUD_OUT_COMPENSATED_NA,
    Delay = constants::CEC_OP_AUD_OUT_COMPENSATED_DELAY,
    NoDelay = constants::CEC_OP_AUD_OUT_COMPENSATED_NO_DELAY,
    PartialDelay = constants::CEC_OP_AUD_OUT_COMPENSATED_PARTIAL_DELAY,
    #[default]
    Invalid(u8),
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum AudioRate {
    Off = constants::CEC_OP_AUD_RATE_OFF,
    WideStandard = constants::CEC_OP_AUD_RATE_WIDE_STD,
    WideFast = constants::CEC_OP_AUD_RATE_WIDE_FAST,
    WideSlow = constants::CEC_OP_AUD_RATE_WIDE_SLOW,
    NarrowStandard = constants::CEC_OP_AUD_RATE_NARROW_STD,
    NarrowFast = constants::CEC_OP_AUD_RATE_NARROW_FAST,
    NarrowSlow = constants::CEC_OP_AUD_RATE_NARROW_SLOW,
}

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq)]
#[bits = 2]
#[repr(u8)]
pub enum AudioFormatId {
    CEA861 = constants::CEC_OP_AUD_FMT_ID_CEA861,
    CEA861Cxt = constants::CEC_OP_AUD_FMT_ID_CEA861_CXT,
    #[default]
    Invalid(u8),
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum AudioOutputCompensated {
    NotApplicable = constants::CEC_OP_AUD_OUT_COMPENSATED_NA,
    Delay = constants::CEC_OP_AUD_OUT_COMPENSATED_DELAY,
    NoDelay = constants::CEC_OP_AUD_OUT_COMPENSATED_NO_DELAY,
    PartialDelay = constants::CEC_OP_AUD_OUT_COMPENSATED_PARTIAL_DELAY,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum BroadcastSystem {
    PalBg = constants::CEC_OP_BCAST_SYSTEM_PAL_BG,
    SecamLq = constants::CEC_OP_BCAST_SYSTEM_SECAM_LQ, /* SECAM L' */
    PalM = constants::CEC_OP_BCAST_SYSTEM_PAL_M,
    NtscM = constants::CEC_OP_BCAST_SYSTEM_NTSC_M,
    PalI = constants::CEC_OP_BCAST_SYSTEM_PAL_I,
    SecamDk = constants::CEC_OP_BCAST_SYSTEM_SECAM_DK,
    SecamBg = constants::CEC_OP_BCAST_SYSTEM_SECAM_BG,
    SecamL = constants::CEC_OP_BCAST_SYSTEM_SECAM_L,
    PalDk = constants::CEC_OP_BCAST_SYSTEM_PAL_DK,
    Other = constants::CEC_OP_BCAST_SYSTEM_OTHER,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum CdcErrorCode {
    None = constants::CEC_OP_CDC_ERROR_CODE_NONE,
    CapUnsupported = constants::CEC_OP_CDC_ERROR_CODE_CAP_UNSUPPORTED,
    WrongState = constants::CEC_OP_CDC_ERROR_CODE_WRONG_STATE,
    Other = constants::CEC_OP_CDC_ERROR_CODE_OTHER,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum ChannelNumberFormat {
    Fmt1Part = constants::CEC_OP_CHANNEL_NUMBER_FMT_1_PART,
    Fmt2Part = constants::CEC_OP_CHANNEL_NUMBER_FMT_2_PART,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum DeckControlMode {
    SkipForward = constants::CEC_OP_DECK_CTL_MODE_SKIP_FWD,
    SkipReverse = constants::CEC_OP_DECK_CTL_MODE_SKIP_REV,
    Stop = constants::CEC_OP_DECK_CTL_MODE_STOP,
    Eject = constants::CEC_OP_DECK_CTL_MODE_EJECT,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum DisplayControl {
    Default = constants::CEC_OP_DISP_CTL_DEFAULT,
    UntilCleared = constants::CEC_OP_DISP_CTL_UNTIL_CLEARED,
    Clear = constants::CEC_OP_DISP_CTL_CLEAR,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum EncFunctionalityState {
    ExtConNotSupported = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_NOT_SUPPORTED,
    ExtConInactive = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_INACTIVE,
    ExtConActive = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_ACTIVE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum ExternalSourceSpecifier {
    ExternalPlug = constants::CEC_OP_EXT_SRC_PLUG,
    ExternalPhysicalAddress = constants::CEC_OP_EXT_SRC_PHYS_ADDR,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum HecFunctionalityState {
    NotSupported = constants::CEC_OP_HEC_FUNC_STATE_NOT_SUPPORTED,
    Inactive = constants::CEC_OP_HEC_FUNC_STATE_INACTIVE,
    Active = constants::CEC_OP_HEC_FUNC_STATE_ACTIVE,
    ActivationField = constants::CEC_OP_HEC_FUNC_STATE_ACTIVATION_FIELD,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum HostFunctionalityState {
    NotSupported = constants::CEC_OP_HOST_FUNC_STATE_NOT_SUPPORTED,
    Inactive = constants::CEC_OP_HOST_FUNC_STATE_INACTIVE,
    Active = constants::CEC_OP_HOST_FUNC_STATE_ACTIVE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum HpdErrorCode {
    None = constants::CEC_OP_HPD_ERROR_NONE,
    InitiatorNotCapable = constants::CEC_OP_HPD_ERROR_INITIATOR_NOT_CAPABLE,
    InitiatorWrongState = constants::CEC_OP_HPD_ERROR_INITIATOR_WRONG_STATE,
    Other = constants::CEC_OP_HPD_ERROR_OTHER,
    NoneNoVideo = constants::CEC_OP_HPD_ERROR_NONE_NO_VIDEO,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum HpdStateState {
    CpEdidDisable = constants::CEC_OP_HPD_STATE_CP_EDID_DISABLE,
    CpEdidEnable = constants::CEC_OP_HPD_STATE_CP_EDID_ENABLE,
    CpEdidDisableEnable = constants::CEC_OP_HPD_STATE_CP_EDID_DISABLE_ENABLE,
    EdidDisable = constants::CEC_OP_HPD_STATE_EDID_DISABLE,
    EdidEnable = constants::CEC_OP_HPD_STATE_EDID_ENABLE,
    EdidDisableEnable = constants::CEC_OP_HPD_STATE_EDID_DISABLE_ENABLE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum MediaInfo {
    UnprotectedMedia = constants::CEC_OP_MEDIA_INFO_UNPROT_MEDIA,
    ProtectedMedia = constants::CEC_OP_MEDIA_INFO_PROT_MEDIA,
    NoMedia = constants::CEC_OP_MEDIA_INFO_NO_MEDIA,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum MenuRequestType {
    Activate = constants::CEC_OP_MENU_REQUEST_ACTIVATE,
    Deactivate = constants::CEC_OP_MENU_REQUEST_DEACTIVATE,
    Query = constants::CEC_OP_MENU_REQUEST_QUERY,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum MenuState {
    Activated = constants::CEC_OP_MENU_STATE_ACTIVATED,
    Deactivated = constants::CEC_OP_MENU_STATE_DEACTIVATED,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum NotProgrammedErrorInfo {
    NoFreeTimer = constants::CEC_OP_PROG_ERROR_NO_FREE_TIMER,
    DateOutOfRange = constants::CEC_OP_PROG_ERROR_DATE_OUT_OF_RANGE,
    RecordingSequenceError = constants::CEC_OP_PROG_ERROR_REC_SEQ_ERROR,
    InvalidExternalPlug = constants::CEC_OP_PROG_ERROR_INV_EXT_PLUG,
    InvalidExternalPhysicalAddress = constants::CEC_OP_PROG_ERROR_INV_EXT_PHYS_ADDR,
    CaUnsupported = constants::CEC_OP_PROG_ERROR_CA_UNSUPP,
    InsufficientCaEntitlements = constants::CEC_OP_PROG_ERROR_INSUF_CA_ENTITLEMENTS,
    ResolutionUnsupported = constants::CEC_OP_PROG_ERROR_RESOLUTION_UNSUPP,
    ParentalLock = constants::CEC_OP_PROG_ERROR_PARENTAL_LOCK,
    ClockFailure = constants::CEC_OP_PROG_ERROR_CLOCK_FAILURE,
    Duplicate = constants::CEC_OP_PROG_ERROR_DUPLICATE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum PowerStatus {
    On = constants::CEC_OP_POWER_STATUS_ON,
    Standby = constants::CEC_OP_POWER_STATUS_STANDBY,
    ToOn = constants::CEC_OP_POWER_STATUS_TO_ON,
    ToStandby = constants::CEC_OP_POWER_STATUS_TO_STANDBY,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum ProgrammedInfo {
    EnoughSpace = constants::CEC_OP_PROG_INFO_ENOUGH_SPACE,
    NotEnoughSpace = constants::CEC_OP_PROG_INFO_NOT_ENOUGH_SPACE,
    MayNotBeEnoughSpace = constants::CEC_OP_PROG_INFO_MIGHT_NOT_BE_ENOUGH_SPACE,
    NoneAvailable = constants::CEC_OP_PROG_INFO_NONE_AVAILABLE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum RecordSourceType {
    Own = constants::CEC_OP_RECORD_SRC_OWN,
    Digital = constants::CEC_OP_RECORD_SRC_DIGITAL,
    Analogue = constants::CEC_OP_RECORD_SRC_ANALOG,
    ExternalPlug = constants::CEC_OP_RECORD_SRC_EXT_PLUG,
    ExternalPhysicalAddress = constants::CEC_OP_RECORD_SRC_EXT_PHYS_ADDR,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum RecordStatusInfo {
    CurrentSource = constants::CEC_OP_RECORD_STATUS_CUR_SRC,
    DigitalService = constants::CEC_OP_RECORD_STATUS_DIG_SERVICE,
    AnalogService = constants::CEC_OP_RECORD_STATUS_ANA_SERVICE,
    ExternalInput = constants::CEC_OP_RECORD_STATUS_EXT_INPUT,
    NoDigitalService = constants::CEC_OP_RECORD_STATUS_NO_DIG_SERVICE,
    NoAnalogueService = constants::CEC_OP_RECORD_STATUS_NO_ANA_SERVICE,
    NoService = constants::CEC_OP_RECORD_STATUS_NO_SERVICE,
    InvalidExternalPlug = constants::CEC_OP_RECORD_STATUS_INVALID_EXT_PLUG,
    InvalidExternalPhysicalAddress = constants::CEC_OP_RECORD_STATUS_INVALID_EXT_PHYS_ADDR,
    UnsupportedCaSystem = constants::CEC_OP_RECORD_STATUS_UNSUP_CA,
    NoCaEntitlements = constants::CEC_OP_RECORD_STATUS_NO_CA_ENTITLEMENTS,
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum RcProfileId {
    ProfileNone = constants::CEC_OP_FEAT_RC_TV_PROFILE_NONE,
    Profile1 = constants::CEC_OP_FEAT_RC_TV_PROFILE_1,
    Profile2 = constants::CEC_OP_FEAT_RC_TV_PROFILE_2,
    Profile3 = constants::CEC_OP_FEAT_RC_TV_PROFILE_3,
    Profile4 = constants::CEC_OP_FEAT_RC_TV_PROFILE_4,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum ServiceIdMethod {
    ByDigitalId = constants::CEC_OP_SERVICE_ID_METHOD_BY_DIG_ID,
    ByChannel = constants::CEC_OP_SERVICE_ID_METHOD_BY_CHANNEL,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum StatusRequest {
    On = constants::CEC_OP_STATUS_REQ_ON,
    Off = constants::CEC_OP_STATUS_REQ_OFF,
    Once = constants::CEC_OP_STATUS_REQ_ONCE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum TimerClearedStatusData {
    Recording = constants::CEC_OP_TIMER_CLR_STAT_RECORDING,
    NoMatching = constants::CEC_OP_TIMER_CLR_STAT_NO_MATCHING,
    NoInfo = constants::CEC_OP_TIMER_CLR_STAT_NO_INFO,
    Cleared = constants::CEC_OP_TIMER_CLR_STAT_CLEARED,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum TunerDisplayInfo {
    Digital = constants::CEC_OP_TUNER_DISPLAY_INFO_DIGITAL,
    None = constants::CEC_OP_TUNER_DISPLAY_INFO_NONE,
    Analogue = constants::CEC_OP_TUNER_DISPLAY_INFO_ANALOGUE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
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
    Number11 = constants::CEC_OP_UI_CMD_NUMBER_11,
    Number12 = constants::CEC_OP_UI_CMD_NUMBER_12,
    Number0OrNumber10 = constants::CEC_OP_UI_CMD_NUMBER_0_OR_NUMBER_10,
    Number1 = constants::CEC_OP_UI_CMD_NUMBER_1,
    Number2 = constants::CEC_OP_UI_CMD_NUMBER_2,
    Number3 = constants::CEC_OP_UI_CMD_NUMBER_3,
    Number4 = constants::CEC_OP_UI_CMD_NUMBER_4,
    Number5 = constants::CEC_OP_UI_CMD_NUMBER_5,
    Number6 = constants::CEC_OP_UI_CMD_NUMBER_6,
    Number7 = constants::CEC_OP_UI_CMD_NUMBER_7,
    Number8 = constants::CEC_OP_UI_CMD_NUMBER_8,
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
    F1Blue = constants::CEC_OP_UI_CMD_F1_BLUE,
    F2Red = constants::CEC_OP_UI_CMD_F2_RED,
    F3Green = constants::CEC_OP_UI_CMD_F3_GREEN,
    F4Yellow = constants::CEC_OP_UI_CMD_F4_YELLOW,
    F5 = constants::CEC_OP_UI_CMD_F5,
    Data = constants::CEC_OP_UI_CMD_DATA,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
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
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Operand)]
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
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Operand)]
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
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Operand)]
    pub struct RcProfileSource: u8 {
        const HAS_DEV_ROOT_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_DEV_ROOT_MENU;
        const HAS_DEV_SETUP_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_DEV_SETUP_MENU;
        const HAS_CONTENTS_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_CONTENTS_MENU;
        const HAS_MEDIA_TOP_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_MEDIA_TOP_MENU;
        const HAS_MEDIA_CONTEXT_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_MEDIA_CONTEXT_MENU;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Operand)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Operand)]
pub struct AnalogueServiceId {
    pub broadcast_type: AnalogueBroadcastType,
    pub frequency: AnalogueFrequency,
    pub broadcast_system: BroadcastSystem,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
        buf.extend([((broadcast_system as u8) << 1) | (service_id_method as u8)]);
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

    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self> {
        use DigitalServiceBroadcastSystem as System;
        use DigitalServiceId as Id;

        if bytes.len() < offset + 7 {
            return Err(Error::InsufficientLength {
                required: 7,
                got: bytes.len() - offset,
            });
        }
        let head = bytes[offset];
        let service_id_method = ServiceIdMethod::try_from_primitive(head & 1)?;
        let broadcast_system = System::try_from_primitive(head >> 1)?;
        if service_id_method == ServiceIdMethod::ByChannel {
            let channel_id = <ChannelId as OperandEncodable>::from_bytes(bytes, offset + 1)
                .map_err(add_error_offset(1))?;
            let reserved = <u16 as OperandEncodable>::from_bytes(bytes, offset + 5)
                .map_err(add_error_offset(5))?;
            Ok(Id::Channel {
                broadcast_system,
                channel_id,
                reserved,
            })
        } else {
            Ok(match broadcast_system {
                System::AribGeneric => Id::AribGeneric(
                    OperandEncodable::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ),
                System::AtscGeneric => Id::AtscGeneric(
                    OperandEncodable::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ),
                System::DvbGeneric => Id::DvbGeneric(
                    OperandEncodable::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ),
                System::AribCs => Id::AribCs(
                    OperandEncodable::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ),
                System::AribBs => Id::AribBs(
                    OperandEncodable::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ),
                System::AribT => Id::AribT(
                    OperandEncodable::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ),
                System::AtscCable => Id::AtscCable(
                    OperandEncodable::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ),
                System::AtscSatellite => Id::AtscSatellite(
                    OperandEncodable::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ),
                System::AtscTerrestrial => Id::AtscTerrestrial(
                    OperandEncodable::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ),
                System::DvbC => Id::DvbC(
                    OperandEncodable::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ),
                System::DvbS => Id::DvbS(
                    OperandEncodable::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ),
                System::DvbS2 => Id::DvbS2(
                    OperandEncodable::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ),
                System::DvbT => Id::DvbT(
                    OperandEncodable::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ),
            })
        }
    }

    fn len(&self) -> usize {
        7
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Operand)]
pub struct Duration {
    pub hours: DurationHours,
    pub minutes: Minute,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Operand)]
pub struct Time {
    pub hour: Hour,
    pub minute: Minute,
}

#[repr(u8)]
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, IntoPrimitive, TryFromPrimitive, Operand,
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
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, IntoPrimitive, TryFromPrimitive, Operand,
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

#[bitfield(u8)]
#[derive(PartialEq, Eq, Operand)]
pub struct AudioFormatIdAndCode {
    #[bits(6)]
    pub code: usize,
    #[bits(2)]
    pub id: AudioFormatId,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Operand)]
pub struct AudioStatus {
    #[bits(7)]
    pub volume: usize,
    pub mute: bool,
}

// TODO: Limit range
#[bitfield(u8)]
#[derive(PartialEq, Eq, PartialOrd)]
pub struct BcdByte {
    #[bits(4)]
    pub ones: usize,
    #[bits(4)]
    pub tens: usize,
}

impl OperandEncodable for BcdByte {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        buf.extend([u8::from(*self)]);
    }

    fn from_bytes(bytes: &[u8], offset: usize) -> Result<BcdByte> {
        if bytes.len() < 1 + offset {
            return Err(Error::InsufficientLength {
                required: 1,
                got: bytes.len() - offset,
            });
        }
        let byte = bytes[offset];
        check_range(byte & 0xF, 0, 10)?;
        check_range(byte, 0, 100)?;
        Ok(BcdByte::from(byte))
    }

    fn len(&self) -> usize {
        1
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Operand)]
pub struct AribData {
    pub transport_stream_id: u16,
    pub service_id: u16,
    pub original_network_id: u16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Operand)]
pub struct AtscData {
    pub transport_stream_id: u16,
    pub program_number: u16,
    pub reserved: u16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Operand)]
pub struct ChannelData {
    pub channel_id: ChannelId,
    pub reserved: u16,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChannelId {
    pub number_format: ChannelNumberFormat,
    pub major_channel: u16,
    pub minor_channel: u16,
}

impl OperandEncodable for ChannelId {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        let number_format = u8::from(self.number_format);
        let major: u16 = u16::from(number_format) | (self.major_channel << 6);
        major.to_bytes(buf);
        self.minor_channel.to_bytes(buf);
    }

    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self> {
        if bytes.len() < offset + 4 {
            return Err(Error::InsufficientLength {
                required: 4,
                got: bytes.len() - offset,
            });
        }
        let major = <u16 as OperandEncodable>::from_bytes(bytes, offset)?;
        let minor_channel = <u16 as OperandEncodable>::from_bytes(bytes, offset + 2)?;
        let number_format = u8::try_from(major & 0x3F).unwrap();
        let number_format = ChannelNumberFormat::try_from_primitive(number_format)?;
        let major_channel = major >> 6;
        Ok(ChannelId {
            number_format,
            major_channel,
            minor_channel,
        })
    }

    fn len(&self) -> usize {
        4
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeviceFeatures {
    pub device_features_1: DeviceFeatures1,
    pub device_features_n: BoundedBufferOperand<14, u8>,
}

impl TaggedLengthBuffer for DeviceFeatures {
    type FixedParam = DeviceFeatures1;

    fn try_new(first: DeviceFeatures1, extra_params: &[u8]) -> Result<DeviceFeatures> {
        Ok(DeviceFeatures {
            device_features_1: first,
            device_features_n: BoundedBufferOperand::<14, u8>::from_bytes(extra_params, 0)?,
        })
    }

    fn fixed_param(&self) -> DeviceFeatures1 {
        self.device_features_1
    }

    fn extra_params(&self) -> &[u8] {
        &self.device_features_n.buffer[..self.device_features_n.len]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Operand)]
pub struct DvbData {
    pub transport_stream_id: u16,
    pub service_id: u16,
    pub original_network_id: u16,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Operand)]
pub struct LatencyFlags {
    #[bits(2)]
    pub audio_out_compensated: AudioOutCompensated,
    pub low_latency_mode: bool,
    #[bits(5)]
    _reserved: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RcProfile {
    pub rc_profile_1: RcProfile1,
    pub rc_profile_n: BoundedBufferOperand<14, u8>,
}

impl TaggedLengthBuffer for RcProfile {
    type FixedParam = RcProfile1;

    fn try_new(first: RcProfile1, extra_params: &[u8]) -> Result<RcProfile> {
        Ok(RcProfile {
            rc_profile_1: first,
            rc_profile_n: BoundedBufferOperand::<14, u8>::from_bytes(extra_params, 0)?,
        })
    }

    fn fixed_param(&self) -> RcProfile1 {
        self.rc_profile_1
    }

    fn extra_params(&self) -> &[u8] {
        &self.rc_profile_n.buffer[..self.rc_profile_n.len]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RcProfile1 {
    RcProfileSource(RcProfileSource),
    RcProfileId(RcProfileId),
}

impl From<RcProfile1> for u8 {
    fn from(profile: RcProfile1) -> u8 {
        match profile {
            RcProfile1::RcProfileSource(profile_source) => profile_source.bits(),
            RcProfile1::RcProfileId(profile_id) => profile_id.into(),
        }
    }
}

impl TryFrom<u8> for RcProfile1 {
    type Error = Error;

    fn try_from(flags: u8) -> Result<RcProfile1> {
        if (flags & 0x40) == 0x40 {
            Ok(RcProfile1::RcProfileSource(
                RcProfileSource::from_bits_retain(flags),
            ))
        } else {
            Ok(RcProfile1::RcProfileId(RcProfileId::try_from_primitive(
                flags,
            )?))
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TunerDeviceInfo {
    Analogue {
        recording: bool,
        tuner_display_info: TunerDisplayInfo,
        service_id: AnalogueServiceId,
    },
    Digital {
        recording: bool,
        tuner_display_info: TunerDisplayInfo,
        service_id: DigitalServiceId,
    },
}

impl OperandEncodable for TunerDeviceInfo {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        match self {
            TunerDeviceInfo::Analogue {
                recording,
                tuner_display_info,
                service_id,
            } => {
                let recording = if *recording { 1u8 } else { 0u8 };
                let display_info = u8::from(*tuner_display_info);
                <u8 as OperandEncodable>::to_bytes(&(recording | (display_info << 1)), buf);
                service_id.to_bytes(buf);
            }
            TunerDeviceInfo::Digital {
                recording,
                tuner_display_info,
                service_id,
            } => {
                let recording = if *recording { 1u8 } else { 0u8 };
                let display_info = u8::from(*tuner_display_info);
                <u8 as OperandEncodable>::to_bytes(&(recording | (display_info << 1)), buf);
                service_id.to_bytes(buf);
            }
        }
    }

    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self> {
        if bytes.len() - offset < 5 {
            return Err(Error::InsufficientLength {
                required: 5,
                got: bytes.len() - offset,
            });
        }
        let head = bytes[offset];
        let recording = (head & 1) == 1;
        let tuner_display_info = TunerDisplayInfo::try_from_primitive(head >> 1)?;
        match bytes.len() - offset {
            5 => Ok(TunerDeviceInfo::Analogue {
                recording,
                tuner_display_info,
                service_id: AnalogueServiceId::from_bytes(bytes, offset)?,
            }),
            8 => Ok(TunerDeviceInfo::Digital {
                recording,
                tuner_display_info,
                service_id: DigitalServiceId::from_bytes(bytes, offset)?,
            }),
            l => Err(Error::InvalidLength {
                got: l,
                expected: vec![5, 8],
            }),
        }
    }

    fn len(&self) -> usize {
        match self {
            TunerDeviceInfo::Analogue { .. } => 5,
            TunerDeviceInfo::Digital { .. } => 8,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self> {
        match bytes.len() - offset {
            1 => Ok(ExternalSource::Plug(bytes[offset])),
            2 => Ok(ExternalSource::PhysicalAddress(
                <PhysicalAddress as OperandEncodable>::from_bytes(bytes, offset)?,
            )),
            l => Err(Error::InvalidLength {
                got: l,
                expected: vec![1, 2],
            }),
        }
    }

    fn len(&self) -> usize {
        match self {
            ExternalSource::Plug(_) => 1,
            ExternalSource::PhysicalAddress(_) => 2,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

    fn from_bytes(bytes: &[u8], offset: usize) -> Result<Self> {
        let record_source_type = RecordSourceType::from_bytes(bytes, offset)?;
        match record_source_type {
            RecordSourceType::Own => Ok(RecordSource::Own),
            RecordSourceType::Digital => Ok(RecordSource::DigitalService(
                DigitalServiceId::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
            )),
            RecordSourceType::Analogue => Ok(RecordSource::AnalogueService(
                AnalogueServiceId::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
            )),
            RecordSourceType::ExternalPlug | RecordSourceType::ExternalPhysicalAddress => {
                Ok(RecordSource::External(
                    ExternalSource::from_bytes(bytes, offset + 1).map_err(add_error_offset(1))?,
                ))
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
}
