use anyhow::Result;
use std::os::fd::AsFd;
use tokio::io::unix::AsyncFd;
use tokio::io::Interest;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};
use udev::{Event, EventType, MonitorBuilder};

use crate::system::SystemHandle;

async fn handle_event(ev: Event, system: &SystemHandle) {
    let Some(tags) = ev.property_value("CURRENT_TAGS") else {
        return;
    };
    if !tags.to_string_lossy().contains(":uaccess:") {
        return;
    }
    let Some(node) = ev.devnode() else {
        return;
    };
    let device = node.to_string_lossy().to_string();
    match ev.event_type() {
        EventType::Add => {
            if let Err(err) = system.find_dev(&device).await {
                error!("Could not add device {device}: {err}");
            }
        }
        EventType::Remove => system.close_dev(device).await,
        _ => (),
    }
}

pub(crate) async fn udev_hotplug(system: SystemHandle, token: CancellationToken) -> Result<()> {
    let monitor = MonitorBuilder::new()?.match_subsystem("cec")?.listen()?;
    let mut iter = monitor.iter();
    let fd = AsyncFd::new(monitor.as_fd())?;
    loop {
        select! {
            _ = token.cancelled() => break Ok(()),
            guard = fd.ready(Interest::READABLE) => {
                let _guard = guard?;
                let Some(ev) = iter.next() else {
                    warn!("Poller said event was present, but it was not");
                    continue;
                };
                handle_event(ev, &system).await;
            },
            _ = fd.ready(Interest::ERROR) => break Ok(()),
        };
    }
}
