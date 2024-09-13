pub const CEC_MAX_MSG_SIZE: usize = 16;

/* cec_msg flags field */
pub const CEC_MSG_FL_REPLY_TO_FOLLOWERS: u32 = 1 << 0;
pub const CEC_MSG_FL_RAW: u32 = 1 << 1;

/* cec_msg tx/rx_status field */
pub const CEC_TX_STATUS_OK: u8 = 1 << 0;
pub const CEC_TX_STATUS_ARB_LOST: u8 = 1 << 1;
pub const CEC_TX_STATUS_NACK: u8 = 1 << 2;
pub const CEC_TX_STATUS_LOW_DRIVE: u8 = 1 << 3;
pub const CEC_TX_STATUS_ERROR: u8 = 1 << 4;
pub const CEC_TX_STATUS_MAX_RETRIES: u8 = 1 << 5;
pub const CEC_TX_STATUS_ABORTED: u8 = 1 << 6;
pub const CEC_TX_STATUS_TIMEOUT: u8 = 1 << 7;

pub const CEC_RX_STATUS_OK: u8 = 1 << 0;
pub const CEC_RX_STATUS_TIMEOUT: u8 = 1 << 1;
pub const CEC_RX_STATUS_FEATURE_ABORT: u8 = 1 << 2;
pub const CEC_RX_STATUS_ABORTED: u8 = 1 << 3;

pub const CEC_LOG_ADDR_INVALID: u16 = 0xff;
pub const CEC_PHYS_ADDR_INVALID: u16 = 0xffff;

/*
 * The maximum number of logical addresses one device can be assigned to.
 * The CEC 2.0 spec allows for only 2 logical addresses at the moment. The
 * Analog Devices CEC hardware supports 3. So let's go wild and go for 4.
 */
pub const CEC_MAX_LOG_ADDRS: usize = 4;

/* The logical addresses defined by CEC 2.0 */
pub const CEC_LOG_ADDR_TV: u16 = 0;
pub const CEC_LOG_ADDR_RECORD_1: u16 = 1;
pub const CEC_LOG_ADDR_RECORD_2: u16 = 2;
pub const CEC_LOG_ADDR_TUNER_1: u16 = 3;
pub const CEC_LOG_ADDR_PLAYBACK_1: u16 = 4;
pub const CEC_LOG_ADDR_AUDIOSYSTEM: u16 = 5;
pub const CEC_LOG_ADDR_TUNER_2: u16 = 6;
pub const CEC_LOG_ADDR_TUNER_3: u16 = 7;
pub const CEC_LOG_ADDR_PLAYBACK_2: u16 = 8;
pub const CEC_LOG_ADDR_RECORD_3: u16 = 9;
pub const CEC_LOG_ADDR_TUNER_4: u16 = 10;
pub const CEC_LOG_ADDR_PLAYBACK_3: u16 = 11;
pub const CEC_LOG_ADDR_BACKUP_1: u16 = 12;
pub const CEC_LOG_ADDR_BACKUP_2: u16 = 13;
pub const CEC_LOG_ADDR_SPECIFIC: u16 = 14;
pub const CEC_LOG_ADDR_UNREGISTERED: u16 = 15; /* as initiator address */
pub const CEC_LOG_ADDR_BROADCAST: u16 = 15; /* as destination address */

/* The logical address types that the CEC device wants to claim */
pub const CEC_LOG_ADDR_TYPE_TV: u16 = 0;
pub const CEC_LOG_ADDR_TYPE_RECORD: u16 = 1;
pub const CEC_LOG_ADDR_TYPE_TUNER: u16 = 2;
pub const CEC_LOG_ADDR_TYPE_PLAYBACK: u16 = 3;
pub const CEC_LOG_ADDR_TYPE_AUDIOSYSTEM: u16 = 4;
pub const CEC_LOG_ADDR_TYPE_SPECIFIC: u16 = 5;
pub const CEC_LOG_ADDR_TYPE_UNREGISTERED: u16 = 6;
/*
 * Switches should use UNREGISTERED.
 * Processors should use SPECIFIC.
 */

pub const CEC_LOG_ADDR_MASK_TV: u16 = 1 << CEC_LOG_ADDR_TV;
pub const CEC_LOG_ADDR_MASK_RECORD: u16 =
    (1 << CEC_LOG_ADDR_RECORD_1) | (1 << CEC_LOG_ADDR_RECORD_2) | (1 << CEC_LOG_ADDR_RECORD_3);
pub const CEC_LOG_ADDR_MASK_TUNER: u16 = (1 << CEC_LOG_ADDR_TUNER_1)
    | (1 << CEC_LOG_ADDR_TUNER_2)
    | (1 << CEC_LOG_ADDR_TUNER_3)
    | (1 << CEC_LOG_ADDR_TUNER_4);
pub const CEC_LOG_ADDR_MASK_PLAYBACK: u16 = (1 << CEC_LOG_ADDR_PLAYBACK_1)
    | (1 << CEC_LOG_ADDR_PLAYBACK_2)
    | (1 << CEC_LOG_ADDR_PLAYBACK_3);
pub const CEC_LOG_ADDR_MASK_AUDIOSYSTEM: u16 = 1 << CEC_LOG_ADDR_AUDIOSYSTEM;
pub const CEC_LOG_ADDR_MASK_BACKUP: u16 =
    (1 << CEC_LOG_ADDR_BACKUP_1) | (1 << CEC_LOG_ADDR_BACKUP_2);
pub const CEC_LOG_ADDR_MASK_SPECIFIC: u16 = 1 << CEC_LOG_ADDR_SPECIFIC;
pub const CEC_LOG_ADDR_MASK_UNREGISTERED: u16 = 1 << CEC_LOG_ADDR_UNREGISTERED;

/*
 * Use this if there is no vendor ID (CEC_G_VENDOR_ID) or if the vendor ID
 * should be disabled (CEC_S_VENDOR_ID)
 */
pub const CEC_VENDOR_ID_NONE: u32 = 0xffffffff;

/* The message handling modes */
/* Modes for initiator */
pub const CEC_MODE_NO_INITIATOR: u32 = 0x0 << 0;
pub const CEC_MODE_INITIATOR: u32 = 0x1 << 0;
pub const CEC_MODE_EXCL_INITIATOR: u32 = 0x2 << 0;
pub const CEC_MODE_INITIATOR_MSK: u32 = 0x0f;

/* Modes for follower */
pub const CEC_MODE_NO_FOLLOWER: u32 = 0x0 << 4;
pub const CEC_MODE_FOLLOWER: u32 = 0x1 << 4;
pub const CEC_MODE_EXCL_FOLLOWER: u32 = 0x2 << 4;
pub const CEC_MODE_EXCL_FOLLOWER_PASSTHRU: u32 = 0x3 << 4;
pub const CEC_MODE_MONITOR_PIN: u32 = 0xd << 4;
pub const CEC_MODE_MONITOR: u32 = 0xe << 4;
pub const CEC_MODE_MONITOR_ALL: u32 = 0xf << 4;
pub const CEC_MODE_FOLLOWER_MSK: u32 = 0xf0;

