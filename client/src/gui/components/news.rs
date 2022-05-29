use crate::{
    assets::{HAXRCORP_4089_FONT, HAXRCORP_4089_FONT_SIZE_2},
    consts,
    gui::views::default::{DefaultViewMessage, Interaction},
    net, Result,
};
use iced::pure::{column, container, scrollable, text, Element};
use rss::Channel;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct News {
    posts: Vec<Post>,
    pub etag: String,
}

impl News {
    /// Tries to fetch the News
    async fn fetch() -> Result<Self> {
        use std::io::BufReader;

        let news = net::query(consts::NEWS_URL).await?;
        let etag = net::get_etag(&news);
        let feed = Channel::read_from(BufReader::new(&news.bytes().await?[..]))?;

        Ok(News {
            posts: feed.items().iter().take(15).map(Post::from).collect(),
            etag,
        })
    }

    /// Returns new News in case remote one is newer
    pub(crate) async fn update(version: String) -> Result<Option<Self>> {
        match net::query_etag(consts::NEWS_URL).await? {
            Some(remote_version) => {
                if version != remote_version {
                    return Ok(Some(Self::fetch().await?));
                } else {
                    tracing::debug!("News up-to-date.");
                    Ok(None)
                }
            },
            // We query the news incase there's no etag to be found
            // to make sure the player stays informed.
            None => Ok(Some(Self::fetch().await?)),
        }
    }

    pub(crate) fn view(&self) -> Element<DefaultViewMessage> {
        use crate::gui::style;
        use iced::Length;

        let mut news = column().spacing(20).padding(25);

        for post in &self.posts {
            news = news.push(post.view());
        }

        container(scrollable(news))
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .style(style::News)
            .into()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Post {
    pub title: String,
    pub description: String,
    pub button_url: String,
}

impl Post {
    pub(crate) fn view(&self) -> Element<DefaultViewMessage> {
        use crate::gui::views::default::secondary_button;

        column()
            .push(
                text(&self.title)
                    .font(HAXRCORP_4089_FONT)
                    .size(HAXRCORP_4089_FONT_SIZE_2),
            )
            .push(text(&self.description).size(18))
            .push(secondary_button(
                "Read More",
                Interaction::ReadMore(self.button_url.clone()),
            ))
            .spacing(8)
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
    fn from(post: &rss::Item) -> Self {
        Post {
            title: post.title().unwrap_or("Missing title").into(),
            description: Self::process_description(post.description()),
            button_url: post.link().unwrap_or("https://www.veloren.net").into(),
        }
    }
}
