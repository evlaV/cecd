/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::{anyhow, bail};
use libc::pid_t;
use nix::sys::signal;
use nix::unistd::Pid;
use std::process::Stdio;
use std::str::FromStr;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use zbus::connection::{Builder, Connection};
use zbus::Address;

pub struct MockDBus {
    _connection: Connection,
    address: Address,
    process: Child,
}

impl MockDBus {
    pub async fn new() -> anyhow::Result<MockDBus> {
        let mut process = Command::new("/usr/bin/dbus-daemon")
            .args([
                "--nofork",
                "--print-address",
                "--config-file=test-dbus.conf",
            ])
            .stdout(Stdio::piped())
            .spawn()?;

        let stdout = BufReader::new(
            process
                .stdout
                .take()
                .ok_or(anyhow!("Couldn't capture stdout"))?,
        );

        let address = stdout
            .lines()
            .next_line()
            .await?
            .ok_or(anyhow!("Failed to read address"))?;

        let address = Address::from_str(address.trim_end())?;
        let connection = Builder::address(address.clone())?.build().await?;

        Ok(MockDBus {
            _connection: connection,
            address,
            process,
        })
    }

    pub fn _shutdown(mut self) -> anyhow::Result<()> {
        let pid = match self.process.id() {
            Some(id) => id,
            None => return Ok(()),
        };
        let pid: pid_t = match pid.try_into() {
            Ok(pid) => pid,
            Err(message) => bail!("Unable to get pid_t from command {message}"),
        };
        signal::kill(Pid::from_raw(pid), signal::Signal::SIGINT)?;
        for _ in [0..10] {
            // Wait for the process to exit synchronously, but not for too long
            if self.process.try_wait()?.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_micros(100));
        }
        Ok(())
    }

    pub fn address(&self) -> Address {
        self.address.clone()
    }
}
