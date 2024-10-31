use anyhow::Result;
use zbus::interface;
use linux_cec::device::AsyncDevice;
use std::path::Path;

struct CecDevice {
    device: AsyncDevice,
}

impl CecDevice {
    pub(crate) async fn open(path: impl AsRef<Path>) -> Result<CecDevice> {
        Ok(CecDevice {
            device: AsyncDevice::open(&path).await?
        })
    }
}

#[interface(name = "com.steampowered.CecDaemon.CecDevice")]
impl CecDevice {
}
