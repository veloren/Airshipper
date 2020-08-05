use crate::Result;
use prometheus::{Encoder, IntCounter, Registry, TextEncoder};

/// Prometheus Metrics
pub struct Metrics {
    registry: Registry,

    downloads_windows: IntCounter,
    downloads_linux: IntCounter,
    downloads_macos: IntCounter,
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

        registry.register(Box::new(downloads_windows.clone()))?;
        registry.register(Box::new(downloads_linux.clone()))?;
        registry.register(Box::new(downloads_macos.clone()))?;

        Ok(Self {
            registry,

            downloads_windows,
            downloads_linux,
            downloads_macos,
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
