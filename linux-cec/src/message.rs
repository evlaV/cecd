use linux_cec_macros::Message;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::ffi::c_char;
use std::mem::size_of;

use crate::operand::*;
use crate::{constants, LogicalAddress, MsgFlags, PhysicalAddress, RxStatus, Timestamp, TxStatus};

pub trait MessageEncodable {
    const OPCODE: Opcode;

    fn to_bytes(&self) -> [u8; 15] {
        let mut raw = [0; 15];
        raw[0] = Self::OPCODE.into();

        let parameters = self.parameters();
        raw[1..=parameters.len() + 1].copy_from_slice(self.parameters());
        raw
    }

    fn parameters(&self) -> &[u8] {
        &[]
    }

    fn len(&self) -> usize;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct MessageBuffer {
    buffer: [u8; 14],
    len: usize,
}

impl MessageBuffer {
    fn parameters(&self) -> &[u8] {
        &self.buffer[..self.len]
    }

    fn len(&self) -> usize {
        usize::min(self.len, 14)
    }
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
    //#[parameter]
    //pub source: RecordSource,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordStatus {
    #[parameter]
    pub status: RecordStatusInfo,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordTvScreen;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClearAnalogueTimer {
    #[parameter]
    pub day_of_month: DayOfMonth,
    #[parameter]
    pub month_of_year: MonthOfYear,
    #[parameter]
    pub start_time: Time,
    #[parameter]
    pub duration: Duration,
    #[parameter]
    pub recording_sequence: RecordingSequence,
    #[parameter]
    pub analogue_broadcast_type: AnalogueBroadcastType,
    #[parameter]
    pub analogue_frequency: AnalogueFrequency,
    #[parameter]
    pub broadcast_system: BroadcastSystem,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClearDigitalTimer {
    #[parameter]
    pub day_of_month: DayOfMonth,
    #[parameter]
    pub month_of_year: MonthOfYear,
    #[parameter]
    pub start_time: Time,
    #[parameter]
    pub duration: Duration,
    #[parameter]
    pub recording_sequence: RecordingSequence,
    #[parameter]
    pub digital_service_identification: DigitalServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClearExtTimer {
    #[parameter]
    pub day_of_month: DayOfMonth,
    #[parameter]
    pub month_of_year: MonthOfYear,
    #[parameter]
    pub start_time: Time,
    #[parameter]
    pub duration: Duration,
    #[parameter]
    pub recording_sequence: RecordingSequence,
    //#[parameter]
    //pub external_source: ExternalSource,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetAnalogueTimer {
    #[parameter]
    pub day_of_month: DayOfMonth,
    #[parameter]
    pub month_of_year: MonthOfYear,
    #[parameter]
    pub start_time: Time,
    #[parameter]
    pub duration: Duration,
    #[parameter]
    pub recording_sequence: RecordingSequence,
    #[parameter]
    pub analogue_broadcast_type: AnalogueBroadcastType,
    #[parameter]
    pub analogue_frequency: AnalogueFrequency,
    #[parameter]
    pub broadcast_system: BroadcastSystem,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetDigitalTimer {
    #[parameter]
    pub day_of_month: DayOfMonth,
    #[parameter]
    pub month_of_year: MonthOfYear,
    #[parameter]
    pub start_time: Time,
    #[parameter]
    pub duration: Duration,
    #[parameter]
    pub recording_sequence: RecordingSequence,
    #[parameter]
    pub digital_service_identification: DigitalServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetExtTimer {
    #[parameter]
    pub day_of_month: DayOfMonth,
    #[parameter]
    pub month_of_year: MonthOfYear,
    #[parameter]
    pub start_time: Time,
    #[parameter]
    pub duration: Duration,
    #[parameter]
    pub recording_sequence: RecordingSequence,
    //#[parameter]
    //pub external_source: ExternalSource,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetTimerProgramTitle {
    buffer: MessageBuffer,
}

impl MessageEncodable for SetTimerProgramTitle {
    const OPCODE: Opcode = Opcode::SetTimerProgramTitle;

    fn parameters(&self) -> &[u8] {
        self.buffer.parameters()
    }

    fn len(&self) -> usize {
        self.buffer.len() + 1
    }
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TimerClearedStatus {
    #[parameter]
    pub timer_cleared_status: TimerClearedStatusData,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TimerStatus {
    data: u8,
    duration_available: Option<Duration>,
}

impl MessageEncodable for TimerStatus {
    const OPCODE: Opcode = Opcode::TimerStatus;

    // TODO

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
    pub version: Version,
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
    pub device_type: PrimaryDeviceType,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetMenuLanguage {
    #[parameter]
    pub language: [c_char; 3],
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeckControl {
    #[parameter]
    pub mode: DeckControlMode,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeckStatus {
    #[parameter]
    pub info: DeckInfo,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveDeckStatus {
    #[parameter]
    pub request: StatusRequest,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Play {
    #[parameter]
    pub mode: PlayMode,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveTunerDeviceStatus {
    #[parameter]
    pub request: StatusRequest,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SelectAnalogueService {
    #[parameter]
    pub analogue_broadcast_type: AnalogueBroadcastType,
    #[parameter]
    pub analogue_frequency: AnalogueFrequency,
    #[parameter]
    pub broadcast_system: BroadcastSystem,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TunerDeviceStatus {
    //#[parameter]
    //pub info: TunerDeviceInfo,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TunerStepDecrement;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TunerStepIncrement;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeviceVendorId {
    #[parameter]
    pub vendor_id: [u8; 3],
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveDeviceVendorId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct VendorCommand {
    buffer: MessageBuffer,
}

impl MessageEncodable for VendorCommand {
    const OPCODE: Opcode = Opcode::VendorCommand;

    fn parameters(&self) -> &[u8] {
        self.buffer.parameters()
    }

    fn len(&self) -> usize {
        self.buffer.len() + 1
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct VendorCommandWithId {
    buffer: MessageBuffer,
}

impl MessageEncodable for VendorCommandWithId {
    const OPCODE: Opcode = Opcode::VendorCommandWithId;

    fn parameters(&self) -> &[u8] {
        self.buffer.parameters()
    }

    fn len(&self) -> usize {
        self.buffer.len() + 1
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct VendorRemoteButtonDown {
    buffer: MessageBuffer,
}

impl MessageEncodable for VendorRemoteButtonDown {
    const OPCODE: Opcode = Opcode::VendorRemoteButtonDown;

    fn parameters(&self) -> &[u8] {
        self.buffer.parameters()
    }

    fn len(&self) -> usize {
        self.buffer.len() + 1
    }
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct VendorRemoteButtonUp;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetOsdString {
    buffer: MessageBuffer,
}

impl MessageEncodable for SetOsdString {
    const OPCODE: Opcode = Opcode::SetOsdString;

    fn parameters(&self) -> &[u8] {
        self.buffer.parameters()
    }

    fn len(&self) -> usize {
        self.buffer.len() + 1
    }
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveOsdName;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetOsdName {
    buffer: MessageBuffer,
}

impl MessageEncodable for SetOsdName {
    const OPCODE: Opcode = Opcode::SetOsdName;

    fn parameters(&self) -> &[u8] {
        self.buffer.parameters()
    }

    fn len(&self) -> usize {
        self.buffer.len() + 1
    }
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MenuRequest {
    #[parameter]
    request_type: MenuRequestType,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MenuStatus {
    #[parameter]
    state: MenuState,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct UserControlPressed {
    #[parameter]
    ui_command: UiCommand,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct UserControlReleased;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveDevicePowerStatus;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReportPowerStatus {
    #[parameter]
    status: PowerStatus,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct FeatureAbort {
    #[parameter]
    opcode: Opcode,
    #[parameter]
    abort_reason: AbortReason,
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
    status: AudioStatus,
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
    audio_rate: AudioRate,
}

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
