//! State which is used by the command line and GUI and also gets saved to disk

use crate::{
    fs,
    gui::widgets::{Changelog, News},
    profiles::Profile,
    Result,
};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, prelude::*};

/// Stores/Caches data needed
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
    pub changelog: Changelog,
    pub news: News,
    pub active_profile: Profile,
}

impl SavedState {
    pub fn empty() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub async fn load() -> Result<Self> {
        let mut contents = String::new();

        match File::open(fs::savedstate_file()).await {
            Ok(mut file) => {
                file.read_to_string(&mut contents).await?;
            },
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    log::debug!("saved state not found. Fallback to default!");
                    return Ok(SavedState::default());
                },
                _ => {
                    log::error!("saved state invalid!");
                    return Err(e.into());
                },
            },
        }
        Ok(ron::de::from_str(&contents)?)
    }

    pub async fn save(self) -> Result<()> {
        let ron = ron::ser::to_string_pretty(&self, ron::ser::PrettyConfig::default())?;

        let path = fs::savedstate_file();
        if let Some(dir) = path.parent() {
            tokio::fs::create_dir_all(dir).await?;
        }

        let mut file = File::create(path).await?;
        file.write_all(ron.as_bytes()).await?;
        file.sync_all().await?;
        Ok(())
    }
}
