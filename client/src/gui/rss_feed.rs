use crate::{
    fs,
    gui::views::{default::DefaultViewMessage, Action},
    net, ClientError, Result,
};
use futures_util::future::join_all;
use iced::Command;
use rss::Channel;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

#[derive(Clone, Debug)]
pub enum RssFeedUpdateStatus {
    NoUpdateRequired,
    UpdateFailed(ClientError),
    Updated(RssFeedData),
}

#[derive(Clone, Debug)]
pub enum RssFeedComponentMessage {
    UpdateRssFeed(RssFeedUpdateStatus),
    ImageFetched {
        url: String,
        result: Result<Vec<u8>>,
    },
}

/// Allows a component to handle updates to an RSS feed that it owns
pub trait RssFeedComponent {
    /// Stores the feed against the component's own state
    fn store_feed(&mut self, rss_feed_data: RssFeedData);

    /// Returns the posts that the component has previously fetched from the RSS feed
    fn posts(&self) -> Vec<RssPost>;
    fn posts_mut(&mut self) -> Vec<&mut RssPost>;

    /// Triggers an update message when an RSS post is updated to signal to the view that
    /// it should refresh
    fn rss_post_update_command(&self, url: String) -> Command<DefaultViewMessage>;

    /// An optional hook that is called after the RSS feed is updated
    fn after_rss_feed_updated(&mut self) {}

    fn handle_update(
        &mut self,
        msg: RssFeedComponentMessage,
    ) -> Option<Command<DefaultViewMessage>> {
        match msg {
            RssFeedComponentMessage::UpdateRssFeed(status) => match status {
                RssFeedUpdateStatus::Updated(rss_feed_data) => {
                    self.store_feed(rss_feed_data);
                    self.after_rss_feed_updated();

                    Some(Command::perform(
                        async { Action::Save },
                        DefaultViewMessage::Action,
                    ))
                },
                RssFeedUpdateStatus::NoUpdateRequired => {
                    // On application startup when there's been no rss update since the
                    // last run the posts will have been de-serialized  without their
                    // image data (which we don't store in the state ron file) so we need
                    // to attempt to fetch the image for them, which
                    // should reside in the cache if it hasn't been
                    // cleared.
                    let posts = self.posts();
                    let commands: Vec<Command<DefaultViewMessage>> = posts
                        .iter()
                        .filter_map(|post| {
                            // Filter out posts that we already have the image for, or
                            // don't have an image URL
                            if post.image_bytes.is_some() || post.image_url.is_none() {
                                return None;
                            }
                            let url = post.image_url.as_ref().unwrap().to_owned();
                            Some(self.rss_post_update_command(url))
                        })
                        .collect();

                    self.after_rss_feed_updated();

                    debug!("Fetching images for {} cached blog posts", commands.len());
                    Some(Command::batch(commands))
                },
                RssFeedUpdateStatus::UpdateFailed(e) => {
                    error!(?e, "Failed to fetch RSS feed");
                    None
                },
            },
            RssFeedComponentMessage::ImageFetched { result, url } => {
                if let Ok(bytes) = result {
                    if let Some(post) = self
                        .posts_mut()
                        .iter_mut()
                        .filter(|post| post.image_url.is_some())
                        .find(|post| post.image_url.as_ref().unwrap() == &url)
                    {
                        post.image_bytes = Some(bytes);
                    }
                }

                None
            },
        }
    }
}

/// Represents the parsed data downloaded from an RSS feed URL, comprised of individual
/// posts and an etag used to check if the RSS feed has been updated since it was last
/// queried
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RssFeedData {
    pub posts: Vec<RssPost>,
    pub etag: String,
}

impl RssFeedData {
    pub async fn update_feed(
        feed_url: &str,
        local_version: String,
    ) -> RssFeedUpdateStatus {
        let fetch = move |local_version: String| async move {
            match net::query_etag(feed_url).await? {
                Some(remote_version) => {
                    if local_version != remote_version {
                        debug!(
                            ?feed_url,
                            "Local version {} does not match remote version {}, \
                             fetching feed",
                            local_version,
                            remote_version
                        );
                        Ok(RssFeedUpdateStatus::Updated(
                            RssFeedData::fetch(feed_url).await?,
                        ))
                    } else {
                        debug!(?feed_url, "RSS feed up-to-date.");
                        Ok(RssFeedUpdateStatus::NoUpdateRequired)
                    }
                },
                // If no etag was found, perform a full update
                None => {
                    warn!(
                        ?feed_url,
                        "No etag found for RSS feed, assuming an update is required."
                    );
                    Ok(RssFeedUpdateStatus::Updated(
                        RssFeedData::fetch(feed_url).await?,
                    ))
                },
            }
        };

        fetch(local_version)
            .await
            .unwrap_or_else(RssFeedUpdateStatus::UpdateFailed)
    }

