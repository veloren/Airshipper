use super::Action;
use crate::{
    assets::{HAXRCORP_4089_FONT, HAXRCORP_4089_FONT_SIZE_3},
    gui,
    gui::{
        components::{Changelog, News},
        style, subscriptions, Result,
    },
    io, net, profiles,
    profiles::Profile,
    ProcessUpdate,
};
use iced::{
    alignment::{Horizontal, Vertical},
    image::Handle,
    pure::{
        button, column, container, pick_list, row, text, text_input, tooltip, Element,
    },
    tooltip::Position,
    Alignment, Command, Image, Length, ProgressBar,
};
use std::path::PathBuf;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref LOG_REGEX: Regex = Regex::new(r"(?:\x{1b}\[\dm)?(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{1,6}Z)(?:\x{1b}\[\dm\s+\x{1b}\[\d{2}m)?\s?(INFO|TRACE|DEBUG|ERROR|WARN)(?:\x{1b}\[\dm\s\x{1b}\[\dm)?\s?((?:[A-Za-z_]+:{0,2})+)\s?(.*)").unwrap();
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DefaultView {
    changelog: Changelog,
    news: News,
    #[serde(skip)]
    state: State,
    #[serde(skip)]
    download_progress: Option<net::Progress>,
    #[serde(skip)]
    show_settings: bool,
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
    Playing(Profile),

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
    ChannelChanged(profiles::Channel),
    WgpuBackendChanged(profiles::WgpuBackend),
    ReadMore(String),
    SetChangelogDisplayCount(usize),
    SettingsPressed,
    OpenLogsPressed,
    Disabled,
    EnvVarsChanged(String),
}

impl DefaultView {
    pub fn subscription(&self) -> iced::Subscription<DefaultViewMessage> {
        match &self.state {
            State::Downloading(url, location, _) => {
                subscriptions::download::file(url, location)
                    .map(DefaultViewMessage::DownloadProgress)
            },
            &State::Playing(ref profile) => {
                subscriptions::process::stream(profile.clone())
                    .map(DefaultViewMessage::ProcessUpdate)
            },
            _ => iced::Subscription::none(),
        }
    }

