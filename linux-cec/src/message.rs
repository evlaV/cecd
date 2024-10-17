use linux_cec_macros::{Message, MessageEnum, Operand};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::operand::OperandEncodable;
use crate::{constants, operand, PhysicalAddress, Result};
#[cfg(test)]
use crate::{Error, Range};

pub trait MessageEncodable: Sized {
    const OPCODE: Opcode;

    fn to_bytes(&self) -> Vec<u8> {
        let mut raw = vec![Self::OPCODE as u8];
        raw.extend(self.parameters());
        raw
    }

    fn to_message(&self) -> Message;
    fn into_message(self) -> Message;
    fn parameters(&self) -> Vec<u8>;
    fn try_from_parameters(params: &[u8]) -> Result<Self>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ActiveSource {
    pub address: PhysicalAddress,
}

#[cfg(test)]
mod test_active_source {
    use super::*;

    #[test]
    fn test_len() {
        assert_eq!(ActiveSource { address: 0 }.len(), 3);
    }

    #[test]
    fn test_encoding() {
        assert_eq!(
            &ActiveSource { address: 0x1234 }.to_bytes(),
            &[Opcode::ActiveSource as u8, 0x12, 0x34]
        );
    }

    #[test]
    fn test_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::ActiveSource as u8, 0x12, 0x34]),
            Ok(Message::ActiveSource(ActiveSource { address: 0x1234 }))
        );
        assert_eq!(
            Message::try_from_bytes(&[Opcode::ActiveSource as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 2,
                quantity: String::from("bytes"),
            })
        );
    }
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ImageViewOn;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TextViewOn;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct InactiveSource {
    pub address: PhysicalAddress,
}

#[cfg(test)]
mod test_inactive_source {
    use super::*;

    #[test]
    fn test_len() {
        assert_eq!(InactiveSource { address: 0 }.len(), 3);
    }

    #[test]
    fn test_encoding() {
        assert_eq!(
            &InactiveSource { address: 0x1234 }.to_bytes(),
            &[Opcode::InactiveSource as u8, 0x12, 0x34]
        );
    }

    #[test]
    fn test_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::InactiveSource as u8, 0x12, 0x34]),
            Ok(Message::InactiveSource(InactiveSource { address: 0x1234 }))
        );
        assert_eq!(
            Message::try_from_bytes(&[Opcode::InactiveSource as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 2,
                quantity: String::from("bytes"),
            })
        );
    }
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RequestActiveSource;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RoutingChange {
    pub original_address: PhysicalAddress,
    pub new_address: PhysicalAddress,
}

#[cfg(test)]
mod test_routing_change {
    use super::*;

    #[test]
    fn test_len() {
        assert_eq!(
            RoutingChange {
                original_address: 0,
                new_address: 0
            }
            .len(),
            5
        );
    }

    #[test]
    fn test_encoding() {
        assert_eq!(
            &RoutingChange {
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
            Ok(Message::RoutingChange(RoutingChange {
                original_address: 0x1234,
                new_address: 0x5678
            }))
        );
        assert_eq!(
            Message::try_from_bytes(&[Opcode::RoutingChange as u8, 0x12, 0x34, 0x56]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(5),
                got: 4,
                quantity: String::from("bytes"),
            })
        );
    }
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RoutingInformation {
    pub address: PhysicalAddress,
}

#[cfg(test)]
mod test_routing_information {
    use super::*;

    #[test]
    fn test_len() {
        assert_eq!(RoutingInformation { address: 0 }.len(), 3);
    }

    #[test]
    fn test_encoding() {
        assert_eq!(
            &RoutingInformation { address: 0x1234 }.to_bytes(),
            &[Opcode::RoutingInformation as u8, 0x12, 0x34]
        );
    }

    #[test]
    fn test_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::RoutingInformation as u8, 0x12, 0x34]),
            Ok(Message::RoutingInformation(RoutingInformation {
                address: 0x1234
            }))
        );
        assert_eq!(
            Message::try_from_bytes(&[Opcode::RoutingInformation as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 2,
                quantity: String::from("bytes"),
            })
        );
    }
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetStreamPath {
    pub address: PhysicalAddress,
}

