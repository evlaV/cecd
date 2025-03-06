/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::{ensure, Result};
use input_linux::Key;
use linux_cec::device::Capabilities;
use linux_cec::operand::UiCommand;
use linux_cec::{FollowerMode, InitiatorMode, LogicalAddressType, VendorId};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::read_dir;
use tokio::sync::broadcast::{channel, Sender};
use tokio::sync::{Mutex, MutexGuard};
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use zbus::connection::{Builder, Connection};
use zbus::proxy;

use crate::config::Config;
use crate::dbus::CecDevice;
use crate::uinput::UInputDevice;
use crate::ArcDevice;

#[derive(Debug)]
pub(crate) struct System {
    osd_name: String,
    pub config: Config,

    pub connection: Connection,
    pub channel: Sender<SystemMessage>,
    system_bus: Connection,
    token: CancellationToken,
    devs: HashMap<PathBuf, CancellationToken>,
}

#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait LoginManager {
    #[zbus(signal)]
    fn prepare_for_sleep(&self, sleep: bool) -> Result<()>;

    fn suspend(&self, interactive: bool) -> Result<()>;
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum SystemMessage {
    Wake,
    Standby,
    ReloadConfig,
}

impl System {
    // Most of these mappings match Linux's rc mapping, but a few are intentionally
    // changed or removed in an opinionated way. These are just the defaults however,
    // so they are easily overridden or unmapped if desired.
    const DEFAULT_MAPPINGS: &[(UiCommand, Key)] = &[
        (UiCommand::Select, Key::Enter),
        (UiCommand::Up, Key::Up),
        (UiCommand::Down, Key::Down),
        (UiCommand::Left, Key::Left),
        (UiCommand::Right, Key::Right),
        (UiCommand::RightUp, Key::RightUp),
        (UiCommand::RightDown, Key::RightDown),
        (UiCommand::LeftUp, Key::LeftUp),
        (UiCommand::LeftDown, Key::LeftDown),
        (UiCommand::DeviceRootMenu, Key::RootMenu),
        (UiCommand::DeviceSetupMenu, Key::Setup),
        (UiCommand::ContentsMenu, Key::Menu),
        (UiCommand::FavoriteMenu, Key::Favorites),
        (UiCommand::Back, Key::Esc),
        (UiCommand::MediaTopMenu, Key::MediaTopMenu),
        (UiCommand::MediaContextSensitiveMenu, Key::ContextMenu),
        (UiCommand::NumberEntryMode, Key::Digits),
        (UiCommand::Number11, Key::Numeric11),
        (UiCommand::Number12, Key::Numeric12),
        (UiCommand::Number0OrNumber10, Key::Num0),
        (UiCommand::Number1, Key::Num1),
        (UiCommand::Number2, Key::Num2),
        (UiCommand::Number3, Key::Num3),
        (UiCommand::Number4, Key::Num4),
        (UiCommand::Number5, Key::Num5),
        (UiCommand::Number6, Key::Num6),
        (UiCommand::Number7, Key::Num7),
        (UiCommand::Number8, Key::Num8),
        (UiCommand::Number9, Key::Num9),
        (UiCommand::Dot, Key::Dot),
        (UiCommand::Enter, Key::Enter),
        (UiCommand::Clear, Key::Clear),
        (UiCommand::NextFavorite, Key::NextFavorite),
        (UiCommand::ChannelUp, Key::ChannelUp),
        (UiCommand::ChannelDown, Key::ChannelDown),
        (UiCommand::PreviousChannel, Key::Previous),
        (UiCommand::SoundSelect, Key::Sound),
        // UiCommand::InputSelect, no good mapping
        (UiCommand::DisplayInformation, Key::Info),
        (UiCommand::Help, Key::Help),
        (UiCommand::PageUp, Key::PageUp),
        (UiCommand::PageDown, Key::PageDown),
        (UiCommand::Power, Key::Power),
        (UiCommand::VolumeUp, Key::VolumeUp),
        (UiCommand::VolumeDown, Key::VolumeDown),
        (UiCommand::Mute, Key::Mute),
        (UiCommand::Play, Key::PlayCD),
        (UiCommand::Stop, Key::StopCD),
        (UiCommand::Pause, Key::PauseCD),
        (UiCommand::Record, Key::Record),
        (UiCommand::Rewind, Key::Rewind),
        (UiCommand::FastForward, Key::FastForward),
        (UiCommand::Eject, Key::EjectCD),
        (UiCommand::SkipForward, Key::NextSong),
        (UiCommand::SkipBackward, Key::PreviousSong),
        (UiCommand::StopRecord, Key::StopRecord),
        (UiCommand::PauseRecord, Key::PauseRecord),
        (UiCommand::Angle, Key::Angle),
        // UiCommand::SubPicture, no good mapping
        (UiCommand::VideoOnDemand, Key::Vod),
        (UiCommand::ElectronicProgramGuide, Key::EPG),
        (UiCommand::TimerProgramming, Key::Time),
        (UiCommand::InitialConfiguration, Key::Config),
        // UiCommand::SelectBroadcastType, no good mapping
        // UiCommand::SelectSoundPresentation, no good mapping
        (UiCommand::AudioDescription, Key::AudioDesc),
        (UiCommand::Internet, Key::WWW),
        (UiCommand::ThreeDMode, Key::Audio3dMode),
        // The "function" keys are intended for operations that have a well-
        // defined end state, e.g. "mute function" does not toggle mute, it
        // specifically enables mute. Since the majority aren't cleanly
        // mappable, these are left unmapped instead of having the possibility
        // of doing the wrong operation.
        // UiCommand::PlayFunction
        // UiCommand::PausePlayFunction
        // UiCommand::RecordFunction
        // UiCommand::PauseRecordFunction
        // UiCommand::StopFunction
        // UiCommand::MuteFunction
        (UiCommand::RestoreVolumeFunction, Key::Unmute),
        // UiCommand::TuneFunction
        // UiCommand::SelectMediaFunction
        // UiCommand::SelectAvInputFunction
        // UiCommand::SelectAudioInputFunction
        (UiCommand::PowerToggleFunction, Key::Power),
        (UiCommand::PowerOffFunction, Key::Sleep),
        (UiCommand::PowerOnFunction, Key::Wakeup),
        (UiCommand::F1Blue, Key::Blue),
        (UiCommand::F2Red, Key::Red),
        (UiCommand::F3Green, Key::Green),
        (UiCommand::F4Yellow, Key::Yellow),
        (UiCommand::F5, Key::F5),
        (UiCommand::Data, Key::Data),
    ];

