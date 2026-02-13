/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::{ensure, Result};
use input_linux::Key;
use linux_cec::device::Capabilities;
use linux_cec::operand::UiCommand;
use linux_cec::{self, FollowerMode, InitiatorMode, LogicalAddressType, PhysicalAddress, VendorId};
use nix::errno::Errno;
use nix::unistd::gethostname;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::{read, read_dir, read_to_string};
use tokio::spawn;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio::sync::{Mutex, MutexGuard};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use zbus::connection::{Builder, Connection};
use zbus::fdo::ObjectManager;
use zbus::names::UniqueName;
use zbus::proxy;
use zbus::zvariant::ObjectPath;

use crate::config::{read_config_file, read_default_config, Config};
use crate::dbus::{CecConfig, CecDevice, Daemon, PATH};
use crate::message_handler::{MessageHandler, MessageHandlerTask};
use crate::uinput::UInputDevice;
use crate::ArcDevice;

#[derive(Debug)]
pub(crate) struct System {
    osd_name: String,
    pub config: Config,
    config_path: Option<PathBuf>,

    pub connection: Connection,
    pub channel: Sender<SystemMessage>,
    system_bus: Connection,
    token: CancellationToken,
    devs: HashMap<PathBuf, CancellationToken>,

    message_handlers: HashMap<u8, MessageHandlerHandle>,
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
    Standby { standby_tv: bool },
    ReloadConfig,
}

