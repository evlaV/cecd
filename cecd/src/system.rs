/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::{ensure, Result};
use linux_cec::device::AsyncDevice;
use linux_cec::operand::VendorId;
use linux_cec::{FollowerMode, InitiatorMode, LogicalAddress};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::read_dir;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, WeakUnboundedSender};
use tokio::sync::{Mutex, MutexGuard};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use zbus::connection::{Builder, Connection};
use zbus::proxy;

use crate::config::Config;
use crate::dbus::CecDevice;
use crate::uinput::UInputDevice;

#[derive(Debug)]
pub(crate) struct System {
    osd_name: String,
    config: Config,

    connection: Connection,
    system_bus: Connection,
    token: CancellationToken,
    devs: HashMap<PathBuf, DeviceHandle>,
}

#[derive(Debug)]
struct DeviceHandle {
    token: CancellationToken,
    channel: WeakUnboundedSender<SystemMessage>,
}

#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait LoginManager {
    #[zbus(signal)]
    fn prepare_for_sleep(&self, sleep: bool) -> Result<()>;
}

#[derive(Debug)]
pub(crate) enum SystemMessage {
    Wake,
}

impl System {
    pub(crate) async fn new(token: CancellationToken) -> Result<System> {
        let connection = Builder::session()?
            .name("com.steampowered.CecDaemon1")?
            .build()
            .await?;

        let system_bus = Connection::system().await?;

        Ok(System {
            osd_name: String::from("CEC Device"),
            config: Config::default(),
            connection,
            system_bus,
            token,
            devs: HashMap::new(),
        })
    }

    async fn find_devs(&mut self) -> Result<Vec<(CecDevice, UnboundedReceiver<SystemMessage>)>> {
        let mut devs = Vec::new();
        let mut add = HashMap::new();
        let mut dir = read_dir("/dev").await?;
        while let Some(entry) = dir.next_entry().await? {
            let name = entry.file_name();
            if !name.to_string_lossy().starts_with("cec") {
                continue;
            }

            let path = entry.path();
            if self.devs.contains_key(&path) {
                continue;
            }

            let pathname = path.display();
            debug!("Scanning cec device {pathname}");

            let token = self.token.child_token();
            let (channel, rx) = unbounded_channel();
            devs.push((CecDevice::open(&path, token.clone()).await?, rx));
            info!("Found cec device at {pathname}");
            add.insert(
                path,
                DeviceHandle {
                    token,
                    channel: channel.downgrade(),
                },
            );
        }
        self.devs.extend(add);
        Ok(devs)
    }

    async fn find_dev(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<(CecDevice, UnboundedReceiver<SystemMessage>)> {
        let pathname = path.as_ref().display();
        debug!("Scanning cec device {pathname}");
        ensure!(
            !self.devs.contains_key(path.as_ref()),
            "Device {pathname} already loaded"
        );
        let token = self.token.child_token();
        let (channel, rx) = unbounded_channel();
        let dev = CecDevice::open(&path, token.clone()).await?;
        info!("Found cec device at {pathname}");
        self.devs.insert(
            path.as_ref().to_path_buf(),
            DeviceHandle {
                token,
                channel: channel.downgrade(),
            },
        );
        Ok((dev, rx))
    }

    pub(crate) fn close_dev(&mut self, path: impl AsRef<Path>) {
        if let Some(handle) = self.devs.remove(path.as_ref()) {
            handle.token.cancel();
        }
    }

    pub(crate) async fn set_config(&mut self, config: Config) -> Result<()> {
        if let Some(ref osd_name) = config.osd_name {
            self.osd_name = osd_name.clone();
        }
        self.config = config;

        let log_addr = if self.config.logical_address != LogicalAddress::UNREGISTERED {
            self.config.logical_address
        } else {
            LogicalAddress::PlaybackDevice1
        };
        debug!("OSD name: {}", self.osd_name);
        debug!("Logical address: {log_addr} ({:x})", log_addr as u8);
        debug!("Vendor ID: {:?}", self.config.vendor_id);

        Ok(())
    }

    pub(crate) async fn configure_dev(
        &self,
        device: Arc<Mutex<AsyncDevice>>,
    ) -> Result<Option<UInputDevice>> {
        let log_addr = if self.config.logical_address != LogicalAddress::UNREGISTERED {
            self.config.logical_address
        } else {
            LogicalAddress::PlaybackDevice1
        };

        let uinput = if !self.config.mappings.is_empty() && !self.config.disable_uinput {
            let mut uinput_dev = UInputDevice::new()?;
            uinput_dev.set_mappings(self.config.mappings.clone())?;
            uinput_dev.set_name(self.osd_name.clone())?;
            uinput_dev.open()?;
            Some(uinput_dev)
        } else {
            None
        };

        let device = device.lock().await;
        device.set_initiator(InitiatorMode::Enabled).await?;
        device.set_osd_name(&self.osd_name).await?;
        device.set_vendor_id(self.config.vendor_id).await?;
        device.set_logical_address(log_addr).await?;
        device.set_follower(FollowerMode::Enabled).await?;

        Ok(uinput)
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub(crate) struct SystemHandle(pub Arc<Mutex<System>>);

impl SystemHandle {
    pub(crate) async fn lock(&self) -> MutexGuard<System> {
        self.0.lock().await
    }

    pub(crate) async fn osd_name(&self) -> String {
        self.lock().await.osd_name.clone()
    }

    pub(crate) async fn vendor_id(&self) -> Option<VendorId> {
        self.lock().await.config.vendor_id
    }

    pub(crate) async fn find_devs(&self) -> Result<Vec<CancellationToken>> {
        let mut tokens = Vec::new();
        let devs;
        let connection;
        {
            let mut system = self.lock().await;
            devs = system.find_devs().await?;
            connection = system.connection.clone();
        }
        for (dev, rx) in devs {
            tokens.push(dev.token.clone());
            dev.register(connection.clone(), self.clone(), rx).await?;
        }
        Ok(tokens)
    }

    pub(crate) async fn find_dev(&self, path: impl AsRef<Path>) -> Result<CancellationToken> {
        let dev;
        let rx;
        let connection;
        {
            let mut system = self.lock().await;
            (dev, rx) = system.find_dev(path).await?;
            connection = system.connection.clone();
        }
        let token = dev.token.clone();
        dev.register(connection.clone(), self.clone(), rx).await?;
        Ok(token)
    }

    pub(crate) async fn close_dev(&self, path: impl AsRef<Path>) {
        let mut system = self.lock().await;
        system.close_dev(path);
    }

    pub(crate) async fn set_config(&self, config: Config) -> Result<()> {
        let mut system = self.lock().await;
        system.set_config(config).await
    }

    pub(crate) async fn run(&mut self) -> Result<()> {
        let login_manager = LoginManagerProxy::new(&self.lock().await.system_bus).await?;
        let mut sleep_stream = login_manager.receive_prepare_for_sleep().await?;
        loop {
            let Some(sleep_message) = sleep_stream.next().await else {
                continue;
            };
            let sleep = match sleep_message.args() {
                Ok(args) => args.sleep,
                Err(e) => {
                    warn!("Failed to receive PrepareForSleep message from logind: {e}");
                    continue;
                }
            };
            if !sleep && self.lock().await.config.wake_tv {
                self.lock().await.devs.retain(|_, dev| {
                    if let Some(channel) = dev.channel.upgrade() {
                        if let Ok(()) = channel.send(SystemMessage::Wake) {
                            return true;
                        }
                    }
                    dev.token.cancel();
                    false
                });
            }
        }
    }
}
