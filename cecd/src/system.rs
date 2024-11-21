use anyhow::{ensure, Result};
use linux_cec::operand::VendorId;
use linux_cec::LogicalAddress;
use num_enum::TryFromPrimitive;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::read_dir;
use tokio::sync::{Mutex, MutexGuard};
use tokio_util::sync::CancellationToken;
use tracing::debug;
use zbus::connection::Connection;

use crate::config::Config;
use crate::dbus::CecDevice;

#[derive(Debug)]
pub(crate) struct System {
    pub osd_name: String,
    pub vendor_id: Option<VendorId>,
    pub log_addr: LogicalAddress,

    connection: Connection,
    token: CancellationToken,
    active: HashMap<PathBuf, CancellationToken>,
}

impl System {
    pub(crate) fn new(connection: Connection, token: CancellationToken) -> System {
        System {
            osd_name: String::from("CEC Device"),
            vendor_id: None,
            log_addr: LogicalAddress::UNREGISTERED,
            connection,
            token,
            active: HashMap::new(),
        }
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
            if self.active.contains_key(&path) {
                continue;
            }

            let pathname = path.display();
            debug!("Scanning cec device {pathname}");

            let token = self.token.child_token();
            devs.push(CecDevice::open(&path, token.clone()).await?);
            add.insert(path, token);
        }
        self.active.extend(add);
        Ok(devs)
    }

    pub(crate) async fn find_dev(&mut self, path: impl AsRef<Path>) -> Result<CecDevice> {
        let pathname = path.as_ref().display();
        debug!("Scanning cec device {pathname}");
        ensure!(
            !self.active.contains_key(path.as_ref()),
            "Device {pathname} already loaded"
        );
        let token = self.token.child_token();
        let dev = CecDevice::open(&path, token.clone()).await?;
        self.active.insert(path.as_ref().to_path_buf(), token);
        Ok(dev)
    }

    pub(crate) fn close_dev(&mut self, path: impl AsRef<Path>) {
        if let Some(token) = self.active.remove(path.as_ref()) {
            token.cancel();
        }
    }

    pub(crate) async fn set_config(&mut self, config: Config) -> Result<()> {
        if let Some(osd_name) = config.osd_name {
            self.osd_name = osd_name;
        }
        if let Some(vendor_id) = config.vendor_id {
            self.vendor_id = Some(VendorId(vendor_id));
        }
        if let Some(logical_address) = config.logical_address {
            self.log_addr = LogicalAddress::try_from_primitive(logical_address)?;
        }
        todo!();
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
        self.lock().await.vendor_id
    }

    pub(crate) async fn find_devs(&self) -> Result<()> {
        let devs;
        let connection;
        {
            let mut system = self.lock().await;
            devs = system.find_devs().await?;
            connection = system.connection.clone();
        }
        for mut dev in devs {
            dev.register(connection.clone(), self.clone()).await?;
        }
        Ok(())
    }

    pub(crate) async fn find_dev(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut dev;
        let connection;
        {
            let mut system = self.lock().await;
            dev = system.find_dev(path).await?;
            connection = system.connection.clone();
        }
        dev.register(connection.clone(), self.clone()).await
    }

    pub(crate) async fn close_dev(&self, path: impl AsRef<Path>) {
        let mut system = self.lock().await;
        system.close_dev(path);
    }

    pub(crate) async fn set_config(&self, config: Config) -> Result<()> {
        let mut system = self.lock().await;
        system.set_config(config).await
    }
}
