use {
    crate::filesystem,
    crate::network,
    crate::profiles::Profile,
    crate::Result,
    async_std::prelude::*,
    serde::{Deserialize, Serialize},
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
    pub changelog: String,
    /// Compare this to decide whether to update the saved state
    pub changelog_etag: String,
    pub news: Vec<network::Post>,
    /// Compare this to decide whether to update the saved state
    pub news_etag: String,
    pub active_profile: Profile,
}

impl SavedState {
    pub async fn load() -> Result<SavedState> {
        let mut contents = String::new();

        let mut file = async_std::fs::File::open(filesystem::get_savedstate_path()).await?;
        file.read_to_string(&mut contents).await?;

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

        // This is a simple way to save at most once every couple seconds
        async_std::task::sleep(std::time::Duration::from_secs(2)).await;

        Ok(())
    }
}
