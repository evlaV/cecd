/*
 * Copyright © 2024 Valve Software
 *
 * Based in part on linux/cec.h
 * Copyright 2016 Cisco Systems, Inc. and/or its affiliates. All rights reserved.
 * SPDX-License-Identifier: BSD-3-Clause
 */

#![cfg_attr(not(feature = "std"), no_std)]

pub mod constants;
pub mod ioctls;
pub mod structs;

pub use constants::*;
pub use ioctls::*;
pub use structs::*;

pub type LogicalAddress = u8;
pub type MessageHandlingMode = u32;
pub type PhysicalAddress = u16;
pub type Timestamp = u64;
