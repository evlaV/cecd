use linux_cec_macros::{MessageEnum, Operand};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::operand::OperandEncodable;
use crate::{cdc, constants, operand, PhysicalAddress, Result};
#[cfg(test)]
use crate::{Error, Range};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, MessageEnum)]
#[repr(u8)]
pub enum Message {
    ActiveSource {
        address: PhysicalAddress,
    } = constants::CEC_MSG_ACTIVE_SOURCE,
    ImageViewOn = constants::CEC_MSG_IMAGE_VIEW_ON,
    TextViewOn = constants::CEC_MSG_TEXT_VIEW_ON,
    InactiveSource {
        address: PhysicalAddress,
    } = constants::CEC_MSG_INACTIVE_SOURCE,
    RequestActiveSource = constants::CEC_MSG_REQUEST_ACTIVE_SOURCE,
    RoutingChange {
        original_address: PhysicalAddress,
        new_address: PhysicalAddress,
    } = constants::CEC_MSG_ROUTING_CHANGE,
    RoutingInformation {
        address: PhysicalAddress,
    } = constants::CEC_MSG_ROUTING_INFORMATION,
    SetStreamPath {
        address: PhysicalAddress,
    } = constants::CEC_MSG_SET_STREAM_PATH,
    Standby = constants::CEC_MSG_STANDBY,
    RecordOff = constants::CEC_MSG_RECORD_OFF,
    RecordOn {
        source: operand::RecordSource,
    } = constants::CEC_MSG_RECORD_ON,
    RecordStatus {
        status: operand::RecordStatusInfo,
    } = constants::CEC_MSG_RECORD_STATUS,
    RecordTvScreen = constants::CEC_MSG_RECORD_TV_SCREEN,
    // TODO: Unit tests
    ClearAnalogueTimer {
        day_of_month: operand::DayOfMonth,
        month_of_year: operand::MonthOfYear,
        start_time: operand::Time,
        duration: operand::Duration,
        recording_sequence: operand::RecordingSequence,
        service_id: operand::AnalogueServiceId,
    } = constants::CEC_MSG_CLEAR_ANALOGUE_TIMER,
    // TODO: Unit tests
    ClearDigitalTimer {
        day_of_month: operand::DayOfMonth,
        month_of_year: operand::MonthOfYear,
        start_time: operand::Time,
        duration: operand::Duration,
        recording_sequence: operand::RecordingSequence,
        service_id: operand::DigitalServiceId,
    } = constants::CEC_MSG_CLEAR_DIGITAL_TIMER,
    // TODO: Unit tests
    ClearExtTimer {
        day_of_month: operand::DayOfMonth,
        month_of_year: operand::MonthOfYear,
        start_time: operand::Time,
        duration: operand::Duration,
        recording_sequence: operand::RecordingSequence,
        external_source: operand::ExternalSource,
    } = constants::CEC_MSG_CLEAR_EXT_TIMER,
    // TODO: Unit tests
    SetAnalogueTimer {
        day_of_month: operand::DayOfMonth,
        month_of_year: operand::MonthOfYear,
        start_time: operand::Time,
        duration: operand::Duration,
        recording_sequence: operand::RecordingSequence,
        service_id: operand::AnalogueServiceId,
    } = constants::CEC_MSG_SET_ANALOGUE_TIMER,
    // TODO: Unit tests
    SetDigitalTimer {
        day_of_month: operand::DayOfMonth,
        month_of_year: operand::MonthOfYear,
        start_time: operand::Time,
        duration: operand::Duration,
        recording_sequence: operand::RecordingSequence,
        service_id: operand::DigitalServiceId,
    } = constants::CEC_MSG_SET_DIGITAL_TIMER,
    // TODO: Unit tests
    SetExtTimer {
        day_of_month: operand::DayOfMonth,
        month_of_year: operand::MonthOfYear,
        start_time: operand::Time,
        duration: operand::Duration,
        recording_sequence: operand::RecordingSequence,
        external_source: operand::ExternalSource,
    } = constants::CEC_MSG_SET_EXT_TIMER,
    // TODO: Unit tests
    SetTimerProgramTitle {
        title: operand::BufferOperand,
    } = constants::CEC_MSG_SET_TIMER_PROGRAM_TITLE,
    // TODO: Unit tests
    TimerClearedStatus {
        timer_cleared_status: operand::TimerClearedStatusData,
    } = constants::CEC_MSG_TIMER_CLEARED_STATUS,
    // TODO: Unit tests
    TimerStatus {
        data: operand::TimerStatusData,
    } = constants::CEC_MSG_TIMER_STATUS,
    // TODO: Unit tests
    CecVersion {
        version: operand::Version,
    } = constants::CEC_MSG_CEC_VERSION,
    GetCecVersion = constants::CEC_MSG_GET_CEC_VERSION,
    GivePhysicalAddr = constants::CEC_MSG_GIVE_PHYSICAL_ADDR,
    GetMenuLanguage = constants::CEC_MSG_GET_MENU_LANGUAGE,
    // TODO: Unit tests
    ReportPhysicalAddr {
        physical_address: PhysicalAddress,
        device_type: operand::PrimaryDeviceType,
    } = constants::CEC_MSG_REPORT_PHYSICAL_ADDR,
    // TODO: Unit tests
    SetMenuLanguage {
        language: [u8; 3],
    } = constants::CEC_MSG_SET_MENU_LANGUAGE,
    // TODO: Unit tests
    DeckControl {
        mode: operand::DeckControlMode,
    } = constants::CEC_MSG_DECK_CONTROL,
    // TODO: Unit tests
    DeckStatus {
        info: operand::DeckInfo,
    } = constants::CEC_MSG_DECK_STATUS,
    // TODO: Unit tests
    GiveDeckStatus {
        request: operand::StatusRequest,
    } = constants::CEC_MSG_GIVE_DECK_STATUS,
    // TODO: Unit tests
    Play {
        mode: operand::PlayMode,
    } = constants::CEC_MSG_PLAY,
    // TODO: Unit tests
    GiveTunerDeviceStatus {
        request: operand::StatusRequest,
    } = constants::CEC_MSG_GIVE_TUNER_DEVICE_STATUS,
    // TODO: Unit tests
    SelectAnalogueService {
        service_id: operand::AnalogueServiceId,
    } = constants::CEC_MSG_SELECT_ANALOGUE_SERVICE,
    // TODO: Unit tests
    SelectDigitalService {
        service_id: operand::DigitalServiceId,
    } = constants::CEC_MSG_SELECT_DIGITAL_SERVICE,
    // TODO: Unit tests
    TunerDeviceStatus {
        info: operand::TunerDeviceInfo,
    } = constants::CEC_MSG_TUNER_DEVICE_STATUS,
    TunerStepDecrement = constants::CEC_MSG_TUNER_STEP_DECREMENT,
    TunerStepIncrement = constants::CEC_MSG_TUNER_STEP_INCREMENT,
    // TODO: Unit tests
    DeviceVendorId {
        vendor_id: operand::VendorId,
    } = constants::CEC_MSG_DEVICE_VENDOR_ID,
    GiveDeviceVendorId = constants::CEC_MSG_GIVE_DEVICE_VENDOR_ID,
    // TODO: Unit tests
    VendorCommand {
        command: operand::BufferOperand,
    } = constants::CEC_MSG_VENDOR_COMMAND,
    // TODO: Unit tests
    VendorCommandWithId {
        vendor_id: operand::VendorId,
        vendor_specific_data: operand::BoundedBufferOperand<11, u8>,
    } = constants::CEC_MSG_VENDOR_COMMAND_WITH_ID,
    // TODO: Unit tests
    VendorRemoteButtonDown {
        rc_code: operand::BufferOperand,
    } = constants::CEC_MSG_VENDOR_REMOTE_BUTTON_DOWN,
    VendorRemoteButtonUp = constants::CEC_MSG_VENDOR_REMOTE_BUTTON_UP,
    // TODO: Unit tests
    SetOsdString {
        display_control: operand::DisplayControl,
        osd_string: operand::BoundedBufferOperand<13, u8>,
    } = constants::CEC_MSG_SET_OSD_STRING,
    GiveOsdName = constants::CEC_MSG_GIVE_OSD_NAME,
    // TODO: Unit tests
    SetOsdName {
        name: operand::BufferOperand,
    } = constants::CEC_MSG_SET_OSD_NAME,
    // TODO: Unit tests
    MenuRequest {
        request_type: operand::MenuRequestType,
    } = constants::CEC_MSG_MENU_REQUEST,
    // TODO: Unit tests
    MenuStatus {
        state: operand::MenuState,
    } = constants::CEC_MSG_MENU_STATUS,
    // TODO: Unit tests
    UserControlPressed {
        ui_command: operand::UiCommand,
    } = constants::CEC_MSG_USER_CONTROL_PRESSED,
    UserControlReleased = constants::CEC_MSG_USER_CONTROL_RELEASED,
    GiveDevicePowerStatus = constants::CEC_MSG_GIVE_DEVICE_POWER_STATUS,
    // TODO: Unit tests
    ReportPowerStatus {
        status: operand::PowerStatus,
    } = constants::CEC_MSG_REPORT_POWER_STATUS,
    // TODO: Unit tests
    FeatureAbort {
        opcode: Opcode,
        abort_reason: operand::AbortReason,
    } = constants::CEC_MSG_FEATURE_ABORT,
    Abort = constants::CEC_MSG_ABORT,
    GiveAudioStatus = constants::CEC_MSG_GIVE_AUDIO_STATUS,
    GiveSystemAudioModeStatus = constants::CEC_MSG_GIVE_SYSTEM_AUDIO_MODE_STATUS,
    // TODO: Unit tests
    ReportAudioStatus {
        status: operand::AudioStatus,
    } = constants::CEC_MSG_REPORT_AUDIO_STATUS,
    // TODO: Unit tests
    ReportShortAudioDescriptor {
        descriptors: operand::BoundedBufferOperand<4, operand::ShortAudioDescriptor>,
    } = constants::CEC_MSG_REPORT_SHORT_AUDIO_DESCRIPTOR,
    // TODO: Unit tests
    RequestShortAudioDescriptor {
        descriptors: operand::BoundedBufferOperand<4, operand::AudioFormatIdAndCode>,
    } = constants::CEC_MSG_REQUEST_SHORT_AUDIO_DESCRIPTOR,
    // TODO: Unit tests
    SetSystemAudioMode {
        status: bool,
    } = constants::CEC_MSG_SET_SYSTEM_AUDIO_MODE,
    // TODO: Unit tests
    SystemAudioModeRequest {
        physical_address: PhysicalAddress,
    } = constants::CEC_MSG_SYSTEM_AUDIO_MODE_REQUEST,
    // TODO: Unit tests
    SystemAudioModeStatus {
        system_audio_status: bool,
    } = constants::CEC_MSG_SYSTEM_AUDIO_MODE_STATUS,
    // TODO: Unit tests
    SetAudioRate {
        audio_rate: operand::AudioRate,
    } = constants::CEC_MSG_SET_AUDIO_RATE,
    /* HDMI 1.4b */
    InitiateArc = constants::CEC_MSG_INITIATE_ARC,
    ReportArcInitiated = constants::CEC_MSG_REPORT_ARC_INITIATED,
    ReportArcTerminated = constants::CEC_MSG_REPORT_ARC_TERMINATED,
    RequestArcInitiation = constants::CEC_MSG_REQUEST_ARC_INITIATION,
    RequestArcTermination = constants::CEC_MSG_REQUEST_ARC_TERMINATION,
    TerminateArc = constants::CEC_MSG_TERMINATE_ARC,
    // TODO: Unit tests
    CdcMessage {
        initiator: PhysicalAddress,
        message: CdcMessage,
    } = constants::CEC_MSG_CDC_MESSAGE,
    /* HDMI 2.0 */
    // TODO: Unit tests
    ReportFeatures {
        version: operand::Version,
        device_types: operand::AllDeviceTypes,
        rc_profile: operand::RcProfile,
        dev_features: operand::DeviceFeatures,
    } = constants::CEC_MSG_REPORT_FEATURES,
    GiveFeatures = constants::CEC_MSG_GIVE_FEATURES,
    // TODO: Unit tests
    RequestCurrentLatency {
        physical_address: PhysicalAddress,
    } = constants::CEC_MSG_REQUEST_CURRENT_LATENCY,
    // TODO: Unit tests
    ReportCurrentLatency {
        physical_address: PhysicalAddress,
        video_latency: operand::Delay,
        flags: operand::LatencyFlags,
        audio_output_delay: Option<operand::Delay>,
    } = constants::CEC_MSG_REPORT_CURRENT_LATENCY,
}

