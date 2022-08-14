// Filesystem

#[cfg(windows)]
pub const DOWNLOAD_FILE: &str = "veloren.zip";
#[cfg(unix)]
pub const DOWNLOAD_FILE: &str = "veloren";

#[cfg(windows)]
pub const VOXYGEN_FILE: &str = "veloren-voxygen.exe";
#[cfg(unix)]
pub const VOXYGEN_FILE: &str = "veloren-voxygen";

#[cfg(windows)]
pub const LOGS_DIR: &str = "userdata\\voxygen\\logs";

#[cfg(unix)]
pub const LOGS_DIR: &str = "userdata/voxygen/logs";

//#[cfg(windows)]
//pub const SERVER_CLI_FILE: &str = "veloren-server-cli.exe";
#[cfg(unix)]
pub const SERVER_CLI_FILE: &str = "veloren-server-cli";

pub const SAVED_STATE_FILE: &str = "airshipper_state.ron";
pub const LOG_FILE: &str = "airshipper.log";

// Networking

// For querying
pub const CHANGELOG_URL: &str =
    "https://gitlab.com/veloren/veloren/raw/nightly/CHANGELOG.md";
// For user linking
pub const NEWS_URL: &str = "https://veloren.net/rss.xml";

pub const COMMUNITY_SHOWCASE_URL: &str = "https://veloren.net/community-showcase/rss.xml";

pub const GITLAB_MERGED_MR_URL: &str =
    "https://gitlab.com/veloren/veloren/-/merge_requests?scope=all&state=merged";
