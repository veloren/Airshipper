use std::{
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    sync::LazyLock,
    time::Duration,
};

use tracing::warn;
use veloren_query_server::{
    client::{QueryClient, QueryClientError},
    proto::ServerInfo,
};

static PING_COUNT: LazyLock<NonZeroU32> = LazyLock::new(|| NonZeroU32::new(5).unwrap());

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

pub async fn perform_ping(
    client: &mut QueryClient,
) -> Result<(Duration, ServerInfo), QueryClientError> {
    let mut avg_ms = 0.0;
    let mut server_info = None;
    let mut last_error = None;

    for attempt in 1..=PING_COUNT.get() {
        match client.server_info().await {
            Ok((new_server_info, ping)) => {
                avg_ms = ping.as_millis() as f32 * (1.0 / attempt as f32)
                    + avg_ms * ((attempt as f32 - 1.0) / attempt as f32);
                server_info = Some(new_server_info);
            },
            Err(error) => {
                last_error = Some(error);
            },
        }
    }

    server_info
        .map(|info| (Duration::from_millis(avg_ms as u64), info))
        .ok_or_else(|| {
            last_error.expect(
                "There must have occurred some error if server_info is not Some()",
            )
        })
}
