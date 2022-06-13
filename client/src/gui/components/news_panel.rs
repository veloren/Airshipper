use crate::{
    assets::POPPINS_LIGHT_FONT,
    consts, fs,
    gui::{
        style::{BlogPostContainerStyle, TransparentButtonStyle, LILAC},
        views::{
            default::{DefaultViewMessage, Interaction},
            Action,
        },
    },
    net, ClientError, Result,
};
use futures_util::future::join_all;
use iced::{
    alignment::{Horizontal, Vertical},
    pure::{button, column, container, image, scrollable, text, Element},
    ContentFit, Length,
};
use iced_native::{image::Handle, Alignment, Command};
use rss::Channel;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NewsPanelComponent {
    news: News,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct News {
    posts: Vec<Post>,
    etag: String,
}

#[derive(Clone, Debug)]
pub enum NewsPanelMessage {
    UpdateNews(NewsUpdateStatus),
    ImageFetched {
        url: String,
        result: Result<Vec<u8>>,
    },
}

#[derive(Clone, Debug)]
pub enum NewsUpdateStatus {
    NoUpdateRequired,
    UpdateFailed(ClientError),
    Updated(News),
}

impl NewsPanelComponent {
    pub fn etag(&self) -> &str {
        &self.news.etag
    }

    /// Tries to fetch the News
    async fn fetch() -> Result<News> {
        use std::io::BufReader;

        let news = net::query(consts::NEWS_URL).await?;
        let etag = net::get_etag(&news);
        let feed = Channel::read_from(BufReader::new(&news.bytes().await?[..]))?;

        let futs = feed
            .items()
            .iter()
            .take(15)
            .map(async move |item| {
                let mut post = Post::from(item);
                if let Some(url) = &post.image_url {
                    if let Ok(bytes) =
                        NewsPanelComponent::fetch_image(url.to_owned()).await
                    {
                        post.image_bytes = Some(bytes);
                    }
                };
                post
            })
            .collect::<Vec<_>>();
        let posts = join_all(futs).await;

        Ok(News { posts, etag })
    }

    /// Attempts to fetch an image for a given URL, retrieving it from the news image
    /// cache if possible, otherwise attempting to fetch it from the URL
    async fn fetch_image(url: String) -> Result<Vec<u8>> {
        let cache_base_path = fs::get_cache_path().join("news_images");
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
                    error!("Failed to fetch bytes of news image from URL {}", url);
                    Err(e.into())
                },
            },
            Err(e) => {
                error!("Failed to download news image from URL {}", url);
                Err(e.into())
            },
        }
    }

    /// Returns new News in case remote one is newer
    pub(crate) async fn update_news(version: String) -> NewsUpdateStatus {
        async fn fetch(version: String) -> Result<NewsUpdateStatus> {
            match net::query_etag(consts::NEWS_URL).await? {
                Some(remote_version) => {
                    if version != remote_version {
                        Ok(NewsUpdateStatus::Updated(
                            NewsPanelComponent::fetch().await?,
                        ))
                    } else {
                        debug!("News up-to-date.");
                        Ok(NewsUpdateStatus::NoUpdateRequired)
                    }
                },
                // We query the news in case there's no etag to be found
                // to make sure the player stays informed.
                None => Ok(NewsUpdateStatus::Updated(
                    NewsPanelComponent::fetch().await?,
                )),
            }
        }

        fetch(version)
            .await
            .unwrap_or_else(NewsUpdateStatus::UpdateFailed)
    }

    pub fn update(
        &mut self,
        msg: NewsPanelMessage,
    ) -> Option<Command<DefaultViewMessage>> {
        match msg {
            NewsPanelMessage::UpdateNews(status) => match status {
                NewsUpdateStatus::Updated(news) => {
                    self.news = news;
                    Some(Command::perform(
                        async { Action::Save },
                        DefaultViewMessage::Action,
                    ))
                },
                NewsUpdateStatus::NoUpdateRequired => {
                    // On application startup when there's been no news update since the
                    // last run the posts will have been de-serialized  without their
                    // image data (which we don't store in the state ron file) so we need
                    // to attempt to fetch the image for them, which
                    // should reside in the cache if it hasn't been
                    // cleared.
                    let commands: Vec<Command<DefaultViewMessage>> = self
                        .news
                        .posts
                        .iter_mut()
                        .filter_map(|post| {
                            // Filter out posts that we already have the image for, or
                            // don't have an image URL
                            if post.image_bytes.is_some() || post.image_url.is_none() {
                                return None;
                            }

                            let url = post.image_url.as_ref().unwrap().to_owned();
                            Some(Command::perform(
                                NewsPanelComponent::fetch_image(url.to_owned()),
                                move |img| {
                                    DefaultViewMessage::NewsPanel(
                                        NewsPanelMessage::ImageFetched {
                                            url: url.to_owned(),
                                            result: img,
                                        },
                                    )
                                },
                            ))
                        })
                        .collect();
                    debug!("Fetching images for {} cached blog posts", commands.len());
                    Some(Command::batch(commands))
                },
                NewsUpdateStatus::UpdateFailed(e) => {
                    error!(?e, "Failed to fetch news");
                    None
                },
            },
            NewsPanelMessage::ImageFetched { result, url } => {
                if let Ok(bytes) = result {
                    if let Some(mut post) = self
                        .news
                        .posts
                        .iter_mut()
                        .filter(|x| x.image_url.is_some())
                        .find(|post| post.image_url.as_ref().unwrap() == &url)
                    {
                        post.image_bytes = Some(bytes);
                    }
                }

                None
            },
        }
    }

    pub(crate) fn view(&self) -> Element<DefaultViewMessage> {
        let mut news = column().spacing(20).padding(20);

        for post in &self.news.posts {
            news = news.push(post.view());
        }

        container(scrollable(news))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Post {
    pub title: String,
    pub description: String,
    pub button_url: String,
    pub image_url: Option<String>,
    #[serde(skip)]
    pub image_bytes: Option<Vec<u8>>,
}

impl Post {
    pub(crate) fn view(&self) -> Element<DefaultViewMessage> {
        let image_container = if let Some(bytes) = &self.image_bytes {
            container(
                image(Handle::from_memory(bytes.clone())).content_fit(ContentFit::Cover),
            )
        } else {
            container(
                text("Loading...")
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
        };

        button(
            column()
                .push(
                    image_container
                        .width(Length::Units(211))
                        .height(Length::Units(119)),
                )
                .push(
                    container(
                        column()
                            .spacing(2)
                            .push(text("Development").size(16).color(LILAC))
                            .push(text(&self.title).size(20).font(POPPINS_LIGHT_FONT))
                            .push(text(&self.description).size(14)),
                    )
                    .width(Length::Fill)
                    .style(BlogPostContainerStyle)
                    .padding(8),
                )
                .align_items(Alignment::Center),
        )
        .on_press(DefaultViewMessage::Interaction(Interaction::OpenURL(
            self.button_url.clone(),
        )))
        .padding(0)
        .style(TransparentButtonStyle)
        .into()
    }

    fn process_description(desc: Option<&str>) -> String {
        match desc {
            Some(desc) => {
                let stripped_html = html2text::from_read(desc.as_bytes(), 400)
                    .lines()
                    .take(3)
                    .filter(|x| !x.contains("[banner]"))
                    .map(|x| format!("{}\n", x))
                    .collect::<String>();
                strip_markdown::strip_markdown(&stripped_html)
            },
            None => "No description found.".into(),
        }
    }
}

impl From<&rss::Item> for Post {
    fn from(item: &rss::Item) -> Self {
        let mut post = Post {
            title: item.title().unwrap_or("Missing title").into(),
            description: Self::process_description(item.description()),
            button_url: item.link().unwrap_or("https://www.veloren.net").into(),
            image_url: None,
            image_bytes: None,
        };

        // If the news item has an enclosure (attached media), check if it's a jpg or png
        // and if it is store the URL against the post for display in the news
        // feed.
        if let Some(enclosure) = &item.enclosure && matches!(enclosure.mime_type.as_str(), "image/jpg" | "image/png") {
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
