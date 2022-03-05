use crate::Result;
use prometheus::{Encoder, IntCounter, IntCounterVec, Opts, Registry, TextEncoder};

/// Prometheus Metrics
pub struct Metrics {
    registry: Registry,
    downloads: IntCounterVec,
    pub uploads: IntCounter,
    http_routes_in: IntCounterVec,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        let downloads = IntCounterVec::new(
            Opts::new(
                "downloads",
                "shows the number of requests which want to download Veloren by os and \
                 arch",
            ),
            &["os", "arch"],
        )?;
        let uploads = IntCounter::new(
            "uploads_total",
            "shows the number of requests which lead to an upload of new artifacts",
        )?;
        let http_routes_in = IntCounterVec::new(
            Opts::new(
                "http_routes_in_total",
                "shows the number of requests per each route",
            ),
            &["http"],
        )?;

        registry.register(Box::new(downloads.clone()))?;
        registry.register(Box::new(uploads.clone()))?;
        registry.register(Box::new(http_routes_in.clone()))?;

        Ok(Self {
            registry,
            downloads,
            uploads,
            http_routes_in,
        })
    }

    /// Will try to identify the platform and increment the respective downloads
    pub fn increment(&self, os: &str, arch: &str) {
        self.downloads.with_label_values(&[os, arch]).inc();
    }

    /// Returns statistics
    pub fn gather(&self) -> Result<String> {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer)?;

        Ok(String::from_utf8(buffer).unwrap())
    }
}

use rocket::{
    fairing::{Fairing, Info, Kind},
    request::Request,
    response::Response,
};
#[crate::async_trait]
impl Fairing for Metrics {
    fn info(&self) -> Info {
        Info {
            name: "Prometheus metric collection",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, _: &mut Response<'r>) {
        if let Some(route) = request.route() {
            let endpoint = route.uri.as_str();
            self.http_routes_in.with_label_values(&[endpoint]).inc();
        }
    }
}
