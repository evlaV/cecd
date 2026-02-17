/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::Result;
use clap::Parser;
#[cfg(not(test))]
use linux_cec::device::{AsyncDevice, AsyncDevicePoller};
use nix::time::{clock_gettime, ClockId};
use std::env;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::UnixDatagram;
use tokio::select;
use tokio::signal::ctrl_c;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::Mutex;
use tokio::task::{spawn, JoinHandle, JoinSet, LocalSet};
use tokio_util::sync::CancellationToken;
use tracing::{debug, trace, warn};
use zbus::connection::{Builder, Connection};

use crate::system::{ConfigTask, System, SystemHandle};
use crate::udev::udev_hotplug;

pub(crate) mod config;
pub(crate) mod dbus;
pub(crate) mod device;
pub(crate) mod message_handler;
pub(crate) mod system;
pub(crate) mod udev;
pub(crate) mod uinput;

#[cfg(test)]
pub mod testing;

#[cfg(test)]
pub use testing::{AsyncDevice, AsyncDevicePoller};

#[derive(Clone, Debug)]
#[repr(transparent)]
pub(crate) struct ArcDevice(Arc<Mutex<AsyncDevice>>);

impl ArcDevice {
    pub async fn open(path: impl AsRef<Path>) -> Result<ArcDevice> {
        Ok(ArcDevice(Arc::new(Mutex::new(
            AsyncDevice::open(&path).await?,
        ))))
    }
}

impl Deref for ArcDevice {
    type Target = Arc<Mutex<AsyncDevice>>;

    fn deref(&self) -> &Arc<Mutex<AsyncDevice>> {
        &self.0
    }
}

#[derive(Debug, Default)]
struct NotifySocket {
    socket: Option<UnixDatagram>,
}

impl NotifySocket {
    async fn setup_socket(&mut self) -> Result<()> {
        if self.socket.is_some() {
            return Ok(());
        }
        let Some(notify_socket) = env::var_os("NOTIFY_SOCKET") else {
            return Ok(());
        };
        let socket = UnixDatagram::unbound()?;
        socket.connect(notify_socket)?;
        self.socket = Some(socket);
        Ok(())
    }

    async fn notify(&mut self, message: &str) -> Result<()> {
        self.setup_socket()
            .await
            .inspect_err(|e| warn!("Couldn't set up systemd notify socket: {e}"))?;
        let Some(ref socket) = self.socket else {
            return Ok(());
        };
        trace!("Sending message to systemd: {message}");
        socket
            .send(message.as_bytes())
            .await
            .inspect_err(|e| warn!("Couldn't notify systemd: {e}"))?;
        Ok(())
    }

    async fn ready(&mut self) -> Result<()> {
        self.notify("READY=1\n").await
    }

    async fn begin_reload(&mut self) -> Result<()> {
        let timestamp = clock_gettime(ClockId::CLOCK_MONOTONIC)
            .inspect_err(|e| warn!("Couldn't get timestamp when notifying systemd: {e}"))?;
        let timestamp = timestamp.tv_sec() * 1_000_000 + timestamp.tv_nsec() / 1_000;
        let notifies = format!("RELOADING=1\nMONOTONIC_USEC={timestamp}\n");
        self.notify(notifies.as_str()).await?;
        Ok(())
    }
}

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

    debug!("cecd starting up");
    let args = Arguments::parse();
    let token = CancellationToken::new();
    let builder = Builder::session()?;
    let system_bus = Connection::system().await?;
    let system = SystemHandle(Arc::new(Mutex::new(
        System::new(token.clone(), builder, system_bus, args.config).await?,
    )));
    system.reconfig().await?;
    ConfigTask::start(system.clone()).await?;
    let system_task = {
        let mut system = system.clone();
        spawn(async move { system.run().await })
    };
    system.setup_dbus().await?;

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
        let devs = system.find_devs().await;
        let devs = if args.enable_hotplug {
            // If we're hotplugging, don't fail on error
            // here; we may get valid devices later
            devs.inspect_err(|err| warn!("Unable to get initial device list: {err}"))
                .unwrap_or_default()
        } else {
            devs?
        };
        for token in devs {
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
            let mut notify_socket = NotifySocket::default();
            loop {
                let mut sighup = signal(SignalKind::hangup())?;
                select! {
                    _ = sighup.recv() => (),
                    () = token.cancelled() => break,
                }
                let _ = notify_socket.begin_reload().await;
                system.reconfig().await?;
                let _ = notify_socket.ready().await;
            }
            Ok(())
        })
    };
    let _guard = token.drop_guard();

    let mut notify_socket = NotifySocket::default();
    let _ = notify_socket.ready().await;

    let mut sigterm = signal(SignalKind::terminate())?;
    select! {
        r = ctrl_c() => r?,
        _ = sigterm.recv() => (),
        () = local => (),
        r = config_reload => r??,
        r = system_task => r??,
    };
    Ok(())
}