/* Userspace has to configure the physical address */
pub const CEC_CAP_PHYS_ADDR: u32 = 1 << 0;
/* Userspace has to configure the logical addresses */
pub const CEC_CAP_LOG_ADDRS: u32 = 1 << 1;
/* Userspace can transmit messages (and thus become follower as well) */
pub const CEC_CAP_TRANSMIT: u32 = 1 << 2;
/*
 * Passthrough all messages instead of processing them.
 */
pub const CEC_CAP_PASSTHROUGH: u32 = 1 << 3;
/* Supports remote control */
pub const CEC_CAP_RC: u32 = 1 << 4;
/* Hardware can monitor all messages, not just directed and broadcast. */
pub const CEC_CAP_MONITOR_ALL: u32 = 1 << 5;
/* Hardware can use CEC only if the HDMI HPD pin is high. */
pub const CEC_CAP_NEEDS_HPD: u32 = 1 << 6;
/* Hardware can monitor CEC pin transitions */
pub const CEC_CAP_MONITOR_PIN: u32 = 1 << 7;
/* CEC_ADAP_G_CONNECTOR_INFO is available */
pub const CEC_CAP_CONNECTOR_INFO: u32 = 1 << 8;

/* Allow a fallback to unregistered */
pub const CEC_LOG_ADDRS_FL_ALLOW_UNREG_FALLBACK: u32 = 1 << 0;
/* Passthrough RC messages to the input subsystem */
pub const CEC_LOG_ADDRS_FL_ALLOW_RC_PASSTHRU: u32 = 1 << 1;
/* CDC-Only device: supports only CDC messages */
pub const CEC_LOG_ADDRS_FL_CDC_ONLY: u32 = 1 << 2;

pub const CEC_CONNECTOR_TYPE_NO_CONNECTOR: u32 = 0;
pub const CEC_CONNECTOR_TYPE_DRM: u32 = 1;

/* Events */

/* Event that occurs when the adapter state changes */
pub const CEC_EVENT_STATE_CHANGE: u32 = 1;
/*
 * This event is sent when messages are lost because the application
 * didn't empty the message queue in time
 */
pub const CEC_EVENT_LOST_MSGS: u32 = 2;
pub const CEC_EVENT_PIN_CEC_LOW: u32 = 3;
pub const CEC_EVENT_PIN_CEC_HIGH: u32 = 4;
pub const CEC_EVENT_PIN_HPD_LOW: u32 = 5;
pub const CEC_EVENT_PIN_HPD_HIGH: u32 = 6;
pub const CEC_EVENT_PIN_5V_LOW: u32 = 7;
pub const CEC_EVENT_PIN_5V_HIGH: u32 = 8;

pub const CEC_EVENT_FL_INITIAL_STATE: u32 = 1 << 0;
pub const CEC_EVENT_FL_DROPPED_EVENTS: u32 = 1 << 1;

/* Messages */

/* One Touch Play Feature */
pub const CEC_MSG_ACTIVE_SOURCE: u8 = 0x82;
pub const CEC_MSG_IMAGE_VIEW_ON: u8 = 0x04;
pub const CEC_MSG_TEXT_VIEW_ON: u8 = 0x0d;

/* Routing Control Feature */

/*
 * Has also:
 * CEC_MSG_ACTIVE_SOURCE
 */

pub const CEC_MSG_INACTIVE_SOURCE: u8 = 0x9d;
pub const CEC_MSG_REQUEST_ACTIVE_SOURCE: u8 = 0x85;
pub const CEC_MSG_ROUTING_CHANGE: u8 = 0x80;
pub const CEC_MSG_ROUTING_INFORMATION: u8 = 0x81;
pub const CEC_MSG_SET_STREAM_PATH: u8 = 0x86;

/* Standby Feature */
pub const CEC_MSG_STANDBY: u8 = 0x36;

/* One Touch Record Feature */
pub const CEC_MSG_RECORD_OFF: u8 = 0x0b;
pub const CEC_MSG_RECORD_ON: u8 = 0x09;
/* Record Source Type Operand (rec_src_type) */
pub const CEC_OP_RECORD_SRC_OWN: u8 = 1;
pub const CEC_OP_RECORD_SRC_DIGITAL: u8 = 2;
pub const CEC_OP_RECORD_SRC_ANALOG: u8 = 3;
pub const CEC_OP_RECORD_SRC_EXT_PLUG: u8 = 4;
pub const CEC_OP_RECORD_SRC_EXT_PHYS_ADDR: u8 = 5;
/* Service Identification Method Operand (service_id_method) */
pub const CEC_OP_SERVICE_ID_METHOD_BY_DIG_ID: u8 = 0;
pub const CEC_OP_SERVICE_ID_METHOD_BY_CHANNEL: u8 = 1;
/* Digital Service Broadcast System Operand (dig_bcast_system) */
pub const CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ARIB_GEN: u8 = 0x00;
pub const CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ATSC_GEN: u8 = 0x01;
pub const CEC_OP_DIG_SERVICE_BCAST_SYSTEM_DVB_GEN: u8 = 0x02;
pub const CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ARIB_BS: u8 = 0x08;
pub const CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ARIB_CS: u8 = 0x09;
pub const CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ARIB_T: u8 = 0x0a;
pub const CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ATSC_CABLE: u8 = 0x10;
pub const CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ATSC_SAT: u8 = 0x11;
pub const CEC_OP_DIG_SERVICE_BCAST_SYSTEM_ATSC_T: u8 = 0x12;
pub const CEC_OP_DIG_SERVICE_BCAST_SYSTEM_DVB_C: u8 = 0x18;
pub const CEC_OP_DIG_SERVICE_BCAST_SYSTEM_DVB_S: u8 = 0x19;
pub const CEC_OP_DIG_SERVICE_BCAST_SYSTEM_DVB_S2: u8 = 0x1a;
pub const CEC_OP_DIG_SERVICE_BCAST_SYSTEM_DVB_T: u8 = 0x1b;
/* Analogue Broadcast Type Operand (ana_bcast_type) */
pub const CEC_OP_ANA_BCAST_TYPE_CABLE: u8 = 0;
pub const CEC_OP_ANA_BCAST_TYPE_SATELLITE: u8 = 1;
pub const CEC_OP_ANA_BCAST_TYPE_TERRESTRIAL: u8 = 2;
/* Broadcast System Operand (bcast_system) */
pub const CEC_OP_BCAST_SYSTEM_PAL_BG: u8 = 0x00;
pub const CEC_OP_BCAST_SYSTEM_SECAM_LQ: u8 = 0x01; /* SECAM L' */
pub const CEC_OP_BCAST_SYSTEM_PAL_M: u8 = 0x02;
pub const CEC_OP_BCAST_SYSTEM_NTSC_M: u8 = 0x03;
pub const CEC_OP_BCAST_SYSTEM_PAL_I: u8 = 0x04;
pub const CEC_OP_BCAST_SYSTEM_SECAM_DK: u8 = 0x05;
pub const CEC_OP_BCAST_SYSTEM_SECAM_BG: u8 = 0x06;
pub const CEC_OP_BCAST_SYSTEM_SECAM_L: u8 = 0x07;
pub const CEC_OP_BCAST_SYSTEM_PAL_DK: u8 = 0x08;
pub const CEC_OP_BCAST_SYSTEM_OTHER: u8 = 0x1f;
/* Channel Number Format Operand (channel_number_fmt) */
pub const CEC_OP_CHANNEL_NUMBER_FMT_1_PART: u8 = 0x01;
pub const CEC_OP_CHANNEL_NUMBER_FMT_2_PART: u8 = 0x02;

