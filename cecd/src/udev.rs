/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::Result;
use std::os::fd::AsFd;
use tokio::io::unix::AsyncFd;
use tokio::io::Interest;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};
use udev::{Event, EventType, MonitorBuilder};

use crate::system::SystemHandle;

async fn handle_event(ev: Event, system: &SystemHandle) {
    debug!("Got udev event {ev:#?}");
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
            () = token.cancelled() => break Ok(()),
            guard = fd.ready(Interest::READABLE) => {
                let mut guard = guard?;
                for ev in iter.by_ref() {
                    handle_event(ev, &system).await;
                };
                guard.clear_ready();
            },
            _ = fd.ready(Interest::ERROR) => break Ok(()),
        };
    }
}
