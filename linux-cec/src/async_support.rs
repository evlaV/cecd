use std::path::Path;
use std::sync::mpsc::{channel, Receiver, RecvError, SendError, Sender};
use std::thread::{spawn, JoinHandle};
use tokio::fs::OpenOptions;
use tokio::sync::oneshot;

use crate::device::{ConnectorInfo, Envelope};
use crate::message::Message;
use crate::{device, Error, FollowerMode, InitiatorMode, LogicalAddress, PhysicalAddress, Result};

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
    SetBlocking(bool, ResultChannel<()>),
    SetInitiator(InitiatorMode, ResultChannel<()>),
    SetFollower(FollowerMode, ResultChannel<()>),
    GetPhysicalAddress(ResultChannel<PhysicalAddress>),
    SetPhysicalAddress(PhysicalAddress, ResultChannel<()>),
    GetLogicalAddresses(ResultChannel<Vec<LogicalAddress>>),
    SetLogicalAddresses(Vec<LogicalAddress>, ResultChannel<()>),
    SetLogicalAddress(LogicalAddress, ResultChannel<()>),
    TransmitMessage(Message, LogicalAddress, ResultChannel<()>),
    ReceiveMessage(u32, ResultChannel<Envelope>),
    GetConnectorInfo(ResultChannel<ConnectorInfo>),
}

pub struct Device {
    thread: Option<JoinHandle<Result<()>>>,
    tx: Sender<DeviceCommand>,
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

        let thread = spawn(move || {
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
        relay!{ self, SetBlocking => blocking }
    }

    pub async fn set_initiator(&self, mode: InitiatorMode) -> Result<()> {
        relay!{ self, SetInitiator => mode }
    }

    pub async fn set_follower(&self, mode: FollowerMode) -> Result<()> {
        relay!{ self, SetFollower => mode }
    }

    pub async fn get_physical_address(&self) -> Result<PhysicalAddress> {
        relay!{ self, GetPhysicalAddress }
    }

    pub async fn set_physical_address(&self, phys_addr: PhysicalAddress) -> Result<()> {
        relay!{ self, SetPhysicalAddress => phys_addr }
    }

    pub async fn get_logical_addresses(&self) -> Result<Vec<LogicalAddress>> {
        relay!{ self, GetLogicalAddresses }
    }

    pub async fn set_logical_addresses(&self, log_addrs: &[LogicalAddress]) -> Result<()> {
        relay!{ self, SetLogicalAddresses => Vec::from(log_addrs) }
    }

    pub async fn set_logical_address(&self, log_addr: LogicalAddress) -> Result<()> {
        relay!{ self, SetLogicalAddress => log_addr }
    }

    pub async fn tx_message(&self, message: &Message, destination: LogicalAddress) -> Result<()> {
        relay!{ self, TransmitMessage => *message, destination }
    }

    pub async fn rx_message(&self, timeout_ms: u32) -> Result<Envelope> {
        relay!{ self, ReceiveMessage => timeout_ms }
    }

    pub async fn get_connector_info(&self) -> Result<ConnectorInfo> {
        relay!{ self, GetConnectorInfo }
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

        let thread = spawn(move || thread.run());
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
                DeviceCommand::SetBlocking(block, rx) => {
                    let _ = rx.send(self.device.set_blocking(block));
                }
                DeviceCommand::SetInitiator(mode, rx) => {
                    let _ = rx.send(self.device.set_initiator(mode));
                }
                DeviceCommand::SetFollower(mode, rx) => {
                    let _ = rx.send(self.device.set_follower(mode));
                }
                DeviceCommand::GetPhysicalAddress(rx) => {
                    let _ = rx.send(self.device.get_physical_address());
                }
                DeviceCommand::SetPhysicalAddress(phys_addr, rx) => {
                    let _ = rx.send(self.device.set_physical_address(phys_addr));
                }
                DeviceCommand::GetLogicalAddresses(rx) => {
                    let _ = rx.send(self.device.get_logical_addresses());
                }
                DeviceCommand::SetLogicalAddresses(log_addrs, rx) => {
                    let _ = rx.send(self.device.set_logical_addresses(&log_addrs));
                }
                DeviceCommand::SetLogicalAddress(log_addr, rx) => {
                    let _ = rx.send(self.device.set_logical_address(log_addr));
                }
                DeviceCommand::TransmitMessage(message, dest, rx) => {
                    let _ = rx.send(self.device.tx_message(&message, dest));
                }
                DeviceCommand::ReceiveMessage(timeout_ms, rx) => {
                    let _ = rx.send(self.device.rx_message(timeout_ms));
                }
                DeviceCommand::GetConnectorInfo(rx) => {
                    let _ = rx.send(self.device.get_connector_info());
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