impl Message {
    pub fn opcode(&self) -> Opcode {
        let opcode = unsafe { *<*const _>::from(self).cast::<u8>() };
        Opcode::try_from_primitive(opcode).unwrap()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, MessageEnum)]
#[repr(u8)]
pub enum CdcMessage {
    HecInquireState {
        terminating_address1: PhysicalAddress,
        terminating_address2: PhysicalAddress,
    } = constants::CEC_MSG_CDC_HEC_INQUIRE_STATE,
    HecReportState {
        physical_address: PhysicalAddress,
        state: cdc::HecState,
        support: cdc::HecSupportField,
        activation: cdc::HecActivationField,
    } = constants::CEC_MSG_CDC_HEC_REPORT_STATE,
    HecSetStateAdjacent {
        terminating_address: PhysicalAddress,
        set_state: bool,
    } = constants::CEC_MSG_CDC_HEC_SET_STATE_ADJACENT,
    HecSetState {
        terminating_address1: PhysicalAddress,
        terminating_address2: PhysicalAddress,
        set_state: bool,
        terminating_addresses: operand::BoundedBufferOperand<3, PhysicalAddress>,
    } = constants::CEC_MSG_CDC_HEC_SET_STATE,
    HecRequestDeactivation {
        terminating_address1: PhysicalAddress,
        terminating_address2: PhysicalAddress,
        terminating_address3: PhysicalAddress,
    } = constants::CEC_MSG_CDC_HEC_REQUEST_DEACTIVATION,
    HecNotifyAlive = constants::CEC_MSG_CDC_HEC_NOTIFY_ALIVE,
    HecDiscover = constants::CEC_MSG_CDC_HEC_DISCOVER,
    HpdSetState(cdc::InputPortHpdState) = constants::CEC_MSG_CDC_HPD_SET_STATE,
    HpdReportState(cdc::HpdStateErrorCode) = constants::CEC_MSG_CDC_HPD_REPORT_STATE,
}

