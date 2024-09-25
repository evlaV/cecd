use bitfield::bitfield;
use bitflags::bitflags;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::constants;

pub type AnalogueFrequency = u16; // TODO: Limit range
pub type DurationHours = BcdByte;
pub type Hour = BcdByte; // TODO: Limit range
pub type Minute = BcdByte; // TODO: Limit range

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum AbortReason {
    UnrecognizedOp = constants::CEC_OP_ABORT_UNRECOGNIZED_OP,
    IncorrectMode = constants::CEC_OP_ABORT_INCORRECT_MODE,
    NoSource = constants::CEC_OP_ABORT_NO_SOURCE,
    InvalidOp = constants::CEC_OP_ABORT_INVALID_OP,
    Refused = constants::CEC_OP_ABORT_REFUSED,
    Undetermined = constants::CEC_OP_ABORT_UNDETERMINED,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum AnalogueBroadcastType {
    Cable = constants::CEC_OP_ANA_BCAST_TYPE_CABLE,
    Satellite = constants::CEC_OP_ANA_BCAST_TYPE_SATELLITE,
    Terrestrial = constants::CEC_OP_ANA_BCAST_TYPE_TERRESTRIAL,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum AudioRate {
    Off = constants::CEC_OP_AUD_RATE_OFF,
    WideStandard = constants::CEC_OP_AUD_RATE_WIDE_STD,
    WideFast = constants::CEC_OP_AUD_RATE_WIDE_FAST,
    WideSlow = constants::CEC_OP_AUD_RATE_WIDE_SLOW,
    NarrowStandard = constants::CEC_OP_AUD_RATE_NARROW_STD,
    NarrowFast = constants::CEC_OP_AUD_RATE_NARROW_FAST,
    NarrowSlow = constants::CEC_OP_AUD_RATE_NARROW_SLOW,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum AudioOutputCompensated {
    NotApplicable = constants::CEC_OP_AUD_OUT_COMPENSATED_NA,
    Delay = constants::CEC_OP_AUD_OUT_COMPENSATED_DELAY,
    NoDelay = constants::CEC_OP_AUD_OUT_COMPENSATED_NO_DELAY,
    PartialDelay = constants::CEC_OP_AUD_OUT_COMPENSATED_PARTIAL_DELAY,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum CdcErrorCode {
    None = constants::CEC_OP_CDC_ERROR_CODE_NONE,
    CapUnsupported = constants::CEC_OP_CDC_ERROR_CODE_CAP_UNSUPPORTED,
    WrongState = constants::CEC_OP_CDC_ERROR_CODE_WRONG_STATE,
    Other = constants::CEC_OP_CDC_ERROR_CODE_OTHER,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum ChannelNumberFormat {
    Fmt1Part = constants::CEC_OP_CHANNEL_NUMBER_FMT_1_PART,
    Fmt2Part = constants::CEC_OP_CHANNEL_NUMBER_FMT_2_PART,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum DeckControlMode {
    SkipForward = constants::CEC_OP_DECK_CTL_MODE_SKIP_FWD,
    SkipReverse = constants::CEC_OP_DECK_CTL_MODE_SKIP_REV,
    Stop = constants::CEC_OP_DECK_CTL_MODE_STOP,
    Eject = constants::CEC_OP_DECK_CTL_MODE_EJECT,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum DisplayControl {
    Default = constants::CEC_OP_DISP_CTL_DEFAULT,
    UntilCleared = constants::CEC_OP_DISP_CTL_UNTIL_CLEARED,
    Clear = constants::CEC_OP_DISP_CTL_CLEAR,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum EncFunctionalityState {
    ExtConNotSupported = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_NOT_SUPPORTED,
    ExtConInactive = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_INACTIVE,
    ExtConActive = constants::CEC_OP_ENC_FUNC_STATE_EXT_CON_ACTIVE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum ExternalSourceSpecifier {
    ExternalPlug = constants::CEC_OP_EXT_SRC_PLUG,
    ExternalPhysicalAddress = constants::CEC_OP_EXT_SRC_PHYS_ADDR,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum HecFunctionalityState {
    NotSupported = constants::CEC_OP_HEC_FUNC_STATE_NOT_SUPPORTED,
    Inactive = constants::CEC_OP_HEC_FUNC_STATE_INACTIVE,
    Active = constants::CEC_OP_HEC_FUNC_STATE_ACTIVE,
    ActivationField = constants::CEC_OP_HEC_FUNC_STATE_ACTIVATION_FIELD,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum HostFunctionalityState {
    NotSupported = constants::CEC_OP_HOST_FUNC_STATE_NOT_SUPPORTED,
    Inactive = constants::CEC_OP_HOST_FUNC_STATE_INACTIVE,
    Active = constants::CEC_OP_HOST_FUNC_STATE_ACTIVE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum HpdErrorCode {
    None = constants::CEC_OP_HPD_ERROR_NONE,
    InitiatorNotCapable = constants::CEC_OP_HPD_ERROR_INITIATOR_NOT_CAPABLE,
    InitiatorWrongState = constants::CEC_OP_HPD_ERROR_INITIATOR_WRONG_STATE,
    Other = constants::CEC_OP_HPD_ERROR_OTHER,
    NoneNoVideo = constants::CEC_OP_HPD_ERROR_NONE_NO_VIDEO,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum HpdStateState {
    CpEdidDisable = constants::CEC_OP_HPD_STATE_CP_EDID_DISABLE,
    CpEdidEnable = constants::CEC_OP_HPD_STATE_CP_EDID_ENABLE,
    CpEdidDisableEnable = constants::CEC_OP_HPD_STATE_CP_EDID_DISABLE_ENABLE,
    EdidDisable = constants::CEC_OP_HPD_STATE_EDID_DISABLE,
    EdidEnable = constants::CEC_OP_HPD_STATE_EDID_ENABLE,
    EdidDisableEnable = constants::CEC_OP_HPD_STATE_EDID_DISABLE_ENABLE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum MediaInfo {
    UnprotectedMedia = constants::CEC_OP_MEDIA_INFO_UNPROT_MEDIA,
    ProtectedMedia = constants::CEC_OP_MEDIA_INFO_PROT_MEDIA,
    NoMedia = constants::CEC_OP_MEDIA_INFO_NO_MEDIA,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum MenuRequestType {
    Activate = constants::CEC_OP_MENU_REQUEST_ACTIVATE,
    Deactivate = constants::CEC_OP_MENU_REQUEST_DEACTIVATE,
    Query = constants::CEC_OP_MENU_REQUEST_QUERY,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum MenuState {
    Activated = constants::CEC_OP_MENU_STATE_ACTIVATED,
    Deactivated = constants::CEC_OP_MENU_STATE_DEACTIVATED,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum PowerStatus {
    On = constants::CEC_OP_POWER_STATUS_ON,
    Standby = constants::CEC_OP_POWER_STATUS_STANDBY,
    ToOn = constants::CEC_OP_POWER_STATUS_TO_ON,
    ToStandby = constants::CEC_OP_POWER_STATUS_TO_STANDBY,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum ProgrammedInfo {
    EnoughSpace = constants::CEC_OP_PROG_INFO_ENOUGH_SPACE,
    NotEnoughSpace = constants::CEC_OP_PROG_INFO_NOT_ENOUGH_SPACE,
    MayNotBeEnoughSpace = constants::CEC_OP_PROG_INFO_MIGHT_NOT_BE_ENOUGH_SPACE,
    NoneAvailable = constants::CEC_OP_PROG_INFO_NONE_AVAILABLE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum RecordSourceType {
    Own = constants::CEC_OP_RECORD_SRC_OWN,
    Digital = constants::CEC_OP_RECORD_SRC_DIGITAL,
    Analogue = constants::CEC_OP_RECORD_SRC_ANALOG,
    ExternalPlug = constants::CEC_OP_RECORD_SRC_EXT_PLUG,
    ExternalPhysicalAddress = constants::CEC_OP_RECORD_SRC_EXT_PHYS_ADDR,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum RcProfileId {
    ProfileNone = constants::CEC_OP_FEAT_RC_TV_PROFILE_NONE,
    Profile1 = constants::CEC_OP_FEAT_RC_TV_PROFILE_1,
    Profile2 = constants::CEC_OP_FEAT_RC_TV_PROFILE_2,
    Profile3 = constants::CEC_OP_FEAT_RC_TV_PROFILE_3,
    Profile4 = constants::CEC_OP_FEAT_RC_TV_PROFILE_4,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum ServiceIdMethod {
    ByDigitalId = constants::CEC_OP_SERVICE_ID_METHOD_BY_DIG_ID,
    ByChannel = constants::CEC_OP_SERVICE_ID_METHOD_BY_CHANNEL,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum StatusRequest {
    On = constants::CEC_OP_STATUS_REQ_ON,
    Off = constants::CEC_OP_STATUS_REQ_OFF,
    Once = constants::CEC_OP_STATUS_REQ_ONCE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum TimerClearedStatusData {
    Recording = constants::CEC_OP_TIMER_CLR_STAT_RECORDING,
    NoMatching = constants::CEC_OP_TIMER_CLR_STAT_NO_MATCHING,
    NoInfo = constants::CEC_OP_TIMER_CLR_STAT_NO_INFO,
    Cleared = constants::CEC_OP_TIMER_CLR_STAT_CLEARED,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum TunerDisplayInfo {
    Digital = constants::CEC_OP_TUNER_DISPLAY_INFO_DIGITAL,
    None = constants::CEC_OP_TUNER_DISPLAY_INFO_NONE,
    Analogue = constants::CEC_OP_TUNER_DISPLAY_INFO_ANALOGUE,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum Version {
    // These first few versions predate CEC specification
    // but are theoretically valid otherwise
    V1_1 = 0,
    V1_2 = 1,
    V1_2a = 2,
    V1_3 = 3,
    V1_3a = constants::CEC_OP_CEC_VERSION_1_3A,
    V1_4 = constants::CEC_OP_CEC_VERSION_1_4,
    V2_0 = constants::CEC_OP_CEC_VERSION_2_0,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
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
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct AllDeviceType: u8 {
        const TV = constants::CEC_OP_ALL_DEVTYPE_TV;
        const RECORDING = constants::CEC_OP_ALL_DEVTYPE_RECORD;
        const TUNER = constants::CEC_OP_ALL_DEVTYPE_TUNER;
        const PLAYBACK = constants::CEC_OP_ALL_DEVTYPE_PLAYBACK;
        const AUDIOSYSTEM = constants::CEC_OP_ALL_DEVTYPE_AUDIOSYSTEM;
        const SWITCH = constants::CEC_OP_ALL_DEVTYPE_SWITCH;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct RcProfileSource: u8 {
        const HAS_DEV_ROOT_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_DEV_ROOT_MENU;
        const HAS_DEV_SETUP_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_DEV_SETUP_MENU;
        const HAS_CONTENTS_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_CONTENTS_MENU;
        const HAS_MEDIA_TOP_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_MEDIA_TOP_MENU;
        const HAS_MEDIA_CONTEXT_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_MEDIA_CONTEXT_MENU;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ServiceId {
    AribData {
        transport_stream_id: u16,
        service_id: u16,
        original_network_id: u16,
    },
    AtscData {
        transport_stream_id: u16,
        program_number: u16,
        reserved: u16,
    },
    DvbData {
        transport_stream_id: u16,
        service_id: u16,
        original_network_id: u16,
    },
    ChannelData {
        channel_id: ChannelId,
        reserved: u16,
    },
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DigitalServiceId {
    pub service_id_method: ServiceIdMethod,
    pub digital_broadcast_system: DigitalServiceBroadcastSystem,
    pub service_id: ServiceId,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Duration {
    pub hours: DurationHours,
    pub minutes: Minute,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time {
    pub hour: Hour,
    pub minute: Minute,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, IntoPrimitive, TryFromPrimitive)]
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

// TODO: Limit range
bitfield! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct BcdByte(u8);
    impl Debug;

    ones, _: 3, 0;
    tens, _: 7, 4;
}

bitfield! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct ChannelId(u16);
    impl Debug;

    channel_number_format, _: 5, 0; // TODO: How do I specify the type?
    major_channel_number, _: 15, 6;
    minor_channel_number, _: 31, 16;
}

bitfield! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct AudioStatus(u8);
    impl Debug;

    audio_mute_status, _: 1, 0;
    audio_volume_satus, _: 7, 2;
}