impl System {
    // Most of these mappings match Linux's rc mapping, but a few are intentionally
    // changed or removed in an opinionated way. These are just the defaults however,
    // so they are easily overridden or unmapped if desired.
    pub(crate) const DEFAULT_MAPPINGS: &[(UiCommand, Key)] = &[
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
        config_path: Option<PathBuf>,
    ) -> Result<System> {
        let connection = builder.name("com.steampowered.CecDaemon1")?.build().await?;
        let (channel, _) = channel(10);

        let hostname = gethostname()
            .ok()
            .and_then(|hostname| hostname.into_string().ok());

        let osd_name = match hostname {
            Some(hostname) if !hostname.is_empty() => hostname,
            _ => String::from("CEC Device"),
        };

        Ok(System {
            osd_name,
            config: Config::default(),
            connection,
            system_bus,
            token,
            devs: HashMap::new(),
            channel,
            config_path,
            message_handlers: HashMap::new(),
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

    pub(crate) async fn reconfig(&mut self) -> Result<()> {
        let config = if let Some(ref config_path) = self.config_path {
            read_config_file(config_path).await?
        } else {
            read_default_config().await?
        };
        self.set_config(config).await
    }

    pub(crate) async fn set_config(&mut self, config: Config) -> Result<()> {
        if let Some(ref osd_name) = config.osd_name {
            self.osd_name.clone_from(osd_name);
        }
        self.config = config;

        if self.config.logical_address == LogicalAddressType::Unregistered {
            self.config.logical_address = LogicalAddressType::Playback;
        }

        if self.config.osd_name.is_none() {
            let hostname = gethostname()
                .ok()
                .and_then(|hostname| hostname.into_string().ok());

            self.config.osd_name = Some(match hostname {
                Some(hostname) if !hostname.is_empty() => hostname,
                _ => String::from("CEC Device"),
            });
        }

        if self.config.mappings.is_empty() {
            self.config.mappings = System::DEFAULT_MAPPINGS.iter().copied().collect();
        }

        debug!("Configuration loaded: {:#?}", self.config);

        self.send_message(SystemMessage::ReloadConfig).await;
        Ok(())
    }

    pub(crate) fn subscribe(&self) -> Receiver<SystemMessage> {
        self.channel.subscribe()
    }

    fn trimmed_osd_name(&self) -> &str {
        if self.osd_name.len() <= 14 {
            return self.osd_name.as_str();
        }
        // TODO: Simplify using floor_char_boundary when we can bump the minimum rust ver to 1.91
        for i in (10..=14).rev() {
            if self.osd_name.is_char_boundary(i) {
                return &self.osd_name.as_str()[..i];
            }
        }
        unreachable!();
    }

    pub(crate) async fn configure_dev(&self, device: ArcDevice) -> Result<Option<UInputDevice>> {
        let uinput = if !self.config.mappings.is_empty() && self.config.uinput {
            match UInputDevice::new() {
                Ok(mut uinput_dev) => {
                    uinput_dev.set_mappings(self.config.mappings.clone())?;
                    uinput_dev.set_name(self.osd_name.clone())?;
                    uinput_dev.open()?;
                    Some(uinput_dev)
                }
                Err(e) => {
                    warn!("Failed to open uinput device: {e}");
                    None
                }
            }
        } else {
            None
        };

        let device = device.lock().await;
        device.set_initiator_mode(InitiatorMode::Enabled).await?;
        let caps = device.get_capabilities().await?;
        debug!("Device has caps: {caps:?}");
        if caps.contains(Capabilities::PHYS_ADDR) {
            if let Ok(Some(physical_address)) = System::find_pa().await {
                debug!("Found physical address {physical_address} in EDID");
                device.set_physical_address(physical_address).await?;
            } else if let Some(physical_address) = self.config.physical_address {
                debug!(
                    "Physical address required but not found, using fallback {}",
                    physical_address
                );
                device.set_physical_address(physical_address).await?;
            } else {
                warn!("Couldn't determine physical address");
                device
                    .set_physical_address(PhysicalAddress::default())
                    .await?;
            }
        }
        if caps.contains(Capabilities::LOG_ADDRS) {
            for i in 0..5 {
                match device.clear_logical_addresses().await {
                    Err(linux_cec::Error::SystemError(e)) if e == Errno::EBUSY => {
                        if i == 4 {
                            return Err(e.into());
                        }
                        sleep(Duration::from_millis(200)).await
                    }
                    Err(e) => return Err(e.into()),
                    Ok(()) => break,
                }
            }
            device.set_osd_name(self.trimmed_osd_name()).await?;
            device.set_vendor_id(self.config.vendor_id).await?;
            device
                .set_logical_address(self.config.logical_address)
                .await?;
        }
        device.set_follower_mode(FollowerMode::Enabled).await?;

        Ok(uinput)
    }

    async fn send_message(&mut self, message: SystemMessage) {
        // This is allowed to fail silently
        let _ = self.channel.send(message);
    }

    fn parse_hdmi_edid_pa(edid: &[u8]) -> Option<PhysicalAddress> {
        const HDMI_OUI: &[u8] = &[0x03, 0x0C, 0x00];
        let mut block = 128;
        // We ignore a lot of the spec and let through a bunch of bad EDIDs.
        // As it turns out, a lot of vendors ship bad EDIDs. We want to be as
        // permissive as possible without misparsing things that are
        // definitively what we're looking for.
        while block + 128 <= edid.len() {
            if edid[block] != 2 && edid[block + 1] < 3 {
                // Requires CTA EDID version 3 or newer
                block += 128;
                continue;
            }
            let end = block
                + (if edid[block + 2] >= 4 {
                    usize::min(edid[block + 2] as usize, 127)
                } else {
                    127
                });
            let mut offset = block + 4;
            while offset + 1 < edid.len() && offset < end {
                let header = edid[offset];
                let size = ((header & 0x1F) + 1) as usize;
                if size < 6 {
                    offset += size;
                    continue;
                }
                if offset + size > end {
                    break;
                }
                if (header & 0xE0) != 0x60 {
                    // Not a vendor specific data block
                    offset += size;
                    continue;
                }
                if &edid[offset + 1..offset + 4] != HDMI_OUI {
                    // Not an HDMI block
                    offset += size;
                    continue;
                }
                return Some(PhysicalAddress::from(
                    ((edid[offset + 4] as u16) << 8) | edid[offset + 5] as u16,
                ));
            }
            block += 128;
        }
        None
    }

    async fn find_pa() -> Result<Option<PhysicalAddress>> {
        let mut pa = None;
        let mut dir = read_dir("/sys/class/drm").await?;
        while let Some(entry) = dir.next_entry().await? {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if !file_name.starts_with("card") && !file_name.contains('-') {
                continue;
            }
            let status = match read_to_string(entry.path().join("status")).await {
                Ok(status) => status,
                Err(e) if e.kind() == ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            };
            if status.trim() != "connected" {
                continue;
            }
            let edid = match read(entry.path().join("edid")).await {
                Ok(edid) => edid,
                Err(e) if e.kind() == ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            };
            let Some(this_pa) = System::parse_hdmi_edid_pa(&edid) else {
                continue;
            };
            if pa.is_some() {
                debug!("Found multiple connected monitors with physical addresses");
                return Ok(None);
            }
            pa = Some(this_pa);
        }
        Ok(pa)
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub(crate) struct SystemHandle(pub Arc<Mutex<System>>);

impl SystemHandle {
    pub(crate) async fn lock(&self) -> MutexGuard<'_, System> {
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
            dev.register(connection.clone(), self.clone()).await?;
        }
        Ok(tokens)
    }

    pub(crate) async fn find_dev(&self, path: impl AsRef<Path>) -> Result<CancellationToken> {
        let dev;
        let connection;
        {
            let mut system = self.lock().await;
            dev = system.find_dev(path).await?;
            connection = system.connection.clone();
        }
        let token = dev.token.clone();
        dev.register(connection.clone(), self.clone()).await?;
        Ok(token)
    }

    pub(crate) async fn close_dev(&self, path: impl AsRef<Path>) {
        let mut system = self.lock().await;
        system.close_dev(path);
    }

    pub(crate) async fn reconfig(&self) -> Result<()> {
        let mut system = self.lock().await;
        system.reconfig().await
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
            let mut system = self.lock().await;
            if !sleep && system.config.wake_tv {
                system.send_message(SystemMessage::Wake).await;
            } else if sleep {
                let standby_tv = system.config.suspend_tv;
                system
                    .send_message(SystemMessage::Standby { standby_tv })
                    .await;
            }
        }
    }

    pub(crate) async fn suspend(&self) -> Result<()> {
        let login_manager = LoginManagerProxy::new(&self.lock().await.system_bus).await?;
        login_manager.suspend(false).await
    }

    pub(crate) async fn setup_dbus(&self) -> Result<()> {
        // We can't have the lock held while we install
        // the ObjectManager to avoid a deadlock
        let connection = self.lock().await.connection.clone();
        let object_server = connection.object_server();

        let daemon_obj = Daemon::new(self);
        object_server
            .at(format!("{PATH}/Daemon"), daemon_obj)
            .await?;

        object_server.at(PATH, ObjectManager {}).await?;
        Ok(())
    }

    pub(crate) async fn get_message_handler(&self, opcode: u8) -> Option<MessageHandler<'_>> {
        let system = self.lock().await;
        system
            .message_handlers
            .get(&opcode)
            .map(|handle| handle.handler.clone())
    }

    pub(crate) async fn register_message_handler(
        &self,
        opcode: impl Into<u8>,
        object: ObjectPath<'_>,
        bus_name: &UniqueName<'_>,
    ) -> Result<bool> {
        let mut system = self.lock().await;
        let opcode = opcode.into();
        if system.message_handlers.contains_key(&opcode) {
            return Ok(false);
        }
        let task =
            MessageHandlerTask::start(&system.connection, self.clone(), opcode, bus_name).await?;
        let handler = MessageHandler::new(&system.connection, &object, bus_name).await?;
        system
            .message_handlers
            .insert(opcode, MessageHandlerHandle { handler, task });
        Ok(true)
    }

    pub(crate) async fn unregister_message_handler(
        &self,
        opcode: impl Into<u8>,
        bus_name: &UniqueName<'_>,
    ) -> Result<bool> {
        let mut system = self.lock().await;
        let opcode = opcode.into();
        let entry = system.message_handlers.entry(opcode);
        let handle = match entry {
            Entry::Occupied(entry) if entry.get().handler.is_name(bus_name) => entry.remove(),
            _ => return Ok(false),
        };
        handle.task.abort();
        Ok(true)
    }

    pub(crate) async fn list_handled_messages(&self) -> HashSet<u8> {
        self.lock().await.message_handlers.keys().copied().collect()
    }
}

#[derive(Debug)]
pub struct ConfigTask {
    channel: Receiver<SystemMessage>,
    connection: Connection,
}

impl ConfigTask {
    pub async fn start(system: SystemHandle) -> Result<JoinHandle<Result<()>>> {
        let channel;
        let connection;
        {
            let system = system.lock().await;
            channel = system.subscribe();
            connection = system.connection.clone();
        }
        let config_obj = CecConfig::new(system).await;
        connection
            .object_server()
            .at(format!("{PATH}/Daemon"), config_obj)
            .await?;
        let task = ConfigTask {
            channel,
            connection,
        };
        Ok(spawn(task.run()))
    }

