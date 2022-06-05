use super::Action;
use crate::{
    gui,
    gui::{
        components::{Changelog, LogoPanelComponent, News},
        style, subscriptions, Result,
    },
    io, net, profiles,
    profiles::Profile,
    ProcessUpdate, Progress,
};
use iced::{
    alignment::{Horizontal, Vertical},
    image::Handle,
    pure::{
        button, column, container, pick_list, row, text, text_input, tooltip,
        widget::{Column, Container},
        Application, Element, Widget,
    },
    tooltip::Position,
    Alignment, Command, Image, Length, Padding, ProgressBar, Renderer,
};
use iced_lazy::Component;
use std::path::PathBuf;

use crate::gui::{
    components::{CommunityShowcaseComponent, GamePanelComponent, GamePanelMessage},
    style::{LeftPanelStyle, TestStyle2, TestStyle3},
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DefaultView {
    changelog: Changelog,
    #[serde(skip)]
    logo_panel_component: LogoPanelComponent,
    #[serde(skip)]
    community_showcase_component: CommunityShowcaseComponent,
    #[serde(skip)]
    game_panel_component: GamePanelComponent,
    news: News,
    #[serde(skip)]
    state: State,
    #[serde(skip)]
    show_settings: bool,
}

#[derive(Debug, Clone)]
pub enum State {
    // do not ask, used for retry.
    QueryingForUpdates(bool),
    //Playing(Profile),
    /// bool indicates whether Veloren can be started offline
    Offline(bool),
}

impl Default for State {
    fn default() -> Self {
        Self::QueryingForUpdates(false)
    }
}

#[derive(Clone, Debug)]
pub enum DefaultViewMessage {
    // Messages
    Action(Action),
    Query,

    // Updates
    ChangelogUpdate(Result<Option<Changelog>>),
    NewsUpdate(Result<Option<News>>),
    #[cfg(windows)]
    LauncherUpdate(Result<Option<self_update::update::Release>>),

    // User Interactions
    Interaction(Interaction),

    // Game Panel Messages
    GamePanel(GamePanelMessage),
}

#[derive(Debug, Clone)]
pub enum Interaction {
    LogLevelChanged(profiles::LogLevel),
    ServerChanged(profiles::Server),
    WgpuBackendChanged(profiles::WgpuBackend),
    ReadMore(String),
    SetChangelogDisplayCount(usize),
    SettingsPressed,
    OpenLogsPressed,
    OpenURL(String),
    Disabled,
    EnvVarsChanged(String),
}

impl DefaultView {
    pub fn subscription(&self) -> iced::Subscription<DefaultViewMessage> {
        self.game_panel_component
            .subscription()
            .map(|msg| DefaultViewMessage::GamePanel(msg))
    }

    pub fn view(&self, active_profile: &Profile) -> Element<DefaultViewMessage> {
        let Self {
            changelog,
            news,
            logo_panel_component,
            community_showcase_component,
            game_panel_component,
            ..
        } = self;

        let left = container(
            column()
                .push(container(logo_panel_component.view()).height(Length::Fill))
                .push(
                    container(community_showcase_component.view()).height(Length::Shrink),
                )
                .push(
                    container(
                        game_panel_component
                            .view()
                            .map(DefaultViewMessage::GamePanel),
                    )
                    .height(Length::Shrink),
                ),
        )
        .height(Length::Fill)
        .width(Length::Units(347))
        .style(LeftPanelStyle);
        let middle = container(changelog.view())
            .height(Length::Fill)
            .width(Length::Fill)
            .style(TestStyle2);
        let right = container(column().push(text("hello3")))
            .height(Length::Fill)
            .width(Length::Units(237))
            .style(TestStyle3);

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
                        Changelog::update(self.changelog.etag.clone()),
                        DefaultViewMessage::ChangelogUpdate,
                    ),
                    Command::perform(
                        News::update(self.news.etag.clone()),
                        DefaultViewMessage::NewsUpdate,
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
            // Updates
            DefaultViewMessage::ChangelogUpdate(update) => match update {
                Ok(Some(changelog)) => {
                    self.changelog = changelog;
                    return Command::perform(
                        async { Action::Save },
                        DefaultViewMessage::Action,
                    );
                },
                Ok(None) => {},
                Err(e) => {
                    tracing::trace!("Failed to update changelog: {}", e);
                },
            },
            DefaultViewMessage::NewsUpdate(update) => match update {
                Ok(Some(news)) => {
                    self.news = news;
                    return Command::perform(
                        async { Action::Save },
                        DefaultViewMessage::Action,
                    );
                },
                Ok(None) => {},
                Err(e) => {
                    tracing::trace!("Failed to update news: {}", e);
                },
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
                Interaction::ReadMore(url) => {
                    if let Err(e) = opener::open(&url) {
                        tracing::error!("failed to open {} : {}", url, e);
                    }
                },
                // TODO: Move to download panel
                Interaction::ServerChanged(new_server) => {
                    tracing::debug!("new server selected {}", new_server);
                    self.state = State::QueryingForUpdates(false);
                    let mut profile = active_profile.clone();
                    profile.server = new_server;
                    let profile2 = profile.clone();
                    return Command::batch(vec![
                        Command::perform(
                            async { Action::UpdateProfile(profile2) },
                            DefaultViewMessage::Action,
                        ),
                        Command::perform(Profile::update(profile), |update| {
                            DefaultViewMessage::GamePanel(GamePanelMessage::GameUpdate(
                                update,
                            ))
                        }),
                    ]);
                },
                // TODO: Move to changelog
                Interaction::SetChangelogDisplayCount(count) => {
                    if count <= 1 {
                        self.changelog.display_count = 1;
                    } else if count >= self.changelog.versions.len() {
                        self.changelog.display_count = self.changelog.versions.len();
                    } else {
                        self.changelog.display_count = count;
                    }
                },
                // TODO: Move all of this to new settings panel
                Interaction::SettingsPressed => {
                    self.show_settings = !self.show_settings;
                },
                Interaction::WgpuBackendChanged(wgpu_backend) => {
                    let mut profile = active_profile.clone();
                    profile.wgpu_backend = wgpu_backend;
                    return Command::perform(
                        async { Action::UpdateProfile(profile) },
                        DefaultViewMessage::Action,
                    );
                },
                Interaction::LogLevelChanged(log_level) => {
                    let mut profile = active_profile.clone();
                    profile.log_level = log_level;
                    return Command::perform(
                        async { Action::UpdateProfile(profile) },
                        DefaultViewMessage::Action,
                    );
                },
                Interaction::OpenLogsPressed => {
                    if let Err(e) = opener::open(active_profile.voxygen_logs_path()) {
                        tracing::error!("Failed to open logs dir: {:?}", e);
                    }
                },
                Interaction::EnvVarsChanged(vars) => {
                    let mut profile = active_profile.clone();
                    profile.env_vars = vars;
                    return Command::perform(
                        async { Action::UpdateProfile(profile) },
                        DefaultViewMessage::Action,
                    );
                },
                Interaction::Disabled => {},
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

pub fn settings_button<'a>(
    interaction: Interaction,
    style: impl iced::button::StyleSheet + 'static,
) -> Element<'a, DefaultViewMessage> {
    let btn: Element<Interaction> = button(Image::new(Handle::from_memory(
        crate::assets::SETTINGS_ICON.to_vec(),
    )))
    .on_press(interaction)
    .width(Length::Units(30))
    .height(Length::Units(30))
    .style(style)
    .padding(2)
    .into();

    let element = btn.map(DefaultViewMessage::Interaction);

    tooltip(element, "Settings", Position::Top)
        .style(style::Tooltip)
        .gap(5)
        .into()
}

pub fn secondary_button<'a>(
    label: impl Into<String>,
    interaction: Interaction,
) -> Element<'a, DefaultViewMessage> {
    secondary_button_with_width(label, interaction, Length::Shrink)
}