    pub async fn fetch(feed_url: &str) -> Result<RssFeedData> {
        use std::io::BufReader;

        let feed_response = net::query(feed_url).await?;
        let etag = net::get_etag(&feed_response);
        let feed = Channel::read_from(BufReader::new(&feed_response.bytes().await?[..]))?;

        let futs = feed
            .items()
            .iter()
            // TODO: Currently we want 15 blog posts and 15 community showcase posts - if this is ever not the case 
            // then this number will need parameterising.
            .take(15)
            .map(move |item| async move {
                let mut post = RssPost::from(item);
                if let Some(url) = &post.image_url {
                    if let Ok(bytes) = RssFeedData::fetch_image(url.to_owned()).await {
                        post.image_bytes = Some(bytes);
                    }
                };
                post
            })
            .collect::<Vec<_>>();
        let posts = join_all(futs).await;

        Ok(RssFeedData { posts, etag })
    }

    /// Attempts to fetch an image for a given URL, retrieving it from the RSS image
    /// cache if possible, otherwise attempting to fetch it from the URL
    pub async fn fetch_image(url: String) -> Result<Vec<u8>> {
        let cache_base_path = fs::get_cache_path().join("rss_images");
        std::fs::create_dir_all(&cache_base_path).map_err(|_| ClientError::IoError)?;

        // Use an MD5 hash as the filename to avoid dealing with special characters and
        // long URLs that would make an unusable filename
        let md5 = &md5::compute(&url);
        let image_cache_path = cache_base_path.join(format!("{:?}", md5));

        if let Ok(cached_bytes) = std::fs::read(&image_cache_path) {
            // Found the image cached locally so use it
            debug!(
                "Retrieved cached image for URL {} from path {}",
                url,
                image_cache_path.to_string_lossy()
            );
            return Ok(cached_bytes);
        }

        match crate::net::client::WEB_CLIENT.get(&*url).send().await {
            Ok(response) => match response.bytes().await {
                Ok(bytes) => {
                    // Image successfully downloaded, write it to the cache before
                    // returning it
                    debug!(
                        "Caching image from URL {} with path {}",
                        url,
                        image_cache_path.to_string_lossy()
                    );
                    std::fs::write(&image_cache_path, &bytes)?;
                    Ok(bytes.to_vec())
                },
                Err(e) => {
                    error!("Failed to fetch bytes of RSS image from URL {}", url);
                    Err(e.into())
                },
            },
            Err(e) => {
                error!("Failed to download RSS image from URL {}", url);
                Err(e.into())
            },
        }
    }
}

/// An individual post parsed from an RSS feed
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RssPost {
    pub title: String,
    pub description: String,
    pub button_url: String,
    pub image_url: Option<String>,
    #[serde(skip)]
    pub image_bytes: Option<Vec<u8>>,
}

impl RssPost {
    fn process_description(desc: Option<&str>) -> String {
        match desc {
            Some(desc) => {
                let wrapped_html = html2text::from_read(desc.as_bytes(), 400);
                if let Ok(html) = wrapped_html {
                    let stripped_html = html
                        .lines()
                        .take(3)
                        .filter(|x| !x.contains("[banner]"))
                        .fold(String::new(), |mut output, b| {
                            use std::fmt::Write;
                            let _ = writeln!(output, "{b}");
                            output
                        });
                    strip_markdown::strip_markdown(&stripped_html)
                } else {
                    "HTML parsing failed.".into()
                }
            },
            None => "No description found.".into(),
        }
    }
}

impl From<&rss::Item> for RssPost {
    fn from(item: &rss::Item) -> Self {
        let mut post = RssPost {
            title: item.title().unwrap_or("Missing title").into(),
            description: Self::process_description(item.description()),
            button_url: item.link().unwrap_or("https://www.veloren.net").into(),
            image_url: None,
            image_bytes: None,
        };

        // If the RSS item has an enclosure (attached media), check if it's a jpg or png
        // and if it is store the URL against the post for display in the RSS
        // feed.
        if let Some(enclosure) = item.enclosure.as_ref().filter(|enclosure| {
            matches!(enclosure.mime_type.as_str(), "image/jpg" | "image/png")
        }) {
            let mut url = enclosure.url.clone();

            // If the image is hosted by the discord CDN, use its ability to provide a
            // resized image to save bandwidth
            if url.starts_with("https://media.discordapp.net") {
                url = format!("{}?width=320&height=240", url);
            };

            post.image_url = Some(url);
        }

        post
    }
}
