use crate::model::{Device, Progress, Snapshot};
use anyhow::{Result, bail};
use iroh::EndpointAddr;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const ALPN: &[u8] = b"hopper/library/1";
const MAX_FRAME: usize = 16 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Deliver {
        id: String,
        payload: crate::home::Payload,
    },
    Podcast {
        command: crate::podcast::Command,
    },
    Pair {
        secret: String,
        device: Device,
        address: EndpointAddr,
    },
    Snapshot {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        address: Option<EndpointAddr>,
        progress: Vec<Progress>,
    },
    Media {
        id: String,
        offset: u64,
        length: u64,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    Delivered {
        receipt: crate::home::Receipt,
    },
    Podcast {
        result: crate::podcast::CommandResult,
    },
    Paired {
        device: Device,
        address: EndpointAddr,
    },
    Snapshot {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        address: Option<EndpointAddr>,
        snapshot: Snapshot,
    },
    Media {
        total: u64,
        length: u64,
    },
    Error {
        message: String,
    },
}

pub async fn write_json<W: AsyncWrite + Unpin, T: Serialize>(w: &mut W, value: &T) -> Result<()> {
    let data = serde_json::to_vec(value)?;
    if data.len() > MAX_FRAME {
        bail!("protocol frame too large");
    }
    w.write_u32(data.len() as u32).await?;
    w.write_all(&data).await?;
    Ok(())
}

pub async fn read_json<R: AsyncRead + Unpin, T: DeserializeOwned>(r: &mut R) -> Result<T> {
    let len = r.read_u32().await? as usize;
    if len > MAX_FRAME {
        bail!("protocol frame too large");
    }
    let mut data = vec![0; len];
    r.read_exact(&mut data).await?;
    Ok(serde_json::from_slice(&data)?)
}
