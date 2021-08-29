use super::Action;
use crate::{
    assets::{HAXRCORP_4089_FONT, HAXRCORP_4089_FONT_SIZE_3},
    gui,
    gui::{
        components::{Changelog, News},
        style, subscriptions, Result,
    },
    io, net, profiles,
    profiles::{LogLevel, Profile},
    ProcessUpdate,
};
use iced::{
    button, image::Handle, pick_list, tooltip::Position, Align, Button, Column, Command,
    Container, Element, HorizontalAlignment, Image, Length, PickList, ProgressBar, Row,
    Text, Tooltip, VerticalAlignment,
};
use std::path::PathBuf;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DefaultView {
    changelog: Changelog,
    news: News,

    #[serde(skip)]
    state: State,

    #[serde(skip)]
    play_button_state: button::State,
    #[serde(skip)]
    settings_button_state: button::State,
    #[serde(skip)]
    open_logs_button_state: button::State,
    #[serde(skip)]
    server_picker_state: pick_list::State<profiles::Server>,
    #[serde(skip)]
    wgpu_backend_picker_state: pick_list::State<profiles::WgpuBackend>,
    #[serde(skip)]
    log_level_picker_state: pick_list::State<profiles::LogLevel>,
    #[serde(skip)]
    download_progress: Option<net::Progress>,
    #[serde(skip)]
    show_settings: bool,
    #[serde(skip)]
    log_level: LogLevel,
}

#[derive(Debug, Clone)]
pub enum State {
    // do not ask, used for retry.
    QueryingForUpdates(bool),
    UpdateAvailable(String),
    /// Url, Download Path, Version
    Downloading(String, PathBuf, String),
    Installing,
    ReadyToPlay,
    Playing(Profile, LogLevel),

    Retry,
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
    GameUpdate(Result<Option<String>>),
    ProcessUpdate(io::ProcessUpdate),
    DownloadProgress(net::Progress),
    InstallDone(Result<Profile>),

    #[cfg(windows)]
    LauncherUpdate(Result<Option<self_update::update::Release>>),

    // User Interactions
    Interaction(Interaction),
}

#[derive(Debug, Clone)]
pub enum Interaction {
    PlayPressed,
    LogLevelChanged(profiles::LogLevel),
    ServerChanged(profiles::Server),
    WgpuBackendChanged(profiles::WgpuBackend),
    ReadMore(String),
    SettingsPressed,
    OpenLogsPressed,
    Disabled,
}

impl DefaultView {
    pub fn subscription(&self) -> iced::Subscription<DefaultViewMessage> {
        match &self.state {
            State::Downloading(url, location, _) => {
                subscriptions::download::file(url, location)
                    .map(DefaultViewMessage::DownloadProgress)
            },
            &State::Playing(ref profile, log_level) => {
                subscriptions::process::stream(profile.clone(), log_level)
                    .map(DefaultViewMessage::ProcessUpdate)
            },
            _ => iced::Subscription::none(),
        }
    }