#[cfg(test)]
mod test_set_stream_path {
    use super::*;

    #[test]
    fn test_len() {
        assert_eq!(SetStreamPath { address: 0 }.len(), 3);
    }

    #[test]
    fn test_encoding() {
        assert_eq!(
            &SetStreamPath { address: 0x1234 }.to_bytes(),
            &[Opcode::SetStreamPath as u8, 0x12, 0x34]
        );
    }

    #[test]
    fn test_decoding() {
        assert_eq!(
            Message::try_from_bytes(&[Opcode::SetStreamPath as u8, 0x12, 0x34]),
            Ok(Message::SetStreamPath(SetStreamPath { address: 0x1234 }))
        );
        assert_eq!(
            Message::try_from_bytes(&[Opcode::SetStreamPath as u8, 0x12]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 2,
                quantity: String::from("bytes"),
            })
        );
    }
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Standby;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordOff;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordOn {
    pub source: operand::RecordSource,
}

#[cfg(test)]
mod test_record_on {
    use super::*;

    #[test]
    fn test_own_len() {
        assert_eq!(
            RecordOn {
                source: operand::RecordSource::Own
            }
            .len(),
            2
        );
    }

    #[test]
    fn test_own_encoding() {
        assert_eq!(
            &RecordOn {
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
            Ok(Message::RecordOn(RecordOn {
                source: operand::RecordSource::Own
            }))
        );

        assert_eq!(
            Message::try_from_bytes(&[Opcode::RecordOn as u8,]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(2),
                got: 1,
                quantity: String::from("bytes"),
            })
        );
    }