pub const CEC_MSG_RECORD_STATUS: u8 = 0x0a;
/* Record Status Operand (rec_status) */
pub const CEC_OP_RECORD_STATUS_CUR_SRC: u8 = 0x01;
pub const CEC_OP_RECORD_STATUS_DIG_SERVICE: u8 = 0x02;
pub const CEC_OP_RECORD_STATUS_ANA_SERVICE: u8 = 0x03;
pub const CEC_OP_RECORD_STATUS_EXT_INPUT: u8 = 0x04;
pub const CEC_OP_RECORD_STATUS_NO_DIG_SERVICE: u8 = 0x05;
pub const CEC_OP_RECORD_STATUS_NO_ANA_SERVICE: u8 = 0x06;
pub const CEC_OP_RECORD_STATUS_NO_SERVICE: u8 = 0x07;
pub const CEC_OP_RECORD_STATUS_INVALID_EXT_PLUG: u8 = 0x09;
pub const CEC_OP_RECORD_STATUS_INVALID_EXT_PHYS_ADDR: u8 = 0x0a;
pub const CEC_OP_RECORD_STATUS_UNSUP_CA: u8 = 0x0b;
pub const CEC_OP_RECORD_STATUS_NO_CA_ENTITLEMENTS: u8 = 0x0c;
pub const CEC_OP_RECORD_STATUS_CANT_COPY_SRC: u8 = 0x0d;
pub const CEC_OP_RECORD_STATUS_NO_MORE_COPIES: u8 = 0x0e;
pub const CEC_OP_RECORD_STATUS_NO_MEDIA: u8 = 0x10;
pub const CEC_OP_RECORD_STATUS_PLAYING: u8 = 0x11;
pub const CEC_OP_RECORD_STATUS_ALREADY_RECORDING: u8 = 0x12;
pub const CEC_OP_RECORD_STATUS_MEDIA_PROT: u8 = 0x13;
pub const CEC_OP_RECORD_STATUS_NO_SIGNAL: u8 = 0x14;
pub const CEC_OP_RECORD_STATUS_MEDIA_PROBLEM: u8 = 0x15;
pub const CEC_OP_RECORD_STATUS_NO_SPACE: u8 = 0x16;
pub const CEC_OP_RECORD_STATUS_PARENTAL_LOCK: u8 = 0x17;
pub const CEC_OP_RECORD_STATUS_TERMINATED_OK: u8 = 0x1a;
pub const CEC_OP_RECORD_STATUS_ALREADY_TERM: u8 = 0x1b;
pub const CEC_OP_RECORD_STATUS_OTHER: u8 = 0x1f;

pub const CEC_MSG_RECORD_TV_SCREEN: u8 = 0x0f;

/* Timer Programming Feature */
pub const CEC_MSG_CLEAR_ANALOGUE_TIMER: u8 = 0x33;
/* Recording Sequence Operand (recording_seq) */
pub const CEC_OP_REC_SEQ_SUNDAY: u8 = 0x01;
pub const CEC_OP_REC_SEQ_MONDAY: u8 = 0x02;
pub const CEC_OP_REC_SEQ_TUESDAY: u8 = 0x04;
pub const CEC_OP_REC_SEQ_WEDNESDAY: u8 = 0x08;
pub const CEC_OP_REC_SEQ_THURSDAY: u8 = 0x10;
pub const CEC_OP_REC_SEQ_FRIDAY: u8 = 0x20;
pub const CEC_OP_REC_SEQ_SATURDAY: u8 = 0x40;
pub const CEC_OP_REC_SEQ_ONCE_ONLY: u8 = 0x00;

pub const CEC_MSG_CLEAR_DIGITAL_TIMER: u8 = 0x99;

pub const CEC_MSG_CLEAR_EXT_TIMER: u8 = 0xa1;
/* External Source Specifier Operand (ext_src_spec) */
pub const CEC_OP_EXT_SRC_PLUG: u8 = 0x04;
pub const CEC_OP_EXT_SRC_PHYS_ADDR: u8 = 0x05;

pub const CEC_MSG_SET_ANALOGUE_TIMER: u8 = 0x34;
pub const CEC_MSG_SET_DIGITAL_TIMER: u8 = 0x97;
pub const CEC_MSG_SET_EXT_TIMER: u8 = 0xa2;

pub const CEC_MSG_SET_TIMER_PROGRAM_TITLE: u8 = 0x67;
pub const CEC_MSG_TIMER_CLEARED_STATUS: u8 = 0x43;
/* Timer Cleared Status Data Operand (timer_cleared_status) */
pub const CEC_OP_TIMER_CLR_STAT_RECORDING: u8 = 0x00;
pub const CEC_OP_TIMER_CLR_STAT_NO_MATCHING: u8 = 0x01;
pub const CEC_OP_TIMER_CLR_STAT_NO_INFO: u8 = 0x02;
pub const CEC_OP_TIMER_CLR_STAT_CLEARED: u8 = 0x80;

pub const CEC_MSG_TIMER_STATUS: u8 = 0x35;
/* Timer Overlap Warning Operand (timer_overlap_warning) */
pub const CEC_OP_TIMER_OVERLAP_WARNING_NO_OVERLAP: u8 = 0;
pub const CEC_OP_TIMER_OVERLAP_WARNING_OVERLAP: u8 = 1;
/* Media Info Operand (media_info) */
pub const CEC_OP_MEDIA_INFO_UNPROT_MEDIA: u8 = 0;
pub const CEC_OP_MEDIA_INFO_PROT_MEDIA: u8 = 1;
pub const CEC_OP_MEDIA_INFO_NO_MEDIA: u8 = 2;
/* Programmed Indicator Operand (prog_indicator) */
pub const CEC_OP_PROG_IND_NOT_PROGRAMMED: u8 = 0;
pub const CEC_OP_PROG_IND_PROGRAMMED: u8 = 1;
/* Programmed Info Operand (prog_info) */
pub const CEC_OP_PROG_INFO_ENOUGH_SPACE: u8 = 0x08;
pub const CEC_OP_PROG_INFO_NOT_ENOUGH_SPACE: u8 = 0x09;
pub const CEC_OP_PROG_INFO_MIGHT_NOT_BE_ENOUGH_SPACE: u8 = 0x0b;
pub const CEC_OP_PROG_INFO_NONE_AVAILABLE: u8 = 0x0a;
/* Not Programmed Error Info Operand (prog_error) */
pub const CEC_OP_PROG_ERROR_NO_FREE_TIMER: u8 = 0x01;
pub const CEC_OP_PROG_ERROR_DATE_OUT_OF_RANGE: u8 = 0x02;
pub const CEC_OP_PROG_ERROR_REC_SEQ_ERROR: u8 = 0x03;
pub const CEC_OP_PROG_ERROR_INV_EXT_PLUG: u8 = 0x04;
pub const CEC_OP_PROG_ERROR_INV_EXT_PHYS_ADDR: u8 = 0x05;
pub const CEC_OP_PROG_ERROR_CA_UNSUPP: u8 = 0x06;
pub const CEC_OP_PROG_ERROR_INSUF_CA_ENTITLEMENTS: u8 = 0x07;
pub const CEC_OP_PROG_ERROR_RESOLUTION_UNSUPP: u8 = 0x08;
pub const CEC_OP_PROG_ERROR_PARENTAL_LOCK: u8 = 0x09;
pub const CEC_OP_PROG_ERROR_CLOCK_FAILURE: u8 = 0x0a;
pub const CEC_OP_PROG_ERROR_DUPLICATE: u8 = 0x0e;

