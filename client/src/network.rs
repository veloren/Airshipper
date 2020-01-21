use crate::profiles::Profile;
use crate::Result;
use async_std::{fs::File, prelude::*};
use isahc::{config::RedirectPolicy, prelude::*};
use reqwest::header::*;
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(windows)]
pub const DOWNLOAD_FILE: &str = "veloren.zip";
#[cfg(unix)]
pub const DOWNLOAD_FILE: &str = "veloren";

// Maybe move this over to downloadbar ?
lazy_static::lazy_static! {
    pub static ref CLIENT: Client = {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            format!("Airshipper/{} ({})", env!("CARGO_PKG_VERSION"), whoami::os())
                .parse()
                .unwrap(),
        );
        ClientBuilder::new()
            .connect_timeout(std::time::Duration::from_secs(30))
            .default_headers(headers)
            .build().expect("FATAL: failed to build reqwest client!")
    };
}

/// Returns the remote version of the profile
pub fn get_newest_version_name(profile: &Profile) -> Result<String> {
    let mut resp = CLIENT.get(&get_version_uri(profile)).send()?;
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
pub fn start_download(profile: &Profile) -> Result<(isahc::Metrics, PathBuf)> {
    log::info!("Downloading {} - {}", profile.name, profile.channel);

    // Download
    std::fs::create_dir_all(&profile.directory)?;

    let mut response = Request::get(get_artifact_uri(&profile))
        .metrics(true)
        .redirect_policy(RedirectPolicy::Follow)
        .body(())
        .expect("error handling")
        .send()
        .expect("error handling");

    let metrics = response.metrics().unwrap().clone();

    let zip_path = profile.directory.join(DOWNLOAD_FILE);
    let zip_path_clone = zip_path.clone();

    async_std::task::spawn(async move {
        let body = response.body_mut();
        let mut buffer = [0; 8000]; // 8KB
        let mut file = File::create(&zip_path_clone)
            .await
            .expect("TODO: error handling");

        loop {
            match body.read(&mut buffer).await {
                Ok(0) => {
                    log::info!("Download finished!");
                    break;
                }
                Ok(x) => {
                    file.write_all(&buffer[0..x])
                        .await
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
    Ok((metrics, zip_path.to_owned()))
}

pub async fn query_changelog() -> Result<String> {
    Ok(
        Request::get("https://gitlab.com/veloren/veloren/raw/master/CHANGELOG.md")
            .body(())
            .expect("Error handling!")
            .send()
            .expect("Error handling!")
            .text()
            .expect("Non UTF-8 Text :/")
            .lines()
            .skip_while(|x| !x.contains(&"## [Unreleased]"))
            .skip(2)
            .take_while(|x| !x.contains(&"## [0.1.0]"))
            .map(|x| format!("{}\n", x))
            .collect(),
    )
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
    let feed = rss::Channel::from_url("https://veloren.net/rss.xml").expect("Error handling timo!");
    let mut posts = Vec::new();

    for post in feed.items() {
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
    // TODO: Check if removing markdown before html is better!
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
pub async fn install(profile: &Profile, zip_path: PathBuf) -> Result<()> {
    // Extract
    log::info!("Unzipping to {:?}", profile.directory);
    let mut zip_file = std::fs::File::open(&zip_path)?;

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
    std::fs::remove_file(profile.directory.join(DOWNLOAD_FILE))?;

    #[cfg(unix)]
    set_permissions(vec![
        &profile.directory.join(crate::VOXYGEN_FILE),
        &profile.directory.join(crate::SERVER_CLI_FILE),
    ])?;

    Ok(())
}

fn get_version_uri(profile: &Profile) -> String {
    format!(
        "{}/version/{}/{}",
        profile.base_server_url,
        whoami::platform(),
        profile.channel
    )
}
fn get_artifact_uri(profile: &Profile) -> String {
    format!(
        "{}/latest/{}/{}",
        profile.base_server_url,
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