    async fn run(mut self) -> Result<()> {
        let config_obj = self
            .connection
            .object_server()
            .interface::<_, CecConfig>(format!("{PATH}/Daemon"))
            .await?;
        loop {
            let message = match self.channel.recv().await {
                Ok(message) => message,
                Err(e) => {
                    warn!("Error receiving message: {e}");
                    return Err(e.into());
                }
            };
            if !matches!(message, SystemMessage::ReloadConfig) {
                continue;
            }
            let emitter = config_obj.signal_emitter();
            config_obj.get_mut().await.reconfigure(emitter).await;
        }
    }
}

#[derive(Debug)]
pub struct MessageHandlerHandle {
    handler: MessageHandler<'static>,
    task: JoinHandle<Result<()>>,
}

#[cfg(test)]
mod test {
    use super::*;

    use linux_cec::device::{Envelope, MessageData};
    use linux_cec::message::{Message, Opcode};
    use linux_cec::operand::AbortReason;
    use linux_cec::{LogicalAddress, PhysicalAddress};
    use std::iter::repeat_n;
    use std::time::Duration;
    use tokio::select;
    use tokio::time::sleep;
    use zbus::zvariant::OwnedObjectPath;
    use zbus::{fdo, interface};

    use crate::testing::setup_dbus_test;

