use clap::{Parser, Subcommand};
use linux_cec::device::Device;
use linux_cec::message::{self, MessageEncodable};
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
    SetOsdName { name: String },
}

fn main() -> Result<()> {
    let args = Arguments::parse();

    let mut dev = Device::open(args.device)?;
    let log_addr = LogicalAddress::RecordingDevice1;

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
        Command::SetOsdName { name } => {
            dev.set_initiator(InitiatorMode::Enabled)?;
            let message = message::SetOsdName::from_str(&name)?;
            dev.tx_message(&message.to_message(), LogicalAddress::Tv)?;
        }
    }
    Ok(())
}