    #[test]
    fn test_digital_len() {
        assert_eq!(
            RecordOn {
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
            &RecordOn {
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
            Ok(Message::RecordOn(RecordOn {
                source: operand::RecordSource::DigitalService(
                    operand::DigitalServiceId::AribGeneric(operand::AribData {
                        transport_stream_id: 0x1234,
                        service_id: 0x5678,
                        original_network_id: 0x9ABC,
                    })
                ),
            }))
        );

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
                quantity: String::from("bytes"),
            })
        );
    }

    #[test]
    fn test_analogue_len() {
        assert_eq!(
            RecordOn {
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
            &RecordOn {
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
            Ok(Message::RecordOn(RecordOn {
                source: operand::RecordSource::AnalogueService(operand::AnalogueServiceId {
                    broadcast_type: operand::AnalogueBroadcastType::Satellite,
                    frequency: 0x1234,
                    broadcast_system: operand::BroadcastSystem::SecamL,
                }),
            }))
        );

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
                quantity: String::from("bytes"),
            })
        );
    }

    #[test]
    fn test_external_plug_len() {
        assert_eq!(
            RecordOn {
                source: operand::RecordSource::External(operand::ExternalSource::Plug(0))
            }
            .len(),
            3
        );
    }

    #[test]
    fn test_external_plug_encoding() {
        assert_eq!(
            &RecordOn {
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
            Ok(Message::RecordOn(RecordOn {
                source: operand::RecordSource::External(operand::ExternalSource::Plug(0x56))
            }))
        );

        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::ExternalPlug as u8,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(3),
                got: 2,
                quantity: String::from("bytes"),
            })
        );
    }

    #[test]
    fn test_external_phys_addr_len() {
        assert_eq!(
            RecordOn {
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
            &RecordOn {
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
            Ok(Message::RecordOn(RecordOn {
                source: operand::RecordSource::External(operand::ExternalSource::PhysicalAddress(
                    0x1234
                ))
            }))
        );

        assert_eq!(
            Message::try_from_bytes(&[
                Opcode::RecordOn as u8,
                operand::RecordSourceType::ExternalPhysicalAddress as u8,
                0x12,
            ]),
            Err(Error::OutOfRange {
                expected: Range::AtLeast(4),
                got: 3,
                quantity: String::from("bytes"),
            })
        );
    }
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordStatus {
    pub status: operand::RecordStatusInfo,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RecordTvScreen;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClearAnalogueTimer {
    pub day_of_month: operand::DayOfMonth,
    pub month_of_year: operand::MonthOfYear,
    pub start_time: operand::Time,
    pub duration: operand::Duration,
    pub recording_sequence: operand::RecordingSequence,
    pub service_id: operand::AnalogueServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClearDigitalTimer {
    pub day_of_month: operand::DayOfMonth,
    pub month_of_year: operand::MonthOfYear,
    pub start_time: operand::Time,
    pub duration: operand::Duration,
    pub recording_sequence: operand::RecordingSequence,
    pub service_id: operand::DigitalServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClearExtTimer {
    pub day_of_month: operand::DayOfMonth,
    pub month_of_year: operand::MonthOfYear,
    pub start_time: operand::Time,
    pub duration: operand::Duration,
    pub recording_sequence: operand::RecordingSequence,
    pub external_source: operand::ExternalSource,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetAnalogueTimer {
    pub day_of_month: operand::DayOfMonth,
    pub month_of_year: operand::MonthOfYear,
    pub start_time: operand::Time,
    pub duration: operand::Duration,
    pub recording_sequence: operand::RecordingSequence,
    pub service_id: operand::AnalogueServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetDigitalTimer {
    pub day_of_month: operand::DayOfMonth,
    pub month_of_year: operand::MonthOfYear,
    pub start_time: operand::Time,
    pub duration: operand::Duration,
    pub recording_sequence: operand::RecordingSequence,
    pub service_id: operand::DigitalServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetExtTimer {
    pub day_of_month: operand::DayOfMonth,
    pub month_of_year: operand::MonthOfYear,
    pub start_time: operand::Time,
    pub duration: operand::Duration,
    pub recording_sequence: operand::RecordingSequence,
    pub external_source: operand::ExternalSource,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetTimerProgramTitle {
    pub title: operand::BufferOperand,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TimerClearedStatus {
    pub timer_cleared_status: operand::TimerClearedStatusData,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TimerStatus {
    data: u8,
    duration_available: Option<operand::Duration>,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct CecVersion {
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
    pub physical_address: PhysicalAddress,
    pub device_type: operand::PrimaryDeviceType,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetMenuLanguage {
    pub language: [u8; 3],
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeckControl {
    pub mode: operand::DeckControlMode,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeckStatus {
    pub info: operand::DeckInfo,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveDeckStatus {
    pub request: operand::StatusRequest,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Play {
    pub mode: operand::PlayMode,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveTunerDeviceStatus {
    pub request: operand::StatusRequest,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SelectAnalogueService {
    pub service_id: operand::AnalogueServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SelectDigitalService {
    pub service_id: operand::DigitalServiceId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TunerDeviceStatus {
    pub info: operand::TunerDeviceInfo,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TunerStepDecrement;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TunerStepIncrement;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeviceVendorId {
    pub vendor_id: operand::VendorId,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveDeviceVendorId;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct VendorCommand {
    pub command: operand::BufferOperand,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct VendorCommandWithId {
    pub vendor_id: operand::VendorId,
    pub vendor_specific_data: operand::BoundedBufferOperand<11, u8>,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct VendorRemoteButtonDown {
    pub rc_code: operand::BufferOperand,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct VendorRemoteButtonUp;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetOsdString {
    pub display_control: operand::DisplayControl,
    pub osd_string: operand::BoundedBufferOperand<13, u8>,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveOsdName;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetOsdName {
    pub name: operand::BufferOperand,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MenuRequest {
    pub request_type: operand::MenuRequestType,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct MenuStatus {
    pub state: operand::MenuState,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct UserControlPressed {
    pub ui_command: operand::UiCommand,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct UserControlReleased;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveDevicePowerStatus;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReportPowerStatus {
    pub status: operand::PowerStatus,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct FeatureAbort {
    pub opcode: Opcode,
    pub abort_reason: operand::AbortReason,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Abort;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveAudioStatus;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveSystemAudioModeStatus;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReportAudioStatus {
    pub status: operand::AudioStatus,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReportShortAudioDescriptor {
    pub descriptors: operand::BoundedBufferOperand<4, operand::ShortAudioDescriptor>,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RequestShortAudioDescriptor {
    pub descriptors: operand::BoundedBufferOperand<4, operand::AudioFormatIdAndCode>,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetSystemAudioMode {
    pub status: bool,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SystemAudioModeRequest {
    pub physical_address: PhysicalAddress,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SystemAudioModeStatus {
    pub system_audio_status: bool,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SetAudioRate {
    pub audio_rate: operand::AudioRate,
}

/* HDMI 1.4b */

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

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct CdcMessage {
    pub initiator: PhysicalAddress,
    pub opcode: CdcOpcode,
    pub params: operand::BoundedBufferOperand<11, u8>, // TODO
}

/* HDMI 2.0 */

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReportFeatures {
    pub version: operand::Version,
    pub device_types: operand::AllDeviceTypes,
    pub rc_profile: operand::RcProfile,
    pub dev_features: operand::DeviceFeatures,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct GiveFeatures;

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct RequestCurrentLatency {
    pub physical_address: PhysicalAddress,
}

#[derive(Message, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReportCurrentLatency {
    pub physical_address: PhysicalAddress,
    pub video_latency: operand::Delay,
    pub flags: operand::LatencyFlags,
    pub audio_output_delay: Option<operand::Delay>,
}

#[repr(u8)]
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand, MessageEnum,
)]
pub enum Opcode {
    ActiveSource = constants::CEC_MSG_ACTIVE_SOURCE,
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
    SetAudioRate = constants::CEC_MSG_SET_AUDIO_RATE,

    /* HDMI 1.4b */
    InitiateArc = constants::CEC_MSG_INITIATE_ARC,
    ReportArcInitiated = constants::CEC_MSG_REPORT_ARC_INITIATED,
    ReportArcTerminated = constants::CEC_MSG_REPORT_ARC_TERMINATED,
    RequestArcInitiation = constants::CEC_MSG_REQUEST_ARC_INITIATION,
    RequestArcTermination = constants::CEC_MSG_REQUEST_ARC_TERMINATION,
    TerminateArc = constants::CEC_MSG_TERMINATE_ARC,
    CdcMessage = constants::CEC_MSG_CDC_MESSAGE,

    /* HDMI 2.0 */
    ReportFeatures = constants::CEC_MSG_REPORT_FEATURES,
    GiveFeatures = constants::CEC_MSG_GIVE_FEATURES,
    RequestCurrentLatency = constants::CEC_MSG_REQUEST_CURRENT_LATENCY,
    ReportCurrentLatency = constants::CEC_MSG_REPORT_CURRENT_LATENCY,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive, Operand)]
pub enum CdcOpcode {
    HecInquireState = constants::CEC_MSG_CDC_HEC_INQUIRE_STATE,
    HecReportState = constants::CEC_MSG_CDC_HEC_REPORT_STATE,
    HecSetStateAdjacent = constants::CEC_MSG_CDC_HEC_SET_STATE_ADJACENT,
    HecSetState = constants::CEC_MSG_CDC_HEC_SET_STATE,
    HecRequestDeactivation = constants::CEC_MSG_CDC_HEC_REQUEST_DEACTIVATION,
    HecNotifyAlive = constants::CEC_MSG_CDC_HEC_NOTIFY_ALIVE,
    HecDiscover = constants::CEC_MSG_CDC_HEC_DISCOVER,
    HpdSetState = constants::CEC_MSG_CDC_HPD_SET_STATE,
    HpdReportState = constants::CEC_MSG_CDC_HPD_REPORT_STATE,
}
