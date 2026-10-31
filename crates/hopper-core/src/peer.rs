use crate::{
    Node,
    model::*,
    random_secret,
    wire::{self, Request, Response},
};
use anyhow::{Context, Result, bail, ensure};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use futures_util::{StreamExt, stream};
use iroh::{
    EndpointAddr,
    endpoint::{Connection, RecvStream, SendStream},
};
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

#[derive(Serialize, Deserialize)]
struct Invitation {
    version: u8,
    address: EndpointAddr,
    secret: String,
}

impl Node {
    pub async fn invite(&self) -> Result<String> {
        // A relay address makes invitations useful outside the LAN. Do not block offline use.
        let _ = tokio::time::timeout(Duration::from_secs(3), self.0.endpoint.online()).await;
        let secret = random_secret();
        let mut invites = self.0.invites.lock().unwrap();
        invites.retain(|_, until| *until > Instant::now());
        invites.insert(secret.clone(), Instant::now() + Duration::from_secs(600));
        let ticket = Invitation {
            version: 1,
            address: self.0.endpoint.addr(),
            secret,
        };
        Ok(format!(
            "hopper1:{}",
            URL_SAFE_NO_PAD.encode(serde_json::to_vec(&ticket)?)
        ))
    }

    pub async fn pair(&self, code: &str) -> Result<()> {
        ensure!(code.len() <= 16384, "invitation too large");
        let raw = code
            .trim()
            .strip_prefix("hopper1:")
            .context("paste a Hopper invitation beginning hopper1:")?;
        let ticket: Invitation = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(raw)?)?;
        ensure!(
            ticket.version == 1 && ticket.address.id != self.0.endpoint.id(),
            "invalid invitation or already this device"
        );
        let expected = ticket.address.id;
        let (_, mut recv) = self
            .request(
                ticket.address,
                Request::Pair {
                    secret: ticket.secret,
                    device: self.0.device.clone(),
                    address: self.0.endpoint.addr(),
                },
            )
            .await?;
        match tokio::time::timeout(Duration::from_secs(15), wire::read_json(&mut recv)).await?? {
            Response::Paired { device, address } => {
                ensure!(
                    device.id == expected.to_string() && address.id == expected,
                    "pairing identity mismatch"
                );
                self.0.store.put(
                    "peer",
                    &device.id,
                    &Peer {
                        device: device.clone(),
                        address,
                    },
                )?;
                self.0.statuses.lock().unwrap().insert(device.id, None);
            }
            Response::Error { message } => bail!("{message}"),
            _ => bail!("unexpected pairing response"),
        }
        self.refresh().await?;
        Ok(())
    }

    pub async fn unpair(&self, id: &str) -> Result<()> {
        self.0.store.remove("peer", id)?;
        self.0.store.remove("snapshot", id)?;
        self.0.statuses.lock().unwrap().remove(id);
        if let Ok(id) = id.parse::<iroh::EndpointId>()
            && let Some(conn) = self.0.connections.lock().await.remove(&id)
        {
            conn.close(0u32.into(), b"device removed");
        }
        Ok(())
    }

    pub(crate) async fn request(
        &self,
        address: EndpointAddr,
        request: Request,
    ) -> Result<(SendStream, RecvStream)> {
        let conn = {
            let mut cache = self.0.connections.lock().await;
            if cache
                .get(&address.id)
                .is_some_and(|c| c.close_reason().is_some())
            {
                cache.remove(&address.id);
            }
            cache.get(&address.id).cloned()
        };
        let conn = match conn {
            Some(c) => c,
            None => {
                let c = tokio::time::timeout(
                    Duration::from_secs(8),
                    self.0.endpoint.connect(address.clone(), wire::ALPN),
                )
                .await
                .context("device connection timed out")??;
                self.0
                    .connections
                    .lock()
                    .await
                    .insert(address.id, c.clone());
                let node = self.clone();
                let connection = c.clone();
                tokio::spawn(async move { node.serve_connection(connection).await });
                c
            }
        };
        let (mut send, recv) = conn.open_bi().await?;
        wire::write_json(&mut send, &request).await?;
        send.finish()?;
        Ok((send, recv))
    }

    pub async fn refresh(&self) -> Result<()> {
        let _guard = self.0.refreshing.lock().await;
        let peers = self.0.store.all::<Peer>("peer")?;
        let updates = self.0.store.all::<Progress>("progress")?;
        let mut pending = stream::iter(peers.into_iter().map(|peer| {
            let node = self.clone();
            let updates = updates.clone();
            async move {
                let result = tokio::time::timeout(Duration::from_secs(12), async {
                    let (_, mut recv) = node
                        .request(
                            peer.address.clone(),
                            Request::Snapshot { progress: updates },
                        )
                        .await?;
                    match wire::read_json(&mut recv).await? {
                        Response::Snapshot { snapshot } => {
                            ensure!(
                                snapshot.device.id == peer.device.id,
                                "catalogue identity mismatch"
                            );
                            ensure!(
                                snapshot.items.len() <= 100000
                                    && snapshot.items.iter().all(|m| valid_id(&m.id)
                                        && m.bytes > 0
                                        && m.bytes < 10_000_000_000_000),
                                "invalid catalogue"
                            );
                            // An unpair may have happened while the network request was pending.
                            ensure!(
                                node.0.store.get::<Peer>("peer", &peer.device.id)?.is_some(),
                                "device removed"
                            );
                            node.0.store.merge_progress(&snapshot.progress)?;
                            node.0.store.put("snapshot", &peer.device.id, &snapshot)?;
                            Ok(())
                        }
                        Response::Error { message } => bail!("{message}"),
                        _ => bail!("unexpected catalogue response"),
                    }
                })
                .await
                .unwrap_or_else(|_| Err(anyhow::anyhow!("device is offline or unreachable")));
                (peer.device.id, result)
            }
        }))
        .buffer_unordered(8);
        while let Some((id, result)) = pending.next().await {
            self.0
                .statuses
                .lock()
                .unwrap()
                .insert(id, result.err().map(|e| e.to_string()));
        }
        Ok(())
    }

    pub(crate) fn start_accepting(&self) {
        let node = self.clone();
        let limit = Arc::new(tokio::sync::Semaphore::new(64));
        tokio::spawn(async move {
            while let Some(incoming) = node.0.endpoint.accept().await {
                let Ok(permit) = limit.clone().try_acquire_owned() else {
                    incoming.refuse();
                    continue;
                };
                let node = node.clone();
                tokio::spawn(async move {
                    let _permit = permit;
                    let Ok(Ok(conn)) =
                        tokio::time::timeout(Duration::from_secs(10), incoming).await
                    else {
                        return;
                    };
                    node.serve_connection(conn).await;
                });
            }
        });
    }

    async fn serve_connection(&self, conn: Connection) {
        // QUIC streams are bidirectional regardless of which device dialed the connection.
        let streams = Arc::new(tokio::sync::Semaphore::new(16));
        while let Ok((mut send, mut recv)) = conn.accept_bi().await {
            let Ok(permit) = streams.clone().try_acquire_owned() else {
                break;
            };
            let node = self.clone();
            let conn = conn.clone();
            tokio::spawn(async move {
                let _permit = permit;
                let result = node.handle_request(&conn, &mut send, &mut recv).await;
                if let Err(e) = result {
                    let _ = wire::write_json(
                        &mut send,
                        &Response::Error {
                            message: e.to_string(),
                        },
                    )
                    .await;
                }
                let _ = send.finish();
            });
        }
    }

    async fn handle_request(
        &self,
        conn: &Connection,
        send: &mut SendStream,
        recv: &mut RecvStream,
    ) -> Result<()> {
        let request =
            tokio::time::timeout(Duration::from_secs(10), wire::read_json::<_, Request>(recv))
                .await??;
        let remote = conn.remote_id().to_string();
        if let Request::Pair {
            secret,
            device,
            address,
        } = request
        {
            ensure!(
                device.id == remote
                    && address.id == conn.remote_id()
                    && !device.name.trim().is_empty()
                    && device.name.len() <= 100,
                "invalid pairing identity"
            );
            {
                let mut invites = self.0.invites.lock().unwrap();
                ensure!(
                    invites
                        .get(&secret)
                        .is_some_and(|until| *until > Instant::now()),
                    "invitation expired, already used, or invalid"
                );
                self.0
                    .store
                    .put("peer", &remote, &Peer { device, address })?;
                invites.remove(&secret);
            }
            self.0
                .connections
                .lock()
                .await
                .insert(conn.remote_id(), conn.clone());
            wire::write_json(
                send,
                &Response::Paired {
                    device: self.0.device.clone(),
                    address: self.0.endpoint.addr(),
                },
            )
            .await?;
            return Ok(());
        }
        ensure!(
            self.0.store.get::<Peer>("peer", &remote)?.is_some(),
            "device is not paired"
        );
        self.0
            .connections
            .lock()
            .await
            .insert(conn.remote_id(), conn.clone());
        match request {
            Request::Deliver { id, payload } => {
                let receipt = self.receive_delivery(&remote, &id, payload).await?;
                wire::write_json(send, &Response::Delivered { receipt }).await?;
            }
            Request::Podcast { command } => {
                let result = self.apply_podcast_command(command).await?;
                wire::write_json(send, &Response::Podcast { result }).await?;
            }
            Request::Snapshot { progress } => {
                self.0.store.merge_progress(&progress)?;
                wire::write_json(
                    send,
                    &Response::Snapshot {
                        snapshot: self.snapshot()?,
                    },
                )
                .await?;
            }
            Request::Media { id, offset, length } => {
                ensure!(valid_id(&id), "invalid media ID");
                let local = self
                    .0
                    .store
                    .get::<LocalMedia>("local", &id)?
                    .context("media is not on this device")?;
                ensure!(
                    crate::import::available(&local),
                    "source file moved or changed; reimport it"
                );
                ensure!(
                    offset < local.media.bytes
                        && length > 0
                        && length <= local.media.bytes - offset,
                    "invalid byte range"
                );
                let mut file = tokio::fs::File::open(&local.path).await?;
                file.seek(std::io::SeekFrom::Start(offset)).await?;
                wire::write_json(
                    send,
                    &Response::Media {
                        total: local.media.bytes,
                        length,
                    },
                )
                .await?;
                tokio::io::copy(&mut file.take(length), send).await?;
            }
            Request::Pair { .. } => unreachable!(),
        }
        Ok(())
    }
}
