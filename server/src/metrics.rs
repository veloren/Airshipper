use crate::Result;
use prometheus::{Encoder, IntCounterVec, Opts, Registry, TextEncoder};

/// Prometheus Metrics
pub struct Metrics {
    registry: Registry,
    downloads: IntCounterVec,
    uploads: IntCounterVec,
    artifact_uploads: IntCounterVec,
    http_routes_in: IntCounterVec,
}

impl Metrics {
    #[allow(clippy::result_large_err)]
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        let downloads = IntCounterVec::new(
            Opts::new(
                "downloads",
                "shows the number of requests which want to download Veloren by os and \
                 arch",
            ),
            &["os", "arch", "channel"],
        )?;
        let uploads = IntCounterVec::new(
            Opts::new(
                "uploads_total",
                "shows the number of requests which lead to an upload of new artifacts",
            ),
            &["channel"],
        )?;
        let artifact_uploads = IntCounterVec::new(
            Opts::new(
                "artifact_uploads",
                "shows the number of artficats that got updated",
            ),
            &["os", "arch", "channel"],
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
        registry.register(Box::new(artifact_uploads.clone()))?;
        registry.register(Box::new(http_routes_in.clone()))?;

        Ok(Self {
            registry,
            downloads,
            uploads,
            artifact_uploads,
            http_routes_in,
        })
    }

    pub fn increment_download(&self, os: &str, arch: &str, channel: &str) {
        self.downloads.with_label_values(&[os, arch, channel]).inc();
    }

    pub fn increment_upload(&self, channel: &str) {
        self.uploads.with_label_values(&[channel]).inc();
    }

    pub fn increment_artifact_upload(&self, os: &str, arch: &str, channel: &str) {
        self.artifact_uploads
            .with_label_values(&[os, arch, channel])
            .inc();
    }

    /// Returns statistics
    #[allow(clippy::result_large_err)]
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