    pub fn view(&mut self, active_profile: &Profile) -> Element<DefaultViewMessage> {
        let Self {
            changelog,
            news,
            state,
            play_button_state,
            settings_button_state,
            download_progress,
            ..
        } = self;

        let logo = Container::new(
            Image::new(Handle::from_memory(crate::assets::VELOREN_LOGO.to_vec()))
                .width(Length::FillPortion(10)),
        );

        let icons = Row::new()
            .width(Length::Fill)
            .height(Length::Units(90))
            .align_items(Align::Center)
            .spacing(10)
            .padding(15)
            .push(logo);

        // Contains title, changelog
        let left = Column::new()
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .padding(15)
            .push(icons)
            .push(changelog.view());

        // Contains the news pane and optionally the settings pane at the bottom
        let mut right = Column::new()
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .push(news.view());

        if self.show_settings {
            let server_picker = widget_with_label_and_tooltip(
                "Server:",
                "The download server used for game files",
                pick_list(
                    &mut self.server_picker_state,
                    Some(active_profile.server),
                    profiles::SERVERS,
                    Interaction::ServerChanged,
                ),
            );

            let wgpu_backend_picker = widget_with_label_and_tooltip(
                "Graphics Mode:",
                "The rendering backend that the game will use. \nLeave on Auto unless \
                 you are experiencing issues",
                pick_list(
                    &mut self.wgpu_backend_picker_state,
                    Some(active_profile.wgpu_backend),
                    profiles::WGPU_BACKENDS,
                    Interaction::WgpuBackendChanged,
                ),
            );

            let log_level_picker = widget_with_label_and_tooltip(
                "Log Level:",
                "Changes the amount of information that the game outputs to its log file",
                pick_list(
                    &mut self.log_level_picker_state,
                    Some(self.log_level),
                    profiles::LOG_LEVELS,
                    Interaction::LogLevelChanged,
                ),
            );

            let open_logs_button = secondary_button(
                &mut self.open_logs_button_state,
                "Open Logs",
                Interaction::OpenLogsPressed,
            );

            let settings = Container::new(
                Row::new()
                    .padding(2)
                    .push(
                        Column::new()
                            .padding(5)
                            .spacing(10)
                            .align_items(Align::End)
                            .push(wgpu_backend_picker)
                            .push(server_picker),
                    )
                    .push(
                        Column::new()
                            .padding(5)
                            .spacing(10)
                            .align_items(Align::End)
                            .push(log_level_picker)
                            .push(open_logs_button),
                    ),
            )
            .width(Length::Fill)
            .style(gui::style::News);

            right = right.push(settings);
        }

        // Contains logo, changelog and news
        let middle = Container::new(Row::new().padding(2).push(left).push(right))
            .height(Length::FillPortion(6))
            .style(style::Middle);

        let download_progress = match state {
            State::Downloading(_, _, _) => {
                if let Some(prog) = download_progress {
                    match prog {
                        net::Progress::Advanced(_msg, percentage) => *percentage as f32,
                        net::Progress::Finished => 100.0,
                        _ => 0.0,
                    }
                } else {
                    0.0
                }
            },
            _ => 0.0,
        };
        let play_button_text = match state {
            State::Downloading(_, _, _) => "Downloading".to_string(),
            State::Installing => "Installing".into(),
            State::QueryingForUpdates(_) => "Loading".into(),
            State::ReadyToPlay => "Play".into(),
            State::Offline(available) => match available {
                true => "Play".into(),
                false => "Retry".into(),
            },
            State::UpdateAvailable(_) => "Update".into(),
            State::Playing(..) => "Playing".into(),
            State::Retry => "Retry".into(),
        };

        let download_text = match state {
            State::Downloading(_, _, _) => self
                .download_progress
                .as_ref()
                .map(|x| x.to_string())
                .unwrap_or_else(|| "Downloading...".to_string()),
            State::Installing => "Installing...".to_string(),
            State::QueryingForUpdates(_) => "Checking for updates...".to_string(),
            State::ReadyToPlay => "Ready to play...".to_string(),
            State::Offline(available) => match available {
                true => "Ready to play offline...".into(),
                false => "Error: Check your internet and retry.".into(),
            },
            State::UpdateAvailable(_) => "Update available!".to_string(),
            State::Playing(..) => "Much fun playing!".to_string(),
            State::Retry => "Error occured. Give it a retry.".to_string(),
        };
        let download_speed = Text::new(&download_text).size(16);
        let download_progressbar =
            ProgressBar::new(0.0..=100.0, download_progress).style(style::Progress);
        let download = Column::new()
            .width(Length::FillPortion(4))
            .spacing(5)
            .push(download_speed)
            .push(download_progressbar);

        let play = primary_button(
            play_button_state,
            play_button_text,
            match state {
                State::ReadyToPlay
                | State::UpdateAvailable(_)
                | State::Offline(_)
                | State::Retry => Interaction::PlayPressed,
                _ => Interaction::Disabled,
            },
            match state {
                State::ReadyToPlay
                | State::UpdateAvailable(_)
                | State::Offline(_)
                | State::Retry => style::PrimaryButton::Enabled,
                _ => style::PrimaryButton::Disabled,
            },
        );

        let settings_button = settings_button(
            settings_button_state,
            Interaction::SettingsPressed,
            style::SettingsButton,
        );

        let bottom = Container::new(
            Row::new()
                .align_items(Align::End)
                .spacing(10)
                .padding(10)
                .push(download)
                .push(settings_button)
                .push(play),
        )
        .style(style::Bottom);

        // Contains everything
        let content = Column::new()
            .padding(2)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(middle)
            .push(bottom);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::Content)
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
                    Command::perform(
                        Profile::update(active_profile.clone()),
                        DefaultViewMessage::GameUpdate,
                    ),
                    #[cfg(windows)]
                    Command::perform(
                        async { tokio::task::block_in_place(crate::windows::query) },
                        DefaultViewMessage::LauncherUpdate,
                    ),
                ]);
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
                    log::trace!("Failed to update changelog: {}", e);
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
                    log::trace!("Failed to update news: {}", e);
                },
            },
            DefaultViewMessage::GameUpdate(update) => match update {
                Ok(Some(version)) => {
                    // Skip asking
                    if let State::QueryingForUpdates(true) = self.state {
                        self.state = State::Downloading(
                            active_profile.url(),
                            active_profile.download_path(),
                            version,
                        );
                    } else {
                        self.state = State::UpdateAvailable(version);
                    }
                },
                Ok(None) => {
                    self.state = State::ReadyToPlay;
                },
                Err(_) => {
                    // Go into offline mode incase game can't be updated.
                    if active_profile.installed() {
                        self.state = State::Offline(true);
                    } else {
                        self.state = State::Offline(false);
                    }
                },
            },
            DefaultViewMessage::ProcessUpdate(update) => match update {
                ProcessUpdate::Line(msg) => {
                    log::info!(target: "output::Veloren","[Veloren] {}", msg);
                },
                ProcessUpdate::Exit(code) => {
                    log::debug!("Veloren exited with {}", code);
                    self.state = State::QueryingForUpdates(false);
                    return Command::perform(
                        Profile::update(active_profile.clone()),
                        DefaultViewMessage::GameUpdate,
                    );
                },
                ProcessUpdate::Error(err) => {
                    log::error!(
                        "Failed to receive an update from Veloren process! {}",
                        err
                    );
                    self.state = State::Retry;
                },
            },
            DefaultViewMessage::DownloadProgress(progress) => match progress {
                net::Progress::Errored(err) => {
                    log::error!("Download failed with: {}", err);
                    self.state = State::Retry;
                    let mut profile = active_profile.clone();
                    profile.version = None;
                    return Command::perform(
                        async { Action::UpdateProfile(profile) },
                        DefaultViewMessage::Action,
                    );
                },
                net::Progress::Finished => {
                    let version = match &self.state {
                        State::Downloading(_, _, version) => version.to_string(),
                        _ => panic!(
                            "Reached impossible state: Downloading while not in \
                             download state!"
                        ),
                    };
                    self.state = State::Installing;
                    return Command::perform(
                        Profile::install(active_profile.clone(), version),
                        DefaultViewMessage::InstallDone,
                    );
                },
                p => self.download_progress = Some(p),
            },
            DefaultViewMessage::InstallDone(profile) => match profile {
                Ok(profile) => {
                    self.state = State::ReadyToPlay;
                    return Command::perform(
                        async { Action::UpdateProfile(profile) },
                        DefaultViewMessage::Action,
                    );
                },
                Err(e) => {
                    log::error!("Installation failed with: {}", e);
                    self.state = State::Retry;
                    let mut profile = active_profile.clone();
                    profile.version = None;
                    return Command::perform(
                        async { Action::UpdateProfile(profile) },
                        DefaultViewMessage::Action,
                    );
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
                Interaction::PlayPressed => match &self.state {
                    State::UpdateAvailable(version) => {
                        self.state = State::Downloading(
                            active_profile.url(),
                            active_profile.download_path(),
                            version.clone(),
                        )
                    },
                    State::ReadyToPlay => {
                        self.state =
                            State::Playing(active_profile.clone(), self.log_level);
                    },
                    State::Retry => {
                        // TODO: Switching state should trigger these commands
                        self.state = State::QueryingForUpdates(true);
                        return Command::batch(vec![
                            Command::perform(
                                Changelog::update(self.changelog.etag.clone()),
                                DefaultViewMessage::ChangelogUpdate,
                            ),
                            Command::perform(
                                News::update(self.news.etag.clone()),
                                DefaultViewMessage::NewsUpdate,
                            ),
                            Command::perform(
                                Profile::update(active_profile.clone()),
                                DefaultViewMessage::GameUpdate,
                            ),
                        ]);
                    },
                    State::Offline(available) => match available {
                        // Play offline
                        true => {
                            self.state =
                                State::Playing(active_profile.clone(), self.log_level);
                        },
                        // Retry
                        false => {
                            // TODO: Switching state should trigger these commands
                            self.state = State::QueryingForUpdates(true);
                            return Command::batch(vec![
                                Command::perform(
                                    Changelog::update(self.changelog.etag.clone()),
                                    DefaultViewMessage::ChangelogUpdate,
                                ),
                                Command::perform(
                                    News::update(self.news.etag.clone()),
                                    DefaultViewMessage::NewsUpdate,
                                ),
                                Command::perform(
                                    Profile::update(active_profile.clone()),
                                    DefaultViewMessage::GameUpdate,
                                ),
                            ]);
                        },
                    },

                    State::Installing
                    | State::Downloading(_, _, _)
                    | State::Playing(..)
                    | State::QueryingForUpdates(_) => {},
                },
                Interaction::ReadMore(url) => {
                    if let Err(e) = opener::open(&url) {
                        log::error!("failed to open {} : {}", url, e);
                    }
                },
                Interaction::ServerChanged(new_server) => {
                    log::debug!("new server selected {}", new_server);
                    self.state = State::QueryingForUpdates(false);
                    let mut profile = active_profile.clone();
                    profile.server = new_server;
                    let profile2 = profile.clone();
                    return Command::batch(vec![
                        Command::perform(
                            async { Action::UpdateProfile(profile2) },
                            DefaultViewMessage::Action,
                        ),
                        Command::perform(
                            Profile::update(profile),
                            DefaultViewMessage::GameUpdate,
                        ),
                    ]);
                },
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
                    self.log_level = log_level;
                },
                Interaction::OpenLogsPressed => {
                    if let Err(e) = opener::open(active_profile.voxygen_logs_path()) {
                        log::error!("Failed to open logs dir: {:?}", e);
                    }
                },
                Interaction::Disabled => {},
            },
        }

        Command::none()
    }
}