pub fn secondary_button_with_width<'a>(
    label: impl Into<String>,
    interaction: Interaction,
    width: Length,
) -> Element<'a, DefaultViewMessage> {
    let btn: Element<Interaction> = button(
        text(label)
            .size(16)
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center),
    )
    .on_press(interaction)
    .width(width)
    .style(style::SecondaryButton)
    .into();

    btn.map(DefaultViewMessage::Interaction)
}

pub fn picklist<'a, T: Clone + Eq + std::fmt::Display + 'static>(
    selected: Option<T>,
    values: &'a [T],
    interaction: impl Fn(T) -> Interaction + 'static,
) -> Element<'a, DefaultViewMessage> {
    let selected = Some(selected.unwrap_or_else(|| values[0].clone()));
    let pick_list: Element<Interaction> = pick_list(values, selected, interaction)
        .width(Length::Units(100))
        .style(style::ServerPickList)
        .padding(4)
        .into();

    pick_list.map(DefaultViewMessage::Interaction)
}

pub fn widget_with_label<'a>(
    label_text: &'a str,
    widget: Element<'a, DefaultViewMessage>,
) -> Element<'a, DefaultViewMessage> {
    row()
        .spacing(10)
        .push(text(label_text).horizontal_alignment(Horizontal::Right))
        .push(widget)
        .into()
}
