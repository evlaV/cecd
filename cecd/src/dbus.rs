use anyhow::Result;
use linux_cec::device::{AsyncDevice, PollResult, PollTimeout};
use linux_cec::message::Message;
use linux_cec::operand::Version;
use linux_cec::Timeout;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::canonicalize;
use tokio::select;
use tokio::sync::Mutex;
use tokio::task::{spawn, JoinHandle};
use tokio_util::sync::CancellationToken;
use zbus::object_server::{InterfaceRef, SignalEmitter};
use zbus::{interface, Connection};

use crate::SystemHandle;

const PATH: &'static str = "/com/steampowered/CecDaemon1";

pub struct CecDevice {
    device: Arc<Mutex<AsyncDevice>>,
    token: CancellationToken,
    poller: Option<JoinHandle<Result<()>>>,
    path: PathBuf,
}

struct PollTask {
    device: Arc<Mutex<AsyncDevice>>,
    token: CancellationToken,
    interface: InterfaceRef<CecDevice>,
    system: SystemHandle,
}

impl CecDevice {
    pub async fn open(path: impl AsRef<Path>) -> Result<CecDevice> {
        let path = canonicalize(path).await?;
        let device = Arc::new(Mutex::new(AsyncDevice::open(&path).await?));
        let token = CancellationToken::new();
        Ok(CecDevice {
            device,
            token,
            path,
            poller: None,
        })
    }

    pub async fn register(&mut self, connection: Connection, system: SystemHandle) -> Result<()> {
        let object_server = connection.object_server();
        let path = self.dbus_path()?;
        let interface = object_server.interface(path).await?;
        let poll_task = PollTask {
            device: self.device.clone(),
            token: self.token.clone(),
            interface,
            system,
        };
        self.poller = Some(spawn(poll_task.run()));
        Ok(())
    }

    pub fn dbus_path(&self) -> Result<String> {
        todo!();
    }
}

#[interface(name = "com.steampowered.CecDaemon1.CecDevice1")]
impl CecDevice {
    #[zbus(signal)]
    async fn received_message(
        signal_emitter: &SignalEmitter<'_>,
        initiator: u8,
        destination: u8,
        timestamp: u64,
        message: &[u8],
    ) -> zbus::Result<()>;
}

impl PollTask {
    async fn run(self) -> Result<()> {
        let poller = self.device.lock().await.get_poller().await?;
        loop {
            select! {
                ev = poller.poll(PollTimeout::NONE) => {
                    self.handle_poll_result(ev?).await?
                }
                _ = self.token.cancelled() => (),
            }
        }
    }

    async fn handle_poll_result(&self, result: PollResult) -> Result<()> {
        if !result.got_message() {
            return Ok(());
        }
        let envelope = self
            .device
            .lock()
            .await
            .rx_message(Timeout::from_ms(10))
            .await?;

        self.interface
            .received_message(
                envelope.initiator.into(),
                envelope.destination.into(),
                envelope.timestamp,
                envelope.message.to_bytes().as_ref(),
            )
            .await?;

        let reply = match envelope.message {
            Message::GetCecVersion => Some(Message::CecVersion {
                version: Version::V2_0,
            }),
            Message::GiveDeviceVendorId => self
                .system
                .vendor_id()
                .await
                .map(|vendor_id| Message::DeviceVendorId { vendor_id }),
            Message::GiveOsdName => Some(Message::SetOsdName {
                name: self.system.osd_name().await,
            }),
            Message::GivePhysicalAddr => todo!(),
            Message::UserControlPressed { .. } => todo!(),
            Message::UserControlReleased => todo!(),
            _ => None,
        };

        todo!();
    }
}
