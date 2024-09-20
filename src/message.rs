use bitflags::bitflags;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{constants, LogicalAddress, MsgFlags, RxStatus, Timestamp, TxStatus};

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
pub enum Opcode {
    ActivateSource = constants::CEC_MSG_ACTIVE_SOURCE,
    ImageViewOn = constants::CEC_MSG_IMAGE_VIEW_ON,
    TextViewOn = constants::CEC_MSG_TEXT_VIEW_ON,
    InactiveSource = constants::CEC_MSG_INACTIVE_SOURCE,
    RequestActiveSource = constants::CEC_MSG_REQUEST_ACTIVE_SOURCE,
    RoutingChange = constants::CEC_MSG_ROUTING_CHANGE,
    RoutingInformation = constants::CEC_MSG_ROUTING_INFORMATION,
    SetStreamPath = constants::CEC_MSG_SET_STREAM_PATH,
    Standby = constants::CEC_MSG_STANDBY,
    RecordOff = constants::CEC_MSG_RECORD_OFF,
    RecordOn = constants::CEC_MSG_RECORD_ON,
    RecordStatus = constants::CEC_MSG_RECORD_STATUS,
    RecordTvScreen = constants::CEC_MSG_RECORD_TV_SCREEN,
    ClearAnalogueTimer = constants::CEC_MSG_CLEAR_ANALOGUE_TIMER,
    ClearDigitalTimer = constants::CEC_MSG_CLEAR_DIGITAL_TIMER,
    ClearExtTimer = constants::CEC_MSG_CLEAR_EXT_TIMER,
    SetAnalogueTimer = constants::CEC_MSG_SET_ANALOGUE_TIMER,
    SetDigitalTimer = constants::CEC_MSG_SET_DIGITAL_TIMER,
    SetExtTimer = constants::CEC_MSG_SET_EXT_TIMER,
    SetTimerProgramTitle = constants::CEC_MSG_SET_TIMER_PROGRAM_TITLE,
    TimerClearedStatus = constants::CEC_MSG_TIMER_CLEARED_STATUS,
    TimerStatus = constants::CEC_MSG_TIMER_STATUS,
    CecVersion = constants::CEC_MSG_CEC_VERSION,
    GetCecVersion = constants::CEC_MSG_GET_CEC_VERSION,
    GivePhysicalAddr = constants::CEC_MSG_GIVE_PHYSICAL_ADDR,
    GetMenuLanguage = constants::CEC_MSG_GET_MENU_LANGUAGE,
    ReportPhysicalAddr = constants::CEC_MSG_REPORT_PHYSICAL_ADDR,
    SetMenuLanguage = constants::CEC_MSG_SET_MENU_LANGUAGE,
    ReportFeatures = constants::CEC_MSG_REPORT_FEATURES, /* HDMI 2.0 */
    GiveFeatures = constants::CEC_MSG_GIVE_FEATURES,     /* HDMI 2.0 */
    DeckControl = constants::CEC_MSG_DECK_CONTROL,
    DeckStatus = constants::CEC_MSG_DECK_STATUS,
    GiveDeckStatus = constants::CEC_MSG_GIVE_DECK_STATUS,
    Play = constants::CEC_MSG_PLAY,
    GiveTunerDeviceStatus = constants::CEC_MSG_GIVE_TUNER_DEVICE_STATUS,
    SelectAnalogueService = constants::CEC_MSG_SELECT_ANALOGUE_SERVICE,
    SelectDigitalService = constants::CEC_MSG_SELECT_DIGITAL_SERVICE,
    TunerDeviceStatus = constants::CEC_MSG_TUNER_DEVICE_STATUS,
    TunerStepDecrement = constants::CEC_MSG_TUNER_STEP_DECREMENT,
    TunerStepIncrement = constants::CEC_MSG_TUNER_STEP_INCREMENT,
    DeviceVendorId = constants::CEC_MSG_DEVICE_VENDOR_ID,
    GiveDeviceVendorId = constants::CEC_MSG_GIVE_DEVICE_VENDOR_ID,
    VendorCommand = constants::CEC_MSG_VENDOR_COMMAND,
    VendorCommandWithId = constants::CEC_MSG_VENDOR_COMMAND_WITH_ID,
    VendorRemoteButtonDown = constants::CEC_MSG_VENDOR_REMOTE_BUTTON_DOWN,
    VendorRemoteButtonUp = constants::CEC_MSG_VENDOR_REMOTE_BUTTON_UP,
    SetOsdString = constants::CEC_MSG_SET_OSD_STRING,
    GiveOsdName = constants::CEC_MSG_GIVE_OSD_NAME,
    SetOsdName = constants::CEC_MSG_SET_OSD_NAME,
    MenuRequest = constants::CEC_MSG_MENU_REQUEST,
    MenuStatus = constants::CEC_MSG_MENU_STATUS,
    UserControlPressed = constants::CEC_MSG_USER_CONTROL_PRESSED,
    UserControlReleased = constants::CEC_MSG_USER_CONTROL_RELEASED,
    GiveDevicePowerStatus = constants::CEC_MSG_GIVE_DEVICE_POWER_STATUS,
    ReportPowerStatus = constants::CEC_MSG_REPORT_POWER_STATUS,
    FeatureAbort = constants::CEC_MSG_FEATURE_ABORT,
    Abort = constants::CEC_MSG_ABORT,
    GiveAudioStatus = constants::CEC_MSG_GIVE_AUDIO_STATUS,
    GiveSystemAudioModeStatus = constants::CEC_MSG_GIVE_SYSTEM_AUDIO_MODE_STATUS,
    ReportAudioStatus = constants::CEC_MSG_REPORT_AUDIO_STATUS,
    ReportShortAudioDescriptor = constants::CEC_MSG_REPORT_SHORT_AUDIO_DESCRIPTOR,
    RequestShortAudioDescriptor = constants::CEC_MSG_REQUEST_SHORT_AUDIO_DESCRIPTOR,
    SetSystemAudioMode = constants::CEC_MSG_SET_SYSTEM_AUDIO_MODE,
    SystemAudioModeRequest = constants::CEC_MSG_SYSTEM_AUDIO_MODE_REQUEST,
    SystemAudioModeStatus = constants::CEC_MSG_SYSTEM_AUDIO_MODE_STATUS,
    SetAudioVolumeLevel = constants::CEC_MSG_SET_AUDIO_VOLUME_LEVEL,
    SetAudioRate = constants::CEC_MSG_SET_AUDIO_RATE,
    InitiateArc = constants::CEC_MSG_INITIATE_ARC,
    ReportArcInitiated = constants::CEC_MSG_REPORT_ARC_INITIATED,
    ReportArcTerminated = constants::CEC_MSG_REPORT_ARC_TERMINATED,
    RequestArcInitiation = constants::CEC_MSG_REQUEST_ARC_INITIATION,
    RequestArcTermination = constants::CEC_MSG_REQUEST_ARC_TERMINATION,
    TerminateArc = constants::CEC_MSG_TERMINATE_ARC,
    RequestCurrentLatency = constants::CEC_MSG_REQUEST_CURRENT_LATENCY,
    ReportCurrentLatency = constants::CEC_MSG_REPORT_CURRENT_LATENCY,
    CdcMessage = constants::CEC_MSG_CDC_MESSAGE,
}

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
pub enum CecVersion {
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
pub enum RecordStatus {
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
pub enum TimerClearedStatus {
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
    #[derive(Debug, Copy, Clone)]
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
    #[derive(Debug, Copy, Clone)]
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
    #[derive(Debug, Copy, Clone)]
    pub struct RcProfileSource: u8 {
        const HAS_DEV_ROOT_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_DEV_ROOT_MENU;
        const HAS_DEV_SETUP_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_DEV_SETUP_MENU;
        const HAS_CONTENTS_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_CONTENTS_MENU;
        const HAS_MEDIA_TOP_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_MEDIA_TOP_MENU;
        const HAS_MEDIA_CONTEXT_MENU = constants::CEC_OP_FEAT_RC_SRC_HAS_MEDIA_CONTEXT_MENU;
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone)]
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

/// CEC message structure.
#[repr(C)]
pub struct CecMessage {
    /// Timestamp in nanoseconds using CLOCK_MONOTONIC. Set by the
    /// driver when the message transmission has finished.
    tx_ts: Timestamp,
    /// Timestamp in nanoseconds using CLOCK_MONOTONIC. Set by the
    /// driver when the message was received.
    rx_ts: Timestamp,
    /// Length in bytes of the message.
    len: u32,
    /**
     * The timeout (in ms) that is used to timeout CEC_RECEIVE.
     * Set to 0 if you want to wait forever. This timeout can also be
     * used with CEC_TRANSMIT as the timeout for waiting for a reply.
     * If 0, then it will use a 1 second timeout instead of waiting
     * forever as is done with CEC_RECEIVE.
     */
    timeout: u32,
    /// The framework assigns a sequence number to messages that are
    /// sent. This can be used to track replies to previously sent messages.
    sequence: u32,
    /// Set to 0.
    flags: MsgFlags,
    /// The message payload.
    msg: [u8; constants::CEC_MAX_MSG_SIZE],
    /**
     * This field is ignored with CEC_RECEIVE and is only used by
     * CEC_TRANSMIT. If non-zero, then wait for a reply with this
     * opcode. Set to CEC_MSG_FEATURE_ABORT if you want to wait for
     * a possible ABORT reply. If there was an error when sending the
     * msg or FeatureAbort was returned, then reply is set to 0.
     * If reply is non-zero upon return, then len/msg are set to
     * the received message.
     * If reply is zero upon return and status has the
     * CEC_TX_STATUS_FEATURE_ABORT bit set, then len/msg are set to
     * the received feature abort message.
     * If reply is zero upon return and status has the
     * CEC_TX_STATUS_MAX_RETRIES bit set, then no reply was seen at
     * all. If reply is non-zero for CEC_TRANSMIT and the message is a
     * broadcast, then -EINVAL is returned.
     * if reply is non-zero, then timeout is set to 1000 (the required
     * maximum response time).
     */
    reply: u8,
    /// The message receive status bits. Set by the driver.
    rx_status: RxStatus,
    /// The message transmit status bits. Set by the driver.
    tx_status: TxStatus,
    /// The number of 'Arbitration Lost' events. Set by the driver.
    tx_arb_lost_cnt: u8,
    /// The number of 'Not Acknowledged' events. Set by the driver.
    tx_nack_cnt: u8,
    /// The number of 'Low Drive Detected' events. Set by the driver.
    tx_low_drive_cnt: u8,
    /// The number of 'Error' events. Set by the driver.
    tx_error_cnt: u8,
}

impl CecMessage {
    /// Return the initiator's logical address.
    pub fn initiator(&self) -> u8 {
        self.msg[0] >> 4
    }

