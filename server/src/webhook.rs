use crate::{
    config::GithubReleaseConfig, models::Artifact, FsStorage, Result,
    ServerError::OctocrabError,
};
use octocrab::{models::repos::Release, GitHubError, Octocrab};
use reqwest::Url;
use serde::Deserialize;

pub fn process(artifacts: Vec<Artifact>, mut db: crate::DbConnection) {
    tokio::spawn(async move {
        for artifact in artifacts {
            if let Err(e) = transfer(artifact, &mut db).await {
                tracing::error!("Failed to transfer artifact: {}.", e);
            }
        }
        if let Err(e) = crate::prune::prune(&mut db).await {
            tracing::error!("Pruning failed: {}.", e);
        }
    });
}

#[tracing::instrument(skip(db))]
async fn transfer(artifact: Artifact, db: &mut crate::DbConnection) -> Result<()> {
    use tokio::{fs::File, io::AsyncWriteExt};

    tracing::info!("Downloading...");

    let mut resp = reqwest::get(&artifact.get_url()).await?;
    let mut file = File::create(&artifact.file_name).await?;
    let mut content = vec![];
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
        content.write_all(&chunk).await?;
    }
    file.sync_data().await?;

    let hash = format!("{:x}", md5::compute(content));
    let remote_hash = get_remote_hash(&resp);

    if hash != remote_hash {
        tracing::error!(
            "Downloaded file has '{}' MD5 hash while remote hash is '{}'. Exiting...",
            hash,
            remote_hash
        );
        // Clean up
        tokio::fs::remove_file(&artifact.file_name).await?;
    } else {
        tracing::debug!("Computed hash: {}, remote_hash: {}", hash, remote_hash);
        tracing::info!("Storing...");

        FsStorage::store(&artifact).await?;

        if let Some(github_release_config) = &crate::CONFIG.github_release_config {
            let upload_to_github_result =
                upload_to_github_release(&artifact.file_name, github_release_config)
                    .await;
            if let Err(e) = upload_to_github_result {
                tracing::error!(?e, "Couldn't upload to github");
            }
        }

        // Update database with new information
        tracing::info!("hash valid. Update database...");
        db.insert_artifact(&artifact).await?;

        // Delete obselete artifact
        tokio::fs::remove_file(&artifact.file_name).await?;
    }
    Ok(())
}

fn get_remote_hash(resp: &reqwest::Response) -> String {
    resp.headers()
        .get(reqwest::header::ETAG)
        .map(|x| x.to_str().expect("always valid ascii?"))
        .unwrap_or("REMOTE_ETAG_MISSING")
        .replace('\"', "")
}

async fn upload_to_github_release(
    file_name: &str,
    github_release_config: &GithubReleaseConfig,
) -> Result<Url> {
    let octocrab = Octocrab::builder()
        .personal_token(github_release_config.github_token.clone())
        .build()?;
    let release = get_github_release(&octocrab, github_release_config).await?;

    //Remove extra %7B?name,label} in the url path.
    //This is required because the github API returns {?name,label}
    //at the end of the upload url, which needs to be removed before
    //using the url.
    let url = release.upload_url.as_str();
    let stripped_url = url.strip_suffix("%7B?name,label}").unwrap_or(url);
    let mut new_url = Url::parse(stripped_url)?;

    //Taken from https://github.com/XAMPPRocky/octocrab/issues/96#issuecomment-863002976
    new_url.set_query(Some(format!("{}={}", "name", file_name).as_str()));

    let file_size = std::fs::metadata(file_name)?.len();
    let file = tokio::fs::File::open(file_name).await?;
    let stream =
        tokio_util::codec::FramedRead::new(file, tokio_util::codec::BytesCodec::new());
    let body = reqwest::Body::wrap_stream(stream);

    let builder = octocrab
        .request_builder(new_url.as_str(), reqwest::Method::POST)
        .header("Content-Type", "application/octet-stream")
        .header("Content-Length", file_size.to_string());

    #[derive(Deserialize)]
    struct DownloadUrl {
        browser_download_url: String,
    }

    let response = builder
        .body(body)
        .send()
        .await?
        .json::<DownloadUrl>()
        .await?;

    let download_url = Url::parse(&response.browser_download_url)?;

    Ok(download_url)
}

///Gets the github release set in config if the release exists, otherwise creates and
/// returns it.
async fn get_github_release(
    octocrab: &Octocrab,
    github_release_config: &GithubReleaseConfig,
) -> Result<Release> {
    let repo_get_result = octocrab
        .repos(
            &github_release_config.github_repository_owner,
            &github_release_config.github_repository,
        )
        .releases()
        .get_by_tag(&github_release_config.github_release)
        .await;

    let repo_result = match repo_get_result {
        Ok(release) => Ok(release),
        Err(octocrab::Error::GitHub {
            source: GitHubError { message, .. },
            ..
        }) if message == "Not Found" => octocrab
            .repos(
                &github_release_config.github_repository_owner,
                &github_release_config.github_repository,
            )
            .releases()
            .create(&github_release_config.github_release)
            .send()
            .await
            .map_err(OctocrabError),
        err => err.map_err(OctocrabError),
    };

    repo_result
}