pub fn primary_button(
    state: &mut button::State,
    label: impl Into<String>,
    interaction: Interaction,
    style: impl button::StyleSheet + 'static,
) -> Element<DefaultViewMessage> {
    let btn: Element<Interaction> = Button::new(
        state,
        Text::new(label)
            .font(HAXRCORP_4089_FONT)
            .size(HAXRCORP_4089_FONT_SIZE_3)
            .height(Length::Fill)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center),
    )
    .on_press(interaction)
    .width(Length::FillPortion(1))
    .height(Length::Units(60))
    .style(style)
    .padding(2)
    .into();

    btn.map(DefaultViewMessage::Interaction)
}

pub fn settings_button(
    state: &mut button::State,
    interaction: Interaction,
    style: impl button::StyleSheet + 'static,
) -> Element<DefaultViewMessage> {
    let btn: Element<Interaction> = Button::new(
        state,
        Image::new(Handle::from_memory(crate::assets::SETTINGS_ICON.to_vec())),
    )
    .on_press(interaction)
    .width(Length::Units(30))
    .height(Length::Units(30))
    .style(style)
    .padding(2)
    .into();

    let element = btn.map(DefaultViewMessage::Interaction);
    Tooltip::new(element, "Settings", Position::Top)
        .style(style::Tooltip)
        .gap(5)
        .into()
}