    #[test]
    fn test_parse_edid_missing() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 80
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(System::parse_hdmi_edid_pa(&edid), None);
    }

    #[test]
    fn test_parse_edid_missing_header() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x00, 0x00, 0x00, 0x00, 0x65, 0x03, 0x0c, 0x00, // 80
            0x12, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(System::parse_hdmi_edid_pa(&edid), None);
    }

    #[test]
    fn test_parse_edid_early() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x00, 0x00, 0x65, 0x03, 0x0c, 0x00, // 80
            0x12, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(
            System::parse_hdmi_edid_pa(&edid),
            Some(PhysicalAddress::from(0x1234))
        );
    }

    #[test]
    fn test_parse_edid_extra_block() {
        let header = repeat_n(0u8, 0x100);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x00, 0x00, 0x65, 0x03, 0x0c, 0x00, // 80
            0x12, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(
            System::parse_hdmi_edid_pa(&edid),
            Some(PhysicalAddress::from(0x1234))
        );
    }

    #[test]
    fn test_parse_edid_island() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 80
            0x65, 0x03, 0x0c, 0x00, 0x12, 0x34, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(
            System::parse_hdmi_edid_pa(&edid),
            Some(PhysicalAddress::from(0x1234))
        );
    }

    #[test]
    fn test_parse_edid_skipped() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x00, 0x00, 0x6b, 0x00, 0x00, 0x00, // 80
            0x65, 0x03, 0x0c, 0x00, 0x12, 0x34, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(System::parse_hdmi_edid_pa(&edid), None);
    }

    #[test]
    fn test_parse_edid_cutoff_0() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x04, 0x00, 0x65, 0x03, 0x0c, 0x00, // 80
            0x12, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(System::parse_hdmi_edid_pa(&edid), None);
    }

    #[test]
    fn test_parse_edid_cutoff_1() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x05, 0x00, 0x65, 0x03, 0x0c, 0x00, // 80
            0x12, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(System::parse_hdmi_edid_pa(&edid), None);
    }

    #[test]
    fn test_parse_edid_cutoff_2() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x06, 0x00, 0x65, 0x03, 0x0c, 0x00, // 80
            0x12, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(System::parse_hdmi_edid_pa(&edid), None);
    }

    #[test]
    fn test_parse_edid_cutoff_3() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x07, 0x00, 0x65, 0x03, 0x0c, 0x00, // 80
            0x12, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(System::parse_hdmi_edid_pa(&edid), None);
    }

    #[test]
    fn test_parse_edid_cutoff_4() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x08, 0x00, 0x65, 0x03, 0x0c, 0x00, // 80
            0x12, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(System::parse_hdmi_edid_pa(&edid), None);
    }

    #[test]
    fn test_parse_edid_cutoff_5() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x09, 0x00, 0x65, 0x03, 0x0c, 0x00, // 80
            0x12, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(System::parse_hdmi_edid_pa(&edid), None);
    }

    #[test]
    fn test_parse_edid_cutoff_6() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x0a, 0x00, 0x65, 0x03, 0x0c, 0x00, // 80
            0x12, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(
            System::parse_hdmi_edid_pa(&edid),
            Some(PhysicalAddress::from(0x1234))
        );
    }

    #[test]
    fn test_parse_edid_cutoff_7() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x0b, 0x00, 0x65, 0x03, 0x0c, 0x00, // 80
            0x12, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(
            System::parse_hdmi_edid_pa(&edid),
            Some(PhysicalAddress::from(0x1234))
        );
    }

    #[test]
    fn test_parse_edid_cutoff_end() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x7f, 0x00, 0x00, 0x00, 0x00, 0x00, // 80
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x65, 0x03, 0x0c, 0x00, 0x12, 0x34, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(System::parse_hdmi_edid_pa(&edid), None);
    }

    #[test]
    fn test_parse_edid_cutoff_invalid_len() {
        let header = repeat_n(0u8, 0x80);
        let edid: [u8; 0x80] = [
            0x02, 0x03, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, // 80
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 88
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 90
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 98
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E8
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F0
            0x00, 0x00, 0x65, 0x03, 0x0c, 0x00, 0x12, 0x34, // F8
        ];
        let edid: Vec<u8> = header.into_iter().chain(edid.into_iter()).collect();
        assert_eq!(System::parse_hdmi_edid_pa(&edid), None);
    }

    #[derive(Debug, PartialEq)]
    struct RemoteMessage {
        device: OwnedObjectPath,
        initiator: u8,
        destination: u8,
        timestamp: u64,
        message: Vec<u8>,
    }

    #[derive(Debug)]
    struct MockMessageHandler {
        last_message: Option<RemoteMessage>,
        abort: Option<AbortReason>,
        channel: Sender<()>,
        timeout: bool,
    }

    impl MockMessageHandler {
        fn new() -> MockMessageHandler {
            MockMessageHandler {
                last_message: None,
                abort: None,
                channel: Sender::new(2),
                timeout: false,
            }
        }
    }

    #[interface(name = "com.steampowered.CecDaemon1.MessageHandler1")]
    impl MockMessageHandler {
        async fn handle_message(
            &mut self,
            device: OwnedObjectPath,
            initiator: u8,
            destination: u8,
            timestamp: u64,
            message: &[u8],
        ) -> fdo::Result<(bool, u8)> {
            self.last_message = Some(RemoteMessage {
                device,
                initiator,
                destination,
                timestamp,
                message: message.to_vec(),
            });
            let _ = self.channel.send(());
            if self.timeout {
                sleep(Duration::from_millis(100)).await;
                let _ = self.channel.send(());
                Ok((true, 0))
            } else if let Some(abort) = self.abort.as_ref() {
                Ok((false, *abort as u8))
            } else {
                Ok((true, 0))
            }
        }
    }

    async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
        let mut dev = dev.lock().await;
        dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
        dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
        Ok(())
    }

    #[tokio::test]
    async fn test_no_handlers() {
        let test = setup_dbus_test(cb, None).await.unwrap();

        assert!(test.system.list_handled_messages().await.is_empty());
    }

    #[tokio::test]
    async fn test_register_handler_drop() {
        let test = setup_dbus_test(cb, None).await.unwrap();
        let new_conn = test.dbus.new_connection().await.unwrap();

        let object_server = new_conn.object_server();
        let path = ObjectPath::try_from("/TestHandler").unwrap();
        assert!(object_server
            .at(&path, MockMessageHandler::new())
            .await
            .unwrap());
        let unique_name = new_conn.unique_name().unwrap();

        assert!(test
            .system
            .register_message_handler(Opcode::GetCecVersion, path.as_ref(), unique_name)
            .await
            .unwrap());
        assert_eq!(
            test.system
                .list_handled_messages()
                .await
                .into_iter()
                .collect::<Vec<_>>(),
            &[Opcode::GetCecVersion as u8]
        );

        new_conn.graceful_shutdown().await;
        sleep(Duration::from_millis(5)).await; // Wait for the drop to propagate
        assert!(test.system.list_handled_messages().await.is_empty());
    }

    #[tokio::test]
    async fn test_register_handler_relay() {
        let test = setup_dbus_test(cb, None).await.unwrap();
        let new_conn = test.dbus.new_connection().await.unwrap();

        let object_server = new_conn.object_server();
        let path = ObjectPath::try_from("/TestHandler").unwrap();
        let handler = MockMessageHandler::new();
        let mut rx = handler.channel.subscribe();
        assert!(object_server.at(&path, handler).await.unwrap());
        let handler = object_server
            .interface::<_, MockMessageHandler>(&path)
            .await
            .unwrap();
        let unique_name = new_conn.unique_name().unwrap();

        assert!(test
            .system
            .register_message_handler(Opcode::GetCecVersion, path.as_ref(), unique_name)
            .await
            .unwrap());

        test.dev
            .lock()
            .await
            .queue_rx_message(Envelope {
                message: MessageData::Valid(Message::GetCecVersion),
                initiator: LogicalAddress::Tv,
                destination: LogicalAddress::PlaybackDevice1,
                timestamp: 0,
                sequence: 1,
            })
            .await;
        select! {
            _ = sleep(Duration::from_millis(50)) => (),
            _ = rx.recv() => (),
        };
        let message = handler.get_mut().await.last_message.take().unwrap();
        assert_eq!(&message.device.as_ref(), test.proxy.inner().path());
        assert_eq!(message.initiator, LogicalAddress::Tv as u8);
        assert_eq!(message.destination, LogicalAddress::PlaybackDevice1 as u8);
        assert_eq!(message.message, &[Opcode::GetCecVersion as u8]);
        assert!(test.dev.lock().await.dequeue_tx_message().await.is_none());
    }

    #[tokio::test]
    async fn test_register_handler_timeout() {
        let test = setup_dbus_test(cb, None).await.unwrap();
        let new_conn = test.dbus.new_connection().await.unwrap();

        let object_server = new_conn.object_server();
        let path = ObjectPath::try_from("/TestHandler").unwrap();
        let mut handler = MockMessageHandler::new();
        let mut rx = handler.channel.subscribe();
        handler.timeout = true;
        assert!(object_server.at(&path, handler).await.unwrap());
        let unique_name = new_conn.unique_name().unwrap();

        assert!(test
            .system
            .register_message_handler(Opcode::GetCecVersion, path.as_ref(), unique_name)
            .await
            .unwrap());

        test.dev
            .lock()
            .await
            .queue_rx_message(Envelope {
                message: MessageData::Valid(Message::GetCecVersion),
                initiator: LogicalAddress::Tv,
                destination: LogicalAddress::PlaybackDevice1,
                timestamp: 0,
                sequence: 1,
            })
            .await;
        rx.recv().await.unwrap();
        assert!(test.dev.lock().await.dequeue_tx_message().await.is_none());
        rx.recv().await.unwrap();
        let (message, address) = test.dev.lock().await.dequeue_tx_message().await.unwrap();
        assert_eq!(address, LogicalAddress::Tv);
        assert_eq!(
            message,
            Message::FeatureAbort {
                opcode: Opcode::GetCecVersion as u8,
                abort_reason: AbortReason::Undetermined,
            }
        );
    }

    #[tokio::test]
    async fn test_register_handler_abort() {
        let test = setup_dbus_test(cb, None).await.unwrap();
        let new_conn = test.dbus.new_connection().await.unwrap();

        let object_server = new_conn.object_server();
        let path = ObjectPath::try_from("/TestHandler").unwrap();
        let mut handler = MockMessageHandler::new();
        let mut rx = handler.channel.subscribe();
        handler.abort = Some(AbortReason::Refused);
        assert!(object_server.at(&path, handler).await.unwrap());
        let unique_name = new_conn.unique_name().unwrap();

        assert!(test
            .system
            .register_message_handler(Opcode::GetCecVersion, path.as_ref(), unique_name)
            .await
            .unwrap());

        test.dev
            .lock()
            .await
            .queue_rx_message(Envelope {
                message: MessageData::Valid(Message::GetCecVersion),
                initiator: LogicalAddress::Tv,
                destination: LogicalAddress::PlaybackDevice1,
                timestamp: 0,
                sequence: 1,
            })
            .await;
        rx.recv().await.unwrap();
        sleep(Duration::from_millis(1)).await;
        let (message, address) = test.dev.lock().await.dequeue_tx_message().await.unwrap();
        assert_eq!(address, LogicalAddress::Tv);
        assert_eq!(
            message,
            Message::FeatureAbort {
                opcode: Opcode::GetCecVersion as u8,
                abort_reason: AbortReason::Refused,
            }
        );
    }

    #[tokio::test]
    async fn test_double_register_handler() {
        let test = setup_dbus_test(cb, None).await.unwrap();
        let new_conn = test.dbus.new_connection().await.unwrap();

        let object_server = new_conn.object_server();
        let unique_name = new_conn.unique_name().unwrap();

        let path = ObjectPath::try_from("/TestHandler").unwrap();
        assert!(object_server
            .at(&path, MockMessageHandler::new())
            .await
            .unwrap());
        assert!(test
            .system
            .register_message_handler(Opcode::GetCecVersion, path.as_ref(), unique_name)
            .await
            .unwrap());

        let path = ObjectPath::try_from("/TestHandler2").unwrap();
        assert!(object_server
            .at(&path, MockMessageHandler::new())
            .await
            .unwrap());
        assert!(!test
            .system
            .register_message_handler(Opcode::GetCecVersion, path.as_ref(), unique_name)
            .await
            .unwrap());

        assert_eq!(
            test.system
                .list_handled_messages()
                .await
                .into_iter()
                .collect::<Vec<_>>(),
            &[Opcode::GetCecVersion as u8]
        );

        assert!(test
            .system
            .register_message_handler(Opcode::CecVersion, path.as_ref(), unique_name)
            .await
            .unwrap());

        assert_eq!(
            test.system.list_handled_messages().await,
            HashSet::from([Opcode::CecVersion as u8, Opcode::GetCecVersion as u8])
        );
    }
}
