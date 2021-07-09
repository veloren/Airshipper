use crate::Result;
use prometheus::{Encoder, IntCounter, IntCounterVec, Opts, Registry, TextEncoder};

/// Prometheus Metrics
pub struct Metrics {
    registry: Registry,

    downloads_windows: IntCounter,
    downloads_linux: IntCounter,
    downloads_macos: IntCounter,
    pub uploads: IntCounter,

    http_routes_in: IntCounterVec,
}

impl Metrics {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        let downloads_windows = IntCounter::new(
            "windows_downloads",
            "shows the number of requests which want to download Veloren for Windows",
        )?;
        let downloads_linux = IntCounter::new(
            "linux_downloads",
            "shows the number of requests which want to download Veloren for Linux",
        )?;
        let downloads_macos = IntCounter::new(
            "macos_downloads",
            "shows the number of requests which want to download Veloren for MacOS",
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

        registry.register(Box::new(downloads_windows.clone()))?;
        registry.register(Box::new(downloads_linux.clone()))?;
        registry.register(Box::new(downloads_macos.clone()))?;
        registry.register(Box::new(uploads.clone()))?;
        registry.register(Box::new(http_routes_in.clone()))?;

        Ok(Self {
            registry,

            downloads_windows,
            downloads_linux,
            downloads_macos,
            uploads,

            http_routes_in,
        })
    }

    /// Will try to identify the platform and increment the respective downloads
    ///
    /// Note: Will ignore unknown platforms
    pub fn increment(&self, platform: &str) {
        match platform.to_lowercase().as_ref() {
            "windows" => self.downloads_windows.inc(),
            "linux" => self.downloads_linux.inc(),
            "macos" => self.downloads_macos.inc(),
            _ => {},
        }
    }

    // Returns statistics
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
            self.http_routes_in.with_label_values(&[&endpoint]).inc();
        }
    }
}
