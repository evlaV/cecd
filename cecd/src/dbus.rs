use anyhow::{anyhow, Result};
use linux_cec::device::{AsyncDevice, PollResult, PollTimeout};
use linux_cec::message::Message;
use linux_cec::operand::AbortReason;
use linux_cec::{FollowerMode, InitiatorMode, LogicalAddress, Timeout};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::canonicalize;
use tokio::select;
use tokio::sync::Mutex;
use tokio::task::{spawn, JoinHandle};
use tokio_util::sync::CancellationToken;
use zbus::object_server::{InterfaceRef, SignalEmitter};
use zbus::{interface, Connection};

use crate::system::SystemHandle;

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
    pub async fn open(path: impl AsRef<Path>, token: CancellationToken) -> Result<CecDevice> {
        let path = canonicalize(path).await?;
        let device = Arc::new(Mutex::new(AsyncDevice::open(&path).await?));
        Ok(CecDevice {
            device,
            token,
            path,
            poller: None,
        })
    }

    pub async fn register(&mut self, connection: Connection, system: SystemHandle) -> Result<()> {
        let device = self.device.clone();
        let osd_name;
        let log_addr;
        let vendor_id;
        {
            let system = system.lock().await;
            osd_name = system.osd_name.clone();
            log_addr = system.log_addr;
            vendor_id = system.vendor_id;
        }
        {
            let device = device.lock().await;
            device.set_osd_name(&osd_name).await?;
            device.set_logical_address(log_addr).await?;
            device.set_vendor_id(vendor_id).await?;
            device.set_follower(FollowerMode::Enabled).await?;
            device.set_initiator(InitiatorMode::Enabled).await?;
        }
        let object_server = connection.object_server();
        let path = self.dbus_path()?;
        let interface = object_server.interface(path).await?;
        let poll_task = PollTask {
            device,
            token: self.token.clone(),
            interface,
            system,
        };
        self.poller = Some(spawn(poll_task.run()));
        Ok(())
    }

    pub fn dbus_path(&self) -> Result<String> {
        let path = self.path.to_str().ok_or(anyhow!("Invalid path supplied"))?;
        let path = path.strip_prefix("/dev").unwrap_or(path);
        let path = path
            .split('/')
            .filter_map(|node| {
                // Capitalize the first letter of all path elements, if present
                let mut chars = node.chars();
                chars
                    .next()
                    .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
            })
            .collect::<String>();
        Ok(format!("{PATH}/{path}"))
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
            Message::UserControlPressed { .. } => todo!(),
            Message::UserControlReleased => todo!(),
            _ => None,
        };

        if let Some(reply) = reply {
            self.device
                .lock()
                .await
                .tx_message(&reply, envelope.initiator)
                .await?;
        } else if envelope.destination != LogicalAddress::BROADCAST {
            let abort = Message::FeatureAbort {
                opcode: envelope.message.opcode(),
                abort_reason: AbortReason::UnrecognizedOp,
            };
            self.device
                .lock()
                .await
                .tx_message(&abort, envelope.initiator)
                .await?;
        }
        Ok(())
    }
}
