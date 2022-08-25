use crate::{
    assets::POPPINS_LIGHT_FONT,
    consts,
    gui::{
        rss_feed::{
            RssFeedComponent, RssFeedComponentMessage, RssFeedData, RssFeedUpdateStatus,
            RssPost,
        },
        style::{
            BlogPostContainerStyle, LoadingBlogPostContainerStyle,
            TransparentButtonStyle, LILAC,
        },
        views::default::{DefaultViewMessage, Interaction},
    },
};
use iced::{
    alignment::{Horizontal, Vertical},
    pure::{button, column, container, image, scrollable, text, Element},
    ContentFit, Length,
};
use iced_native::{image::Handle, Alignment, Command};
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

    fn rss_post_update_command(&self, url: String) -> Command<DefaultViewMessage> {
        // TODO: All of this except the specific DefaultViewMessage is the same for every
        // RssComponent so could be better encapsulated within the RssFeedComponent trait.
        Command::perform(RssFeedData::fetch_image(url.to_owned()), move |img| {
            DefaultViewMessage::NewsPanel(NewsPanelMessage::RssUpdate(
                RssFeedComponentMessage::ImageFetched {
                    url: url.to_owned(),
                    result: img,
                },
            ))
        })
    }
}
impl NewsPanelComponent {
    pub fn etag(&self) -> &str {
        &self.etag
    }

    /// Returns new News in case remote one is newer
    pub(crate) async fn update_news(local_version: String) -> RssFeedUpdateStatus {
        RssFeedData::update_feed(consts::NEWS_URL, local_version).await
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
        let mut news = column().spacing(20).padding(20);

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

        let image_container = if let Some(bytes) = &post.image_bytes {
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
            .style(LoadingBlogPostContainerStyle)
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
                            .push(text(&post.title).size(20).font(POPPINS_LIGHT_FONT))
                            .push(text(&post.description).size(14)),
                    )
                    .width(Length::Fill)
                    .style(BlogPostContainerStyle)
                    .padding(8),
                )
                .align_items(Alignment::Center),
        )
        .on_press(DefaultViewMessage::Interaction(Interaction::OpenURL(
            post.button_url.clone(),
        )))
        .padding(0)
        .style(TransparentButtonStyle)
        .into()
    }
}
