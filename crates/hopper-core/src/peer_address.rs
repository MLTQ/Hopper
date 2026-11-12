use anyhow::Result;
use iroh::{
    Endpoint, EndpointAddr, EndpointId, SecretKey,
    endpoint::{BindOpts, presets},
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

fn port(id: EndpointId) -> u16 {
    let bytes = id.as_bytes();
    49152 + (u16::from_be_bytes([bytes[0], bytes[1]]) % 16384)
}

pub(crate) async fn bind(key: SecretKey, local_only: bool) -> Result<Endpoint> {
    let builder = || {
        let builder = if local_only {
            Endpoint::builder(presets::Minimal)
        } else {
            Endpoint::builder(presets::N0)
        };
        builder
            .secret_key(key.clone())
            .alpns(vec![crate::wire::ALPN.to_vec()])
    };
    let port = port(key.public());
    let stable = builder()
        .clear_ip_transports()
        .bind_addr(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))?
        .bind_addr_with_opts(
            SocketAddr::from((Ipv6Addr::UNSPECIFIED, port)),
            BindOpts::default().set_is_required(false),
        )?
        .bind()
        .await;
    // Another process may use the preferred port. Pairing and discovery also carry the actual address.
    match stable {
        Ok(endpoint) => Ok(endpoint),
        Err(_) => Ok(builder().bind().await?),
    }
}

pub(crate) fn dial_address(mut address: EndpointAddr) -> EndpointAddr {
    let port = port(address.id);
    let hints: Vec<_> = address
        .ip_addrs()
        .copied()
        .filter(|a| match a.ip() {
            IpAddr::V4(ip) => ip.is_private() || ip.is_loopback() || ip.is_link_local(),
            IpAddr::V6(ip) => ip.is_unique_local() || ip.is_loopback(),
        })
        .map(|mut a| {
            a.set_port(port);
            a
        })
        .collect();
    for hint in hints {
        address = address.with_ip_addr(hint);
    }
    address
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, Node};
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use serde_json::Value;

    #[tokio::test]
    async fn old_lan_ticket_connects_and_refreshes_the_saved_address() {
        let tmp = tempfile::tempdir().unwrap();
        let start = |name: &str| {
            Node::start(Config {
                data_dir: tmp.path().join(name),
                name: name.into(),
                local_only: true,
            })
        };
        let (host, _) = start("desktop").await.unwrap();
        let (player, _) = start("player").await.unwrap();
        let invite = host.invite().await.unwrap();
        let mut ticket: Value = serde_json::from_slice(
            &URL_SAFE_NO_PAD
                .decode(invite.strip_prefix("hopper1:").unwrap())
                .unwrap(),
        )
        .unwrap();
        let id = host.0.endpoint.id();
        ticket["address"] = serde_json::to_value(
            EndpointAddr::new(id).with_ip_addr("127.0.0.1:1".parse().unwrap()),
        )
        .unwrap();
        let legacy = format!(
            "hopper1:{}",
            URL_SAFE_NO_PAD.encode(serde_json::to_vec(&ticket).unwrap())
        );
        player.pair(&legacy).await.unwrap();
        let peer = player
            .0
            .store
            .get::<crate::model::Peer>("peer", &id.to_string())
            .unwrap()
            .unwrap();
        assert!(peer.address.ip_addrs().any(|a| a.port() == port(id)));
        player.close().await;
        host.close().await;
    }
}