/* System Information Feature */
pub const CEC_MSG_CEC_VERSION: u8 = 0x9e;
/* CEC Version Operand (cec_version) */
pub const CEC_OP_CEC_VERSION_1_3A: u8 = 4;
pub const CEC_OP_CEC_VERSION_1_4: u8 = 5;
pub const CEC_OP_CEC_VERSION_2_0: u8 = 6;

pub const CEC_MSG_GET_CEC_VERSION: u8 = 0x9f;
pub const CEC_MSG_GIVE_PHYSICAL_ADDR: u8 = 0x83;
pub const CEC_MSG_GET_MENU_LANGUAGE: u8 = 0x91;
pub const CEC_MSG_REPORT_PHYSICAL_ADDR: u8 = 0x84;
/* Primary Device Type Operand (prim_devtype) */
pub const CEC_OP_PRIM_DEVTYPE_TV: u8 = 0;
pub const CEC_OP_PRIM_DEVTYPE_RECORD: u8 = 1;
pub const CEC_OP_PRIM_DEVTYPE_TUNER: u8 = 3;
pub const CEC_OP_PRIM_DEVTYPE_PLAYBACK: u8 = 4;
pub const CEC_OP_PRIM_DEVTYPE_AUDIOSYSTEM: u8 = 5;
pub const CEC_OP_PRIM_DEVTYPE_SWITCH: u8 = 6;
pub const CEC_OP_PRIM_DEVTYPE_PROCESSOR: u8 = 7;

pub const CEC_MSG_SET_MENU_LANGUAGE: u8 = 0x32;
pub const CEC_MSG_REPORT_FEATURES: u8 = 0xa6; /* HDMI 2.0 */
/* All Device Types Operand (all_device_types) */
pub const CEC_OP_ALL_DEVTYPE_TV: u8 = 0x80;
pub const CEC_OP_ALL_DEVTYPE_RECORD: u8 = 0x40;
pub const CEC_OP_ALL_DEVTYPE_TUNER: u8 = 0x20;
pub const CEC_OP_ALL_DEVTYPE_PLAYBACK: u8 = 0x10;
pub const CEC_OP_ALL_DEVTYPE_AUDIOSYSTEM: u8 = 0x08;
pub const CEC_OP_ALL_DEVTYPE_SWITCH: u8 = 0x04;
/*
 * And if you wondering what happened to PROCESSOR devices: those should
 * be mapped to a SWITCH.
 */

/* Valid for RC Profile and Device Feature operands */
pub const CEC_OP_FEAT_EXT: u8 = 0x80; /* Extension bit */
/* RC Profile Operand (rc_profile) */
pub const CEC_OP_FEAT_RC_TV_PROFILE_NONE: u8 = 0x00;
pub const CEC_OP_FEAT_RC_TV_PROFILE_1: u8 = 0x02;
pub const CEC_OP_FEAT_RC_TV_PROFILE_2: u8 = 0x06;
pub const CEC_OP_FEAT_RC_TV_PROFILE_3: u8 = 0x0a;
pub const CEC_OP_FEAT_RC_TV_PROFILE_4: u8 = 0x0e;
pub const CEC_OP_FEAT_RC_SRC_HAS_DEV_ROOT_MENU: u8 = 0x50;
pub const CEC_OP_FEAT_RC_SRC_HAS_DEV_SETUP_MENU: u8 = 0x48;
pub const CEC_OP_FEAT_RC_SRC_HAS_CONTENTS_MENU: u8 = 0x44;
pub const CEC_OP_FEAT_RC_SRC_HAS_MEDIA_TOP_MENU: u8 = 0x42;
pub const CEC_OP_FEAT_RC_SRC_HAS_MEDIA_CONTEXT_MENU: u8 = 0x41;
/* Device Feature Operand (dev_features) */
pub const CEC_OP_FEAT_DEV_HAS_RECORD_TV_SCREEN: u8 = 0x40;
pub const CEC_OP_FEAT_DEV_HAS_SET_OSD_STRING: u8 = 0x20;
pub const CEC_OP_FEAT_DEV_HAS_DECK_CONTROL: u8 = 0x10;
pub const CEC_OP_FEAT_DEV_HAS_SET_AUDIO_RATE: u8 = 0x08;
pub const CEC_OP_FEAT_DEV_SINK_HAS_ARC_TX: u8 = 0x04;
pub const CEC_OP_FEAT_DEV_SOURCE_HAS_ARC_RX: u8 = 0x02;
pub const CEC_OP_FEAT_DEV_HAS_SET_AUDIO_VOLUME_LEVEL: u8 = 0x01;

pub const CEC_MSG_GIVE_FEATURES: u8 = 0xa5; /* HDMI 2.0 */

/* Deck Control Feature */
pub const CEC_MSG_DECK_CONTROL: u8 = 0x42;
/* Deck Control Mode Operand (deck_control_mode) */
pub const CEC_OP_DECK_CTL_MODE_SKIP_FWD: u8 = 1;
pub const CEC_OP_DECK_CTL_MODE_SKIP_REV: u8 = 2;
pub const CEC_OP_DECK_CTL_MODE_STOP: u8 = 3;
pub const CEC_OP_DECK_CTL_MODE_EJECT: u8 = 4;

