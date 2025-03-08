/*
 * Copyright © 2024 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::{ensure, Result};
#[cfg(not(test))]
use input_linux::sys::BUS_CEC;
#[cfg(test)]
use input_linux::InputEvent;
#[cfg(not(test))]
use input_linux::{EventKind, InputId, UInputHandle};
use input_linux::{EventTime, Key, KeyEvent, KeyState, SynchronizeEvent};
use linux_cec::operand::UiCommand;
#[cfg(not(test))]
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use std::collections::HashMap;
#[cfg(test)]
use std::collections::VecDeque;
#[cfg(not(test))]
use std::fs::OpenOptions;
#[cfg(not(test))]
use std::os::fd::{IntoRawFd, RawFd};
use std::time::SystemTime;
use tracing::{debug, warn};

pub(crate) struct UInputDevice {
    mappings: HashMap<UiCommand, Key>,
    #[cfg(not(test))]
    handle: UInputHandle<RawFd>,
    #[cfg(test)]
    queue: VecDeque<InputEvent>,
    name: String,
    open: bool,
}

impl UInputDevice {
    #[cfg(not(test))]
    pub(crate) fn new() -> Result<UInputDevice> {
        let rawfd = OpenOptions::new()
            .write(true)
            .create(false)
            .open("/dev/uinput")?
            .into_raw_fd();

        let mut flags = OFlag::from_bits_retain(fcntl(rawfd, FcntlArg::F_GETFL)?);
        flags.set(OFlag::O_NONBLOCK, true);
        fcntl(rawfd, FcntlArg::F_SETFL(flags))?;

        Ok(UInputDevice {
            mappings: HashMap::new(),
            handle: UInputHandle::new(rawfd),
            name: String::new(),
            open: false,
        })
    }

    #[cfg(test)]
    pub(crate) fn new() -> Result<UInputDevice> {
        Ok(UInputDevice {
            mappings: HashMap::new(),
            queue: VecDeque::new(),
            name: String::new(),
            open: false,
        })
    }

    pub(crate) fn set_mappings(&mut self, mappings: HashMap<UiCommand, Key>) -> Result<()> {
        ensure!(!self.open, "Cannot change mappings after opening");
        self.mappings = mappings;
        Ok(())
    }

    pub(crate) fn set_name(&mut self, name: String) -> Result<()> {
        ensure!(!self.open, "Cannot change name after opening");
        self.name = name;
        Ok(())
    }

    #[cfg(not(test))]
    pub(crate) fn open(&mut self) -> Result<()> {
        ensure!(!self.open, "Cannot reopen uinput handle");

        self.handle.set_evbit(EventKind::Key)?;
        for key in self.mappings.values() {
            self.handle.set_keybit(*key)?;
        }

        // TODO: Come up with better values for this
        let input_id = InputId {
            bustype: BUS_CEC,
            vendor: 0,
            product: 0,
            version: 0,
        };
        self.handle
            .create(&input_id, self.name.as_bytes(), 0, &[])?;
        self.open = true;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn open(&mut self) -> Result<()> {
        ensure!(!self.open, "Cannot reopen uinput handle");
        self.open = true;
        Ok(())
    }

    fn system_time() -> Result<EventTime> {
        let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        Ok(EventTime::new(
            duration.as_secs().try_into()?,
            duration.subsec_micros().into(),
        ))
    }

    fn send_key_event(&mut self, key: Key, value: KeyState) -> Result<()> {
        let tv = UInputDevice::system_time().unwrap_or_else(|err| {
            warn!("System time error: {err}");
            EventTime::default()
        });

        let ev = KeyEvent::new(tv, key, value);
        let syn = SynchronizeEvent::report(tv);
        #[cfg(not(test))]
        self.handle.write(&[*ev.as_ref(), *syn.as_ref()])?;
        #[cfg(test)]
        self.queue.extend(&[*ev.as_ref(), *syn.as_ref()]);
        Ok(())
    }

    pub(crate) fn key_down(&mut self, uikey: UiCommand) -> Result<()> {
        let Some(key) = self.mappings.get(&uikey) else {
            debug!("Mapping for {uikey} not found");
            return Ok(());
        };

        self.send_key_event(*key, KeyState::PRESSED)
    }

    pub(crate) fn key_up(&mut self, uikey: UiCommand) -> Result<()> {
        let Some(key) = self.mappings.get(&uikey) else {
            debug!("Mapping for {uikey} not found");
            return Ok(());
        };

        self.send_key_event(*key, KeyState::RELEASED)
    }

    #[cfg(test)]
    pub(crate) fn get_next_event(&mut self) -> Option<InputEvent> {
        self.queue.pop_front()
    }
}
