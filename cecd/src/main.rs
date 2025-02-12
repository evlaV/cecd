/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::Result;
use clap::Parser;
#[cfg(not(test))]
use linux_cec::device::AsyncDevice;
#[cfg(not(test))]
use std::ops::Deref;
#[cfg(not(test))]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::select;
use tokio::signal::ctrl_c;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::Mutex;
use tokio::task::{spawn, JoinHandle, JoinSet, LocalSet};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

use crate::config::{read_config_file, read_default_config};
use crate::system::{System, SystemHandle};
use crate::udev::udev_hotplug;

pub(crate) mod config;
pub(crate) mod dbus;
pub(crate) mod device;
pub(crate) mod system;
pub(crate) mod udev;
pub(crate) mod uinput;

#[cfg(test)]
pub mod testing;

#[cfg(not(test))]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub(crate) struct ArcDevice(Arc<Mutex<AsyncDevice>>);

#[cfg(not(test))]
impl ArcDevice {
    pub async fn open(path: impl AsRef<Path>) -> Result<ArcDevice> {
        Ok(ArcDevice(Arc::new(Mutex::new(
            AsyncDevice::open(&path).await?,
        ))))
    }
}

#[cfg(not(test))]
impl Deref for ArcDevice {
    type Target = Arc<Mutex<AsyncDevice>>;

    fn deref(&self) -> &Arc<Mutex<AsyncDevice>> {
        &self.0
    }
}

#[cfg(test)]
pub use testing::ArcDevice;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    /// Which device to listen on. If parameter isn't specified, then cecd
    /// will attempt to detect the all available CEC devices in /dev.
    device: Option<PathBuf>,

    #[arg(short, long)]
    /// Enable hotplugging of CEC device. If enabled, the -d argument will be
    /// ignored and cecd will instead use the first available cec device if
    /// present, or wait for one to appear if not.
    enable_hotplug: bool,

    #[arg(short, long)]
    /// Override the default configuration paths and use a custom config file.
    config: Option<PathBuf>,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Arguments::parse();
    let token = CancellationToken::new();
    let system = SystemHandle(Arc::new(Mutex::new(System::new(token.clone()).await?)));
    let config = if let Some(ref config_path) = args.config {
        read_config_file(config_path).await?
    } else {
        read_default_config().await?
    };
    system.set_config(config).await?;
    let system_task = {
        let mut system = system.clone();
        spawn(async move { system.run().await })
    };

    debug!("cecd starting up");
    debug!("OSD name: {}", system.osd_name().await);
    debug!(
        "Vendor ID: {}",
        match system.vendor_id().await {
            Some(x) => format!("{x}"),
            None => String::from("none"),
        }
    );

    let mut joinset = JoinSet::new();
    if let Some(device) = args.device {
        let token = system.find_dev(device).await?;
        joinset.spawn(async move {
            token.cancelled().await;
        });
    } else {
        for token in system.find_devs().await? {
            joinset.spawn(async move {
                token.cancelled().await;
            });
        }
    }

    let local = LocalSet::new();

    if args.enable_hotplug {
        let system = system.clone();
        let token = token.clone();
        local.spawn_local(udev_hotplug(system, token));
    } else {
        if joinset.is_empty() {
            warn!("No devices found");
            return Ok(());
        }
        local.spawn_local(async move {
            joinset.join_all().await;
        });
    }

    let config_reload: JoinHandle<Result<()>> = {
        let token = token.clone();
        spawn(async move {
            loop {
                let mut sighup = signal(SignalKind::hangup())?;
                select! {
                    _ = sighup.recv() => (),
                    _ = token.cancelled() => break,
                }
                let config = if let Some(ref config_path) = args.config {
                    read_config_file(config_path).await?
                } else {
                    read_default_config().await?
                };
                system.set_config(config).await?;
            }
            Ok(())
        })
    };
    let _guard = token.drop_guard();

    let mut sigterm = signal(SignalKind::terminate())?;
    select! {
        r = ctrl_c() => r?,
        _ = sigterm.recv() => (),
        _ = local => (),
        r = config_reload => r??,
        r = system_task => r??,
    };
    Ok(())
}
