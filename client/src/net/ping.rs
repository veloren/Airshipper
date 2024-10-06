use std::net::{IpAddr, SocketAddr};

use tracing::warn;
use veloren_query_server::client::QueryClient;

pub async fn create_client(address: &str, port: u16) -> Option<QueryClient> {
    let addr = match address.parse::<IpAddr>() {
        Ok(addr) => SocketAddr::new(addr, port),
        Err(_error) => tokio::net::lookup_host(format!("{address}:{port}"))
            .await
            .inspect_err(|error| warn!(?address, ?error, "Host lookup failed"))
            .ok()?
            .next()
            .or_else(|| {
                warn!(?address, "Host lookup returned no IP addresses");
                None
            })?,
    };

    Some(QueryClient::new(addr))
}