impl CdcMessage {
    pub fn opcode(&self) -> CdcOpcode {
        let opcode = unsafe { *<*const _>::from(self).cast::<u8>() };
        CdcOpcode::try_from_primitive(opcode).unwrap()
    }
}

impl OperandEncodable for CdcMessage {
    fn to_bytes(&self, buf: &mut impl Extend<u8>) {
        let bytes = CdcMessage::to_bytes(self);
        buf.extend(bytes.into_iter());
    }

    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        CdcMessage::try_from_bytes(bytes)
    }

    fn len(&self) -> usize {
        CdcMessage::len(self)
    }
}

#[cfg(test)]
mod test_active_source {
    use super::*;

    #[test]
    fn test_len() {
        assert_eq!(Message::ActiveSource { address: 0 }.len(), 3);
    }

    #[test]
    fn test_opcode() {
        assert_eq!(
            Message::ActiveSource { address: 0 }.opcode(),
            Opcode::ActiveSource
        );
    }

    #[test]
    fn test_encoding() {
        assert_eq!(
            &Message::ActiveSource { address: 0x1234 }.to_bytes(),
            &[Opcode::ActiveSource as u8, 0x12, 0x34]
        );
    }

    #[test]
    fn test_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::ActiveSource as u8, 0x12, 0x34]),
            Ok(Message::ActiveSource { address: 0x1234 })
        );
    }

    #[test]
    fn test_decoding_missing_operand() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::ActiveSource as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 1,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::ActiveSource as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 2,
                quantity: "bytes",
            })
        );
    }
}

