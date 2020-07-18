use crate::{consts, gui::Message, net, Result};
use iced::{scrollable, Element};
use rss::Channel;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct News {
    posts: Vec<Post>,
    pub etag: String,

    #[serde(skip)]
    news_scrollable_state: scrollable::State,
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
            ..Default::default()
        })
    }

    /// Returns new News incase remote one is newer
    pub(crate) async fn update(version: String) -> Result<Option<Self>> {
        let remote_version = net::query_etag(consts::NEWS_URL).await?;
        if version != remote_version {
            return Ok(Some(Self::fetch().await?));
        } else {
            log::debug!("News up-to-date.");
        }
        Ok(None)
    }

    pub(crate) fn view(&mut self) -> Element<Message> {
        use crate::gui::style;
        use iced::{Container, Length, Scrollable};

        let mut news = Scrollable::new(&mut self.news_scrollable_state)
            .spacing(20)
            .padding(25);

        for post in &mut self.posts {
            news = news.push(post.view());
        }

        Container::new(news)
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

    #[serde(skip)]
    pub btn_state: iced::button::State,
}

impl Post {
    pub(crate) fn view(&mut self) -> Element<Message> {
        use crate::gui::widgets::{secondary_button, Interaction};
        use iced::{Column, Text};

        Column::new()
            .push(Text::new(&self.title).size(20))
            .push(Text::new(&self.description).size(16))
            .push(secondary_button(
                &mut self.btn_state,
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

            btn_state: Default::default(),
        }
    }
}
