use linux_cec_macros::{Message, Operand};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::operand::OperandEncodable;
use crate::{constants, operand, PhysicalAddress};

pub trait MessageEncodable {
    const OPCODE: Opcode;

    fn to_bytes(&self) -> [u8; 15] {
        let mut raw = [0; 15];
        raw[0] = Self::OPCODE.into();

        let parameters = self.parameters();
        raw[1..=parameters.len() + 1].copy_from_slice(&self.parameters());
        raw
    }

    fn parameters(&self) -> Vec<u8> {
        Vec::new()
    }

    fn len(&self) -> usize;
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ActivateSource {
    #[parameter]
    pub address: PhysicalAddress,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ImageViewOn;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TextViewOn;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct InactiveSource {
    #[parameter]
    pub address: PhysicalAddress,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RequestActiveSource;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RoutingChange {
    #[parameter]
    pub original_address: PhysicalAddress,
    #[parameter]
    pub new_address: PhysicalAddress,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RoutingInformation {
    #[parameter]
    pub address: PhysicalAddress,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetStreamPath {
    #[parameter]
    pub address: PhysicalAddress,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Standby;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordOff;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordOn {
    #[parameter]
    pub record_source_type: operand::RecordSourceType,
    #[parameter]
    pub source: operand::RecordSource,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordStatus {
    #[parameter]
    pub status: operand::RecordStatusInfo,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordTvScreen;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClearAnalogueTimer {
    #[parameter]
    pub day_of_month: operand::DayOfMonth,
    #[parameter]
    pub month_of_year: operand::MonthOfYear,
    #[parameter]
    pub start_time: operand::Time,
    #[parameter]
    pub duration: operand::Duration,
    #[parameter]
    pub recording_sequence: operand::RecordingSequence,
    #[parameter]
    pub service_id: operand::AnalogueServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClearDigitalTimer {
    #[parameter]
    pub day_of_month: operand::DayOfMonth,
    #[parameter]
    pub month_of_year: operand::MonthOfYear,
    #[parameter]
    pub start_time: operand::Time,
    #[parameter]
    pub duration: operand::Duration,
    #[parameter]
    pub recording_sequence: operand::RecordingSequence,
    #[parameter]
    pub service_id: operand::DigitalServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClearExtTimer {
    #[parameter]
    pub day_of_month: operand::DayOfMonth,
    #[parameter]
    pub month_of_year: operand::MonthOfYear,
    #[parameter]
    pub start_time: operand::Time,
    #[parameter]
    pub duration: operand::Duration,
    #[parameter]
    pub recording_sequence: operand::RecordingSequence,
    #[parameter]
    pub external_source: operand::ExternalSource,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetAnalogueTimer {
    #[parameter]
    pub day_of_month: operand::DayOfMonth,
    #[parameter]
    pub month_of_year: operand::MonthOfYear,
    #[parameter]
    pub start_time: operand::Time,
    #[parameter]
    pub duration: operand::Duration,
    #[parameter]
    pub recording_sequence: operand::RecordingSequence,
    #[parameter]
    pub service_id: operand::AnalogueServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetDigitalTimer {
    #[parameter]
    pub day_of_month: operand::DayOfMonth,
    #[parameter]
    pub month_of_year: operand::MonthOfYear,
    #[parameter]
    pub start_time: operand::Time,
    #[parameter]
    pub duration: operand::Duration,
    #[parameter]
    pub recording_sequence: operand::RecordingSequence,
    #[parameter]
    pub service_id: operand::DigitalServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetExtTimer {
    #[parameter]
    pub day_of_month: operand::DayOfMonth,
    #[parameter]
    pub month_of_year: operand::MonthOfYear,
    #[parameter]
    pub start_time: operand::Time,
    #[parameter]
    pub duration: operand::Duration,
    #[parameter]
    pub recording_sequence: operand::RecordingSequence,
    #[parameter]
    pub external_source: operand::ExternalSource,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetTimerProgramTitle {
    #[parameter]
    pub title: operand::BufferOperand,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TimerClearedStatus {
    #[parameter]
    pub timer_cleared_status: operand::TimerClearedStatusData,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TimerStatus {
    data: u8,
    duration_available: Option<operand::Duration>,
}

impl MessageEncodable for TimerStatus {
    const OPCODE: Opcode = Opcode::TimerStatus;

    fn parameters(&self) -> Vec<u8> {
        let mut params = vec![self.data];
        if let Some(duration) = self.duration_available {
            duration.to_bytes(&mut params);
        }
        params
    }

    fn len(&self) -> usize {
        if self.duration_available.is_some() {
            3
        } else {
            1
        }
    }
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct CecVersion {
    #[parameter]
    pub version: operand::Version,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GetCecVersion;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GivePhysicalAddr;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GetMenuLanguage;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReportPhysicalAddr {
    #[parameter]
    pub physical_address: PhysicalAddress,
    #[parameter]
    pub device_type: operand::PrimaryDeviceType,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetMenuLanguage {
    #[parameter]
    pub language: [u8; 3],
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeckControl {
    #[parameter]
    pub mode: operand::DeckControlMode,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeckStatus {
    #[parameter]
    pub info: operand::DeckInfo,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveDeckStatus {
    #[parameter]
    pub request: operand::StatusRequest,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Play {
    #[parameter]
    pub mode: operand::PlayMode,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveTunerDeviceStatus {
    #[parameter]
    pub request: operand::StatusRequest,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SelectAnalogueService {
    #[parameter]
    pub service_id: operand::AnalogueServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SelectDigitalService {
    #[parameter]
    pub service_id: operand::DigitalServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TunerDeviceStatus {
    #[parameter]
    pub info: operand::TunerDeviceInfo,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TunerStepDecrement;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TunerStepIncrement;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeviceVendorId {
    #[parameter]
    pub vendor_id: operand::VendorId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveDeviceVendorId;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct VendorCommand {
    #[parameter]
    pub command: operand::BufferOperand,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct VendorCommandWithId {
    #[parameter]
    pub vendor_id: operand::VendorId,
    #[parameter]
    pub vendor_specific_data: operand::BoundedBufferOperand<11>,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct VendorRemoteButtonDown {
    rc_code: operand::BufferOperand,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct VendorRemoteButtonUp;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetOsdString {
    #[parameter]
    display_control: operand::DisplayControl,
    #[parameter]
    osd_string: operand::BoundedBufferOperand<13>,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveOsdName;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetOsdName {
    #[parameter]
    name: operand::BufferOperand,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MenuRequest {
    #[parameter]
    request_type: operand::MenuRequestType,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MenuStatus {
    #[parameter]
    state: operand::MenuState,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct UserControlPressed {
    #[parameter]
    ui_command: operand::UiCommand,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct UserControlReleased;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveDevicePowerStatus;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReportPowerStatus {
    #[parameter]
    status: operand::PowerStatus,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct FeatureAbort {
    #[parameter]
    opcode: Opcode,
    #[parameter]
    abort_reason: operand::AbortReason,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Abort;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveAudioStatus;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveSystemAudioModeStatus;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReportAudioStatus {
    #[parameter]
    status: operand::AudioStatus,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReportShortAudioDescriptor {
    short_audio_descriptor: [operand::ShortAudioDescriptor; 4],
    count: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RequestShortAudioDescriptor {
    audio_format_id_and_code: [operand::AudioFormatIdAndCode; 4],
    count: usize,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetSystemAudioMode {
    #[parameter]
    status: bool,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SystemAudioModeRequest {
    #[parameter]
    physical_address: PhysicalAddress,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SystemAudioModeStatus {
    #[parameter]
    system_audio_status: bool,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetAudioRate {
    #[parameter]
    audio_rate: operand::AudioRate,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct InitiateArc;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReportArcInitiated;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReportArcTerminated;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RequestArcInitiation;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RequestArcTermination;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TerminateArc;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CdcMessage;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
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
