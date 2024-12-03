use nix::poll::PollTimeout;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, RecvError, SendError, Sender};
use std::thread::{self, JoinHandle};
use tokio::fs::OpenOptions;
use tokio::sync::oneshot;

use crate::device::{ConnectorInfo, Envelope, PollResult, PollStatus};
use crate::message::Message;
use crate::operand::VendorId;
use crate::{
    device, Error, FollowerMode, InitiatorMode, LogicalAddress, PhysicalAddress, Result, Timeout,
};

macro_rules! relay {
    ($self:expr, $message:ident) => {
        let (tx, rx) = oneshot::channel();
        $self.tx
            .send(DeviceCommand::$message(tx))?;
        rx.await?
    };

    ($self:expr, $message:ident => $($args:expr),*) => {
        let (tx, rx) = oneshot::channel();
        $self.tx
            .send(DeviceCommand::$message($($args,)* tx))?;
        rx.await?
    };
}

type ResultChannel<T> = oneshot::Sender<Result<T>>;

enum DeviceCommand {
    Drop,
    GetPoller(ResultChannel<DevicePoller>),
    SetBlocking(bool, ResultChannel<()>),
    SetInitiator(InitiatorMode, ResultChannel<()>),
    SetFollower(FollowerMode, ResultChannel<()>),
    GetPhysicalAddress(ResultChannel<PhysicalAddress>),
    SetPhysicalAddress(PhysicalAddress, ResultChannel<()>),
    GetLogicalAddresses(ResultChannel<Vec<LogicalAddress>>),
    SetLogicalAddresses(Vec<LogicalAddress>, ResultChannel<()>),
    SetLogicalAddress(LogicalAddress, ResultChannel<()>),
    GetOsdName(ResultChannel<String>),
    SetOsdName(String, ResultChannel<()>),
    GetVendorId(ResultChannel<Option<VendorId>>),
    SetVendorId(Option<VendorId>, ResultChannel<()>),
    TransmitMessage(Message, LogicalAddress, ResultChannel<()>),
    ReceiveMessage(Timeout, ResultChannel<Envelope>),
    HandleStatus(PollStatus, ResultChannel<Vec<PollResult>>),
    GetConnectorInfo(ResultChannel<ConnectorInfo>),
    ActivateSource(bool, ResultChannel<()>),
    Standby(LogicalAddress, ResultChannel<()>),
}

pub struct Device {
    thread: Option<JoinHandle<Result<()>>>,
    tx: Sender<DeviceCommand>,
}

pub struct DevicePoller {
    thread: Option<JoinHandle<Result<()>>>,
    tx: Sender<PollerCommand>,
}

enum PollerCommand {
    Drop,
    Poll(PollTimeout, ResultChannel<PollStatus>),
}

struct DeviceThread {
    device: device::Device,
    rx: Receiver<DeviceCommand>,
}

impl Device {
    pub async fn open(path: impl AsRef<Path>) -> Result<Device> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(path)
            .await?;

        let (tx, rx) = channel();
        let (start_tx, start_rx) = oneshot::channel();
        let file = file.into_std().await;

        let thread = thread::spawn(move || {
            let device = device::Device::try_from(file)?;
            let mut thread = DeviceThread { device, rx };
            let _ = start_tx.send(());
            thread.run()
        });
        start_rx.await?;

        Ok(Device {
            thread: Some(thread),
            tx,
        })
    }

    pub async fn set_blocking(&self, blocking: bool) -> Result<()> {
        relay! { self, SetBlocking => blocking }
    }

    pub async fn get_poller(&self) -> Result<DevicePoller> {
        relay! { self, GetPoller }
    }

    pub async fn set_initiator(&self, mode: InitiatorMode) -> Result<()> {
        relay! { self, SetInitiator => mode }
    }

    pub async fn set_follower(&self, mode: FollowerMode) -> Result<()> {
        relay! { self, SetFollower => mode }
    }

    pub async fn get_physical_address(&self) -> Result<PhysicalAddress> {
        relay! { self, GetPhysicalAddress }
    }

    pub async fn set_physical_address(&self, phys_addr: PhysicalAddress) -> Result<()> {
        relay! { self, SetPhysicalAddress => phys_addr }
    }

    pub async fn get_logical_addresses(&self) -> Result<Vec<LogicalAddress>> {
        relay! { self, GetLogicalAddresses }
    }

    pub async fn set_logical_addresses(&self, log_addrs: &[LogicalAddress]) -> Result<()> {
        relay! { self, SetLogicalAddresses => Vec::from(log_addrs) }
    }

    pub async fn set_logical_address(&self, log_addr: LogicalAddress) -> Result<()> {
        relay! { self, SetLogicalAddress => log_addr }
    }

    pub async fn get_osd_name(&self) -> Result<String> {
        relay! { self, GetOsdName }
    }

    pub async fn set_osd_name(&self, name: &str) -> Result<()> {
        relay! { self, SetOsdName => name.to_string() }
    }

    pub async fn get_vendor_id(&self) -> Result<Option<VendorId>> {
        relay! { self, GetVendorId }
    }

    pub async fn set_vendor_id(&self, vendor_id: Option<VendorId>) -> Result<()> {
        relay! { self, SetVendorId => vendor_id }
    }

    pub async fn tx_message(&self, message: &Message, destination: LogicalAddress) -> Result<()> {
        relay! { self, TransmitMessage => *message, destination }
    }

    pub async fn rx_message(&self, timeout: Timeout) -> Result<Envelope> {
        relay! { self, ReceiveMessage => timeout }
    }

    pub async fn handle_status(&self, status: PollStatus) -> Result<Vec<PollResult>> {
        relay! { self, HandleStatus => status }
    }

    pub async fn get_connector_info(&self) -> Result<ConnectorInfo> {
        relay! { self, GetConnectorInfo }
    }

    pub async fn activate_source(&self, text_view: bool) -> Result<()> {
        relay! { self, ActivateSource => text_view }
    }

    pub async fn standby(&self, target: LogicalAddress) -> Result<()> {
        relay! { self, Standby => target }
    }

    pub async fn close(mut self) -> Result<()> {
        self.tx.send(DeviceCommand::Drop)?;
        let Some(thread) = self.thread.take() else {
            return Ok(());
        };
        thread.join().unwrap()
    }
}

