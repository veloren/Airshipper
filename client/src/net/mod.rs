/// stream file download with progress tracking.
mod download;

mod client;

pub use client::*;
pub use download::*;

/// Returns the download url if a new version of airshipper has been released.
#[cfg(windows)]
pub async fn check_win_update() -> crate::Result<Option<String>> {
    use crate::{consts, net};
    use semver::Version;

    let resp = net::query(&format!("{}/download/latest", consts::UPDATE_SERVER)).await?;
    if resp.status().is_success() {
        let text = resp.text().await?;
        let lines = text.lines().take(2).collect::<Vec<&str>>();
        let (version, url) = (
            // Incase the remote version cannot be parsed we default to the current one.
            Version::parse(lines[0].trim()).unwrap_or_else(|_| {
                log::warn!("Ignoring corrupted remote version!");
                Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
            }),
            lines[1].trim(),
        );

        if version > Version::parse(env!("CARGO_PKG_VERSION")).unwrap() {
            Ok(Some(url.into()))
        } else {
            Ok(None)
        }
    } else {
        Err(format!(
            "Couldn't check for airshipper updates. Server returned: {}",
            resp.text().await?
        )
        .into())
    }
}
