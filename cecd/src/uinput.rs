use anyhow::{ensure, Result};
use input_linux::sys::BUS_CEC;
use input_linux::{EventKind, EventTime, InputId, Key, KeyEvent, KeyState, UInputHandle};
use linux_cec::operand::UiCommand;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::os::fd::{IntoRawFd, RawFd};
use std::slice::from_ref;
use std::time::SystemTime;
use tracing::{debug, warn};

pub(crate) struct UInputDevice {
    mappings: HashMap<UiCommand, Key>,
    handle: UInputHandle<RawFd>,
    name: String,
    open: bool,
}

impl UInputDevice {
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

    fn system_time() -> Result<EventTime> {
        let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        Ok(EventTime::new(
            duration.as_secs().try_into()?,
            duration.subsec_micros().into(),
        ))
    }

    fn send_key_event(&self, key: Key, value: KeyState) -> Result<()> {
        let tv = UInputDevice::system_time().unwrap_or_else(|err| {
            warn!("System time error: {err}");
            EventTime::default()
        });

        let ev = KeyEvent::new(tv, key, value);
        self.handle.write(from_ref(ev.as_ref()))?;
        Ok(())
    }

    pub(crate) fn key_down(&self, uikey: UiCommand) -> Result<()> {
        let Some(key) = self.mappings.get(&uikey) else {
            debug!("Mapping for {uikey} not found");
            return Ok(());
        };

        self.send_key_event(*key, KeyState::PRESSED)
    }

    pub(crate) fn key_up(&self, uikey: UiCommand) -> Result<()> {
        let Some(key) = self.mappings.get(&uikey) else {
            debug!("Mapping for {uikey} not found");
            return Ok(());
        };

        self.send_key_event(*key, KeyState::RELEASED)
    }
}