pub const CEC_MSG_DECK_STATUS: u8 = 0x1b;
/* Deck Info Operand (deck_info) */
pub const CEC_OP_DECK_INFO_PLAY: u8 = 0x11;
pub const CEC_OP_DECK_INFO_RECORD: u8 = 0x12;
pub const CEC_OP_DECK_INFO_PLAY_REV: u8 = 0x13;
pub const CEC_OP_DECK_INFO_STILL: u8 = 0x14;
pub const CEC_OP_DECK_INFO_SLOW: u8 = 0x15;
pub const CEC_OP_DECK_INFO_SLOW_REV: u8 = 0x16;
pub const CEC_OP_DECK_INFO_FAST_FWD: u8 = 0x17;
pub const CEC_OP_DECK_INFO_FAST_REV: u8 = 0x18;
pub const CEC_OP_DECK_INFO_NO_MEDIA: u8 = 0x19;
pub const CEC_OP_DECK_INFO_STOP: u8 = 0x1a;
pub const CEC_OP_DECK_INFO_SKIP_FWD: u8 = 0x1b;
pub const CEC_OP_DECK_INFO_SKIP_REV: u8 = 0x1c;
pub const CEC_OP_DECK_INFO_INDEX_SEARCH_FWD: u8 = 0x1d;
pub const CEC_OP_DECK_INFO_INDEX_SEARCH_REV: u8 = 0x1e;
pub const CEC_OP_DECK_INFO_OTHER: u8 = 0x1f;

pub const CEC_MSG_GIVE_DECK_STATUS: u8 = 0x1a;
/* Status Request Operand (status_req) */
pub const CEC_OP_STATUS_REQ_ON: u8 = 1;
pub const CEC_OP_STATUS_REQ_OFF: u8 = 2;
pub const CEC_OP_STATUS_REQ_ONCE: u8 = 3;

pub const CEC_MSG_PLAY: u8 = 0x41;
/* Play Mode Operand (play_mode) */
pub const CEC_OP_PLAY_MODE_PLAY_FWD: u8 = 0x24;
pub const CEC_OP_PLAY_MODE_PLAY_REV: u8 = 0x20;
pub const CEC_OP_PLAY_MODE_PLAY_STILL: u8 = 0x25;
pub const CEC_OP_PLAY_MODE_PLAY_FAST_FWD_MIN: u8 = 0x05;
pub const CEC_OP_PLAY_MODE_PLAY_FAST_FWD_MED: u8 = 0x06;
pub const CEC_OP_PLAY_MODE_PLAY_FAST_FWD_MAX: u8 = 0x07;
pub const CEC_OP_PLAY_MODE_PLAY_FAST_REV_MIN: u8 = 0x09;
pub const CEC_OP_PLAY_MODE_PLAY_FAST_REV_MED: u8 = 0x0a;
pub const CEC_OP_PLAY_MODE_PLAY_FAST_REV_MAX: u8 = 0x0b;
pub const CEC_OP_PLAY_MODE_PLAY_SLOW_FWD_MIN: u8 = 0x15;
pub const CEC_OP_PLAY_MODE_PLAY_SLOW_FWD_MED: u8 = 0x16;
pub const CEC_OP_PLAY_MODE_PLAY_SLOW_FWD_MAX: u8 = 0x17;
pub const CEC_OP_PLAY_MODE_PLAY_SLOW_REV_MIN: u8 = 0x19;
pub const CEC_OP_PLAY_MODE_PLAY_SLOW_REV_MED: u8 = 0x1a;
pub const CEC_OP_PLAY_MODE_PLAY_SLOW_REV_MAX: u8 = 0x1b;

/* Tuner Control Feature */
pub const CEC_MSG_GIVE_TUNER_DEVICE_STATUS: u8 = 0x08;
pub const CEC_MSG_SELECT_ANALOGUE_SERVICE: u8 = 0x92;
pub const CEC_MSG_SELECT_DIGITAL_SERVICE: u8 = 0x93;
pub const CEC_MSG_TUNER_DEVICE_STATUS: u8 = 0x07;
/* Recording Flag Operand (rec_flag) */
pub const CEC_OP_REC_FLAG_NOT_USED: u8 = 0;
pub const CEC_OP_REC_FLAG_USED: u8 = 1;
/* Tuner Display Info Operand (tuner_display_info) */
pub const CEC_OP_TUNER_DISPLAY_INFO_DIGITAL: u8 = 0;
pub const CEC_OP_TUNER_DISPLAY_INFO_NONE: u8 = 1;
pub const CEC_OP_TUNER_DISPLAY_INFO_ANALOGUE: u8 = 2;

pub const CEC_MSG_TUNER_STEP_DECREMENT: u8 = 0x06;
pub const CEC_MSG_TUNER_STEP_INCREMENT: u8 = 0x05;

/* Vendor Specific Commands Feature */

/*
 * Has also:
 * CEC_MSG_CEC_VERSION
 * CEC_MSG_GET_CEC_VERSION
 */
pub const CEC_MSG_DEVICE_VENDOR_ID: u8 = 0x87;
pub const CEC_MSG_GIVE_DEVICE_VENDOR_ID: u8 = 0x8c;
pub const CEC_MSG_VENDOR_COMMAND: u8 = 0x89;
pub const CEC_MSG_VENDOR_COMMAND_WITH_ID: u8 = 0xa0;
pub const CEC_MSG_VENDOR_REMOTE_BUTTON_DOWN: u8 = 0x8a;
pub const CEC_MSG_VENDOR_REMOTE_BUTTON_UP: u8 = 0x8b;

/* OSD Display Feature */
pub const CEC_MSG_SET_OSD_STRING: u8 = 0x64;
/* Display Control Operand (disp_ctl) */
pub const CEC_OP_DISP_CTL_DEFAULT: u8 = 0x00;
pub const CEC_OP_DISP_CTL_UNTIL_CLEARED: u8 = 0x40;
pub const CEC_OP_DISP_CTL_CLEAR: u8 = 0x80;

/* Device OSD Transfer Feature */
pub const CEC_MSG_GIVE_OSD_NAME: u8 = 0x46;
pub const CEC_MSG_SET_OSD_NAME: u8 = 0x47;

/* Device Menu Control Feature */
pub const CEC_MSG_MENU_REQUEST: u8 = 0x8d;
/* Menu Request Type Operand (menu_req) */
pub const CEC_OP_MENU_REQUEST_ACTIVATE: u8 = 0x00;
pub const CEC_OP_MENU_REQUEST_DEACTIVATE: u8 = 0x01;
pub const CEC_OP_MENU_REQUEST_QUERY: u8 = 0x02;

pub const CEC_MSG_MENU_STATUS: u8 = 0x8e;
/* Menu State Operand (menu_state) */
pub const CEC_OP_MENU_STATE_ACTIVATED: u8 = 0x00;
pub const CEC_OP_MENU_STATE_DEACTIVATED: u8 = 0x01;

