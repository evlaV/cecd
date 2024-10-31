use anyhow::Result;
use clap::Parser;
use linux_cec::device::AsyncDevice;
use tokio::fs::read_dir;
use tracing::{debug, error, info};

pub(crate) mod dbus;

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

async fn find_dev() -> Result<Option<AsyncDevice>> {
    let mut dir = read_dir("/dev").await?;
    while let Some(entry) = dir.next_entry().await? {
        let name = entry.file_name();
        if !name.to_string_lossy().starts_with("cec") {
            continue;
        }

        let path = entry.path();
        let pathname = path.display();
        debug!("Scanning cec device {pathname}");
        match AsyncDevice::open(&path).await {
            Ok(device) => {
                info!("Found cec device at {pathname}");
                return Ok(Some(device));
            }
            Err(e) => {
                error!("Failed to attach to cec device at {pathname}: {e}");
            }
        }
    }
    Ok(None)
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Arguments::parse();

    let device = if let (false, Some(path)) = (args.allow_hotplug, args.device) {
        Some(AsyncDevice::open(path).await?)
    } else {
        find_dev().await?
    };

    Ok(())
}
