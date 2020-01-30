//! State which is used by the command line and GUI and also gets saved to disk

use crate::{filesystem, network, profiles::Profile, Result};
use async_std::prelude::*;
use serde::{Deserialize, Serialize};

/// Stores/Caches data needed
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub changelog: String,
    /// Compare this to decide whether to update the saved state
    pub changelog_etag: String,
    pub news: Vec<network::Post>,
    /// Compare this to decide whether to update the saved state
    pub news_etag: String,
    pub active_profile: Profile,
}

impl State {
    pub async fn install_profile(&mut self) -> Result<()> {
        self.active_profile = self.active_profile.clone().install().await?;
        Ok(())
    }

    pub async fn update_profile(&mut self) -> Result<isahc::Metrics> {
        self.active_profile.start_download()
    }

    pub async fn check_for_profile_update(&mut self) -> Result<String> {
        self.active_profile.check_for_update().await
    }

    pub async fn start_profile(&mut self) -> Result<()> {
        self.active_profile.start()
    }

    pub async fn load() -> Result<Self> {
        let mut contents = String::new();

        match async_std::fs::File::open(filesystem::get_savedstate_path()).await {
            Ok(mut file) => {
                file.read_to_string(&mut contents).await?;
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    log::debug!("saved state not found. Fallback to default!");
                    return Ok(State::default());
                }
                _ => {
                    log::error!("saved state invalid!");
                    return Err(e.into());
                }
            },
        }
        Ok(ron::de::from_str(&contents)?)
    }

    pub async fn save(self) -> Result<()> {
        let ron = ron::ser::to_string(&self)?;

        let path = filesystem::get_savedstate_path();
        if let Some(dir) = path.parent() {
            async_std::fs::create_dir_all(dir).await?;
        }

        let mut file = async_std::fs::File::create(path).await?;
        file.write_all(ron.as_bytes()).await?;
        Ok(())
    }
}
