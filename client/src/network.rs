//! Takes care of all network operations

use crate::filesystem;
use crate::profiles::Profile;
use crate::Result;
use async_std::{fs::File, prelude::*};
use isahc::{config::RedirectPolicy, prelude::*};
use serde::{Deserialize, Serialize};
use std::io::BufReader;
use std::path::PathBuf;

pub const DOWNLOAD_SERVER: &str = "https://download.veloren.net";

const CHANGELOG_URL: &str = "https://gitlab.com/veloren/veloren/raw/master/CHANGELOG.md";
const NEWS_URL: &str = "https://veloren.net/rss.xml";

/// Use this method when making requests
/// it will include required defaults to make secure https requests.
pub async fn request<T: ToString>(url: T) -> Result<Response<isahc::Body>> {
    Ok(Request::get(url.to_string())
        .redirect_policy(RedirectPolicy::Follow)
        .timeout(std::time::Duration::from_secs(20))
        .header(
            "User-Agent",
            &format!("Airshipper/{}", env!("CARGO_PKG_VERSION")),
        )
        .body(())?
        .send()?)
}

/// Returns the remote version of the profile
pub async fn get_version(profile: &Profile) -> Result<String> {
    let mut resp = request(&get_version_uri(profile)).await?;
    if resp.status().is_success() {
        Ok(resp.text()?)
    } else {
        Err(format!(
            "Couldn't download version information. Server returned '{}'",
            resp.status()
        )
        .into())
    }
}

/// Starts a download of the zip to target directory
pub fn start_download(profile: &Profile) -> Result<isahc::Metrics> {
    log::info!("Downloading {} - {}", profile.name, profile.channel);

    std::fs::create_dir_all(&profile.directory)?;

    let mut response = Request::get(get_artifact_uri(&profile))
        .metrics(true)
        .redirect_policy(RedirectPolicy::Follow)
        .body(())?
        .send()?;

    let metrics = response.metrics().unwrap().clone();

    let zip_path = profile.directory.join(filesystem::DOWNLOAD_FILE);

    async_std::task::spawn(async move {
        let body = response.body_mut();
        let mut buffer = [0; 8000]; // 8KB
        // TODO: deal with this error!
        let mut file = File::create(&zip_path).await.expect("failed to create file for download!");

        loop {
            match body.read(&mut buffer).await {
                Ok(0) => {
                    log::info!("Download finished!");
                    break;
                }
                Ok(x) => {
                    file.write_all(&buffer[0..x])
                        .await
                        // TODO: deal with this error!
                        .expect("TODO: error handling");
                    for i in 0..x {
                        buffer[i] = 0;
                    }
                }
                Err(e) => {
                    log::error!("ERROR: {}", e);
                    break;
                }
            }
        }
    });
    Ok(metrics)
}

pub async fn compare_changelog_etag(cached: &str) -> Result<bool> {
    Ok(request(CHANGELOG_URL)
        .await?
        .headers()
        .get("etag")
        .map(|x| x.to_str().unwrap()) // Etag will always be a valid UTF-8 due to it being ASCII
        .unwrap_or("MissingEtag") // TODO: Decide whether to throw an error if etag does not exist
        != cached)
}

pub async fn compare_news_etag(cached: &str) -> Result<bool> {
    Ok(request(CHANGELOG_URL)
        .await?
        .headers()
        .get("etag")
        .map(|x| x.to_str().unwrap()) // Etag will always be a valid UTF-8 due to it being ASCII
        .unwrap_or("MissingEtag") // TODO: Decide whether to throw an error if etag does not exist
        != cached)
}

pub async fn query_changelog() -> Result<String> {
    Ok(request(CHANGELOG_URL)
        .await?
        .text()?
        .lines()
        .skip_while(|x| !x.contains(&"## [Unreleased]"))
        .skip(2)
        .take_while(|x| !x.contains(&"## [0.1.0]"))
        .map(|x| format!("{}\n", x))
        .collect())
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub title: String,
    pub description: String,
    pub button_url: String,

    #[serde(skip)]
    pub btn_state: iced::button::State,
}

/// Returns a list of Posts with title, description and button url.
pub async fn query_news() -> Result<Vec<Post>> {
    let mut response = isahc::get(NEWS_URL)?;
    let feed = rss::Channel::read_from(BufReader::new(response.body_mut()))?;
    let mut posts = Vec::new();

    for post in feed.items().iter().take(15) {
        // Only take the latest posts
        posts.push(Post {
            title: post.title().unwrap_or("Missing title").into(),
            description: process_description(post.description().unwrap_or("No description found.")),
            button_url: post.link().unwrap_or("https://www.veloren.net").into(),

            btn_state: Default::default(),
        });
    }

    Ok(posts)
}

fn process_description(post: &str) -> String {
    // TODO: Play with the width!
    let stripped_html = html2text::from_read(post.as_bytes(), 400)
        .lines()
        .take(3)
        .filter(|x| !x.contains("[banner]"))
        .map(|x| format!("{}\n", x))
        .collect::<String>();
    let stripped_markdown = strip_markdown::strip_markdown(&stripped_html);
    stripped_markdown
}

/// Unzips to target directory and changes permissions
pub async fn install(profile: &Profile) -> Result<()> {
    // Extract
    log::info!("Unzipping to {:?}", profile.directory);
    let mut zip_file = std::fs::File::open(&profile.directory.join(filesystem::DOWNLOAD_FILE))?;

    let mut archive = zip::ZipArchive::new(&mut zip_file)?;

    for i in 1..archive.len() {
        let mut file = archive.by_index(i)?;
        let path = profile.directory.join(file.sanitized_name());

        if file.is_dir() {
            std::fs::create_dir_all(path)?;
        } else {
            let mut target = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;

            std::io::copy(&mut file, &mut target)?;
        }
    }

    // Delete downloaded zip
    log::trace!("Extracted files, deleting zip archive.");
    std::fs::remove_file(profile.directory.join(filesystem::DOWNLOAD_FILE))?;

    #[cfg(unix)]
    set_permissions(vec![
        &profile.directory.join(filesystem::VOXYGEN_FILE),
        &profile.directory.join(filesystem::SERVER_CLI_FILE),
    ])?;

    Ok(())
}

fn get_version_uri(profile: &Profile) -> String {
    format!(
        "{}/version/{}/{}",
        DOWNLOAD_SERVER,
        whoami::platform(),
        profile.channel
    )
}
fn get_artifact_uri(profile: &Profile) -> String {
    format!(
        "{}/latest/{}/{}",
        DOWNLOAD_SERVER,
        whoami::platform(),
        profile.channel
    )
}

/// Tries to set executable permissions on linux
#[cfg(unix)]
fn set_permissions(files: Vec<&PathBuf>) -> Result<()> {
    use std::process::Command;
    for file in files {
        Command::new("chmod").arg("+x").arg(file).spawn()?;
    }
    Ok(())
}
