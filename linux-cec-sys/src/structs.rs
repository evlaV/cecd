/*
 * Copyright © 2024 Valve Software
 *
 * Based in part on linux/cec.h
 * Copyright 2016 Cisco Systems, Inc. and/or its affiliates. All rights reserved.
 * SPDX-License-Identifier: BSD-3-Clause
 */

#![allow(non_camel_case_types)]

use bitflags::bitflags;
use core::ffi::{c_char, c_uchar};

use crate::constants::*;
use crate::{LogicalAddress, PhysicalAddress, Timestamp};

/// CEC message structure.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct cec_msg {
    /// Timestamp in nanoseconds using `CLOCK_MONOTONIC`. Set by the
    /// driver when the message transmission has finished.
    pub tx_ts: Timestamp,
    /// Timestamp in nanoseconds using `CLOCK_MONOTONIC`. Set by the
    /// driver when the message was received.
    pub rx_ts: Timestamp,
    /// Length in bytes of the message.
    pub len: u32,
    /**
     * The timeout (in ms) that is used to timeout `CEC_RECEIVE`.
     * Set to 0 if you want to wait forever. This timeout can also be
     * used with `CEC_TRANSMIT` as the timeout for waiting for a reply.
     * If 0, then it will use a 1 second timeout instead of waiting
     * forever as is done with `CEC_RECEIVE`.
     */
    pub timeout: u32,
    /// The framework assigns a sequence number to messages that are
    /// sent. This can be used to track replies to previously sent messages.
    pub sequence: u32,
    /// Set to 0.
    pub flags: CEC_MSG_FL,
    /// The message payload.
    pub msg: [u8; CEC_MAX_MSG_SIZE],
    /**
     * This field is ignored with `CEC_RECEIVE` and is only used by
     * `CEC_TRANSMIT`. If non-zero, then wait for a reply with this
     * opcode. Set to `CEC_MSG_FEATURE_ABORT` if you want to wait for
     * a possible `ABORT` reply. If there was an error when sending the
     * msg or `FeatureAbort` was returned, then reply is set to 0.
     * If reply is non-zero upon return, then len/msg are set to
     * the received message.
     * If reply is zero upon return and status has the
     * `CEC_TX_STATUS_FEATURE_ABORT` bit set, then len/msg are set to
     * the received feature abort message.
     * If reply is zero upon return and status has the
     * `CEC_TX_STATUS_MAX_RETRIES` bit set, then no reply was seen at
     * all. If reply is non-zero for `CEC_TRANSMIT` and the message is a
     * broadcast, then `-EINVAL` is returned.
     * if reply is non-zero, then timeout is set to 1000 (the required
     * maximum response time).
     */
    pub reply: u8,
    /// The message receive status bits. Set by the driver.
    pub rx_status: CEC_RX_STATUS,
    /// The message transmit status bits. Set by the driver.
    pub tx_status: CEC_TX_STATUS,
    /// The number of 'Arbitration Lost' events. Set by the driver.
    pub tx_arb_lost_cnt: u8,
    /// The number of 'Not Acknowledged' events. Set by the driver.
    pub tx_nack_cnt: u8,
    /// The number of 'Low Drive Detected' events. Set by the driver.
    pub tx_low_drive_cnt: u8,
    /// The number of 'Error' events. Set by the driver.
    pub tx_error_cnt: u8,
}

bitflags! {
    /// A bitflag struct for values of the CEC_MSG_FL_* constants
    #[derive(Debug, Copy, Clone, Default)]
    pub struct CEC_MSG_FL: u32 {
        const REPLY_TO_FOLLOWERS = CEC_MSG_FL_REPLY_TO_FOLLOWERS;
        const RAW = CEC_MSG_FL_RAW;
    }
}