#[cfg(test)]
mod test_inactive_source {
    use super::*;

    #[test]
    fn test_len() {
        assert_eq!(Message::InactiveSource { address: 0 }.len(), 3);
    }

    #[test]
    fn test_opcode() {
        assert_eq!(
            Message::InactiveSource { address: 0 }.opcode(),
            Opcode::InactiveSource
        );
    }

    #[test]
    fn test_encoding() {
        assert_eq!(
            &Message::InactiveSource { address: 0x1234 }.to_bytes(),
            &[Opcode::InactiveSource as u8, 0x12, 0x34]
        );
    }

    #[test]
    fn test_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::InactiveSource as u8, 0x12, 0x34]),
            Ok(Message::InactiveSource { address: 0x1234 })
        );
    }

    #[test]
    fn test_decoding_missing_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::InactiveSource as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 2,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::InactiveSource as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 1,
                quantity: "bytes",
            })
        );
    }
}

#[cfg(test)]
mod test_routing_change {
    use super::*;

    #[test]
    fn test_len() {
        assert_eq!(
            Message::RoutingChange {
                original_address: 0,
                new_address: 0
            }
            .len(),
            5
        );
    }

    #[test]
    fn test_opcode() {
        assert_eq!(
            Message::RoutingChange {
                original_address: 0x1234,
                new_address: 0x5678
            }
            .opcode(),
            Opcode::RoutingChange
        );
    }