pub const CEC_MSG_USER_CONTROL_PRESSED: u8 = 0x44;
/* UI Command Operand (ui_cmd) */
pub const CEC_OP_UI_CMD_SELECT: u8 = 0x00;
pub const CEC_OP_UI_CMD_UP: u8 = 0x01;
pub const CEC_OP_UI_CMD_DOWN: u8 = 0x02;
pub const CEC_OP_UI_CMD_LEFT: u8 = 0x03;
pub const CEC_OP_UI_CMD_RIGHT: u8 = 0x04;
pub const CEC_OP_UI_CMD_RIGHT_UP: u8 = 0x05;
pub const CEC_OP_UI_CMD_RIGHT_DOWN: u8 = 0x06;
pub const CEC_OP_UI_CMD_LEFT_UP: u8 = 0x07;
pub const CEC_OP_UI_CMD_LEFT_DOWN: u8 = 0x08;
pub const CEC_OP_UI_CMD_DEVICE_ROOT_MENU: u8 = 0x09;
pub const CEC_OP_UI_CMD_DEVICE_SETUP_MENU: u8 = 0x0a;
pub const CEC_OP_UI_CMD_CONTENTS_MENU: u8 = 0x0b;
pub const CEC_OP_UI_CMD_FAVORITE_MENU: u8 = 0x0c;
pub const CEC_OP_UI_CMD_BACK: u8 = 0x0d;
pub const CEC_OP_UI_CMD_MEDIA_TOP_MENU: u8 = 0x10;
pub const CEC_OP_UI_CMD_MEDIA_CONTEXT_SENSITIVE_MENU: u8 = 0x11;
pub const CEC_OP_UI_CMD_NUMBER_ENTRY_MODE: u8 = 0x1d;
pub const CEC_OP_UI_CMD_NUMBER_11: u8 = 0x1e;
pub const CEC_OP_UI_CMD_NUMBER_12: u8 = 0x1f;
pub const CEC_OP_UI_CMD_NUMBER_0_OR_NUMBER_10: u8 = 0x20;
pub const CEC_OP_UI_CMD_NUMBER_1: u8 = 0x21;
pub const CEC_OP_UI_CMD_NUMBER_2: u8 = 0x22;
pub const CEC_OP_UI_CMD_NUMBER_3: u8 = 0x23;
pub const CEC_OP_UI_CMD_NUMBER_4: u8 = 0x24;
pub const CEC_OP_UI_CMD_NUMBER_5: u8 = 0x25;
pub const CEC_OP_UI_CMD_NUMBER_6: u8 = 0x26;
pub const CEC_OP_UI_CMD_NUMBER_7: u8 = 0x27;
pub const CEC_OP_UI_CMD_NUMBER_8: u8 = 0x28;
pub const CEC_OP_UI_CMD_NUMBER_9: u8 = 0x29;
pub const CEC_OP_UI_CMD_DOT: u8 = 0x2a;
pub const CEC_OP_UI_CMD_ENTER: u8 = 0x2b;
pub const CEC_OP_UI_CMD_CLEAR: u8 = 0x2c;
pub const CEC_OP_UI_CMD_NEXT_FAVORITE: u8 = 0x2f;
pub const CEC_OP_UI_CMD_CHANNEL_UP: u8 = 0x30;
pub const CEC_OP_UI_CMD_CHANNEL_DOWN: u8 = 0x31;
pub const CEC_OP_UI_CMD_PREVIOUS_CHANNEL: u8 = 0x32;
pub const CEC_OP_UI_CMD_SOUND_SELECT: u8 = 0x33;
pub const CEC_OP_UI_CMD_INPUT_SELECT: u8 = 0x34;
pub const CEC_OP_UI_CMD_DISPLAY_INFORMATION: u8 = 0x35;
pub const CEC_OP_UI_CMD_HELP: u8 = 0x36;
pub const CEC_OP_UI_CMD_PAGE_UP: u8 = 0x37;
pub const CEC_OP_UI_CMD_PAGE_DOWN: u8 = 0x38;
pub const CEC_OP_UI_CMD_POWER: u8 = 0x40;
pub const CEC_OP_UI_CMD_VOLUME_UP: u8 = 0x41;
pub const CEC_OP_UI_CMD_VOLUME_DOWN: u8 = 0x42;
pub const CEC_OP_UI_CMD_MUTE: u8 = 0x43;
pub const CEC_OP_UI_CMD_PLAY: u8 = 0x44;
pub const CEC_OP_UI_CMD_STOP: u8 = 0x45;
pub const CEC_OP_UI_CMD_PAUSE: u8 = 0x46;
pub const CEC_OP_UI_CMD_RECORD: u8 = 0x47;
pub const CEC_OP_UI_CMD_REWIND: u8 = 0x48;
pub const CEC_OP_UI_CMD_FAST_FORWARD: u8 = 0x49;
pub const CEC_OP_UI_CMD_EJECT: u8 = 0x4a;
pub const CEC_OP_UI_CMD_SKIP_FORWARD: u8 = 0x4b;
pub const CEC_OP_UI_CMD_SKIP_BACKWARD: u8 = 0x4c;
pub const CEC_OP_UI_CMD_STOP_RECORD: u8 = 0x4d;
pub const CEC_OP_UI_CMD_PAUSE_RECORD: u8 = 0x4e;
pub const CEC_OP_UI_CMD_ANGLE: u8 = 0x50;
pub const CEC_OP_UI_CMD_SUB_PICTURE: u8 = 0x51;
pub const CEC_OP_UI_CMD_VIDEO_ON_DEMAND: u8 = 0x52;
pub const CEC_OP_UI_CMD_ELECTRONIC_PROGRAM_GUIDE: u8 = 0x53;
pub const CEC_OP_UI_CMD_TIMER_PROGRAMMING: u8 = 0x54;
pub const CEC_OP_UI_CMD_INITIAL_CONFIGURATION: u8 = 0x55;
pub const CEC_OP_UI_CMD_SELECT_BROADCAST_TYPE: u8 = 0x56;
pub const CEC_OP_UI_CMD_SELECT_SOUND_PRESENTATION: u8 = 0x57;
pub const CEC_OP_UI_CMD_AUDIO_DESCRIPTION: u8 = 0x58;
pub const CEC_OP_UI_CMD_INTERNET: u8 = 0x59;
pub const CEC_OP_UI_CMD_3D_MODE: u8 = 0x5a;
pub const CEC_OP_UI_CMD_PLAY_FUNCTION: u8 = 0x60;
pub const CEC_OP_UI_CMD_PAUSE_PLAY_FUNCTION: u8 = 0x61;
pub const CEC_OP_UI_CMD_RECORD_FUNCTION: u8 = 0x62;
pub const CEC_OP_UI_CMD_PAUSE_RECORD_FUNCTION: u8 = 0x63;
pub const CEC_OP_UI_CMD_STOP_FUNCTION: u8 = 0x64;
pub const CEC_OP_UI_CMD_MUTE_FUNCTION: u8 = 0x65;
pub const CEC_OP_UI_CMD_RESTORE_VOLUME_FUNCTION: u8 = 0x66;
pub const CEC_OP_UI_CMD_TUNE_FUNCTION: u8 = 0x67;
pub const CEC_OP_UI_CMD_SELECT_MEDIA_FUNCTION: u8 = 0x68;
pub const CEC_OP_UI_CMD_SELECT_AV_INPUT_FUNCTION: u8 = 0x69;
pub const CEC_OP_UI_CMD_SELECT_AUDIO_INPUT_FUNCTION: u8 = 0x6a;
pub const CEC_OP_UI_CMD_POWER_TOGGLE_FUNCTION: u8 = 0x6b;
pub const CEC_OP_UI_CMD_POWER_OFF_FUNCTION: u8 = 0x6c;
pub const CEC_OP_UI_CMD_POWER_ON_FUNCTION: u8 = 0x6d;
pub const CEC_OP_UI_CMD_F1_BLUE: u8 = 0x71;
pub const CEC_OP_UI_CMD_F2_RED: u8 = 0x72;
pub const CEC_OP_UI_CMD_F3_GREEN: u8 = 0x73;
pub const CEC_OP_UI_CMD_F4_YELLOW: u8 = 0x74;
pub const CEC_OP_UI_CMD_F5: u8 = 0x75;
pub const CEC_OP_UI_CMD_DATA: u8 = 0x76;
/* UI Broadcast Type Operand (ui_bcast_type) */
pub const CEC_OP_UI_BCAST_TYPE_TOGGLE_ALL: u8 = 0x00;
pub const CEC_OP_UI_BCAST_TYPE_TOGGLE_DIG_ANA: u8 = 0x01;
pub const CEC_OP_UI_BCAST_TYPE_ANALOGUE: u8 = 0x10;
pub const CEC_OP_UI_BCAST_TYPE_ANALOGUE_T: u8 = 0x20;
pub const CEC_OP_UI_BCAST_TYPE_ANALOGUE_CABLE: u8 = 0x30;
pub const CEC_OP_UI_BCAST_TYPE_ANALOGUE_SAT: u8 = 0x40;
pub const CEC_OP_UI_BCAST_TYPE_DIGITAL: u8 = 0x50;
pub const CEC_OP_UI_BCAST_TYPE_DIGITAL_T: u8 = 0x60;
pub const CEC_OP_UI_BCAST_TYPE_DIGITAL_CABLE: u8 = 0x70;
pub const CEC_OP_UI_BCAST_TYPE_DIGITAL_SAT: u8 = 0x80;
pub const CEC_OP_UI_BCAST_TYPE_DIGITAL_COM_SAT: u8 = 0x90;
pub const CEC_OP_UI_BCAST_TYPE_DIGITAL_COM_SAT2: u8 = 0x91;
pub const CEC_OP_UI_BCAST_TYPE_IP: u8 = 0xa0;
/* UI Sound Presentation Control Operand (ui_snd_pres_ctl) */
pub const CEC_OP_UI_SND_PRES_CTL_DUAL_MONO: u8 = 0x10;
pub const CEC_OP_UI_SND_PRES_CTL_KARAOKE: u8 = 0x20;
pub const CEC_OP_UI_SND_PRES_CTL_DOWNMIX: u8 = 0x80;
pub const CEC_OP_UI_SND_PRES_CTL_REVERB: u8 = 0x90;
pub const CEC_OP_UI_SND_PRES_CTL_EQUALIZER: u8 = 0xa0;
pub const CEC_OP_UI_SND_PRES_CTL_BASS_UP: u8 = 0xb1;
pub const CEC_OP_UI_SND_PRES_CTL_BASS_NEUTRAL: u8 = 0xb2;
pub const CEC_OP_UI_SND_PRES_CTL_BASS_DOWN: u8 = 0xb3;
pub const CEC_OP_UI_SND_PRES_CTL_TREBLE_UP: u8 = 0xc1;
pub const CEC_OP_UI_SND_PRES_CTL_TREBLE_NEUTRAL: u8 = 0xc2;
pub const CEC_OP_UI_SND_PRES_CTL_TREBLE_DOWN: u8 = 0xc3;

