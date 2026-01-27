/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::{anyhow, bail, Result};
use linux_cec::device::MessageData;
use linux_cec::message::{Message, Opcode};
use linux_cec::operand::{OperandEncodable, UiCommand};
use linux_cec::{LogicalAddress, LogicalAddressType, PhysicalAddress, Timeout};
use num_enum::TryFromPrimitive;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use tokio::fs::canonicalize;
use tokio::sync::broadcast::Sender;
use tokio::task::{spawn, JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};
use zbus::{fdo, interface, Connection};

use crate::config::Config;
use crate::device::{DeviceTask, KeyRepeat};
use crate::system::{SystemHandle, SystemMessage};
use crate::uinput::UInputDevice;
use crate::ArcDevice;

fn into_fdo_error<T: Display>(val: T) -> fdo::Error {
    fdo::Error::Failed(format!("{val}"))
}

pub const PATH: &str = "/com/steampowered/CecDaemon1";

#[derive(Debug)]
pub struct CecConfig {
    system: SystemHandle,
    cached_config: Config,
}

impl CecConfig {
    pub async fn new(system: SystemHandle) -> CecConfig {
        let cached_config = system.lock().await.config.clone();
        CecConfig {
            system,
            cached_config,
        }
    }

    pub async fn reconfigure(&mut self, emitter: &SignalEmitter<'_>) {
        let mut new_config = self.system.lock().await.config.clone();
        if new_config.logical_address == LogicalAddressType::Unregistered {
            new_config.logical_address = LogicalAddressType::Playback;
        }
        let old_config = self.cached_config.clone();
        self.cached_config = new_config;

        if self.cached_config.osd_name != old_config.osd_name {
            if let Err(e) = self.osd_name_changed(emitter).await {
                warn!("Failed to emit OsdName changed: {e}");
            }
        }

        if self.cached_config.vendor_id != old_config.vendor_id {
            if let Err(e) = self.vendor_id_changed(emitter).await {
                warn!("Failed to emit VendorId changed: {e}");
            }
        }

        if self.cached_config.logical_address != old_config.logical_address {
            if let Err(e) = self.logical_address_changed(emitter).await {
                warn!("Failed to emit LogicalAddress changed: {e}");
            }
        }

        if self.cached_config.mappings != old_config.mappings {
            if let Err(e) = self.mappings_changed(emitter).await {
                warn!("Failed to emit Mappings changed: {e}");
            }
        }

        if self.cached_config.wake_tv != old_config.wake_tv {
            if let Err(e) = self.wake_tv_changed(emitter).await {
                warn!("Failed to emit WakeTv changed: {e}");
            }
        }

        if self.cached_config.suspend_tv != old_config.suspend_tv {
            if let Err(e) = self.suspend_tv_changed(emitter).await {
                warn!("Failed to emit SuspendTv changed: {e}");
            }
        }

        if self.cached_config.allow_standby != old_config.allow_standby {
            if let Err(e) = self.allow_standby_changed(emitter).await {
                warn!("Failed to emit AllowStandby changed: {e}");
            }
        }

        if self.cached_config.disable_uinput != old_config.disable_uinput {
            if let Err(e) = self.disable_uinput_changed(emitter).await {
                warn!("Failed to emit DisableUinput changed: {e}");
            }
        }
    }
}

#[interface(name = "com.steampowered.CecDaemon1.Config1")]
impl CecConfig {
    #[zbus(property)]
    pub async fn osd_name(&self) -> &str {
        self.cached_config.osd_name.as_deref().unwrap_or("")
    }

    #[zbus(property)]
    pub async fn vendor_id(&self) -> i32 {
        self.cached_config.vendor_id.map_or(-1, Into::<i32>::into)
    }

    #[zbus(property)]
    pub async fn logical_address(&self) -> u8 {
        self.cached_config.logical_address.into()
    }

    #[zbus(property)]
    pub async fn mappings(&self) -> Vec<(Vec<u8>, u16)> {
        self.cached_config
            .mappings
            .iter()
            .map(|(k, v)| {
                let mut kv = Vec::new();
                k.to_bytes(&mut kv);
                (kv, (*v).into())
            })
            .collect()
    }

