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
use tracing::{error, trace};
use udev::{Event, EventType, MonitorBuilder};

use crate::system::SystemHandle;

async fn handle_cec_event(ev: Event, system: &SystemHandle) {
    trace!("Got udev event {ev:#?}");
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

async fn handle_drm_event(ev: Event, system: &SystemHandle) {
    trace!("Got udev event {ev:#?}");
    if ev.property_value("HOTPLUG").is_none() {
        return;
    }
    let _ = system.reconfig().await;
}

pub(crate) async fn udev_hotplug(system: SystemHandle, token: CancellationToken) -> Result<()> {
    let cec_monitor = MonitorBuilder::new()?.match_subsystem("cec")?.listen()?;
    let drm_monitor = MonitorBuilder::new()?
        .match_subsystem_devtype("drm", "drm_minor")?
        .listen()?;
    let mut cec_iter = cec_monitor.iter();
    let mut drm_iter = drm_monitor.iter();
    let cec_fd = AsyncFd::new(cec_monitor.as_fd())?;
    let drm_fd = AsyncFd::new(drm_monitor.as_fd())?;
    loop {
        select! {
            () = token.cancelled() => break Ok(()),
            guard = cec_fd.ready(Interest::READABLE) => {
                let mut guard = guard?;
                for ev in cec_iter.by_ref() {
                    handle_cec_event(ev, &system).await;
                };
                guard.clear_ready();
            },
            guard = drm_fd.ready(Interest::READABLE) => {
                let mut guard = guard?;
                for ev in drm_iter.by_ref() {
                    handle_drm_event(ev, &system).await;
                };
                guard.clear_ready();
            },
            _ = cec_fd.ready(Interest::ERROR) => break Ok(()),
            _ = drm_fd.ready(Interest::ERROR) => break Ok(()),
        };
    }
}