bitflags! {
    /// A bitflag struct for values of the CEC_TX_STATUS_* constants
    #[derive(Debug, Copy, Clone)]
    pub struct CEC_TX_STATUS: u8 {
        const OK = CEC_TX_STATUS_OK;
        const ARB_LOST = CEC_TX_STATUS_ARB_LOST;
        const NACK = CEC_TX_STATUS_NACK;
        const LOW_DRIVE = CEC_TX_STATUS_LOW_DRIVE;
        const ERROR = CEC_TX_STATUS_ERROR;
        const MAX_RETRIES = CEC_TX_STATUS_MAX_RETRIES;
        const ABORTED = CEC_TX_STATUS_ABORTED;
        const TIMEOUT = CEC_TX_STATUS_TIMEOUT;
    }
}

bitflags! {
    /// A bitflag struct for values of the CEC_RX_STATUS_* constants
    #[derive(Debug, Copy, Clone)]
    pub struct CEC_RX_STATUS: u8 {
        const OK = CEC_RX_STATUS_OK;
        const TIMEOUT = CEC_RX_STATUS_TIMEOUT;
        const FEATURE_ABORT = CEC_RX_STATUS_FEATURE_ABORT;
        const ABORTED = CEC_RX_STATUS_ABORTED;
    }
}

impl cec_msg {
    /// Return the initiator's logical address.
    #[inline]
    #[must_use]
    pub fn initiator(&self) -> u8 {
        self.msg[0] >> 4
    }

    /// Return the destination's logical address.
    #[inline]
    #[must_use]
    pub fn destination(&self) -> u8 {
        self.msg[0] & 0xf
    }

    /// Return the opcode of the message, None for poll
    #[inline]
    #[must_use]
    pub fn opcode(&self) -> Option<u8> {
        if self.len > 1 {
            Some(self.msg[1])
        } else {
            None
        }
    }

    /// Return true if this is a broadcast message.
    #[inline]
    #[must_use]
    pub fn is_broadcast(&self) -> bool {
        (self.msg[0] & 0xf) == 0xf
    }

    /**
     * Initialize the message structure.
     * `initiator` is the logical address of the initiator and
     * `destination` the logical address of the destination (`0xf` for broadcast).
     *
     * The whole structure is zeroed, the len field is set to 1 (i.e. a poll
     * message) and the initiator and destination are filled in.
     */
    #[inline]
    #[must_use]
    pub fn new(initiator: LogicalAddress, destination: LogicalAddress) -> cec_msg {
        let mut msg = cec_msg {
            tx_ts: 0,
            rx_ts: 0,
            len: 1,
            timeout: 0,
            sequence: 0,
            flags: CEC_MSG_FL::empty(),
            msg: [0; 16],
            reply: 0,
            rx_status: CEC_RX_STATUS::empty(),
            tx_status: CEC_TX_STATUS::empty(),
            tx_arb_lost_cnt: 0,
            tx_nack_cnt: 0,
            tx_low_drive_cnt: 0,
            tx_error_cnt: 0,
        };
        msg.msg[0] = (initiator << 4) | destination;

        msg
    }

    #[inline]
    #[must_use]
    pub fn with_timeout(mut self, timeout_ms: u32) -> cec_msg {
        self.timeout = timeout_ms;
        self
    }

    #[inline]
    #[must_use]
    pub fn from_timeout(timeout_ms: u32) -> cec_msg {
        cec_msg {
            tx_ts: 0,
            rx_ts: 0,
            len: 0,
            timeout: timeout_ms,
            sequence: 0,
            flags: CEC_MSG_FL::empty(),
            msg: [0; 16],
            reply: 0,
            rx_status: CEC_RX_STATUS::empty(),
            tx_status: CEC_TX_STATUS::empty(),
            tx_arb_lost_cnt: 0,
            tx_nack_cnt: 0,
            tx_low_drive_cnt: 0,
            tx_error_cnt: 0,
        }
    }

    /**
     * Fill in destination/initiator in a reply message.
     *
     * Set the msg destination to the orig initiator and the msg initiator to the
     * orig destination. Note that msg and orig may be the same pointer, in which
     * case the change is done in place.
     */
    #[inline]
    pub fn set_reply_to(&mut self, orig: &cec_msg) {
        /* The destination becomes the initiator and vice versa */
        self.msg[0] = (orig.destination() << 4) | orig.initiator();
        self.reply = 0;
        self.timeout = 0;
    }