    /// Return the destination's logical address.
    pub fn destination(&self) -> u8 {
        self.msg[0] & 0xf
    }

    /// Return the opcode of the message, None for poll
    pub fn raw_opcode(&self) -> Option<u8> {
        if self.len > 1 {
            Some(self.msg[1])
        } else {
            None
        }
    }

    /// Return true if this is a broadcast message.
    pub fn is_broadcast(&self) -> bool {
        (self.msg[0] & 0xf) == 0xf
    }

    /**
     * Initialize the message structure.
     * @initiator: the logical address of the initiator
     * @destination: the logical address of the destination (0xf for broadcast)
     *
     * The whole structure is zeroed, the len field is set to 1 (i.e. a poll
     * message) and the initiator and destination are filled in.
     */
    pub fn new(initiator: LogicalAddress, destination: LogicalAddress) -> CecMessage {
        let mut msg = CecMessage {
            tx_ts: 0,
            rx_ts: 0,
            len: 1,
            timeout: 0,
            sequence: 0,
            flags: MsgFlags::empty(),
            msg: [0; 16],
            reply: 0,
            rx_status: RxStatus::empty(),
            tx_status: TxStatus::empty(),
            tx_arb_lost_cnt: 0,
            tx_nack_cnt: 0,
            tx_low_drive_cnt: 0,
            tx_error_cnt: 0,
        };
        msg.msg[0] = (initiator << 4) | destination;

        msg
    }

