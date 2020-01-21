use {
    crate::network,
    crate::profiles::Profile,
    async_std::prelude::*,
    directories,
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone)]
pub enum LoadError {
    FileError,
    FormatError,
}

#[derive(Debug, Clone)]
pub enum SaveError {
    DirectoryError,
    FileError,
    WriteError,
    FormatError,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SavedState {
    pub changelog: String,
    pub news: Vec<network::Post>,
    pub active_profile: Profile,
}

impl SavedState {
    pub async fn load() -> Result<SavedState, LoadError> {
        let mut contents = String::new();

        let mut file = async_std::fs::File::open(get_savedstate_path())
            .await
            .map_err(|_| LoadError::FileError)?;

        file.read_to_string(&mut contents)
            .await
            .map_err(|_| LoadError::FileError)?;

        ron::de::from_str(&contents).map_err(|_| LoadError::FormatError)
    }

    pub async fn save(self) -> Result<(), SaveError> {
        let ron = ron::ser::to_string(&self).map_err(|_| SaveError::FormatError)?;

        let path = get_savedstate_path();

        if let Some(dir) = path.parent() {
            async_std::fs::create_dir_all(dir)
                .await
                .map_err(|_| SaveError::DirectoryError)?;
        }

        {
            let mut file = async_std::fs::File::create(path)
                .await
                .map_err(|_| SaveError::FileError)?;

            file.write_all(ron.as_bytes())
                .await
                .map_err(|_| SaveError::WriteError)?;
        }

        // This is a simple way to save at most once every couple seconds
        async_std::task::sleep(std::time::Duration::from_secs(2)).await;

        Ok(())
    }
}

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