    #[zbus(property)]
    pub async fn wake_tv(&self) -> bool {
        self.cached_config.wake_tv
    }

    #[zbus(property)]
    pub async fn suspend_tv(&self) -> bool {
        self.cached_config.suspend_tv
    }

    #[zbus(property)]
    pub async fn allow_standby(&self) -> bool {
        self.cached_config.allow_standby
    }

    #[zbus(property)]
    pub async fn disable_uinput(&self) -> bool {
        self.cached_config.disable_uinput
    }

    pub async fn reload(&self) -> fdo::Result<()> {
        self.system.reconfig().await.map_err(into_fdo_error)
    }
}

pub struct CecDevice {
    pub device: ArcDevice,
    pub token: CancellationToken,
    channel: Option<Sender<SystemMessage>>,
    path: PathBuf,
    dbus_path: OwnedObjectPath,
    pub cached_phys_addr: u16,
    pub cached_log_addrs: Vec<u8>,
    pub cached_vendor_id: i32,
    key_repeat: HashMap<u8, (UiCommand, CancellationToken, JoinHandle<Result<()>>)>,
    pub uinput: Option<UInputDevice>,
}

impl CecDevice {
    pub async fn open(path: impl AsRef<Path>, token: CancellationToken) -> Result<CecDevice> {
        let path = canonicalize(path).await?;
        let device = ArcDevice::open(&path).await?;

        let dbus_path = path.to_str().ok_or(anyhow!("Invalid path supplied"))?;
        let dbus_path = dbus_path.strip_prefix("/dev").unwrap_or(dbus_path);
        let dbus_path = dbus_path
            .split('/')
            .filter_map(|node| {
                // Capitalize the first letter of all path elements, if present
                let mut chars = node.chars();
                chars
                    .next()
                    .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
            })
            .collect::<String>();
        let dbus_path = OwnedObjectPath::try_from(format!("{PATH}/Devices/{dbus_path}"))?;

        Ok(CecDevice {
            device,
            token,
            path,
            dbus_path,
            channel: None,
            cached_phys_addr: 0xFFFF,
            cached_log_addrs: Vec::new(),
            cached_vendor_id: -1,
            key_repeat: HashMap::new(),
            uinput: None,
        })
    }

    pub async fn register(self, connection: Connection, system: SystemHandle) -> Result<()> {
        debug!("Registering CEC device {} on bus", self.path.display());

        let object_server = connection.object_server();
        let dbus_path = self.dbus_path.clone();
        object_server.at(dbus_path.as_ref(), self).await?;

        let interface = object_server.interface(dbus_path.as_ref()).await?;
        let sender;
        let receiver;
        {
            let system = system.lock().await;
            sender = system.channel.clone();
            receiver = system.subscribe();
        }
        let task = DeviceTask::new(interface.clone(), system, receiver, connection).await?;
        spawn(task.run());
        let mut interface = interface.get_mut().await;
        interface.channel = Some(sender);
        info!("Device {dbus_path} registered");
        Ok(())
    }

    pub fn dbus_path(&self) -> &ObjectPath<'_> {
        &self.dbus_path
    }

    pub async fn send_system_message(&self, message: SystemMessage) -> Result<()> {
        let Some(ref tx) = self.channel else {
            bail!("Device task has not started");
        };
        tx.send(message)?;
        Ok(())
    }
}