pub const CEC_MSG_USER_CONTROL_RELEASED: u8 = 0x45;

/* Remote Control Passthrough Feature */

/*
 * Has also:
 * CEC_MSG_USER_CONTROL_PRESSED
 * CEC_MSG_USER_CONTROL_RELEASED
 */

/* Power Status Feature */
pub const CEC_MSG_GIVE_DEVICE_POWER_STATUS: u8 = 0x8f;
pub const CEC_MSG_REPORT_POWER_STATUS: u8 = 0x90;
/* Power Status Operand (pwr_state) */
pub const CEC_OP_POWER_STATUS_ON: u8 = 0;
pub const CEC_OP_POWER_STATUS_STANDBY: u8 = 1;
pub const CEC_OP_POWER_STATUS_TO_ON: u8 = 2;
pub const CEC_OP_POWER_STATUS_TO_STANDBY: u8 = 3;

/* General Protocol Messages */
pub const CEC_MSG_FEATURE_ABORT: u8 = 0x00;
/* Abort Reason Operand (reason) */
pub const CEC_OP_ABORT_UNRECOGNIZED_OP: u8 = 0;
pub const CEC_OP_ABORT_INCORRECT_MODE: u8 = 1;
pub const CEC_OP_ABORT_NO_SOURCE: u8 = 2;
pub const CEC_OP_ABORT_INVALID_OP: u8 = 3;
pub const CEC_OP_ABORT_REFUSED: u8 = 4;
pub const CEC_OP_ABORT_UNDETERMINED: u8 = 5;

pub const CEC_MSG_ABORT: u8 = 0xff;

/* System Audio Control Feature */

/*
 * Has also:
 * CEC_MSG_USER_CONTROL_PRESSED
 * CEC_MSG_USER_CONTROL_RELEASED
 */
pub const CEC_MSG_GIVE_AUDIO_STATUS: u8 = 0x71;
pub const CEC_MSG_GIVE_SYSTEM_AUDIO_MODE_STATUS: u8 = 0x7d;
pub const CEC_MSG_REPORT_AUDIO_STATUS: u8 = 0x7a;
/* Audio Mute Status Operand (aud_mute_status) */
pub const CEC_OP_AUD_MUTE_STATUS_OFF: u8 = 0;
pub const CEC_OP_AUD_MUTE_STATUS_ON: u8 = 1;

pub const CEC_MSG_REPORT_SHORT_AUDIO_DESCRIPTOR: u8 = 0xa3;
pub const CEC_MSG_REQUEST_SHORT_AUDIO_DESCRIPTOR: u8 = 0xa4;
pub const CEC_MSG_SET_SYSTEM_AUDIO_MODE: u8 = 0x72;
/* System Audio Status Operand (sys_aud_status) */
pub const CEC_OP_SYS_AUD_STATUS_OFF: u8 = 0;
pub const CEC_OP_SYS_AUD_STATUS_ON: u8 = 1;

pub const CEC_MSG_SYSTEM_AUDIO_MODE_REQUEST: u8 = 0x70;
pub const CEC_MSG_SYSTEM_AUDIO_MODE_STATUS: u8 = 0x7e;
/* Audio Format ID Operand (audio_format_id) */
pub const CEC_OP_AUD_FMT_ID_CEA861: u8 = 0;
pub const CEC_OP_AUD_FMT_ID_CEA861_CXT: u8 = 1;

pub const CEC_MSG_SET_AUDIO_VOLUME_LEVEL: u8 = 0x73;

