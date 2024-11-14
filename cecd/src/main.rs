use anyhow::Result;
use clap::Parser;
use linux_cec::device::AsyncDevice;
use linux_cec::operand::{BufferOperand, VendorId};
use std::str::FromStr;
use std::sync::Arc;
use tokio::fs::read_dir;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

pub(crate) mod dbus;

#[derive(Debug)]
struct System {
    osd_name: String,
    vendor_id: Option<VendorId>,
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub(crate) struct SystemHandle(Arc<Mutex<System>>);

impl SystemHandle {
    pub(crate) async fn osd_name(&self) -> BufferOperand {
        let osd_name = &self.0.lock().await.osd_name;
        let truncated = osd_name
            .char_indices()
            .map_while(|(index, ch)| if index <= 14 { Some(ch) } else { None })
            .collect::<String>();
        BufferOperand::from_str(truncated.as_str()).unwrap()
    }

    pub(crate) async fn vendor_id(&self) -> Option<VendorId> {
        self.0.lock().await.vendor_id
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    /// Which device to listen on. If parameter isn't specified, then cecd
    /// will attempt to detect the first available cec device in /dev.
    device: Option<String>,

    #[arg(short, long, default_value_t = true)]
    /// Enable hotplugging of CEC device. If enabled, the -d argument will be
    /// ignored and cecd will instead use the first available cec device if
    /// present, or wait for one to appear if not.
    allow_hotplug: bool,
}

async fn find_devs() -> Result<()> {
    let mut dir = read_dir("/dev").await?;
    while let Some(entry) = dir.next_entry().await? {
        let name = entry.file_name();
        if !name.to_string_lossy().starts_with("cec") {
            continue;
        }

        let path = entry.path();
        let pathname = path.display();
        debug!("Scanning cec device {pathname}");
        todo!();
    }
    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Arguments::parse();

    todo!();

    Ok(())
}
