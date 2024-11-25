/*
 * Copyright © 2024 Valve Software
 *
 * Based in part on linux/cec.h
 * Copyright 2016 Cisco Systems, Inc. and/or its affiliates. All rights reserved.
 * SPDX-License-Identifier: BSD-3-Clause
 */

use nix::{ioctl_read, ioctl_readwrite, ioctl_write_ptr};

use crate::structs::*;
use crate::{MessageHandlingMode, PhysicalAddress};

/* Adapter capabilities */
ioctl_readwrite!(adapter_get_capabilities, b'a', 0, cec_caps);

/*
 * phys_addr is either 0 (if this is the CEC root device)
 * or a valid physical address obtained from the sink's EDID
 * as read by this CEC device (if this is a source device)
 * or a physical address obtained and modified from a sink
 * EDID and used for a sink CEC device.
 * If nothing is connected, then phys_addr is 0xffff.
 * See HDMI 1.4b, section 8.7 (Physical Address).
 *
 * The CEC_ADAP_S_PHYS_ADDR ioctl may not be available if that is handled
 * internally.
 */
ioctl_read!(adapter_get_physical_address, b'a', 1, PhysicalAddress);
ioctl_write_ptr!(adapter_set_physical_address, b'a', 2, PhysicalAddress);

/*
 * Configure the CEC adapter. It sets the device type and which
 * logical types it will try to claim. It will return which
 * logical addresses it could actually claim.
 * An error is returned if the adapter is disabled or if there
 * is no physical address assigned.
 */
ioctl_read!(adapter_get_logical_addresses, b'a', 3, cec_log_addrs);
ioctl_readwrite!(adapter_set_logical_addresses, b'a', 4, cec_log_addrs);

/* Transmit/receive a CEC command */
ioctl_readwrite!(transmit_message, b'a', 5, cec_msg);
ioctl_readwrite!(receive_message, b'a', 6, cec_msg);

/* Dequeue CEC events */
ioctl_readwrite!(dequeue_event, b'a', 7, cec_event);

/*
 * Get and set the message handling mode for this filehandle.
 */
ioctl_read!(get_mode, b'a', 8, MessageHandlingMode);
ioctl_write_ptr!(set_mode, b'a', 9, MessageHandlingMode);

/* Get the connector info */
ioctl_read!(adapter_get_connector_info, b'a', 10, cec_connector_info);
