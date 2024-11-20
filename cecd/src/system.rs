use anyhow::{ensure, Result};
use linux_cec::operand::VendorId;
use linux_cec::LogicalAddress;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::read_dir;
use tokio::sync::{Mutex, MutexGuard};
use tracing::debug;
use zbus::connection::Connection;

use crate::dbus::CecDevice;

#[derive(Debug)]
pub(crate) struct System {
    pub osd_name: String,
    pub vendor_id: Option<VendorId>,
    pub log_addr: LogicalAddress,

    connection: Connection,
    active: HashSet<PathBuf>,
}

impl System {
    pub(crate) fn new(connection: Connection) -> System {
        System {
            osd_name: String::from("CEC Device"),
            vendor_id: None,
            log_addr: LogicalAddress::UNREGISTERED,
            connection,
            active: HashSet::new(),
        }
    }

    pub(crate) async fn find_devs(&mut self) -> Result<Vec<CecDevice>> {
        let mut devs = Vec::new();
        let mut add = HashSet::new();
        let mut dir = read_dir("/dev").await?;
        while let Some(entry) = dir.next_entry().await? {
            let name = entry.file_name();
            if !name.to_string_lossy().starts_with("cec") {
                continue;
            }

            let path = entry.path();
            if self.active.contains(&path) {
                continue;
            }

            let pathname = path.display();
            debug!("Scanning cec device {pathname}");

            devs.push(CecDevice::open(&path).await?);
            add.insert(path);
        }
        self.active.extend(add);
        Ok(devs)
    }

    pub(crate) async fn find_dev(&mut self, path: impl AsRef<Path>) -> Result<CecDevice> {
        let pathname = path.as_ref().display();
        debug!("Scanning cec device {pathname}");
        ensure!(
            !self.active.contains(path.as_ref()),
            "Device {pathname} already loaded"
        );
        let dev = CecDevice::open(&path).await?;
        self.active.insert(path.as_ref().to_path_buf());
        Ok(dev)
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
}
