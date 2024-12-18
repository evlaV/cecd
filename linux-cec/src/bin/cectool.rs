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
#[command(version, about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    command: Command,

    #[arg(short, long, default_value_t = String::from("/dev/cec0"))]
    device: String,
}

#[derive(Subcommand)]
enum Command {
    GetConnectorInfo,
    GetPhysicalAddress,
    GetLogicalAddress,
    SetLogicalAddress { log_addr: LogicalAddress },
    ClearLogicalAddress,
    SetOsdName { name: String },
    SetActive,
    Standby,
    VolumeUp {
        #[arg(default_value_t = LogicalAddress::Tv)]
        target: LogicalAddress,
    },
    VolumeDown {
        #[arg(default_value_t = LogicalAddress::Tv)]
        target: LogicalAddress,
    },
    Mute {
        #[arg(default_value_t = LogicalAddress::Tv)]
        target: LogicalAddress,
    },
    SendKey {
        key: UiCommand,
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
        Command::SendKey { key, target } => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            dev.press_user_control(key, target)?;
            dev.release_user_control(target)?;
        }
    }
    Ok(())
}
