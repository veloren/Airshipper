use async_std_resolver::{
    config::{ResolverConfig, ResolverOpts},
    resolver,
};
use std::{net::IpAddr, str::FromStr, sync::Arc, time::Duration};
use surge_ping::{PingIdentifier, PingSequence};
use tracing::{debug, warn};

#[derive(Clone, Debug)]
pub struct PingResult {
    pub ping: Option<u128>,
    pub server_address: String,
}

pub async fn ping(
    clients: (Arc<surge_ping::Client>, Arc<surge_ping::Client>),
    server_address: String,
    identifier: u16,
) -> PingResult {
    // If the server address is already an IP address, use it unmodified, otherwise
    // attempt to resolve the server address to an IP using DNS
    let ip_addr = match IpAddr::from_str(&server_address) {
        Ok(ip_addr) => Some(ip_addr),
        Err(_) => {
            debug!(
                "Server address {} is not an IP address, attempting DNS resolution...",
                server_address
            );

            let resolver =
                resolver(ResolverConfig::default(), ResolverOpts::default()).await;

            // The server address is not an IP address, so attempt to resolve it via DNS
            if let Ok(resolver) = resolver {
                let result = resolver
                    .lookup_ip(server_address.as_str())
                    .await
                    .map(|x| x.iter().next())
                    .ok()
                    .flatten();

                debug!(
                    "DNS resolution of address {} result: {:?}",
                    server_address, result
                );
                result
            } else {
                warn!(
                    "Skipping DNS resolution of address {} as no DNS resolver was \
                     provided",
                    server_address
                );
                None
            }
        },
    };

    let ping = match ip_addr {
        Some(ip) => {
            let client = if ip.is_ipv4() { clients.0 } else { clients.1 };

            const PAYLOAD: [u8; 56] = [0; 56];

            let mut pinger = client.pinger(ip, PingIdentifier(identifier)).await;
            pinger.timeout(Duration::from_secs(5));

            Some(
                match pinger.ping(PingSequence(identifier), &PAYLOAD).await {
                    Ok((_, dur)) => Some(dur.as_millis()),
                    Err(e) => {
                        debug!(?e, "Failed to ping host: {}", pinger.host);
                        None
                    },
                },
            )
        },
        None => None,
    }
    .flatten();

    PingResult {
        ping,
        server_address,
    }
}