    pub(crate) fn with_timeout(timeout_ms: u32) -> CecMessage {
        CecMessage {
            tx_ts: 0,
            rx_ts: 0,
            len: 0,
            timeout: timeout_ms,
            sequence: 0,
            flags: MsgFlags::empty(),
            msg: [0; 16],
            reply: 0,
            rx_status: RxStatus::empty(),
            tx_status: TxStatus::empty(),
            tx_arb_lost_cnt: 0,
            tx_nack_cnt: 0,
            tx_low_drive_cnt: 0,
            tx_error_cnt: 0,
        }
    }

    /**
     * Fill in destination/initiator in a reply message.
     * @orig: the original message structure
     *
     * Set the msg destination to the orig initiator and the msg initiator to the
     * orig destination. Note that msg and orig may be the same pointer, in which
     * case the change is done in place.
     */
    fn set_reply_to(&mut self, orig: &CecMessage) {
        /* The destination becomes the initiator and vice versa */
        self.msg[0] = (orig.destination() << 4) | orig.initiator();
        self.reply = 0;
        self.timeout = 0;
    }

    /// Return true if this message contains the result of an earlier non-blocking transmit
    pub fn recv_is_tx_result(&self) -> bool {
        self.sequence != 0 && !self.tx_status.is_empty() && self.rx_status.is_empty()
    }

    /// Return true if this message contains the reply of an earlier non-blocking transmit
    pub fn recv_is_rx_result(&self) -> bool {
        self.sequence != 0 && self.tx_status.is_empty() && !self.rx_status.is_empty()
    }

    pub fn status_is_ok(&self) -> bool {
        if !self.tx_status.is_empty() && !self.tx_status.contains(TxStatus::OK) {
            return false;
        }
        if !self.rx_status.is_empty() && !self.rx_status.contains(RxStatus::OK) {
            return false;
        }
        if self.tx_status.is_empty() && self.rx_status.is_empty() {
            return false;
        }
        !self.rx_status.contains(RxStatus::FEATURE_ABORT)
    }

    pub fn opcode(&self) -> Option<Opcode> {
        let raw = self.raw_opcode()?;
        Opcode::try_from(raw).ok()
    }
}