impl From<device::Device> for Device {
    fn from(device: device::Device) -> Device {
        let (tx, rx) = channel();
        let mut thread = DeviceThread { device, rx };

        let thread = thread::spawn(move || thread.run());
        Device {
            thread: Some(thread),
            tx,
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        let _ = self.tx.send(DeviceCommand::Drop);
        let _ = self.thread.take().unwrap().join();
    }
}

impl DeviceThread {
    fn run(&mut self) -> Result<()> {
        loop {
            match self.rx.recv()? {
                DeviceCommand::Drop => break,
                DeviceCommand::GetPoller(tx) => {
                    let _ = tx.send(self.device.get_poller().map(DevicePoller::from));
                }
                DeviceCommand::SetBlocking(block, tx) => {
                    let _ = tx.send(self.device.set_blocking(block));
                }
                DeviceCommand::SetInitiator(mode, tx) => {
                    let _ = tx.send(self.device.set_initiator(mode));
                }
                DeviceCommand::SetFollower(mode, tx) => {
                    let _ = tx.send(self.device.set_follower(mode));
                }
                DeviceCommand::GetPhysicalAddress(tx) => {
                    let _ = tx.send(self.device.get_physical_address());
                }
                DeviceCommand::SetPhysicalAddress(phys_addr, tx) => {
                    let _ = tx.send(self.device.set_physical_address(phys_addr));
                }
                DeviceCommand::GetLogicalAddresses(tx) => {
                    let _ = tx.send(self.device.get_logical_addresses());
                }
                DeviceCommand::SetLogicalAddresses(log_addrs, tx) => {
                    let _ = tx.send(self.device.set_logical_addresses(&log_addrs));
                }
                DeviceCommand::SetLogicalAddress(log_addr, tx) => {
                    let _ = tx.send(self.device.set_logical_address(log_addr));
                }
                DeviceCommand::GetOsdName(tx) => {
                    let _ = tx.send(self.device.get_osd_name());
                }
                DeviceCommand::SetOsdName(name, tx) => {
                    let _ = tx.send(self.device.set_osd_name(&name));
                }
                DeviceCommand::GetVendorId(tx) => {
                    let _ = tx.send(self.device.get_vendor_id());
                }
                DeviceCommand::SetVendorId(vendor_id, tx) => {
                    let _ = tx.send(self.device.set_vendor_id(vendor_id));
                }
                DeviceCommand::TransmitMessage(message, dest, tx) => {
                    let _ = tx.send(self.device.tx_message(&message, dest));
                }
                DeviceCommand::ReceiveMessage(timeout, tx) => {
                    let _ = tx.send(self.device.rx_message(timeout));
                }
                DeviceCommand::HandleStatus(status, tx) => {
                    let _ = tx.send(self.device.handle_status(status));
                }
                DeviceCommand::GetConnectorInfo(tx) => {
                    let _ = tx.send(self.device.get_connector_info());
                }
                DeviceCommand::ActivateSource(text_view, tx) => {
                    let _ = tx.send(self.device.activate_source(text_view));
                }
                DeviceCommand::Standby(target, tx) => {
                    let _ = tx.send(self.device.standby(target));
                }
            }
        }
        Ok(())
    }
}

impl From<RecvError> for Error {
    fn from(error: RecvError) -> Error {
        Error::UnknownError(error.to_string())
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(error: SendError<T>) -> Error {
        Error::UnknownError(error.to_string())
    }
}

impl From<oneshot::error::RecvError> for Error {
    fn from(error: oneshot::error::RecvError) -> Error {
        Error::UnknownError(error.to_string())
    }
}

impl From<device::DevicePoller> for DevicePoller {
    fn from(poller: device::DevicePoller) -> DevicePoller {
        let (tx, rx) = channel();

        let thread = thread::spawn(move || {
            loop {
                match rx.recv()? {
                    PollerCommand::Drop => break,
                    PollerCommand::Poll(timeout, tx) => {
                        let _ = tx.send(poller.poll(timeout));
                    }
                }
            }
            Ok(())
        });
        DevicePoller {
            thread: Some(thread),
            tx,
        }
    }
}

impl DevicePoller {
    pub async fn poll(&self, timeout: PollTimeout) -> Result<PollStatus> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(PollerCommand::Poll(timeout, tx))?;
        rx.await?
    }
}

impl Drop for DevicePoller {
    fn drop(&mut self) {
        let _ = self.tx.send(PollerCommand::Drop);
        let _ = self.thread.take().unwrap().join();
    }
}