#[interface(name = "com.steampowered.CecDaemon1.CecDevice1")]
impl CecDevice {
    #[zbus(signal)]
    async fn received_message(
        signal_emitter: &SignalEmitter<'_>,
        initiator: u8,
        destination: u8,
        timestamp: u64,
        message: &[u8],
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn user_control_pressed(
        signal_emitter: &SignalEmitter<'_>,
        button: &[u8],
        initiator: u8,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn user_control_released(
        signal_emitter: &SignalEmitter<'_>,
        initiator: u8,
    ) -> zbus::Result<()>;

    #[zbus(property)]
    async fn logical_addresses(&self) -> Vec<u8> {
        self.cached_log_addrs.clone()
    }

    #[zbus(property)]
    async fn physical_address(&self) -> u16 {
        self.cached_phys_addr
    }

    #[zbus(property)]
    async fn vendor_id(&self) -> i32 {
        self.cached_vendor_id
    }

    async fn set_osd_name(&self, name: &str) -> fdo::Result<()> {
        self.device
            .lock()
            .await
            .set_osd_name(name)
            .await
            .map_err(into_fdo_error)
    }

    async fn set_active_source(&self, phys_addr: i32) -> fdo::Result<()> {
        let phys_addr = match <_ as TryInto<u16>>::try_into(phys_addr) {
            Ok(phys_addr) => Some(PhysicalAddress::from(phys_addr)),
            Err(_) => None,
        };
        self.device
            .lock()
            .await
            .set_active_source(phys_addr)
            .await
            .map_err(into_fdo_error)
    }

    async fn wake(&self) -> fdo::Result<()> {
        self.send_system_message(SystemMessage::Wake)
            .await
            .map_err(into_fdo_error)
    }

    async fn standby(&self, target: u8) -> fdo::Result<()> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        self.device
            .lock()
            .await
            .standby(target)
            .await
            .map_err(into_fdo_error)
    }

    async fn press_user_control(&mut self, button: &[u8], target: u8) -> fdo::Result<()> {
        let log_addr = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let key = UiCommand::try_from_bytes(button).map_err(into_fdo_error)?;
        if let Some((current_key, token, _)) = self.key_repeat.get(&target) {
            if &key == current_key {
                return Ok(());
            }
            token.cancel();
            let (_, _, handle) = self.key_repeat.remove(&target).unwrap();
            handle
                .await
                .map_err(into_fdo_error)?
                .map_err(into_fdo_error)?;
        }
        let token = self.token.child_token();
        let task = KeyRepeat {
            device: self.device.clone(),
            token: token.clone(),
            log_addr,
            key,
        };
        let handle = spawn(task.run());
        self.key_repeat.insert(target, (key, token, handle));
        Ok(())
    }

    async fn release_user_control(&mut self, target: u8) -> fdo::Result<()> {
        if let Some((_, token, handle)) = self.key_repeat.remove(&target) {
            token.cancel();
            handle
                .await
                .map_err(into_fdo_error)?
                .map_err(into_fdo_error)
        } else {
            let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
            self.device
                .lock()
                .await
                .release_user_control(target)
                .await
                .map_err(into_fdo_error)
        }
    }

    async fn press_once_user_control(&mut self, button: &[u8], target: u8) -> fdo::Result<()> {
        let log_addr = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let key = UiCommand::try_from_bytes(button).map_err(into_fdo_error)?;
        if let Some((_, token, _)) = self.key_repeat.get(&target) {
            token.cancel();
            let (current_key, _, handle) = self.key_repeat.remove(&target).unwrap();
            handle
                .await
                .map_err(into_fdo_error)?
                .map_err(into_fdo_error)?;
            if key == current_key {
                return Ok(());
            }
        }
        let device = self.device.lock().await;
        device
            .press_user_control(key, log_addr)
            .await
            .map_err(into_fdo_error)?;
        device
            .release_user_control(log_addr)
            .await
            .map_err(into_fdo_error)
    }

    async fn volume_up(&self, target: u8) -> fdo::Result<()> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let device = self.device.lock().await;
        device
            .press_user_control(UiCommand::VolumeUp, target)
            .await
            .map_err(into_fdo_error)?;
        device
            .release_user_control(target)
            .await
            .map_err(into_fdo_error)
    }

