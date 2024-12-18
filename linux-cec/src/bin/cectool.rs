/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use clap::{Parser, Subcommand};
use linux_cec::device::Device;
use linux_cec::message::Message;
use linux_cec::operand::{BufferOperand, UiCommand};
use linux_cec::{InitiatorMode, LogicalAddress, Result};
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
        /// The desired logical address
        log_addr: LogicalAddress,
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
    SendCommand {
        /// The name of the specified command key
        key: UiCommand,
        /// The logical address of the target device
        #[arg(default_value_t = LogicalAddress::Tv)]
        target: LogicalAddress,
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
            println!("Physical address: {:04x}", dev.get_physical_address()?);
        }
        Command::GetLogicalAddress => {
            for addr in dev.get_logical_addresses()? {
                println!("Logical address: {addr} ({:x})", addr as u8);
            }
        }
        Command::SetLogicalAddress { log_addr } => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            dev.set_logical_address(log_addr)?;
        }
        Command::ClearLogicalAddress => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            dev.clear_logical_addresses()?;
        }
        Command::SetOsdName { name } => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            let message = Message::SetOsdName {
                name: BufferOperand::from_str(&name)?,
            };
            dev.tx_message(&message, LogicalAddress::Tv)?;
        }
        Command::SetActive => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            let message = Message::RequestActiveSource {};
            dev.tx_message(&message, LogicalAddress::Tv)?;
        }
        Command::Standby => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            let message = Message::Standby {};
            dev.tx_message(&message, LogicalAddress::Tv)?;
        }
        Command::VolumeUp { target } => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            dev.press_user_control(UiCommand::VolumeUp, target)?;
            dev.release_user_control(target)?;
        }
        Command::VolumeDown { target } => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            dev.press_user_control(UiCommand::VolumeDown, target)?;
            dev.release_user_control(target)?;
        }
        Command::Mute { target } => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            dev.press_user_control(UiCommand::Mute, target)?;
            dev.release_user_control(target)?;
        }
        Command::SendCommand { key, target } => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            dev.press_user_control(key, target)?;
            dev.release_user_control(target)?;
        }
    }
    Ok(())
}
