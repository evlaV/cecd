use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::signal::ctrl_c;
use tokio::sync::Mutex;
use tokio::task::LocalSet;
use tokio_util::sync::CancellationToken;
use tracing::debug;
use tracing_subscriber;
use zbus::connection::Builder;

use crate::config::read_default_config;
use crate::system::{System, SystemHandle};
use crate::udev::udev_hotplug;

pub(crate) mod config;
pub(crate) mod dbus;
pub(crate) mod system;
pub(crate) mod udev;
pub(crate) mod uinput;

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
    tracing_subscriber::fmt::init();

    let args = Arguments::parse();
    let connection = Builder::session()?
        .name("com.steampowered.CecDaemon1")?
        .build()
        .await?;

    let token = CancellationToken::new();
    let system = SystemHandle(Arc::new(Mutex::new(System::new(connection, token.clone()))));
    system.set_config(read_default_config().await?).await?;

    debug!("cecd starting up");
    debug!("OSD name: {}", system.osd_name().await);
    debug!(
        "Vendor ID: {}",
        match system.vendor_id().await {
            Some(x) => format!("{x}"),
            None => String::from("none"),
        }
    );

    if let Some(device) = args.device {
        system.find_dev(device).await?;
    } else {
        system.find_devs().await?;
    }

    let local = LocalSet::new();

    if args.allow_hotplug {
        let system = system.clone();
        let token = token.clone();
        local.spawn_local(udev_hotplug(system, token));
    }

    ctrl_c().await?;

    Ok(())
}