/* Audio Rate Control Feature */
pub const CEC_MSG_SET_AUDIO_RATE: u8 = 0x9a;
/* Audio Rate Operand (audio_rate) */
pub const CEC_OP_AUD_RATE_OFF: u8 = 0;
pub const CEC_OP_AUD_RATE_WIDE_STD: u8 = 1;
pub const CEC_OP_AUD_RATE_WIDE_FAST: u8 = 2;
pub const CEC_OP_AUD_RATE_WIDE_SLOW: u8 = 3;
pub const CEC_OP_AUD_RATE_NARROW_STD: u8 = 4;
pub const CEC_OP_AUD_RATE_NARROW_FAST: u8 = 5;
pub const CEC_OP_AUD_RATE_NARROW_SLOW: u8 = 6;

/* Audio Return Channel Control Feature */
pub const CEC_MSG_INITIATE_ARC: u8 = 0xc0;
pub const CEC_MSG_REPORT_ARC_INITIATED: u8 = 0xc1;
pub const CEC_MSG_REPORT_ARC_TERMINATED: u8 = 0xc2;
pub const CEC_MSG_REQUEST_ARC_INITIATION: u8 = 0xc3;
pub const CEC_MSG_REQUEST_ARC_TERMINATION: u8 = 0xc4;
pub const CEC_MSG_TERMINATE_ARC: u8 = 0xc5;

/* Dynamic Audio Lipsync Feature */
/* Only for CEC 2.0 and up */
pub const CEC_MSG_REQUEST_CURRENT_LATENCY: u8 = 0xa7;
pub const CEC_MSG_REPORT_CURRENT_LATENCY: u8 = 0xa8;
/* Low Latency Mode Operand (low_latency_mode) */
pub const CEC_OP_LOW_LATENCY_MODE_OFF: u8 = 0;
pub const CEC_OP_LOW_LATENCY_MODE_ON: u8 = 1;
/* Audio Output Compensated Operand (audio_out_compensated) */
pub const CEC_OP_AUD_OUT_COMPENSATED_NA: u8 = 0;
pub const CEC_OP_AUD_OUT_COMPENSATED_DELAY: u8 = 1;
pub const CEC_OP_AUD_OUT_COMPENSATED_NO_DELAY: u8 = 2;
pub const CEC_OP_AUD_OUT_COMPENSATED_PARTIAL_DELAY: u8 = 3;

/* Capability Discovery and Control Feature */
pub const CEC_MSG_CDC_MESSAGE: u8 = 0xf8;
/* Ethernet-over-HDMI: nobody ever does this... */
pub const CEC_MSG_CDC_HEC_INQUIRE_STATE: u8 = 0x00;
pub const CEC_MSG_CDC_HEC_REPORT_STATE: u8 = 0x01;
/* HEC Functionality State Operand (hec_func_state) */
pub const CEC_OP_HEC_FUNC_STATE_NOT_SUPPORTED: u8 = 0;
pub const CEC_OP_HEC_FUNC_STATE_INACTIVE: u8 = 1;
pub const CEC_OP_HEC_FUNC_STATE_ACTIVE: u8 = 2;
pub const CEC_OP_HEC_FUNC_STATE_ACTIVATION_FIELD: u8 = 3;
/* Host Functionality State Operand (host_func_state) */
pub const CEC_OP_HOST_FUNC_STATE_NOT_SUPPORTED: u8 = 0;
pub const CEC_OP_HOST_FUNC_STATE_INACTIVE: u8 = 1;
pub const CEC_OP_HOST_FUNC_STATE_ACTIVE: u8 = 2;
/* ENC Functionality State Operand (enc_func_state) */
pub const CEC_OP_ENC_FUNC_STATE_EXT_CON_NOT_SUPPORTED: u8 = 0;
pub const CEC_OP_ENC_FUNC_STATE_EXT_CON_INACTIVE: u8 = 1;
pub const CEC_OP_ENC_FUNC_STATE_EXT_CON_ACTIVE: u8 = 2;
/* CDC Error Code Operand (cdc_errcode) */
pub const CEC_OP_CDC_ERROR_CODE_NONE: u8 = 0;
pub const CEC_OP_CDC_ERROR_CODE_CAP_UNSUPPORTED: u8 = 1;
pub const CEC_OP_CDC_ERROR_CODE_WRONG_STATE: u8 = 2;
pub const CEC_OP_CDC_ERROR_CODE_OTHER: u8 = 3;
/* HEC Support Operand (hec_support) */
pub const CEC_OP_HEC_SUPPORT_NO: u8 = 0;
pub const CEC_OP_HEC_SUPPORT_YES: u8 = 1;
/* HEC Activation Operand (hec_activation) */
pub const CEC_OP_HEC_ACTIVATION_ON: u8 = 0;
pub const CEC_OP_HEC_ACTIVATION_OFF: u8 = 1;

pub const CEC_MSG_CDC_HEC_SET_STATE_ADJACENT: u8 = 0x02;
pub const CEC_MSG_CDC_HEC_SET_STATE: u8 = 0x03;
/* HEC Set State Operand (hec_set_state) */
pub const CEC_OP_HEC_SET_STATE_DEACTIVATE: u8 = 0;
pub const CEC_OP_HEC_SET_STATE_ACTIVATE: u8 = 1;

pub const CEC_MSG_CDC_HEC_REQUEST_DEACTIVATION: u8 = 0x04;
pub const CEC_MSG_CDC_HEC_NOTIFY_ALIVE: u8 = 0x05;
pub const CEC_MSG_CDC_HEC_DISCOVER: u8 = 0x06;
/* Hotplug Detect messages */
pub const CEC_MSG_CDC_HPD_SET_STATE: u8 = 0x10;
/* HPD State Operand (hpd_state) */
pub const CEC_OP_HPD_STATE_CP_EDID_DISABLE: u8 = 0;
pub const CEC_OP_HPD_STATE_CP_EDID_ENABLE: u8 = 1;
pub const CEC_OP_HPD_STATE_CP_EDID_DISABLE_ENABLE: u8 = 2;
pub const CEC_OP_HPD_STATE_EDID_DISABLE: u8 = 3;
pub const CEC_OP_HPD_STATE_EDID_ENABLE: u8 = 4;
pub const CEC_OP_HPD_STATE_EDID_DISABLE_ENABLE: u8 = 5;
pub const CEC_MSG_CDC_HPD_REPORT_STATE: u8 = 0x11;
/* HPD Error Code Operand (hpd_error) */
pub const CEC_OP_HPD_ERROR_NONE: u8 = 0;
pub const CEC_OP_HPD_ERROR_INITIATOR_NOT_CAPABLE: u8 = 1;
pub const CEC_OP_HPD_ERROR_INITIATOR_WRONG_STATE: u8 = 2;
pub const CEC_OP_HPD_ERROR_OTHER: u8 = 3;
pub const CEC_OP_HPD_ERROR_NONE_NO_VIDEO: u8 = 4;

/* End of Messages */