    /// Return true if this message contains the result of an earlier non-blocking transmit
    #[inline]
    #[must_use]
    pub fn recv_is_tx_result(&self) -> bool {
        self.sequence != 0 && !self.tx_status.is_empty() && self.rx_status.is_empty()
    }

    /// Return true if this message contains the reply of an earlier non-blocking transmit
    #[inline]
    #[must_use]
    pub fn recv_is_rx_result(&self) -> bool {
        self.sequence != 0 && self.tx_status.is_empty() && !self.rx_status.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn status_is_ok(&self) -> bool {
        if !self.tx_status.is_empty() && !self.tx_status.contains(CEC_TX_STATUS::OK) {
            return false;
        }
        if !self.rx_status.is_empty() && !self.rx_status.contains(CEC_RX_STATUS::OK) {
            return false;
        }
        if self.tx_status.is_empty() && self.rx_status.is_empty() {
            return false;
        }
        !self.rx_status.contains(CEC_RX_STATUS::FEATURE_ABORT)
    }
}

bitflags! {
    /// A bitflag struct for values of the CEC_LOG_ADDR_MASK_* constants
    #[derive(Debug, Copy, Clone, Default)]
    pub struct CEC_LOG_ADDR_MASK: u16 {
        const TV = CEC_LOG_ADDR_MASK_TV;
        const MASK_RECORD = CEC_LOG_ADDR_MASK_RECORD;
        const TUNER = CEC_LOG_ADDR_MASK_TUNER;
        const PLAYBACK = CEC_LOG_ADDR_MASK_PLAYBACK;
        const AUDIOSYSTEM = CEC_LOG_ADDR_MASK_AUDIOSYSTEM;
        const BACKUP = CEC_LOG_ADDR_MASK_BACKUP;
        const SPECIFIC = CEC_LOG_ADDR_MASK_SPECIFIC;
        const UNREGISTERED = CEC_LOG_ADDR_MASK_UNREGISTERED;
    }
}

bitflags! {
    /// A bitflag struct for values of the CEC_CAP_* constants
    #[derive(Debug, Copy, Clone, Default)]
    pub struct CEC_CAP: u32 {
        /// Userspace has to configure the physical address
        const PHYS_ADDR = CEC_CAP_PHYS_ADDR;
        /// Userspace has to configure the logical addresses
        const LOG_ADDRS = CEC_CAP_LOG_ADDRS;
        /// Userspace can transmit messages (and thus become follower as well)
        const TRANSMIT = CEC_CAP_TRANSMIT;
        /// Passthrough all messages instead of processing them.
        const PASSTHROUGH = CEC_CAP_PASSTHROUGH;
        /// Supports remote control
        const RC = CEC_CAP_RC;
        /// Hardware can monitor all messages, not just directed and broadcast.
        const MONITOR_ALL = CEC_CAP_MONITOR_ALL;
        /// Hardware can use CEC only if the HDMI HPD pin is high.
        const NEEDS_HPD = CEC_CAP_NEEDS_HPD;
        /// Hardware can monitor CEC pin transitions
        const MONITOR_PIN = CEC_CAP_MONITOR_PIN;
        /// CEC_ADAP_G_CONNECTOR_INFO is available
        const CONNECTOR_INFO = CEC_CAP_CONNECTOR_INFO;
        /// CEC_MSG_FL_REPLY_VENDOR_ID is available
        const REPLY_VENDOR_ID = CEC_CAP_REPLY_VENDOR_ID;
    }
}

/// CEC capabilities structure.
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct cec_caps {
    /// Name of the CEC device driver.
    pub driver: [c_char; 32],
    /// Name of the CEC device. `driver` + `name` must be unique.
    pub name: [c_char; 32],
    /// Number of available logical addresses.
    pub available_log_addrs: u32,
    /// Capabilities of the CEC adapter.
    pub capabilities: CEC_CAP,
    /// version of the CEC adapter framework.
    pub version: u32,
}

/// CEC logical addresses structure
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct cec_log_addrs {
    /// The claimed logical addresses. Set by the driver.
    pub log_addr: [u8; CEC_MAX_LOG_ADDRS],
    /// Current logical address mask. Set by the driver.
    pub log_addr_mask: CEC_LOG_ADDR_MASK,
    /// The CEC version that the adapter should implement. Set by the caller.
    pub cec_version: u8,
    /// How many logical addresses should be claimed. Set by the caller.
    pub num_log_addrs: u8,
    /// The vendor ID of the device. Set by the caller.
    pub vendor_id: VendorId,
    /// Flags.
    pub flags: CEC_LOG_ADDRS_FL,
    /// The OSD name of the device. Set by the caller.
    pub osd_name: [c_uchar; 15],
    /// The primary device type for each logical address. Set by the caller.
    pub primary_device_type: [u8; CEC_MAX_LOG_ADDRS],
    /// The logical address types. Set by the caller.
    pub log_addr_type: [u8; CEC_MAX_LOG_ADDRS],

    /* CEC 2.0 */
    /// CEC 2.0: all device types represented by the logical address. Set by the caller.
    pub all_device_types: [u8; CEC_MAX_LOG_ADDRS],
    /// CEC 2.0: The logical address features. Set by the caller.
    pub features: [[u8; 12]; CEC_MAX_LOG_ADDRS],
}

impl cec_log_addrs {
    /* Helper functions to identify the 'special' CEC devices */

