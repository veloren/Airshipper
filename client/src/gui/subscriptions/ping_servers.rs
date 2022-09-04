use crate::gui::components::ServerBrowserEntry;
use futures_util::{
    stream,
    stream::{FuturesUnordered, Stream},
};
use iced::{futures, Subscription};
use std::{collections::HashSet, net::IpAddr, str::FromStr, time::Duration};
use tracing::debug;

pub fn ping_servers(servers: Vec<String>) -> iced::Subscription<PingResult> {
    Subscription::from_recipe(PingServers(servers))
}

pub struct PingServers(Vec<String>);

#[derive(Clone, Debug)]
pub struct PingResult {
    pub server_address: String,
    pub ping: Option<u128>,
}

impl<H, I> iced_native::subscription::Recipe<H, I> for PingServers
where
    H: std::hash::Hasher,
{
    type Output = PingResult;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
        self.0.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        use iced::futures::stream::StreamExt;

        do_pings(self.0).boxed()
    }
}

pub fn do_pings(servers: Vec<String>) -> impl Stream<Item = PingResult> {
    let client_v4 = surge_ping::Client::new(&surge_ping::Config::default()).unwrap();
    let client_v6 = surge_ping::Client::new(
        &surge_ping::Config::builder()
            .kind(surge_ping::ICMP::V6)
            .build(),
    )
    .unwrap();

    let futures = FuturesUnordered::new();

    for (i, server_address) in servers.iter().enumerate() {
        futures.push(ping(
            (client_v4.clone(), client_v6.clone()),
            server_address.clone(),
            i as u16,
        ));
    }

    futures
}

async fn ping(
    clients: (surge_ping::Client, surge_ping::Client),
    server_address: String,
    identifier: u16,
) -> PingResult {
    use async_std_resolver::{
        config,
        config::{ResolverConfig, ResolverOpts},
        resolver,
    };
    use surge_ping::{PingIdentifier, PingSequence};

    let ip_addr = match IpAddr::from_str(&server_address) {
        Ok(ip_addr) => Some(ip_addr),
        Err(_) => {
            debug!(
                "Server address {} is not an IP address, attempting DNS resolution...",
                server_address
            );
            // The server address is not an IP address, so attempt to resolve it via DNS
            let resolver = resolver(ResolverConfig::default(), ResolverOpts::default())
                .await
                .unwrap();
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
        },
    };

    match ip_addr {
        Some(ip) => {
            let mut client = if ip.is_ipv4() { clients.0 } else { clients.1 };

            let payload = [0; 56];
            let ping_identifier = PingIdentifier(identifier);

            let mut pinger = client.pinger(ip, ping_identifier).await;
            pinger.timeout(Duration::from_secs(5));

            let ping = match pinger.ping(PingSequence(identifier), &payload).await {
                Ok((_, dur)) => Some(dur.as_millis()),
                Err(e) => {
                    debug!("No.{}: {} ping {}", identifier, pinger.host, e);
                    None
                },
            };

            PingResult {
                server_address,
                ping,
            }
        },
        None => PingResult {
            server_address,
            ping: None,
        },
    }
}
