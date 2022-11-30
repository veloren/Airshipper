use crate::{
    consts,
    gui::{
        custom_widgets::heading_with_rule,
        rss_feed::{
            RssFeedComponent, RssFeedComponentMessage, RssFeedData, RssFeedUpdateStatus,
            RssPost,
        },
        style::{
            LoadingBlogPostContainerStyle, NextPrevTextButtonStyle,
            TransparentButtonStyle,
        },
        views::default::{DefaultViewMessage, Interaction},
    },
};
use iced::{
    alignment::{Horizontal, Vertical},
    pure::{button, column, container, row, text, Element},
    ContentFit, Length, Padding,
};
use iced_native::{image::Handle, widget::Image, Command};
use rand::{prelude::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct CommunityShowcaseComponent {
    posts: Vec<CommunityPost>,
    etag: String,
    offset: usize,
}

#[derive(Clone, Debug)]
pub enum PostOffsetChange {
    Increment,
    Decrement,
}

#[derive(Clone, Debug)]
pub enum CommunityShowcasePanelMessage {
    RssUpdate(RssFeedComponentMessage),
    PostOffsetChange(PostOffsetChange),
}

impl RssFeedComponent for CommunityShowcaseComponent {
    fn store_feed(&mut self, rss_feed: RssFeedData) {
        self.posts = rss_feed
            .posts
            .into_iter()
            .map(|rss_post| CommunityPost { rss_post })
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
            DefaultViewMessage::CommunityShowcasePanel(
                CommunityShowcasePanelMessage::RssUpdate(
                    RssFeedComponentMessage::ImageFetched {
                        url: url.to_owned(),
                        result: img,
                    },
                ),
            )
        })
    }

    fn after_rss_feed_updated(&mut self) {
        // Shuffle Community Showcase posts each time they're loaded so that users
        // see different posts even if they never click the next/prev buttons.
        self.posts.shuffle(&mut thread_rng());
    }
}

impl CommunityShowcaseComponent {
    pub fn etag(&self) -> &str {
        &self.etag
    }

    /// Returns new Community Showcase Posts in case remote one is newer
    pub(crate) async fn update_community_posts(
        local_version: String,
    ) -> RssFeedUpdateStatus {
        RssFeedData::update_feed(consts::COMMUNITY_SHOWCASE_URL, local_version).await
    }

    pub fn update(
        &mut self,
        msg: CommunityShowcasePanelMessage,
    ) -> Option<Command<DefaultViewMessage>> {
        match msg {
            CommunityShowcasePanelMessage::RssUpdate(rss_msg) => {
                self.handle_update(rss_msg)
            },
            CommunityShowcasePanelMessage::PostOffsetChange(post_offset_change) => {
                match post_offset_change {
                    PostOffsetChange::Increment => {
                        self.offset = min(self.offset + 1, self.posts.len() - 1);
                    },
                    PostOffsetChange::Decrement => {
                        self.offset = (self.offset - 1).clamp(0, self.posts.len() - 1)
                    },
                };

                None
            },
        }
    }

    pub fn view(&self) -> Element<DefaultViewMessage> {
        let current_post = if let Some(post) = self.posts.get(self.offset) {
            container(post.view()).width(Length::Fill)
        } else {
            container(text("Nothing to show"))
        };

        let mut prev_button = button("<< Prev").style(NextPrevTextButtonStyle);
        if self.offset > 0 {
            prev_button =
                prev_button.on_press(DefaultViewMessage::CommunityShowcasePanel(
                    CommunityShowcasePanelMessage::PostOffsetChange(
                        PostOffsetChange::Decrement,
                    ),
                ));
        }

        let mut next_button = button("Next >>").style(NextPrevTextButtonStyle);
        if self.offset < max(self.posts.len(), 1) - 1 {
            next_button =
                next_button.on_press(DefaultViewMessage::CommunityShowcasePanel(
                    CommunityShowcasePanelMessage::PostOffsetChange(
                        PostOffsetChange::Increment,
                    ),
                ));
        }

        column()
            .push(heading_with_rule("Community Showcase"))
            .push(
                container(
                    column().push(current_post).push(
                        row()
                            .push(prev_button)
                            .width(Length::Shrink)
                            .push(container(" ").width(Length::Fill))
                            .push(next_button)
                            .width(Length::Shrink),
                    ),
                )
                .width(Length::Fill)
                .padding(Padding::from([10, 20])),
            )
            .into()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CommunityPost {
    pub rss_post: RssPost,
}

impl CommunityPost {
    pub(crate) fn view(&self) -> Element<DefaultViewMessage> {
        let post = &self.rss_post;

        // TODO: Tooltip with post description once Iced supports tooltip layering
        // correctly. Currently tooltips get mixed up with the changelog text
        // because Iced doesn't support multiple layers correctly.
        let image_container = if let Some(bytes) = &post.image_bytes {
            container(
                Image::new(Handle::from_memory(bytes.clone()))
                    .content_fit(ContentFit::Cover),
            )
            .height(Length::Units(180))
        } else {
            container(text("Loading..."))
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .style(LoadingBlogPostContainerStyle)
                .height(Length::Units(180))
                .width(Length::Fill)
        };
        button(image_container)
            .style(TransparentButtonStyle)
            .on_press(DefaultViewMessage::Interaction(Interaction::OpenURL(
                post.button_url.clone(),
            )))
            .width(Length::Fill)
            .into()
    }
}