    #[inline]
    #[must_use]
    pub fn is_2nd_tv(&self) -> bool {
        /*
         * It is a second TV if the logical address is 14 or 15 and the
         * primary device type is a TV.
         */
        self.num_log_addrs != 0
            && self.log_addr[0] >= CEC_LOG_ADDR_SPECIFIC
            && self.primary_device_type[0] == CEC_OP_PRIM_DEVTYPE_TV
    }

    #[inline]
    #[must_use]
    pub fn is_processor(&self) -> bool {
        /*
         * It is a processor if the logical address is 12-15 and the
         * primary device type is a Processor.
         */
        self.num_log_addrs != 0
            && self.log_addr[0] >= CEC_LOG_ADDR_BACKUP_1
            && self.primary_device_type[0] == CEC_OP_PRIM_DEVTYPE_PROCESSOR
    }

    #[inline]
    #[must_use]
    pub fn is_switch(&self) -> bool {
        /*
         * It is a switch if the logical address is 15 and the
         * primary device type is a Switch and the CDC-Only flag is not set.
         */
        self.num_log_addrs == 1
            && self.log_addr[0] == CEC_LOG_ADDR_UNREGISTERED
            && self.primary_device_type[0] == CEC_OP_PRIM_DEVTYPE_SWITCH
            && !self.flags.contains(CEC_LOG_ADDRS_FL::CDC_ONLY)
    }

