use crate::{consts, gui::views::default::DefaultViewMessage, net, Result};
use iced::{scrollable, Element};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Changelog {
    pub text: String,
    pub etag: String,

    #[serde(skip)]
    changelog_scrollable_state: scrollable::State,
}

impl Changelog {
    async fn fetch() -> Result<Self> {
        let changelog = net::query(consts::CHANGELOG_URL).await?;

        Ok(Changelog {
            etag: net::get_etag(&changelog),
            text: changelog
                .text()
                .await?
                .lines()
                .skip_while(|x| !x.contains(&"## [Unreleased]"))
                .skip(2)
                .take_while(|x| !x.contains(&"## [0.1.0]"))
                .map(|x| format!("{}\n", x))
                .collect(),
            ..Default::default()
        })
    }

    /// Returns new Changelog incase remote one is newer
    pub async fn update(version: String) -> Result<Option<Self>> {
        match net::query_etag(consts::CHANGELOG_URL).await? {
            Some(remote_version) => {
                if version != remote_version {
                    return Ok(Some(Self::fetch().await?));
                } else {
                    log::debug!("Changelog up-to-date.");
                    Ok(None)
                }
            },
            // We query the changelog incase there's no etag to be found
            // to make sure the player stays informed.
            None => Ok(Some(Self::fetch().await?)),
        }
    }

    pub fn view(&mut self) -> Element<DefaultViewMessage> {
        use iced::{Length, Scrollable, Text};

        Scrollable::new(&mut self.changelog_scrollable_state)
            .height(Length::Fill)
            .padding(15)
            .spacing(20)
            .push(Text::new(self.text.clone()).size(18))
            .into()
    }
}
