// Filesystem

#[cfg(windows)]
pub const DOWNLOAD_FILE: &str = "veloren.zip";
#[cfg(unix)]
pub const DOWNLOAD_FILE: &str = "veloren";

#[cfg(windows)]
pub const VOXYGEN_FILE: &str = "veloren-voxygen.exe";
#[cfg(unix)]
pub const VOXYGEN_FILE: &str = "veloren-voxygen";

//#[cfg(windows)]
//pub const SERVER_CLI_FILE: &str = "veloren-server-cli.exe";
#[cfg(unix)]
pub const SERVER_CLI_FILE: &str = "veloren-server-cli";

pub const SAVED_STATE_FILE: &str = "airshipper_state.ron";
pub const LOG_FILE: &str = "airshipper.log";

// Networking

pub const DOWNLOAD_SERVER: &str = "https://download.veloren.net";

// For querying
pub const CHANGELOG_URL: &str =
    "https://gitlab.com/veloren/veloren/raw/master/CHANGELOG.md";
// For user linking
pub const CHANGELOG_URL_LINK: &str =
    "https://gitlab.com/veloren/veloren/-/blob/master/CHANGELOG.md";
pub const NEWS_URL: &str = "https://veloren.net/rss.xml";