    #[inline]
    #[must_use]
    pub fn is_cdc_only(&self) -> bool {
        /*
         * It is a CDC-only device if the logical address is 15 and the
         * primary device type is a Switch and the CDC-Only flag is set.
         */
        self.num_log_addrs == 1
            && self.log_addr[0] == CEC_LOG_ADDR_UNREGISTERED
            && self.primary_device_type[0] == CEC_OP_PRIM_DEVTYPE_SWITCH
            && self.flags.contains(CEC_LOG_ADDRS_FL::CDC_ONLY)
    }
}

bitflags! {
    /// A bitflag struct for values of the CEC_LOG_ADDRS_FL_* constants
    #[derive(Debug, Copy, Clone, Default)]
    pub struct CEC_LOG_ADDRS_FL: u32 {
        /// Allow a fallback to unregistered
        const ALLOW_UNREG_FALLBACK = CEC_LOG_ADDRS_FL_ALLOW_UNREG_FALLBACK;
        /// Passthrough RC messages to the input subsystem
        const ALLOW_RC_PASSTHRU = CEC_LOG_ADDRS_FL_ALLOW_RC_PASSTHRU;
        /// CDC-Only device: supports only CDC messages
        const CDC_ONLY = CEC_LOG_ADDRS_FL_CDC_ONLY;
    }
}

/// Tells which drm connector is associated with the CEC adapter.
#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct cec_drm_connector_info {
    /// drm card number
    pub card_no: u32,
    /// drm connector ID
    pub connector_id: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union cec_connector_info_union {
    /// drm connector info
    pub drm: cec_drm_connector_info,
    /// Array to pad the union
    pub raw: [u32; 16],
}

/// Tells if and which connector is associated with the CEC adapter.
#[derive(Clone)]
#[repr(C)]
pub struct cec_connector_info {
    /// Connector type (if any)
    pub ty: u32,
    pub data: cec_connector_info_union,
}

impl Default for cec_connector_info {
    #[inline]
    fn default() -> cec_connector_info {
        cec_connector_info {
            ty: CEC_CONNECTOR_TYPE_NO_CONNECTOR,
            data: cec_connector_info_union { raw: [0; 16] },
        }
    }
}

bitflags! {
    /// A bitflag struct for values of the CEC_EVENT_FL_* constants
    #[derive(Debug, Copy, Clone, Default)]
    pub struct CEC_EVENT_FL: u32 {
        const INITIAL_STATE = CEC_EVENT_FL_INITIAL_STATE;
        const DROPPED_EVENTS = CEC_EVENT_FL_DROPPED_EVENTS;
    }
}

/// Used when the CEC adapter changes state.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct cec_event_state_change {
    /// The current physical address
    pub phys_addr: PhysicalAddress,
    /// The current logical address mask
    pub log_addr_mask: CEC_LOG_ADDR_MASK,
    /** If non-zero, then HDMI connector information is available.
     *  This field is only valid if CEC_CAP_CONNECTOR_INFO is set. If that
     *  capability is set and @have_conn_info is zero, then that indicates
     *  that the HDMI connector device is not instantiated, either because
     *  the HDMI driver is still configuring the device or because the HDMI
     *  device was unbound.
     */
    pub have_conn_info: u16,
}

/// Tells you how many messages were lost.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct cec_event_lost_msgs {
    /// How many messages were lost.
    pub lost_msgs: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union cec_event_union {
    /// The event payload for CEC_EVENT_STATE_CHANGE.
    pub state_change: cec_event_state_change,
    /// The event payload for CEC_EVENT_LOST_MSGS.
    pub lost_msgs: cec_event_lost_msgs,
    /// Array to pad the union.
    pub raw: [u32; 16],
}

/// CEC event structure
#[derive(Clone)]
#[repr(C)]
pub struct cec_event {
    /// The timestamp of when the event was sent.
    pub ts: Timestamp,
    /// The event.
    pub event: u32,
    /// Event flags.
    pub flags: CEC_EVENT_FL,
    pub data: cec_event_union,
}

impl Default for cec_event {
    #[inline]
    fn default() -> cec_event {
        cec_event {
            ts: 0,
            event: 0,
            flags: CEC_EVENT_FL::default(),
            data: cec_event_union { raw: [0; 16] },
        }
    }
}

/// Convenience type for the non-zero null vendor ID
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct VendorId(u32);

impl Default for VendorId {
    #[inline]
    fn default() -> Self {
        VendorId(CEC_VENDOR_ID_NONE)
    }
}

impl TryFrom<u32> for VendorId {
    type Error = ();

    #[inline]
    fn try_from(val: u32) -> Result<VendorId, ()> {
        if val < 0x1_00_00_00 || val == CEC_VENDOR_ID_NONE {
            Ok(VendorId(val))
        } else {
            Err(())
        }
    }
}

impl From<VendorId> for u32 {
    #[inline]
    fn from(val: VendorId) -> u32 {
        val.0
    }
}

impl VendorId {
    #[inline]
    #[must_use]
    pub fn is_none(self) -> bool {
        self.0 == CEC_VENDOR_ID_NONE
    }

    #[inline]
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.0 < 0x1_00_00_00
    }
}