    pub(crate) async fn new(
        token: CancellationToken,
        builder: Builder<'_>,
        system_bus: Connection,
    ) -> Result<System> {
        let connection = builder.name("com.steampowered.CecDaemon1")?.build().await?;
        let (channel, _) = channel(10);

        Ok(System {
            osd_name: String::from("CEC Device"),
            config: Config::default(),
            connection,
            system_bus,
            token,
            devs: HashMap::new(),
            channel,
        })
    }

    pub(crate) async fn find_devs(&mut self) -> Result<Vec<CecDevice>> {
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
            devs.push(CecDevice::open(&path, token.clone()).await?);
            info!("Found cec device at {pathname}");
            add.insert(path, token);
        }
        self.devs.extend(add);
        Ok(devs)
    }

    pub(crate) async fn find_dev(&mut self, path: impl AsRef<Path>) -> Result<CecDevice> {
        let pathname = path.as_ref().display();
        debug!("Scanning cec device {pathname}");
        ensure!(
            !self.devs.contains_key(path.as_ref()),
            "Device {pathname} already loaded"
        );
        let token = self.token.child_token();
        let dev = CecDevice::open(&path, token.clone()).await?;
        info!("Found cec device at {pathname}");
        self.devs.insert(path.as_ref().to_path_buf(), token);
        Ok(dev)
    }

    pub(crate) fn close_dev(&mut self, path: impl AsRef<Path>) {
        if let Some(token) = self.devs.remove(path.as_ref()) {
            token.cancel();
        }
    }

    pub(crate) async fn set_config(&mut self, config: Config) -> Result<()> {
        if let Some(ref osd_name) = config.osd_name {
            self.osd_name = osd_name.clone();
        }
        self.config = config;

        if self.config.logical_address == LogicalAddressType::Unregistered {
            self.config.logical_address = LogicalAddressType::Playback;
        }

        if self.config.mappings.is_empty() {
            self.config.mappings = HashMap::from_iter(System::DEFAULT_MAPPINGS.iter().copied());
        }

        debug!("Configuration loaded: {:#?}", self.config);

        self.send_message(SystemMessage::ReloadConfig).await
    }

    pub(crate) async fn configure_dev(&self, device: ArcDevice) -> Result<Option<UInputDevice>> {
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
        device.set_initiator_mode(InitiatorMode::Enabled).await?;
        let caps = device.get_capabilities().await?;
        debug!("Device has caps: {caps:?}");
        if caps.contains(Capabilities::LOG_ADDRS) {
            device.clear_logical_addresses().await?;
            device.set_osd_name(&self.osd_name).await?;
            device.set_vendor_id(self.config.vendor_id).await?;
            device
                .set_logical_address(self.config.logical_address)
                .await?;
        }
        device.set_follower_mode(FollowerMode::Enabled).await?;

        Ok(uinput)
    }

    async fn send_message(&mut self, message: SystemMessage) -> Result<()> {
        if !self.devs.is_empty() {
            self.channel.send(message)?;
        }
        Ok(())
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
        for dev in devs {
            tokens.push(dev.token.clone());
            dev.register(
                connection.clone(),
                self.clone(),
                self.lock().await.channel.clone(),
            )
            .await?;
        }
        Ok(tokens)
    }

    pub(crate) async fn find_dev(&self, path: impl AsRef<Path>) -> Result<CancellationToken> {
        let dev;
        let channel;
        let connection;
        {
            let mut system = self.lock().await;
            dev = system.find_dev(path).await?;
            channel = system.channel.clone();
            connection = system.connection.clone();
        }
        let token = dev.token.clone();
        dev.register(connection.clone(), self.clone(), channel)
            .await?;
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
                self.lock().await.send_message(SystemMessage::Wake).await?;
            } else if sleep && self.lock().await.config.suspend_tv {
                self.lock()
                    .await
                    .send_message(SystemMessage::Standby)
                    .await?;
            }
        }
    }

    pub(crate) async fn suspend(&self) -> Result<()> {
        let login_manager = LoginManagerProxy::new(&self.lock().await.system_bus).await?;
        login_manager.suspend(false).await
    }
}