pub fn secondary_button(
    state: &mut button::State,
    label: impl Into<String>,
    interaction: Interaction,
) -> Element<DefaultViewMessage> {
    let btn: Element<Interaction> = Button::new(
        state,
        Text::new(label)
            .size(16)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center),
    )
    .on_press(interaction)
    .style(style::SecondaryButton)
    .into();

    btn.map(DefaultViewMessage::Interaction)
}

pub fn pick_list<'a, T: Clone + std::cmp::Eq + std::fmt::Display>(
    state: &'a mut pick_list::State<T>,
    selected: Option<T>,
    values: &'a [T],
    interaction: impl Fn(T) -> Interaction + 'static,
) -> Element<'a, DefaultViewMessage> {
    let selected = Some(selected.unwrap_or_else(|| values[0].clone()));
    let pick_list: Element<Interaction> =
        PickList::new(state, values, selected, interaction)
            .width(Length::Units(100))
            .style(style::ServerPickList)
            .padding(4)
            .into();

    pick_list.map(DefaultViewMessage::Interaction)
}

pub fn widget_with_label_and_tooltip<'a>(
    label_text: &'a str,
    tooltip_text: &'a str,
    widget: Element<'a, DefaultViewMessage>,
) -> Element<'a, DefaultViewMessage> {
    // The tooltip cannot be attached to the actual pick list since they both use
    // overlays and aren't (yet) compatible with each other (it results in
    // the picklist not working at all).
    Row::new()
        .spacing(10)
        .push(
            Tooltip::new(
                Text::new(label_text).horizontal_alignment(HorizontalAlignment::Right),
                tooltip_text,
                Position::Top,
            )
            .style(style::Tooltip)
            .gap(5),
        )
        .push(widget)
        .into()
}