    #[test]
    fn test_encoding() {
        assert_eq!(
            &Message::RoutingChange {
                original_address: 0x1234,
                new_address: 0x5678
            }
            .to_bytes(),
            &[Opcode::RoutingChange as u8, 0x12, 0x34, 0x56, 0x78]
        );
    }

    #[test]
    fn test_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::RoutingChange as u8, 0x12, 0x34, 0x56, 0x78]),
            Ok(Message::RoutingChange {
                original_address: 0x1234,
                new_address: 0x5678
            })
        );
    }

    #[test]
    fn test_decoding_missing_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::RoutingChange as u8, 0x12, 0x34, 0x56]),
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
            Message::try_from_bytes(&[Opcode::RoutingChange as u8, 0x12, 0x34]),
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
            Message::try_from_bytes(&[Opcode::RoutingChange as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 2,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operands() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::RoutingChange as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 1,
                quantity: "bytes",
            })
        );
    }
}

#[cfg(test)]
mod test_routing_information {
    use super::*;

    #[test]
    fn test_len() {
        assert_eq!(Message::RoutingInformation { address: 0 }.len(), 3);
    }

    #[test]
    fn test_opcode() {
        assert_eq!(
            Message::RoutingInformation { address: 0 }.opcode(),
            Opcode::RoutingInformation
        );
    }

    #[test]
    fn test_encoding() {
        assert_eq!(
            &Message::RoutingInformation { address: 0x1234 }.to_bytes(),
            &[Opcode::RoutingInformation as u8, 0x12, 0x34]
        );
    }

    #[test]
    fn test_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::RoutingInformation as u8, 0x12, 0x34]),
            Ok(Message::RoutingInformation { address: 0x1234 })
        );
    }

    #[test]
    fn test_decoding_missing_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::RoutingInformation as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 2,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::RoutingInformation as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 1,
                quantity: "bytes",
            })
        );
    }
}

#[cfg(test)]
mod test_set_stream_path {
    use super::*;

    #[test]
    fn test_len() {
        assert_eq!(Message::SetStreamPath { address: 0 }.len(), 3);
    }

    #[test]
    fn test_opcode() {
        assert_eq!(
            Message::SetStreamPath { address: 0 }.opcode(),
            Opcode::SetStreamPath
        );
    }

    #[test]
    fn test_encoding() {
        assert_eq!(
            &Message::SetStreamPath { address: 0x1234 }.to_bytes(),
            &[Opcode::SetStreamPath as u8, 0x12, 0x34]
        );
    }

    #[test]
    fn test_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::SetStreamPath as u8, 0x12, 0x34]),
            Ok(Message::SetStreamPath { address: 0x1234 })
        );
    }

    #[test]
    fn test_decoding_missing_byte() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::SetStreamPath as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 2,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_missing_operand() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::SetStreamPath as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 1,
                quantity: "bytes",
            })
        );
    }
}

#[cfg(test)]
mod test_record_on {
    use super::*;

    #[test]
    fn test_decode_all_missing() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::RecordOn as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(2),
                got: 1,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_opcode() {
        assert_eq!(
            Message::RecordOn {
                source: operand::RecordSource::Own
            }
            .opcode(),
            Opcode::RecordOn
        );
    }

    #[test]
    fn test_own_len() {
        assert_eq!(
            Message::RecordOn {
                source: operand::RecordSource::Own
            }
            .len(),
            2
        );
    }

    #[test]
    fn test_own_encoding() {
        assert_eq!(
            &Message::RecordOn {
                source: operand::RecordSource::Own
            }
            .to_bytes(),
            &[Opcode::RecordOn as u8, operand::RecordSourceType::Own as u8]
        );
    }

