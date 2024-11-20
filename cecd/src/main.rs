use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::connection::Builder;

use crate::system::{System, SystemHandle};

pub(crate) mod dbus;
pub(crate) mod system;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    /// Which device to listen on. If parameter isn't specified, then cecd
    /// will attempt to detect the all available CEC devices in /dev.
    device: Option<String>,

    #[arg(short, long, default_value_t = true)]
    /// Enable hotplugging of CEC device. If enabled, the -d argument will be
    /// ignored and cecd will instead use the first available cec device if
    /// present, or wait for one to appear if not.
    allow_hotplug: bool,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Arguments::parse();
    let connection = Builder::session()?
        .name("com.steampowered.CecDaemon1")?
        .build()
        .await?;

    let system = SystemHandle(Arc::new(Mutex::new(System::new(connection))));

    if let Some(device) = args.device {
        system.find_dev(device).await?;
    } else {
        system.find_devs().await?;
    }

    if args.allow_hotplug {
        todo!();
    }

    todo!();

    Ok(())
}
