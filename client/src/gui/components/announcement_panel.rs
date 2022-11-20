use crate::{
    assets::{POPPINS_MEDIUM_FONT, UP_RIGHT_ARROW_ICON},
    consts::{AIRSHIPPER_RELEASE_URL, SUPPORTED_SERVER_API_VERSION},
    gui::{
        style::{
            button::ButtonStyle, container::ContainerStyle, text::TextStyle,
            AirshipperTheme,
        },
        views::{
            default::{DefaultViewMessage, Interaction},
            Action,
        },
    },
    net,
    profiles::Profile,
    Result,
};
use iced::{
    alignment::Vertical,
    widget::{button, column, container, image, image::Handle, row, text, Text},
    Alignment, Command, Element, Length, Padding, Renderer,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Clone, Debug)]
pub enum AnnouncementPanelMessage {
    UpdateAnnouncement(Result<Option<AnnouncementPanelComponent>>),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AnnouncementPanelComponent {
    pub announcement_message: Option<String>,
    pub announcement_last_change: chrono::DateTime<chrono::Utc>,
    pub api_version: u32,
}

impl AnnouncementPanelComponent {
    #[allow(clippy::while_let_on_iterator)]
    async fn fetch(profile: &Profile) -> Result<Self> {
        #[derive(Deserialize)]
        pub struct Version {
            version: u32,
        }

        #[derive(Deserialize)]
        pub struct Announcement {
            message: Option<String>,
            last_change: chrono::DateTime<chrono::Utc>,
        }

        let version = net::query(profile.api_version_url())
            .await?
            .json::<Version>()
            .await?;
        let announcement = net::query(profile.announcement_url())
            .await?
            .json::<Announcement>()
            .await?;

        Ok(AnnouncementPanelComponent {
            announcement_message: announcement.message,
            announcement_last_change: announcement.last_change,
            api_version: version.version,
        })
    }

    /// Returns new Announcement incase remote one is newer
    pub async fn update_announcement(
        profile: Profile,
        last_change: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<Self>> {
        debug!("Announcement fetching...");
        let new = Self::fetch(&profile).await?;
        Ok(if new.announcement_last_change != last_change {
            debug!("Announcement is newer");
            Some(new)
        } else {
            debug!("Announcement is same as before");
            None
        })
    }

    pub fn update(
        &mut self,
        msg: AnnouncementPanelMessage,
    ) -> Option<Command<DefaultViewMessage>> {
        match msg {
            AnnouncementPanelMessage::UpdateAnnouncement(result) => match result {
                Ok(Some(announcement)) => {
                    *self = announcement;
                    Some(Command::perform(
                        async { Action::Save },
                        DefaultViewMessage::Action,
                    ))
                },
                Ok(None) => None,
                Err(e) => {
                    tracing::trace!("Failed to update announcement: {}", e);
                    None
                },
            },
        }
    }

    pub fn view(&self) -> Element<DefaultViewMessage, AirshipperTheme> {
        let update = SUPPORTED_SERVER_API_VERSION != self.api_version;
        let rowtext = match (update, &self.announcement_message) {
            (false, None) => {
                return container("").into();
            },
            (true, None) => {
                "Airshipper is outdated, please update to the latest release!".to_string()
            },
            (false, Some(msg)) => {
                let date: chrono::DateTime<chrono::Local> =
                    self.announcement_last_change.into();
                format!("News from {}: {}", date.format("%Y-%m-%d %H:%M"), msg)
            },
            (true, Some(msg)) => {
                format!("Airshipper is outdated! News: {}", msg)
            },
        };

        let mut content_row = row![
            container(
                Text::new(&rowtext)
                    .style(TextStyle::Dark)
                    .font(POPPINS_MEDIUM_FONT),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(Vertical::Center)
            .padding(Padding::from([1, 0, 0, 16])),
        ];
        if update {
            content_row = content_row.push(
                container(
                    button(
                        row![
                            text("Download Airshipper").size(14),
                            image(Handle::from_memory(UP_RIGHT_ARROW_ICON.to_vec(),))
                        ]
                        .spacing(5)
                        .align_items(Alignment::Center),
                    )
                    .on_press(DefaultViewMessage::Interaction(Interaction::OpenURL(
                        AIRSHIPPER_RELEASE_URL.to_string(),
                    )))
                    .padding(Padding::from([2, 10, 2, 12]))
                    .height(Length::Units(20))
                    .style(ButtonStyle::AirshipperDownload),
                )
                .padding(Padding::from([0, 20, 0, 0]))
                .height(Length::Fill)
                .align_y(Vertical::Center)
                .width(Length::Shrink),
            );
        }

        let top_row = row![column![
            container(content_row.height(Length::Fill)).align_y(Vertical::Center),
        ]]
        .height(Length::Units(50));

        let col = column![].push(
            container(top_row)
                .width(Length::Fill)
                .style(ContainerStyle::Announcement),
        );

        let announcement_container = container(col);
        announcement_container.into()
    }
}
