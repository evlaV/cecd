/*
 * Copyright © 2026 Valve Software
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use anyhow::Result;
use cecd_proxy::MessageHandler1Proxy;
use linux_cec::device::Envelope;
use linux_cec::message::Message;
use linux_cec::operand::AbortReason;
use linux_cec::LogicalAddress;
use std::time::Duration;
use tokio::select;
use tokio::sync::oneshot::{channel, Sender};
use tokio::task::{spawn, JoinHandle};
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tracing::warn;
use zbus::fdo::DBusProxy;
use zbus::names::{BusName, OwnedUniqueName, UniqueName};
use zbus::zvariant::ObjectPath;
use zbus::Connection;

use crate::system::SystemHandle;

#[derive(Debug, Clone)]
pub struct MessageHandler<'proxy> {
    proxy: MessageHandler1Proxy<'proxy>,
}

impl<'proxy> MessageHandler<'proxy> {
    pub async fn new<'a, 'b>(
        connection: &Connection,
        object: &ObjectPath<'a>,
        bus_name: &UniqueName<'b>,
    ) -> Result<MessageHandler<'proxy>> {
        let proxy = MessageHandler1Proxy::builder(connection)
            .destination(BusName::Unique(bus_name.to_owned()))?
            .path(object.to_owned())?
            .build()
            .await?;
        Ok(MessageHandler::<'proxy> { proxy })
    }

    fn handle_reply(&self, res: zbus::Result<(bool, u8)>) -> Option<AbortReason> {
        match res {
            Ok((true, _)) => None,
            Ok((false, abort_reason)) => {
                Some(abort_reason.try_into().unwrap_or(AbortReason::Undetermined))
            }
            Err(err) => {
                warn!("Remote handler raised an error: {err}");
                Some(AbortReason::Undetermined)
            }
        }
    }

    pub fn is_name(&self, name: &UniqueName<'_>) -> bool {
        let BusName::Unique(this_name) = self.proxy.inner().destination() else {
            return false;
        };
        this_name == name
    }

    pub async fn handle(
        &self,
        device: &ObjectPath<'_>,
        opcode: u8,
        envelope: &Envelope,
    ) -> Option<(Message, LogicalAddress)> {
        #[cfg(not(test))]
        const TIMEOUT: Duration = Duration::from_millis(500);
        #[cfg(test)]
        const TIMEOUT: Duration = Duration::from_millis(50);
        let message = envelope.message.to_bytes();
        select! {
            () = sleep(TIMEOUT) => {
                warn!("Remote didn't reply in time");
                Some((
                    Message::FeatureAbort {
                        opcode,
                        abort_reason: AbortReason::Undetermined,
                    },
                    envelope.initiator,
                ))
            },
            res = self.proxy.handle_message(
                device,
                envelope.initiator.into(),
                envelope.destination.into(),
                envelope.timestamp,
                message.as_ref(),
            ) => {
                self.handle_reply(res).map(|abort_reason| (
                    Message::FeatureAbort { opcode, abort_reason },
                    envelope.initiator
                ))
            }
        }
    }
}

#[derive(Debug)]
pub struct MessageHandlerTask {
    connection: Connection,
    system: SystemHandle,
    opcode: u8,
    bus_name: OwnedUniqueName,
    started: Option<Sender<()>>,
}

impl MessageHandlerTask {
    pub async fn start(
        connection: &Connection,
        system: SystemHandle,
        opcode: u8,
        bus_name: &UniqueName<'_>,
    ) -> Result<JoinHandle<Result<()>>> {
        let (tx, rx) = channel();

        let handler = MessageHandlerTask {
            connection: connection.clone(),
            system,
            opcode,
            bus_name: OwnedUniqueName::from(bus_name.as_ref()),
            started: Some(tx),
        };
        let task = spawn(handler.run());
        // We need to wait for the receiver to be created before it's safe to
        // return, otherwise we can miss the signal
        let _ = rx.await;
        Ok(task)
    }

    async fn run_impl(&mut self) -> Result<()> {
        let dbus_proxy = DBusProxy::new(&self.connection).await?;
        let mut receiver = dbus_proxy.receive_name_owner_changed().await?;
        let _ = self.started.take().map(|tx| tx.send(()));
        while let Some(message) = receiver.next().await {
            let Ok(args) = message.args() else {
                todo!();
            };
            let BusName::Unique(bus_name) = args.name else {
                continue;
            };
            if bus_name != self.bus_name || args.new_owner.is_some() {
                continue;
            }
            break;
        }
        Ok(())
    }

    async fn run(mut self) -> Result<()> {
        let res = self
            .run_impl()
            .await
            .inspect_err(|err| warn!("Failed to listen for MessageHandler1 shutdown: {err}"));
        let shutdown = self
            .system
            .unregister_message_handler(self.opcode, &UniqueName::from(&self.bus_name))
            .await
            .inspect_err(|err| {
                warn!(
                    "Failed unregister MessageHandler1 or opcode {:02x}: {err}",
                    self.opcode
                );
            });
        res.and(shutdown.map(|_| ()))
    }
}