    pub fn view(&self, active_profile: &Profile) -> Element<DefaultViewMessage> {
        let Self {
            changelog,
            news,
            state,
            //play_button_state,
            //settings_button_state,
            download_progress,
            ..
        } = self;

        let logo = container(
            Image::new(Handle::from_memory(crate::assets::VELOREN_LOGO.to_vec()))
                .width(Length::FillPortion(10)),
        );

        let icons = row()
            .width(Length::Fill)
            .height(Length::Units(90))
            .align_items(Alignment::Center)
            .spacing(10)
            .padding(15)
            .push(logo);

        // Contains title, changelog
        let left = column()
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .padding(15)
            .push(icons)
            .push(changelog.view());

        // Contains the news pane and optionally the settings pane at the bottom
        let mut right = column()
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .push(news.view());

        if self.show_settings {
            let server_picker = tooltip(
                widget_with_label(
                    "Server:",
                    picklist(
                        Some(active_profile.server),
                        profiles::SERVERS,
                        Interaction::ServerChanged,
                    ),
                ),
                "The download server used for game files",
                Position::Top,
            )
            .style(style::Tooltip)
            .gap(5);

            let channel_picker = tooltip(
                widget_with_label(
                    "Channel:",
                    picklist(
                        Some(active_profile.channel),
                        profiles::CHANNELS,
                        Interaction::ChannelChanged,
                    ),
                ),
                "The download channel used for game files",
                Position::Top,
            )
            .style(style::Tooltip)
            .gap(5);

            let wgpu_backend_picker = tooltip(
                widget_with_label(
                    "Graphics Mode:",
                    picklist(
                        Some(active_profile.wgpu_backend),
                        profiles::WGPU_BACKENDS,
                        Interaction::WgpuBackendChanged,
                    ),
                ),
                "The rendering backend that the game will use. \nLeave on Auto unless \
                 you are experiencing issues",
                Position::Top,
            )
            .style(style::Tooltip)
            .gap(5);

            let log_level_picker = tooltip(
                widget_with_label(
                    "Log Level:",
                    picklist(
                        Some(active_profile.log_level),
                        profiles::LOG_LEVELS,
                        Interaction::LogLevelChanged,
                    ),
                ),
                "Changes the amount of information that the game outputs to its log file",
                Position::Top,
            )
            .style(style::Tooltip)
            .gap(5);

            let open_logs_button = secondary_button_with_width(
                "Open Logs",
                Interaction::OpenLogsPressed,
                Length::Fill,
            );

            let env_vars = tooltip(
                widget_with_label(
                    "Env vars:",
                    text_input("FOO=foo, BAR=bar", &active_profile.env_vars, |vars| {
                        DefaultViewMessage::Interaction(Interaction::EnvVarsChanged(vars))
                    })
                    .width(Length::Fill)
                    .into(),
                ),
                "Environment variables set when running Voxygen",
                Position::Top,
            )
            .style(style::Tooltip)
            .gap(5);

            let settings = container(
                column()
                    .padding(2)
                    .align_items(Alignment::End)
                    .push(
                        row()
                            .padding(5)
                            .align_items(Alignment::Center)
                            .push(wgpu_backend_picker),
                    )
                    .push(
                        row()
                            .padding(5)
                            .align_items(Alignment::Center)
                            .push(server_picker)
                            .push(channel_picker),
                    )
                    .push(
                        row()
                            .padding(5)
                            .align_items(Alignment::Center)
                            .push(log_level_picker)
                            .push(open_logs_button),
                    )
                    .push(row().padding(5).spacing(10).push(env_vars)),
            )
            .padding(10)
            .width(Length::Fill)
            .style(gui::style::News);

            right = right.push(settings);
        }

        // Contains logo, changelog and news
        let middle = container(row().padding(2).push(left).push(right))
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
            State::Downloading(_, _, _) => "Downloading",
            State::Installing => "Installing",
            State::QueryingForUpdates(_) => "Loading",
            State::ReadyToPlay => "Play",
            State::Offline(available) => match available {
                true => "Play",
                false => "Retry",
            },
            State::UpdateAvailable(_) => "Update",
            State::Playing(..) => "Playing",
            State::Retry => "Retry",
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
        let download_speed = text(&download_text).size(16);
        let download_progressbar =
            ProgressBar::new(0.0..=100.0, download_progress).style(style::Progress);
        let download = column()
            .width(Length::FillPortion(4))
            .spacing(5)
            .push(download_speed)
            .push(download_progressbar);

        let play = primary_button(
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

        let settings_button =
            settings_button(Interaction::SettingsPressed, style::SettingsButton);

        let bottom = container(
            row()
                .align_items(Alignment::End)
                .spacing(10)
                .padding(10)
                .push(download)
                .push(settings_button)
                .push(play),
        )
        .style(style::Bottom);

        // Contains everything
        let content = column()
            .padding(2)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(middle)
            .push(bottom);

        container(content)
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
                    if let Some(cap) = LOG_REGEX.captures(&msg) {
                        if let (Some(level), Some(target), Some(msg)) =
                            (cap.get(2), cap.get(3), cap.get(4))
                        {
                            let target = target.as_str();
                            let msg = msg.as_str();

                            match level.as_str() {
                                "TRACE" => tracing::trace!(
                                    target: "voxygen",
                                    "{} {}",
                                    target,
                                    msg,
                                ),
                                "DEBUG" => tracing::debug!(
                                    target: "voxygen",
                                    "{} {}",
                                    target,
                                    msg,
                                ),
                                "INFO" => tracing::info!(
                                    target: "voxygen",
                                    "{} {}",
                                    target,
                                    msg,
                                ),
                                "WARN" => tracing::warn!(
                                    target: "voxygen",
                                    "{} {}",
                                    target,
                                    msg,
                                ),
                                "ERROR" => tracing::error!(
                                    target: "voxygen",
                                    "{} {}",
                                    target,
                                    msg,
                                ),
                                _ => tracing::info!(target: "voxygen","{}", msg),
                            }
                        } else {
                            tracing::info!(target: "voxygen","{}", msg);
                        }
                    } else {
                        tracing::info!(target: "voxygen","{}", msg);
                    }
                },
                ProcessUpdate::Exit(code) => {
                    tracing::debug!("Veloren exited with {}", code);
                    self.state = State::QueryingForUpdates(false);
                    return Command::perform(
                        Profile::update(active_profile.clone()),
                        DefaultViewMessage::GameUpdate,
                    );
                },
                ProcessUpdate::Error(err) => {
                    tracing::error!(
                        "Failed to receive an update from Veloren process! {}",
                        err
                    );
                    self.state = State::Retry;
                },
            },
            DefaultViewMessage::DownloadProgress(progress) => match progress {
                net::Progress::Errored(err) => {
                    tracing::error!("Download failed with: {}", err);
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
                    tracing::error!("Installation failed with: {}", e);
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
                        self.state = State::Playing(active_profile.clone());
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
                            self.state = State::Playing(active_profile.clone());
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
                        tracing::error!("failed to open {} : {}", url, e);
                    }
                },
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
                        Command::perform(
                            Profile::update(profile),
                            DefaultViewMessage::GameUpdate,
                        ),
                    ]);
                },
                Interaction::ChannelChanged(new_channel) => {
                    tracing::debug!("new channel selected {}", new_channel);
                    self.state = State::QueryingForUpdates(false);
                    let mut profile = active_profile.clone();
                    profile.channel = new_channel;
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
                Interaction::SetChangelogDisplayCount(count) => {
                    if count <= 1 {
                        self.changelog.display_count = 1;
                    } else if count >= self.changelog.versions.len() {
                        self.changelog.display_count = self.changelog.versions.len();
                    } else {
                        self.changelog.display_count = count;
                    }
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
            },
        }

        Command::none()
    }
}

pub fn primary_button<'a>(
    label: &'static str,
    interaction: Interaction,
    style: impl iced::button::StyleSheet + 'static,
) -> Element<'a, DefaultViewMessage> {
    let btn: Element<Interaction> = button(
        text(label)
            .font(HAXRCORP_4089_FONT)
            .size(HAXRCORP_4089_FONT_SIZE_3)
            .height(Length::Fill)
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center),
    )
    .on_press(interaction)
    .width(Length::FillPortion(1))
    .height(Length::Units(60))
    .style(style)
    .padding(2)
    .into();

    btn.map(DefaultViewMessage::Interaction)
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