    async fn volume_down(&self, target: u8) -> fdo::Result<()> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let device = self.device.lock().await;
        device
            .press_user_control(UiCommand::VolumeDown, target)
            .await
            .map_err(into_fdo_error)?;
        device
            .release_user_control(target)
            .await
            .map_err(into_fdo_error)
    }

    async fn mute(&self, target: u8) -> fdo::Result<()> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let device = self.device.lock().await;
        device
            .press_user_control(UiCommand::Mute, target)
            .await
            .map_err(into_fdo_error)?;
        device
            .release_user_control(target)
            .await
            .map_err(into_fdo_error)
    }

    async fn get_audio_status(&self, target: u8) -> fdo::Result<(u8, bool)> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let reply = self
            .device
            .lock()
            .await
            .tx_rx_message(
                &Message::GiveAudioStatus,
                target,
                Opcode::ReportAudioStatus,
                Timeout::MAX,
            )
            .await
            .map_err(into_fdo_error)?;
        let MessageData::Valid(Message::ReportAudioStatus { status }) = reply.message else {
            return Err(fdo::Error::Failed(String::from("Invalid reply")));
        };
        Ok((
            status.volume().try_into().map_err(into_fdo_error)?,
            status.mute(),
        ))
    }

    async fn send_raw_message(&self, raw_message: &[u8], target: u8) -> fdo::Result<u32> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let raw_message = Message::try_from_bytes(raw_message).map_err(into_fdo_error)?;
        self.device
            .lock()
            .await
            .tx_message(&raw_message, target)
            .await
            .map_err(into_fdo_error)
    }

    async fn send_receive_raw_message(
        &self,
        raw_message: &[u8],
        target: u8,
        opcode: u8,
        timeout: u16,
    ) -> fdo::Result<Vec<u8>> {
        let target = LogicalAddress::try_from_primitive(target).map_err(into_fdo_error)?;
        let raw_message = Message::try_from_bytes(raw_message).map_err(into_fdo_error)?;
        let reply = self
            .device
            .lock()
            .await
            .tx_rx_message(
                &raw_message,
                target,
                Opcode::try_from_primitive(opcode).map_err(into_fdo_error)?,
                Timeout::from_ms(timeout.into()),
            )
            .await
            .map_err(into_fdo_error)?;
        Ok(reply.message.to_bytes())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::system::System;
    use crate::testing::{setup_dbus_test, DBusTest};
    use cecd_proxy::Config1Proxy;
    use input_linux::Key;
    use linux_cec::device::Capabilities;
    use linux_cec::VendorId;
    use nix::unistd::gethostname;

    async fn setup_config_test(
        config: &Config,
    ) -> Result<(DBusTest<'static>, Config1Proxy<'static>)> {
        async fn cb(dev: ArcDevice) -> anyhow::Result<()> {
            let mut dev = dev.lock().await;
            dev.set_caps(Capabilities::LOG_ADDRS | Capabilities::TRANSMIT);
            dev.set_phys_addr(PhysicalAddress::from(0x1000)).await;
            Ok(())
        }
        let test = setup_dbus_test(cb, Some(config.clone())).await?;

        let config_obj = CecConfig::new(test.system.clone()).await;
        test.connection
            .object_server()
            .at(format!("{PATH}/Daemon"), config_obj)
            .await?;

        let config_proxy = Config1Proxy::builder(&test.connection).build().await?;

        Ok((test, config_proxy))
    }

    #[tokio::test]
    async fn test_default_config_readout() {
        let config = Config::default();
        let (_test, config_proxy) = setup_config_test(&config).await.unwrap();

        assert_eq!(
            config_proxy.osd_name().await.unwrap(),
            config
                .osd_name
                .unwrap_or_else(|| gethostname().unwrap().into_string().unwrap())
        );
        assert_eq!(
            config_proxy.vendor_id().await.unwrap(),
            config.vendor_id.map(Into::<i32>::into).unwrap_or(-1)
        );
        assert_eq!(
            config_proxy.logical_address().await.unwrap(),
            (if config.logical_address == LogicalAddressType::Unregistered {
                LogicalAddressType::Playback
            } else {
                config.logical_address
            })
            .into()
        );
        assert_eq!(
            HashMap::from_iter(config_proxy.mappings().await.unwrap()),
            (if !config.mappings.is_empty() {
                config
                    .mappings
                    .iter()
                    .map(|(k, v)| {
                        let mut kv = Vec::new();
                        k.to_bytes(&mut kv);
                        (kv, (*v).into())
                    })
                    .collect::<HashMap<_, _>>()
            } else {
                System::DEFAULT_MAPPINGS
                    .iter()
                    .map(|(k, v)| {
                        let mut kv = Vec::new();
                        k.to_bytes(&mut kv);
                        (kv, (*v).into())
                    })
                    .collect::<HashMap<_, _>>()
            })
        );
        assert_eq!(config_proxy.wake_tv().await.unwrap(), config.wake_tv);
        assert_eq!(config_proxy.suspend_tv().await.unwrap(), config.suspend_tv);
        assert_eq!(
            config_proxy.allow_standby().await.unwrap(),
            config.allow_standby
        );
        assert_eq!(
            config_proxy.disable_uinput().await.unwrap(),
            config.disable_uinput
        );
    }

    #[tokio::test]
    async fn test_osd_name_config_readout() {
        let mut config = Config::default();
        config.osd_name = Some(String::from("CEC2"));
        let (_test, config_proxy) = setup_config_test(&config).await.unwrap();

        assert_eq!(
            config_proxy.osd_name().await.unwrap(),
            config.osd_name.unwrap_or_default()
        );
    }

    #[tokio::test]
    async fn test_vendor_id_config_readout() {
        let mut config = Config::default();
        config.vendor_id = Some(VendorId([0x12, 0x34, 0x56]));
        let (_test, config_proxy) = setup_config_test(&config).await.unwrap();

        assert_eq!(
            config_proxy.vendor_id().await.unwrap(),
            config.vendor_id.map(Into::<i32>::into).unwrap_or(-1)
        );
    }

    #[tokio::test]
    async fn test_logical_address_config_readout() {
        let mut config = Config::default();
        config.logical_address = LogicalAddressType::AudioSystem;
        let (_test, config_proxy) = setup_config_test(&config).await.unwrap();

        assert_eq!(
            config_proxy.logical_address().await.unwrap(),
            config.logical_address.into()
        );
    }

    #[tokio::test]
    async fn test_mappings_config_readout() {
        let mut config = Config::default();
        config.mappings = [(UiCommand::Enter, Key::Enter)].into();
        let (_test, config_proxy) = setup_config_test(&config).await.unwrap();

        assert_eq!(
            HashMap::from_iter(config_proxy.mappings().await.unwrap()),
            config
                .mappings
                .iter()
                .map(|(k, v)| {
                    let mut kv = Vec::new();
                    k.to_bytes(&mut kv);
                    (kv, (*v).into())
                })
                .collect::<HashMap<_, _>>()
        );
    }

    #[tokio::test]
    async fn test_wake_tv_config_readout() {
        let mut config = Config::default();
        config.wake_tv = !config.wake_tv;
        let (_test, config_proxy) = setup_config_test(&config).await.unwrap();

        assert_eq!(config_proxy.wake_tv().await.unwrap(), config.wake_tv);
    }

    #[tokio::test]
    async fn test_suspend_tv_config_readout() {
        let mut config = Config::default();
        config.suspend_tv = !config.suspend_tv;
        let (_test, config_proxy) = setup_config_test(&config).await.unwrap();

        assert_eq!(config_proxy.suspend_tv().await.unwrap(), config.suspend_tv);
    }

    #[tokio::test]
    async fn test_allow_standby_config_readout() {
        let mut config = Config::default();
        config.allow_standby = !config.allow_standby;
        let (_test, config_proxy) = setup_config_test(&config).await.unwrap();

        assert_eq!(
            config_proxy.allow_standby().await.unwrap(),
            config.allow_standby
        );
    }

    #[tokio::test]
    async fn test_disabled_uinput_config_readout() {
        let mut config = Config::default();
        config.disable_uinput = !config.disable_uinput;
        let (_test, config_proxy) = setup_config_test(&config).await.unwrap();

        assert_eq!(
            config_proxy.disable_uinput().await.unwrap(),
            config.disable_uinput
        );
    }
}
