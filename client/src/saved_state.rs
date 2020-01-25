use {
    crate::network,
    crate::profiles::Profile,
    crate::Result,
    async_std::prelude::*,
    directories,
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

        let mut file = async_std::fs::File::open(get_savedstate_path()).await?;
        file.read_to_string(&mut contents).await?;

        Ok(ron::de::from_str(&contents)?)
    }

    pub async fn save(self) -> Result<()> {
        let ron = ron::ser::to_string(&self)?;

        let path = get_savedstate_path();

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

/// TODO: Rewrite it as such that it will choose either
/// the cwd or standard path based on where it is (e.g installed in /usr/bin, or laying around in /home/...)
pub fn get_savedstate_path() -> std::path::PathBuf {
    let mut path = if let Some(project_dirs) =
        directories::ProjectDirs::from("net", "veloren", "airshipper")
    {
        project_dirs.config_dir().into()
    } else {
        std::env::current_dir().unwrap_or(std::path::PathBuf::new())
    };

    path.push("saved_state.ron");
    path
}

/// TODO: Rewrite it as such that it will choose either
/// the cwd or standard path based on where it is (e.g installed in /usr/bin, or laying around in /home/...)
pub fn get_profiles_path() -> std::path::PathBuf {
    let mut path = if let Some(project_dirs) =
        directories::ProjectDirs::from("net", "veloren", "airshipper")
    {
        project_dirs.data_dir().into()
    } else {
        std::env::current_dir().unwrap_or(std::path::PathBuf::new())
    };

    path.push("profiles");
    path
}
