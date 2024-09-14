use crate::constants;

type Opcode = u8;
type Timestamp = u64;
type RxStatusFlags = u8;
type TxStatusFlags = u8;
type LogicalAddress = u8;

/**
 * CecMessage - CEC message structure.
 * @tx_ts:	Timestamp in nanoseconds using CLOCK_MONOTONIC. Set by the
 *		driver when the message transmission has finished.
 * @rx_ts:	Timestamp in nanoseconds using CLOCK_MONOTONIC. Set by the
 *		driver when the message was received.
 * @len:	Length in bytes of the message.
 * @timeout:	The timeout (in ms) that is used to timeout CEC_RECEIVE.
 *		Set to 0 if you want to wait forever. This timeout can also be
 *		used with CEC_TRANSMIT as the timeout for waiting for a reply.
 *		If 0, then it will use a 1 second timeout instead of waiting
 *		forever as is done with CEC_RECEIVE.
 * @sequence:	The framework assigns a sequence number to messages that are
 *		sent. This can be used to track replies to previously sent
 *		messages.
 * @flags:	Set to 0.
 * @msg:	The message payload.
 * @reply:	This field is ignored with CEC_RECEIVE and is only used by
 *		CEC_TRANSMIT. If non-zero, then wait for a reply with this
 *		opcode. Set to CEC_MSG_FEATURE_ABORT if you want to wait for
 *		a possible ABORT reply. If there was an error when sending the
 *		msg or FeatureAbort was returned, then reply is set to 0.
 *		If reply is non-zero upon return, then len/msg are set to
 *		the received message.
 *		If reply is zero upon return and status has the
 *		CEC_TX_STATUS_FEATURE_ABORT bit set, then len/msg are set to
 *		the received feature abort message.
 *		If reply is zero upon return and status has the
 *		CEC_TX_STATUS_MAX_RETRIES bit set, then no reply was seen at
 *		all. If reply is non-zero for CEC_TRANSMIT and the message is a
 *		broadcast, then -EINVAL is returned.
 *		if reply is non-zero, then timeout is set to 1000 (the required
 *		maximum response time).
 * @rx_status:	The message receive status bits. Set by the driver.
 * @tx_status:	The message transmit status bits. Set by the driver.
 * @tx_arb_lost_cnt: The number of 'Arbitration Lost' events. Set by the driver.
 * @tx_nack_cnt: The number of 'Not Acknowledged' events. Set by the driver.
 * @tx_low_drive_cnt: The number of 'Low Drive Detected' events. Set by the
 *		driver.
 * @tx_error_cnt: The number of 'Error' events. Set by the driver.
 */
#[repr(C)]
pub struct CecMessage {
    tx_ts: Timestamp,
    rx_ts: Timestamp,
    len: u32,
    timeout: u32,
    sequence: u32,
    flags: u32,
    msg: [u8; constants::CEC_MAX_MSG_SIZE],
    reply: Opcode,
    rx_status: u8,
    tx_status: u8,
    tx_arb_lost_cnt: u8,
    tx_nack_cnt: u8,
    tx_low_drive_cnt: u8,
    tx_error_cnt: u8,
}

impl CecMessage {
    /**
     * cec_msg_initiator - return the initiator's logical address.
     */
    pub fn initiator(&self) -> u8 {
        self.msg[0] >> 4
    }

    /**
     * cec_msg_destination - return the destination's logical address.
     */
    pub fn destination(&self) -> u8 {
        self.msg[0] & 0xf
    }

    /**
     * cec_msg_opcode - return the opcode of the message, None for poll
     */
    pub fn opcode(&self) -> Option<Opcode> {
        if self.len > 1 {
            Some(self.msg[1])
        } else {
            None
        }
    }

    /**
     * is_broadcast - return true if this is a broadcast message.
     */
    pub fn is_broadcast(&self) -> bool {
        (self.msg[0] & 0xf) == 0xf
    }

    /**
     * new - initialize the message structure.
     * @initiator:	the logical address of the initiator
     * @destination:the logical address of the destination (0xf for broadcast)
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
            flags: 0,
            msg: [0; 16],
            reply: 0,
            rx_status: 0,
            tx_status: 0,
            tx_arb_lost_cnt: 0,
            tx_nack_cnt: 0,
            tx_low_drive_cnt: 0,
            tx_error_cnt: 0,
        };
        msg.msg[0] = (initiator << 4) | destination;

        msg
    }

    /**
     * set_reply_to - fill in destination/initiator in a reply message.
     * @orig:	the original message structure
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

    /**
     * recv_is_tx_result - return true if this message contains the
     *			       result of an earlier non-blocking transmit
     */
    fn recv_is_tx_result(&self) -> bool {
        self.sequence != 0 && self.tx_status != 0 && self.rx_status == 0
    }

    /**
     * recv_is_rx_result - return true if this message contains the
     *			       reply of an earlier non-blocking transmit
     */
    fn recv_is_rx_result(&self) -> bool {
        self.sequence != 0 && self.tx_status == 0 && self.rx_status != 0
    }

    fn status_is_ok(&self) -> bool {
        if self.tx_status != 0 && (self.tx_status & constants::CEC_TX_STATUS_OK) == 0 {
            return false;
        }
        if self.rx_status != 0 && (self.rx_status & constants::CEC_RX_STATUS_OK) == 0 {
            return false;
        }
        if self.tx_status == 0 && self.rx_status == 0 {
            return false;
        }
        (self.rx_status & constants::CEC_RX_STATUS_FEATURE_ABORT) == 0
    }
}
