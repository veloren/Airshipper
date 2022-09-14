use super::Action;
use crate::{
    channels::Channels,
    gui::components::{ChangelogPanelComponent, LogoPanelComponent, NewsPanelComponent},
    profiles::Profile,
};
use iced::{
    pure::{column, container, row, Element},
    Command, Length,
};

use crate::gui::{
    components::{
        AnnouncementPanelComponent, AnnouncementPanelMessage, ChangelogPanelMessage,
        CommunityShowcaseComponent, CommunityShowcasePanelMessage, GamePanelComponent,
        GamePanelMessage, NewsPanelMessage, ServerBrowserPanelComponent,
        ServerBrowserPanelMessage, SettingsPanelComponent, SettingsPanelMessage,
    },
    rss_feed::RssFeedComponentMessage::UpdateRssFeed,
    style::SidePanelStyle,
};

#[cfg(windows)]
use crate::gui::Result;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DefaultView {
    changelog_panel_component: ChangelogPanelComponent,
    announcement_panel_component: AnnouncementPanelComponent,
    #[serde(skip)]
    logo_panel_component: LogoPanelComponent,
    community_showcase_component: CommunityShowcaseComponent,
    #[serde(skip)]
    game_panel_component: GamePanelComponent,
    news_panel_component: NewsPanelComponent,
    #[serde(skip)]
    settings_panel_component: SettingsPanelComponent,
    #[serde(skip)]
    server_browser_panel_component: ServerBrowserPanelComponent,
    #[serde(skip)]
    show_settings: bool,
    #[serde(skip)]
    show_server_browser: bool,
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
    AnnouncementPanel(AnnouncementPanelMessage),
    CommunityShowcasePanel(CommunityShowcasePanelMessage),
    NewsPanel(NewsPanelMessage),
    SettingsPanel(SettingsPanelMessage),
    ServerBrowserPanel(ServerBrowserPanelMessage),
}

#[derive(Debug, Clone)]
pub enum Interaction {
    SettingsPressed,
    ToggleServerBrowser,
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
            announcement_panel_component,
            news_panel_component,
            logo_panel_component,
            community_showcase_component,
            game_panel_component,
            settings_panel_component,
            server_browser_panel_component,
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
                .push(
                    container(game_panel_component.view(active_profile))
                        .height(Length::Shrink),
                ),
        )
        .height(Length::Fill)
        .width(Length::Units(347))
        .style(SidePanelStyle);

        let mut main_row = row().push(left);

        if !self.show_server_browser {
            let middle = container(
                column()
                    .push(
                        container(announcement_panel_component.view())
                            .height(Length::Shrink),
                    )
                    .push(
                        container(changelog_panel_component.view()).height(Length::Fill),
                    ),
            )
            .height(Length::Fill)
            .width(Length::Fill);
            let right = container(news_panel_component.view())
                .height(Length::Fill)
                .width(Length::Units(237))
                .style(SidePanelStyle);

            main_row = main_row.push(middle).push(right);
        } else {
            let server_browser = container(server_browser_panel_component.view())
                .height(Length::Fill)
                .width(Length::Fill);
            main_row = main_row.push(server_browser);
        }

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
                            active_profile.channel.clone(),
                        ),
                        |update| {
                            DefaultViewMessage::ChangelogPanel(
                                ChangelogPanelMessage::UpdateChangelog(update),
                            )
                        },
                    ),
                    Command::perform(ServerBrowserPanelComponent::fetch(), |update| {
                        DefaultViewMessage::ServerBrowserPanel(
                            ServerBrowserPanelMessage::UpdateServerList(update),
                        )
                    }),
                    Command::perform(
                        AnnouncementPanelComponent::update_announcement(
                            active_profile.clone(),
                            self.announcement_panel_component.announcement_last_change,
                        ),
                        |update| {
                            DefaultViewMessage::AnnouncementPanel(
                                AnnouncementPanelMessage::UpdateAnnouncement(update),
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
            DefaultViewMessage::AnnouncementPanel(msg) => {
                if let Some(command) = self.announcement_panel_component.update(msg) {
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
            DefaultViewMessage::ServerBrowserPanel(msg) => {
                if let Some(command) = self.server_browser_panel_component.update(msg) {
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
                Interaction::ToggleServerBrowser => {
                    self.show_server_browser = !self.show_server_browser;

                    // If toggling the server browser panel resulted in it being hidden,
                    // deselect the selected server to switch the
                    // Launch button back to saying "Launch" instead of "Connect to
                    // selected server"
                    if !self.show_server_browser {
                        return Command::perform(async {}, |_| {
                            DefaultViewMessage::ServerBrowserPanel(
                                ServerBrowserPanelMessage::SelectServerEntry(None),
                            )
                        });
                    }
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