    #[test]
    fn test_own_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Own as u8
            ]),
            Ok(Message::RecordOn {
                source: operand::RecordSource::Own
            })
        );
    }

    #[test]
    fn test_digital_len() {
        assert_eq!(
            Message::RecordOn {
                source: operand::RecordSource::DigitalService(
                    operand::DigitalServiceId::AribGeneric(operand::AribData {
                        transport_stream_id: 0,
                        service_id: 0,
                        original_network_id: 0,
                    })
                ),
            }
            .len(),
            9
        );
    }

    #[test]
    fn test_digital_encoding() {
        assert_eq!(
            &Message::RecordOn {
                source: operand::RecordSource::DigitalService(
                    operand::DigitalServiceId::AribGeneric(operand::AribData {
                        transport_stream_id: 0x1234,
                        service_id: 0x5678,
                        original_network_id: 0x9ABC,
                    })
                ),
            }
            .to_bytes(),
            &[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Digital as u8,
                operand::DigitalServiceBroadcastSystem::AribGeneric as u8,
                0x12,
                0x34,
                0x56,
                0x78,
                0x9A,
                0xBC
            ]
        );
    }

    #[test]
    fn test_digital_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Digital as u8,
                operand::DigitalServiceBroadcastSystem::AribGeneric as u8,
                0x12,
                0x34,
                0x56,
                0x78,
                0x9A,
                0xBC
            ]),
            Ok(Message::RecordOn {
                source: operand::RecordSource::DigitalService(
                    operand::DigitalServiceId::AribGeneric(operand::AribData {
                        transport_stream_id: 0x1234,
                        service_id: 0x5678,
                        original_network_id: 0x9ABC,
                    })
                ),
            })
        );
    }

    #[test]
    fn test_digital_decoding_missing_bytes_1() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Digital as u8,
                operand::DigitalServiceBroadcastSystem::AribGeneric as u8,
                0x12,
                0x34,
                0x56,
                0x78,
                0x9A
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(9),
                got: 8,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_digital_decoding_missing_bytes_2() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Digital as u8,
                operand::DigitalServiceBroadcastSystem::AribGeneric as u8,
                0x12,
                0x34,
                0x56,
                0x78
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(9),
                got: 7,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_digital_decoding_missing_bytes_3() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Digital as u8,
                operand::DigitalServiceBroadcastSystem::AribGeneric as u8,
                0x12,
                0x34,
                0x56
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(9),
                got: 6,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_digital_decoding_missing_bytes_4() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Digital as u8,
                operand::DigitalServiceBroadcastSystem::AribGeneric as u8,
                0x12,
                0x34
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(9),
                got: 5,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_digital_decoding_missing_bytes_5() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Digital as u8,
                operand::DigitalServiceBroadcastSystem::AribGeneric as u8,
                0x12,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(9),
                got: 4,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_digital_decoding_missing_operand_1() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Digital as u8,
                operand::DigitalServiceBroadcastSystem::AribGeneric as u8,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(9),
                got: 3,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_digital_decoding_missing_operand_2() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Digital as u8,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(9),
                got: 2,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_analogue_len() {
        assert_eq!(
            Message::RecordOn {
                source: operand::RecordSource::AnalogueService(operand::AnalogueServiceId {
                    broadcast_type: operand::AnalogueBroadcastType::Cable,
                    frequency: 1,
                    broadcast_system: operand::BroadcastSystem::NtscM,
                }),
            }
            .len(),
            6
        );
    }

    #[test]
    fn test_analogue_encoding() {
        assert_eq!(
            &Message::RecordOn {
                source: operand::RecordSource::AnalogueService(operand::AnalogueServiceId {
                    broadcast_type: operand::AnalogueBroadcastType::Satellite,
                    frequency: 0x1234,
                    broadcast_system: operand::BroadcastSystem::SecamL,
                }),
            }
            .to_bytes(),
            &[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Analogue as u8,
                operand::AnalogueBroadcastType::Satellite as u8,
                0x12,
                0x34,
                operand::BroadcastSystem::SecamL as u8
            ]
        );
    }

    #[test]
    fn test_analogue_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Analogue as u8,
                operand::AnalogueBroadcastType::Satellite as u8,
                0x12,
                0x34,
                operand::BroadcastSystem::SecamL as u8
            ]),
            Ok(Message::RecordOn {
                source: operand::RecordSource::AnalogueService(operand::AnalogueServiceId {
                    broadcast_type: operand::AnalogueBroadcastType::Satellite,
                    frequency: 0x1234,
                    broadcast_system: operand::BroadcastSystem::SecamL,
                }),
            })
        );
    }

    #[test]
    fn test_analogue_decoding_missing_operands_1() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Analogue as u8,
                operand::AnalogueBroadcastType::Satellite as u8,
                0x12,
                0x34
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(6),
                got: 5,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_analogue_decoding_missing_operands_1_and_byte() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Analogue as u8,
                operand::AnalogueBroadcastType::Satellite as u8,
                0x12,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(5),
                got: 4,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_analogue_decoding_missing_operands_2() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Analogue as u8,
                operand::AnalogueBroadcastType::Satellite as u8,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(5),
                got: 3,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_analogue_decoding_missing_operands_3() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::Analogue as u8,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 2,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_external_plug_len() {
        assert_eq!(
            Message::RecordOn {
                source: operand::RecordSource::External(operand::ExternalSource::Plug(0))
            }
            .len(),
            3
        );
    }

    #[test]
    fn test_external_plug_encoding() {
        assert_eq!(
            &Message::RecordOn {
                source: operand::RecordSource::External(operand::ExternalSource::Plug(0x56))
            }
            .to_bytes(),
            &[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::ExternalPlug as u8,
                0x56,
            ]
        );
    }

    #[test]
    fn test_external_plug_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::ExternalPlug as u8,
                0x56,
            ]),
            Ok(Message::RecordOn {
                source: operand::RecordSource::External(operand::ExternalSource::Plug(0x56))
            })
        );
    }

    #[test]
    fn test_external_plug_decoding_missing_operand() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::ExternalPlug as u8,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 2,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_external_phys_addr_len() {
        assert_eq!(
            Message::RecordOn {
                source: operand::RecordSource::External(operand::ExternalSource::PhysicalAddress(
                    0
                ))
            }
            .len(),
            4
        );
    }

    #[test]
    fn test_external_phys_addr_encoding() {
        assert_eq!(
            &Message::RecordOn {
                source: operand::RecordSource::External(operand::ExternalSource::PhysicalAddress(
                    0x1234
                ))
            }
            .to_bytes(),
            &[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::ExternalPhysicalAddress as u8,
                0x12,
                0x34
            ]
        );
    }

    #[test]
    fn test_external_phys_addr_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::ExternalPhysicalAddress as u8,
                0x12,
                0x34
            ]),
            Ok(Message::RecordOn {
                source: operand::RecordSource::External(operand::ExternalSource::PhysicalAddress(
                    0x1234
                ))
            })
        );
    }

    #[test]
    fn test_external_phys_addr_decoding_missing_byte() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::ExternalPhysicalAddress as u8,
                0x12,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(4),
                got: 3,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_external_phys_addr_decoding_missing_operand() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::ExternalPhysicalAddress as u8,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(4),
                got: 2,
                quantity: "bytes",
            })
        );
    }

    #[test]
    fn test_decoding_invalid_operand() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::RecordOn as u8, 0xFE]),
            Err(Error::InvalidValueForType {
                ty: "RecordSourceType",
                value: String::from("254"),
            })
        );
    }
}

