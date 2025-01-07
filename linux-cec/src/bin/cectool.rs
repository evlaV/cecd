/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use clap::{Parser, Subcommand};
use linux_cec::device::Device;
use linux_cec::message::Message;
use linux_cec::operand::{BufferOperand, UiCommand};
use linux_cec::{FollowerMode, InitiatorMode, LogicalAddress, LogicalAddressType, Result};
use num_enum::TryFromPrimitive;
use std::str::FromStr;

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = "Basic tool for interfacing with the Linux CEC subsystem"
)]
struct Arguments {
    #[command(subcommand)]
    command: Command,

    /// Which CEC device to use
    #[arg(short, long, default_value_t = String::from("/dev/cec0"))]
    device: String,
}

#[derive(Subcommand)]
enum Command {
    /// Get the connector information from the CEC subsystem
    GetConnectorInfo,
    /// Get the physical address of the CEC adapter
    GetPhysicalAddress,
    /// Get the logical address of the CEC adapter
    GetLogicalAddress,
    /// Set the logical address of the CEC adapter
    SetLogicalAddress {
        /// The desired logical address type
        log_addr: LogicalAddressType,
    },
    /// Clear the logical address of the CEC adapter
    ClearLogicalAddress,
    /// Set the name displayed of this device as displayed on an OSD
    SetOsdName {
        /// The desired name, with a maximum of 14 bytes
        name: String,
    },
    /// Make this the active device
    SetActive,
    /// Standby all connected devices
    Standby,
    /// Increase volume
    VolumeUp {
        /// The logical address of the target device
        #[arg(default_value_t = LogicalAddress::Tv)]
        target: LogicalAddress,
    },
    /// Decrease volume
    VolumeDown {
        /// The logical address of the target device
        #[arg(default_value_t = LogicalAddress::Tv)]
        target: LogicalAddress,
    },
    /// Toggle mute
    Mute {
        /// The logical address of the target device
        #[arg(default_value_t = LogicalAddress::Tv)]
        target: LogicalAddress,
    },
    /// Send a specified user command key
    #[allow(clippy::enum_variant_names)]
    SendCommand {
        /// The name of the specified command key
        key: UiCommand,
        /// The logical address of the target device
        #[arg(default_value_t = LogicalAddress::Tv)]
        target: LogicalAddress,
    },
    /// Monitor CEC traffic
    Monitor {
        /// Monitor traffic to all destinations, not just broadcast and direct
        #[arg(short, long)]
        all: bool,
        /// Suppress printing polling messages
        #[arg(short = 'p', long)]
        suppress_poll: bool,
    },
}

fn main() -> Result<()> {
    let args = Arguments::parse();

    let mut dev = Device::open(args.device)?;

    match args.command {
        Command::GetConnectorInfo => {
            println!("Connector info: {:?}", dev.get_connector_info());
        }
        Command::GetPhysicalAddress => {
            println!("Physical address: {}", dev.get_physical_address()?);
        }
        Command::GetLogicalAddress => {
            for addr in dev.get_logical_addresses()? {
                println!("Logical address: {addr} ({:x})", addr as u8);
            }
        }
        Command::SetLogicalAddress { log_addr } => {
            dev.set_initiator_mode(InitiatorMode::Enabled)?;
            dev.set_logical_address(log_addr)?;
        }
        Command::ClearLogicalAddress => {
            dev.set_initiator_mode(InitiatorMode::Enabled)?;
            dev.clear_logical_addresses()?;
        }
        Command::SetOsdName { name } => {
            dev.set_initiator_mode(InitiatorMode::Enabled)?;
            let message = Message::SetOsdName {
                name: BufferOperand::from_str(&name)?,
            };
            dev.tx_message(&message, LogicalAddress::Tv)?;
        }
        Command::SetActive => {
            dev.set_initiator_mode(InitiatorMode::Enabled)?;
            let message = Message::RequestActiveSource {};
            dev.tx_message(&message, LogicalAddress::Tv)?;
        }
        Command::Standby => {
            dev.set_initiator_mode(InitiatorMode::Enabled)?;
            let message = Message::Standby {};
            dev.tx_message(&message, LogicalAddress::Tv)?;
        }
        Command::VolumeUp { target } => {
            dev.set_initiator_mode(InitiatorMode::Enabled)?;
            dev.press_user_control(UiCommand::VolumeUp, target)?;
            dev.release_user_control(target)?;
        }
        Command::VolumeDown { target } => {
            dev.set_initiator_mode(InitiatorMode::Enabled)?;
            dev.press_user_control(UiCommand::VolumeDown, target)?;
            dev.release_user_control(target)?;
        }
        Command::Mute { target } => {
            dev.set_initiator_mode(InitiatorMode::Enabled)?;
            dev.press_user_control(UiCommand::Mute, target)?;
            dev.release_user_control(target)?;
        }
        Command::SendCommand { key, target } => {
            dev.set_initiator_mode(InitiatorMode::Enabled)?;
            dev.press_user_control(key, target)?;
            dev.release_user_control(target)?;
        }
        Command::Monitor { all, suppress_poll } => {
            dev.set_initiator_mode(InitiatorMode::Disabled)?;
            dev.set_follower_mode(if all {
                FollowerMode::MonitorAll
            } else {
                FollowerMode::Monitor
            })?;
            loop {
                let message = dev.rx_raw_message(0)?;
                let bytes = &message.msg[1..message.len as usize];
                if suppress_poll && bytes.is_empty() {
                    continue;
                }
                let initiator = message.initiator();
                let destination = message.destination();
                println!(
                    "Message @ {}: {} ({:x}) -> {} ({:x})",
                    message.rx_ts,
                    LogicalAddress::try_from_primitive(initiator).unwrap(),
                    initiator,
                    if destination == 15 {
                        String::from("broadcast")
                    } else {
                        format!(
                            "{}",
                            LogicalAddress::try_from_primitive(destination).unwrap()
                        )
                    },
                    destination,
                );
                if bytes.is_empty() {
                    println!("  poll");
                } else {
                    println!("  raw: {bytes:?}");
                    if let Ok(decoded) = Message::try_from_bytes(bytes) {
                        println!("  decoded: {decoded:#?}");
                    } else {
                        println!("  decoding failed");
                    }
                }
            }
        }
    }
    Ok(())
}
