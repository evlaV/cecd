use clap::{Parser, Subcommand};
use linux_cec::device::Device;
use linux_cec::message::{self, MessageEncodable};
use linux_cec::operand::UiCommand;
use linux_cec::{InitiatorMode, LogicalAddress, Result};
use num_enum::TryFromPrimitive;
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
    SetLogicalAddress { log_addr: u8 },
    SetOsdName { name: String },
    SetActive,
    Standby,
    SendKey { key: UiCommand },
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
                println!("Logical address: {:x}", addr as u8);
            }
        }
        Command::SetLogicalAddress { log_addr } => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            let log_addr = LogicalAddress::try_from_primitive(log_addr)?;
            dev.set_logical_address(log_addr)?;
        }
        Command::SetOsdName { name } => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            let message = message::SetOsdName::from_str(&name)?;
            dev.tx_message(&message.to_message(), LogicalAddress::Tv)?;
        }
        Command::SetActive => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            let message = message::RequestActiveSource {};
            dev.tx_message(&message.to_message(), LogicalAddress::Tv)?;
        }
        Command::Standby => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            let message = message::Standby {};
            dev.tx_message(&message.to_message(), LogicalAddress::Tv)?;
        }
        Command::SendKey { key } => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            let message = message::UserControlPressed { ui_command: key };
            dev.tx_message(&message.to_message(), LogicalAddress::BROADCAST)?;
            let message = message::UserControlReleased {};
            dev.tx_message(&message.to_message(), LogicalAddress::BROADCAST)?;
        }
    }
    Ok(())
}