#[cfg(test)]
mod test_record_status {
    use super::*;

    #[test]
    fn test_len() {
        assert_eq!(
            Message::RecordStatus {
                status: operand::RecordStatusInfo::CurrentSource
            }
            .len(),
            2
        );
    }

    #[test]
    fn test_opcode() {
        assert_eq!(
            Message::RecordStatus {
                status: operand::RecordStatusInfo::CurrentSource
            }
            .opcode(),
            Opcode::RecordStatus
        );
    }

    #[test]
    fn test_encode() {
        assert_eq!(
            Message::RecordStatus {
                status: operand::RecordStatusInfo::CurrentSource
            }
            .to_bytes(),
            &[
                Opcode::RecordStatus as u8,
                operand::RecordStatusInfo::CurrentSource as u8
            ]
        );
    }

    #[test]
    fn test_decode() {
        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordStatus as u8,
                operand::RecordStatusInfo::CurrentSource as u8
            ]),
            Ok(Message::RecordStatus {
                status: operand::RecordStatusInfo::CurrentSource
            })
        );
    }

    #[test]
    fn test_decode_missing_operand() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::RecordStatus as u8]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(2),
                got: 1,
                quantity: "bytes"
            })
        );
    }

    #[test]
    fn test_decode_invalid_operand() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::RecordStatus as u8, 0xFE]),
            Err(Error::InvalidValueForType {
                ty: "RecordStatusInfo",
                value: String::from("254"),
            })
        );
    }
}
