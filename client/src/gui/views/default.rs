use super::Action;
use crate::{
    channels::Channels,
    gui::{
        components::{ChangelogPanelComponent, LogoPanelComponent, NewsPanelComponent},
        Result,
    },
    profiles::Profile,
};
use iced::{
    pure::{column, container, row, Element},
    Command, Length,
};

use crate::gui::{
    components::{
        ChangelogPanelMessage, CommunityShowcaseComponent, CommunityShowcasePanelMessage,
        GamePanelComponent, GamePanelMessage, NewsPanelMessage, SettingsPanelComponent,
        SettingsPanelMessage,
    },
    rss_feed::RssFeedComponentMessage::UpdateRssFeed,
    style::SidePanelStyle,
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DefaultView {
    changelog_panel_component: ChangelogPanelComponent,
    #[serde(skip)]
    logo_panel_component: LogoPanelComponent,
    community_showcase_component: CommunityShowcaseComponent,
    #[serde(skip)]
    game_panel_component: GamePanelComponent,
    news_panel_component: NewsPanelComponent,
    #[serde(skip)]
    settings_panel_component: SettingsPanelComponent,
    #[serde(skip)]
    show_settings: bool,
}

#[derive(Clone, Debug)]
pub enum DefaultViewMessage {
    // Messages
    Action(Action),
    Query,

    #[cfg(windows)]
    LauncherUpdate(Result<Option<self_update::update::Release>>),

    // User Interactions
    Interaction(Interaction),

    // Panel-specific messages
    GamePanel(GamePanelMessage),
    ChangelogPanel(ChangelogPanelMessage),
    CommunityShowcasePanel(CommunityShowcasePanelMessage),
    NewsPanel(NewsPanelMessage),
    SettingsPanel(SettingsPanelMessage),
}

#[derive(Debug, Clone)]
pub enum Interaction {
    SettingsPressed,
    OpenURL(String),
}

impl DefaultView {
    pub fn subscription(&self) -> iced::Subscription<DefaultViewMessage> {
        self.game_panel_component
            .subscription()
            .map(DefaultViewMessage::GamePanel)
    }

    pub fn view(&self, active_profile: &Profile) -> Element<DefaultViewMessage> {
        let Self {
            changelog_panel_component,
            news_panel_component,
            logo_panel_component,
            community_showcase_component,
            game_panel_component,
            settings_panel_component,
            ..
        } = self;

        let left_middle_contents = if self.show_settings {
            settings_panel_component.view(active_profile)
        } else {
            community_showcase_component.view()
        };

        let left = container(
            column()
                .push(container(logo_panel_component.view()).height(Length::Fill))
                .push(container(left_middle_contents).height(Length::Shrink))
                .push(container(game_panel_component.view()).height(Length::Shrink)),
        )
        .height(Length::Fill)
        .width(Length::Units(347))
        .style(SidePanelStyle);
        let middle = container(changelog_panel_component.view())
            .height(Length::Fill)
            .width(Length::Fill);
        let right = container(news_panel_component.view())
            .height(Length::Fill)
            .width(Length::Units(237))
            .style(SidePanelStyle);

        let main_row = row().push(left).push(middle).push(right);

        container(main_row)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn update(
        &mut self,
        msg: DefaultViewMessage,
        active_profile: &Profile,
    ) -> Command<DefaultViewMessage> {
        match msg {
            // Messages
            // Will be handled by main view
            DefaultViewMessage::Action(_) => {},
            DefaultViewMessage::Query => {
                return Command::batch(vec![
                    Command::perform(
                        NewsPanelComponent::update_news(
                            self.news_panel_component.etag().to_owned(),
                        ),
                        |update| {
                            DefaultViewMessage::NewsPanel(NewsPanelMessage::RssUpdate(
                                UpdateRssFeed(update),
                            ))
                        },
                    ),
                    Command::perform(
                        ChangelogPanelComponent::update_changelog(
                            self.changelog_panel_component.etag.clone(),
                        ),
                        |update| {
                            DefaultViewMessage::ChangelogPanel(
                                ChangelogPanelMessage::UpdateChangelog(update),
                            )
                        },
                    ),
                    Command::perform(
                        CommunityShowcaseComponent::update_community_posts(
                            self.community_showcase_component.etag().to_owned(),
                        ),
                        |update| {
                            DefaultViewMessage::CommunityShowcasePanel(
                                CommunityShowcasePanelMessage::RssUpdate(UpdateRssFeed(
                                    update,
                                )),
                            )
                        },
                    ),
                    Command::perform(
                        Channels::fetch(active_profile.channel_url()),
                        |channels| {
                            DefaultViewMessage::SettingsPanel(
                                SettingsPanelMessage::ChannelsLoaded(channels),
                            )
                        },
                    ),
                    Command::perform(Profile::update(active_profile.clone()), |update| {
                        DefaultViewMessage::GamePanel(GamePanelMessage::GameUpdate(
                            update,
                        ))
                    }),
                    #[cfg(windows)]
                    Command::perform(
                        async { tokio::task::block_in_place(crate::windows::query) },
                        DefaultViewMessage::LauncherUpdate,
                    ),
                ]);
            },

            DefaultViewMessage::GamePanel(msg) => {
                if let Some(command) =
                    self.game_panel_component.update(msg, active_profile)
                {
                    return command;
                }
            },
            DefaultViewMessage::ChangelogPanel(msg) => {
                if let Some(command) = self.changelog_panel_component.update(msg) {
                    return command;
                }
            },
            DefaultViewMessage::CommunityShowcasePanel(msg) => {
                if let Some(command) = self.community_showcase_component.update(msg) {
                    return command;
                }
            },
            DefaultViewMessage::NewsPanel(msg) => {
                if let Some(command) = self.news_panel_component.update(msg) {
                    return command;
                }
            },

            DefaultViewMessage::SettingsPanel(msg) => {
                if let Some(command) =
                    self.settings_panel_component.update(msg, active_profile)
                {
                    return command;
                }
            },

            #[cfg(windows)]
            DefaultViewMessage::LauncherUpdate(update) => {
                if let Ok(Some(release)) = update {
                    return Command::perform(
                        async { Action::LauncherUpdate(release) },
                        DefaultViewMessage::Action,
                    );
                }
            },

            // User Interaction
            DefaultViewMessage::Interaction(interaction) => match interaction {
                Interaction::SettingsPressed => {
                    self.show_settings = !self.show_settings;
                },
                Interaction::OpenURL(url) => {
                    if let Err(e) = opener::open(url) {
                        tracing::error!(
                            "Failed to open gitlab changelog website: {:?}",
                            e
                        );
                    }
                },
            },
        }

        Command::none()
    }
}
