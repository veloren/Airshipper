use crate::{
    Result,
    assets::POPPINS_LIGHT_FONT,
    consts,
    gui::{
        rss_feed::{
            RssFeedComponent, RssFeedComponentMessage, RssFeedData, RssFeedUpdateStatus,
            RssPost,
        },
        style::{button::ButtonStyle, container::ContainerStyle, text::TextStyle},
        views::default::{DefaultViewMessage, Interaction},
        widget::*,
    },
};
use iced::{
    Alignment, Command, ContentFit, Length,
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, image, image::Handle, scrollable, text},
};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NewsPanelComponent {
    posts: Vec<NewsPost>,
    etag: String,
}

#[derive(Clone, Debug)]
pub enum NewsPanelMessage {
    RssUpdate(RssFeedComponentMessage),
}

impl RssFeedComponent for NewsPanelComponent {
    const IMAGE_HEIGHT: u32 = 117;
    const NAME: &str = "news";

    fn store_feed(&mut self, rss_feed: RssFeedData) {
        self.posts = rss_feed
            .posts
            .into_iter()
            .map(|rss_post| NewsPost { rss_post })
            .collect();
        self.etag = rss_feed.etag;
    }

    fn posts(&self) -> Vec<RssPost> {
        self.posts.iter().map(|x| x.rss_post.clone()).collect()
    }

    fn posts_mut(&mut self) -> Vec<&mut RssPost> {
        self.posts.iter_mut().map(|x| &mut x.rss_post).collect()
    }

    fn image_fetched(url: String, result: Result<Handle>) -> DefaultViewMessage {
        DefaultViewMessage::NewsPanel(NewsPanelMessage::RssUpdate(
            RssFeedComponentMessage::ImageFetched { url, result },
        ))
    }
}
impl NewsPanelComponent {
    // 16:9 Aspect ratio
    const IMAGE_WIDTH: u32 = 208;

    pub fn etag(&self) -> &str {
        &self.etag
    }

    /// Returns new News in case remote one is newer
    pub(crate) async fn update_news(local_version: String) -> RssFeedUpdateStatus {
        RssFeedData::update_feed(
            consts::NEWS_URL,
            local_version,
            Self::NAME,
            Self::IMAGE_HEIGHT,
        )
        .await
    }

    pub fn update(
        &mut self,
        msg: NewsPanelMessage,
    ) -> Option<Command<DefaultViewMessage>> {
        match msg {
            NewsPanelMessage::RssUpdate(rss_msg) => self.handle_update(rss_msg),
        }
    }

    pub(crate) fn view(&self) -> Element<DefaultViewMessage> {
        let mut news = column![].spacing(20).padding(20);

        for post in &self.posts {
            news = news.push(post.view());
        }

        container(scrollable(news))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NewsPost {
    pub rss_post: RssPost,
}

impl NewsPost {
    pub(crate) fn view(&self) -> Element<DefaultViewMessage> {
        let post = &self.rss_post;

        let image_container = if let Some(handle) = &post.image {
            container(
                image(handle.clone())
                    .content_fit(ContentFit::Cover)
                    .width(Length::Fixed(NewsPanelComponent::IMAGE_WIDTH as f32))
                    .height(Length::Fixed(NewsPanelComponent::IMAGE_HEIGHT as f32)),
            )
        } else {
            container(
                text("Loading...")
                    .size(14)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .style(ContainerStyle::LoadingBlogPost)
        };

        button(
            column![]
                .push(
                    image_container
                        .width(Length::Fixed(NewsPanelComponent::IMAGE_WIDTH as f32))
                        .height(Length::Fixed(NewsPanelComponent::IMAGE_HEIGHT as f32)),
                )
                .push(
                    container(
                        column![]
                            .spacing(3)
                            .push(text("Development").size(12).style(TextStyle::Lilac))
                            .push(text(&post.title).size(16).font(POPPINS_LIGHT_FONT))
                            .push(text(&post.description).size(11).line_height(1.5)),
                    )
                    .width(Length::Fill)
                    .style(ContainerStyle::BlogPost)
                    .padding(8),
                )
                .align_items(Alignment::Center),
        )
        .on_press(DefaultViewMessage::Interaction(Interaction::OpenURL(
            post.button_url.clone(),
        )))
        .padding(0)
        .style(ButtonStyle::Transparent)
        .into()
    }
}
