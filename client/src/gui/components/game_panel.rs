use crate::{
    assets::{DOWNLOAD_ICON, POPPINS_BOLD_FONT, POPPINS_MEDIUM_FONT, SETTINGS_ICON},
    gui::{
        custom_widgets::heading_with_rule,
        style::{
            button::{ButtonState, ButtonStyle, DownloadButtonStyle},
            text::TextStyle,
            tooltip::TooltipStyle,
        },
        subscriptions,
        views::{
            default::{DefaultViewMessage, Interaction, Interaction::SettingsPressed},
            Action,
        },
        widget::*,
    },
    io::ProcessUpdate,
    net::Progress,
    profiles::Profile,
    Result,
};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, image, image::Handle, progress_bar, row, text,
        tooltip, tooltip::Position, Image,
    },
    Alignment, Command, Length, Padding,
};
use std::{path::PathBuf, time::Duration};

use lazy_static::lazy_static;
use regex::Regex;
use tracing::debug;

lazy_static! {
    static ref LOG_REGEX: Regex = Regex::new(r"(?:\x{1b}\[\dm)?(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{1,6}Z)(?:\x{1b}\[\dm\s+\x{1b}\[\d{2}m)?\s?(INFO|TRACE|DEBUG|ERROR|WARN)(?:\x{1b}\[\dm\s\x{1b}\[\dm)?\s?((?:[A-Za-z_]+:{0,2})+)\s?(.*)").unwrap();
}

#[derive(Clone, Debug)]
pub enum GamePanelMessage {
    GameUpdate(Result<Option<String>>),
    ProcessUpdate(ProcessUpdate),
    DownloadProgress(Progress),
    InstallDone(Result<Profile>),
    PlayPressed,
    ServerBrowserServerChanged(Option<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum DownloadState {
    Starting,
    InProgress,
}

#[derive(Clone, Debug)]
pub enum GamePanelState {
    QueryingForUpdates(bool),
    UpdateAvailable(String),
    Downloading {
        state: DownloadState,
        url: String,
        download_path: PathBuf,
        version: String,
    },
    Installing,
    ReadyToPlay,
    Playing(Profile),
    Offline(bool),
    Retry,
}

#[derive(Clone, Debug)]
pub struct GamePanelComponent {
    state: GamePanelState,
    download_progress: Option<Progress>,
    selected_server_browser_address: Option<String>,
}

impl Default for GamePanelComponent {
    fn default() -> Self {
        Self {
            state: GamePanelState::QueryingForUpdates(false),
            download_progress: None,
            selected_server_browser_address: None,
        }
    }
}

impl GamePanelComponent {
    pub fn subscription(&self) -> iced::Subscription<GamePanelMessage> {
        match &self.state {
            GamePanelState::Downloading {
                url, download_path, ..
            } => subscriptions::download::file(url, download_path)
                .map(GamePanelMessage::DownloadProgress),
            &GamePanelState::Playing(ref profile) => subscriptions::process::stream(
                profile.clone(),
                self.selected_server_browser_address.clone(),
            )
            .map(GamePanelMessage::ProcessUpdate),
            _ => iced::Subscription::none(),
        }
    }

    pub fn update(
        &mut self,
        msg: GamePanelMessage,
        active_profile: &Profile,
    ) -> Option<Command<DefaultViewMessage>> {
        let (next_state, command) = match msg {
            GamePanelMessage::PlayPressed => match &self.state {
                GamePanelState::UpdateAvailable(version) => (
                    Some(GamePanelState::Downloading {
                        download_path: active_profile.download_path(),
                        url: active_profile.url(),
                        state: DownloadState::Starting,
                        version: version.to_owned(),
                    }),
                    None,
                ),
                GamePanelState::ReadyToPlay => {
                    (Some(GamePanelState::Playing(active_profile.clone())), None)
                },
                GamePanelState::Retry => (
                    Some(GamePanelState::QueryingForUpdates(true)),
                    Some(Command::perform(
                        Profile::update(active_profile.clone()),
                        |update| {
                            DefaultViewMessage::GamePanel(GamePanelMessage::GameUpdate(
                                update,
                            ))
                        },
                    )),
                ),
                GamePanelState::Offline(available) => {
                    match available {
                        // Play offline
                        true => {
                            (Some(GamePanelState::Playing(active_profile.clone())), None)
                        },
                        // Retry
                        false => {
                            // The game has never been downloaded so the only option is to
                            // retry the download
                            (
                                Some(GamePanelState::QueryingForUpdates(true)),
                                Some(Command::perform(
                                    Profile::update(active_profile.clone()),
                                    |update| {
                                        DefaultViewMessage::GamePanel(
                                            GamePanelMessage::GameUpdate(update),
                                        )
                                    },
                                )),
                            )
                        },
                    }
                },

                GamePanelState::Installing
                | GamePanelState::Downloading { .. }
                | GamePanelState::Playing(..)
                | GamePanelState::QueryingForUpdates(_) => (None, None),
            },
            GamePanelMessage::DownloadProgress(progress) => match progress {
                Progress::Errored(err) => {
                    tracing::error!("Download failed with: {}", err);
                    let mut profile = active_profile.clone();
                    profile.version = None;
                    (
                        Some(GamePanelState::Retry),
                        Some(Command::perform(
                            async { Action::UpdateProfile(profile) },
                            DefaultViewMessage::Action,
                        )),
                    )
                },
                Progress::Finished => {
                    let version = match &self.state {
                        GamePanelState::Downloading { version, .. } => {
                            version.to_string()
                        },
                        _ => panic!(
                            "Reached impossible state: Downloading while not in \
                             download state!"
                        ),
                    };
                    (
                        Some(GamePanelState::Installing),
                        Some(Command::perform(
                            Profile::install(active_profile.clone(), version),
                            |result| {
                                DefaultViewMessage::GamePanel(
                                    GamePanelMessage::InstallDone(result),
                                )
                            },
                        )),
                    )
                },
                p => {
                    let next_state = match &self.state {
                        GamePanelState::Downloading {
                            state,
                            url,
                            download_path,
                            version,
                        } => {
                            self.download_progress = Some(p);
                            // If we received a progress update and are still in the
                            // Starting download state, transition to the InProgress state
                            if *state == DownloadState::Starting {
                                Some(GamePanelState::Downloading {
                                    state: DownloadState::InProgress,
                                    url: url.clone(),
                                    download_path: download_path.clone(),
                                    version: version.clone(),
                                })
                            } else {
                                None
                            }
                        },
                        _ => panic!("Received progress update for non-existent download"),
                    };

                    (next_state, None)
                },
            },
            GamePanelMessage::InstallDone(profile) => match profile {
                Ok(profile) => (
                    Some(GamePanelState::ReadyToPlay),
                    Some(Command::perform(
                        async { Action::UpdateProfile(profile) },
                        DefaultViewMessage::Action,
                    )),
                ),
                Err(_e) => {
                    // TODO: Fix
                    // tracing::error!("Installation failed with: {}", e);
                    let mut profile = active_profile.clone();
                    profile.version = None;
                    (
                        Some(GamePanelState::Retry),
                        Some(Command::perform(
                            async { Action::UpdateProfile(profile) },
                            DefaultViewMessage::Action,
                        )),
                    )
                },
            },
            // TODO: Move this out of GamePanelComponent? This code handles redirecting
            // voxygen output to Airshipper's log output
            GamePanelMessage::ProcessUpdate(update) => match update {
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
                    (None, None)
                },
                ProcessUpdate::Exit(code) => {
                    debug!("Veloren exited with {}", code);
                    (
                        Some(GamePanelState::QueryingForUpdates(false)),
                        Some(Command::perform(
                            Profile::update(active_profile.clone()),
                            |update| {
                                DefaultViewMessage::GamePanel(
                                    GamePanelMessage::GameUpdate(update),
                                )
                            },
                        )),
                    )
                },
                ProcessUpdate::Error(err) => {
                    tracing::error!(
                        "Failed to receive an update from Veloren process! {}",
                        err
                    );
                    (Some(GamePanelState::Retry), None)
                },
            },
            GamePanelMessage::GameUpdate(update) => {
                let next_state = match update {
                    // The update check succeeded and found a new version
                    Ok(Some(version)) => {
                        if let GamePanelState::QueryingForUpdates(true) = self.state {
                            // The retry button was pressed so immediately attempt the
                            // download rather than
                            // requiring the user to click Update again.
                            GamePanelState::Downloading {
                                url: active_profile.url(),
                                download_path: active_profile.download_path(),
                                version,
                                state: DownloadState::Starting,
                            }
                        } else {
                            GamePanelState::UpdateAvailable(version)
                        }
                    },
                    // The update check succeeded but the game is already up-to-date
                    Ok(None) => GamePanelState::ReadyToPlay,
                    // The update check failed, so go to offline mode, allowing the user
                    // to play the previously downloaded version if present
                    Err(_) => {
                        if active_profile.installed() {
                            GamePanelState::Offline(true)
                        } else {
                            GamePanelState::Offline(false)
                        }
                    },
                };
                (Some(next_state), None)
            },
            GamePanelMessage::ServerBrowserServerChanged(server_address) => {
                self.selected_server_browser_address = server_address;
                (None, None)
            },
        };

        if let Some(state) = next_state {
            self.set_state(state);
        }

        command
    }

    pub fn view(&self, active_profile: &Profile) -> Element<DefaultViewMessage> {
        // TODO: Improve this with actual game version / date (requires changes to
        // Airshipper Server)
        let mut version_string = "Pre-Alpha".to_owned();
        if let Some(version) = &active_profile.version {
            version_string.push_str(format!(" ({})", &version[..7]).as_str())
        }

        column![]
            .push(heading_with_rule::<DefaultViewMessage>("Game Version"))
            .push(
                container(
                    row![]
                        .height(Length::Units(30))
                        .push(
                            container(
                                text(version_string).size(15).style(TextStyle::LightGrey),
                            )
                            .align_y(Vertical::Bottom)
                            .width(Length::Fill)
                            .height(Length::Fill),
                        )
                        .push(
                            tooltip(
                                container(
                                    button(image(Handle::from_memory(
                                        SETTINGS_ICON.to_vec(),
                                    )))
                                    .style(ButtonStyle::Settings)
                                    .on_press(
                                        DefaultViewMessage::Interaction(SettingsPressed),
                                    ),
                                )
                                .center_y(),
                                "Settings",
                                Position::Left,
                            )
                            .size(18)
                            .style(TooltipStyle::Default)
                            .gap(5),
                        ),
                )
                .padding(Padding::from([0, 20])),
            )
            .push(
                container(self.download_area())
                    .width(Length::Fill)
                    .padding(Padding::from([10, 20, 20, 20])),
            )
            .into()
    }
}

impl GamePanelComponent {
    fn set_state(&mut self, state: GamePanelState) {
        debug!("GamePanel state: {:?} -> {:?}", self.state, state);
        self.state = state;
    }
    fn download_area(&self) -> Element<DefaultViewMessage> {
        match &self.state {
            GamePanelState::Downloading { state, .. }
                if *state == DownloadState::InProgress =>
            {
                // When the game is downloading, the download progress bar and related
                // stats replace the Launch / Update button
                let (percent, total, downloaded, bytes_per_sec, remaining) = match self
                    .download_progress
                    .as_ref()
                    .unwrap_or(&Progress::Started)
                {
                    Progress::Advanced(progress_data) => (
                        progress_data.percent_complete as f32,
                        progress_data.total_bytes,
                        progress_data.downloaded_bytes,
                        progress_data.bytes_per_sec,
                        progress_data.remaining,
                    ),
                    Progress::Finished => (100.0, 0, 0, 0, Duration::from_secs(0)),
                    _ => (0.0, 0, 0, 0, Duration::from_secs(0)),
                };

                let download_rate = bytes_per_sec as f32 / 1_000_000.0;

                let progress_text =
                    format!("{} MB / {} MB", downloaded / 1_000_000, total / 1_000_000);

                let mut download_stats_row = row![]
                    .push(Image::new(Handle::from_memory(DOWNLOAD_ICON.to_vec())))
                    .push(
                        text(progress_text)
                            .horizontal_alignment(Horizontal::Right)
                            .size(17),
                    )
                    .spacing(5)
                    .align_items(Alignment::Center);

                if download_rate >= f32::EPSILON {
                    let seconds = remaining.as_secs() % 60;
                    let minutes = (remaining.as_secs() / 60) % 60;
                    let hours = (remaining.as_secs() / 60) / 60;

                    let remaining_text = if hours > 0 {
                        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                    } else {
                        format!("{:02}:{:02}", minutes, seconds)
                    };

                    download_stats_row = download_stats_row
                        .push(text("@").vertical_alignment(Vertical::Center).size(17))
                        .push(
                            text(format!("{:.1} MB/s", download_rate))
                                .font(POPPINS_BOLD_FONT)
                                .size(17)
                                .width(Length::Fill),
                        )
                        .push(
                            row![]
                                .push(
                                    text(remaining_text).font(POPPINS_BOLD_FONT).size(17),
                                )
                                .push(text("left").size(17))
                                .spacing(2)
                                .width(Length::Shrink),
                        );
                }

                container(
                    column![]
                        .push(text("Downloading").font(POPPINS_BOLD_FONT).size(20))
                        .push(
                            container(download_stats_row).padding(Padding::from([5, 0])),
                        )
                        .push(
                            progress_bar(0.0..=100.0f32, percent)
                                .height(Length::Units(28)),
                        ),
                )
                .into()
            },
            _ => {
                // For all other states, the button is shown with different text/styling
                // dependant on the state
                let (button_text, button_style, enabled, custom_font_size) =
                    match &self.state {
                        GamePanelState::ReadyToPlay
                            if self.selected_server_browser_address.is_some() =>
                        {
                            (
                                "Connect to selected server",
                                ButtonStyle::Download(DownloadButtonStyle::Launch(
                                    ButtonState::Enabled,
                                )),
                                true,
                                Some(25),
                            )
                        },
                        GamePanelState::ReadyToPlay => (
                            "Launch",
                            ButtonStyle::Download(DownloadButtonStyle::Launch(
                                ButtonState::Enabled,
                            )),
                            true,
                            None,
                        ),
                        GamePanelState::Offline(true) => (
                            "Play Offline",
                            ButtonStyle::Download(DownloadButtonStyle::Launch(
                                ButtonState::Enabled,
                            )),
                            true,
                            None,
                        ),
                        GamePanelState::Offline(false) => (
                            "Try Again",
                            ButtonStyle::Download(DownloadButtonStyle::Update(
                                ButtonState::Enabled,
                            )),
                            true,
                            None,
                        ),
                        GamePanelState::Downloading { state, .. }
                            if *state == DownloadState::Starting =>
                        {
                            (
                                "Starting...",
                                ButtonStyle::Download(DownloadButtonStyle::Update(
                                    ButtonState::Disabled,
                                )),
                                false,
                                None,
                            )
                        },
                        GamePanelState::Retry => (
                            "Retry",
                            ButtonStyle::Download(DownloadButtonStyle::Update(
                                ButtonState::Enabled,
                            )),
                            true,
                            None,
                        ),
                        GamePanelState::Installing => (
                            "Installing...",
                            ButtonStyle::Download(DownloadButtonStyle::Launch(
                                ButtonState::Disabled,
                            )),
                            false,
                            None,
                        ),
                        GamePanelState::QueryingForUpdates(_) => (
                            "Loading...",
                            ButtonStyle::Download(DownloadButtonStyle::Update(
                                ButtonState::Disabled,
                            )),
                            false,
                            None,
                        ),
                        GamePanelState::Playing(_) => (
                            "Playing",
                            ButtonStyle::Download(DownloadButtonStyle::Launch(
                                ButtonState::Disabled,
                            )),
                            false,
                            None,
                        ),
                        GamePanelState::UpdateAvailable(_) => (
                            "Update",
                            ButtonStyle::Download(DownloadButtonStyle::Update(
                                ButtonState::Enabled,
                            )),
                            true,
                            None,
                        ),
                        _ => unreachable!(),
                    };

                let mut launch_button = button(
                    text(button_text)
                        .font(POPPINS_BOLD_FONT)
                        .size(custom_font_size.unwrap_or(45))
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center)
                        .width(Length::Fill),
                )
                .style(button_style)
                .width(Length::FillPortion(3))
                .height(Length::Units(75));

                if enabled {
                    launch_button = launch_button.on_press(
                        DefaultViewMessage::GamePanel(GamePanelMessage::PlayPressed),
                    );
                }

                let server_browser_button = button(
                    text("Server Browser")
                        .font(POPPINS_MEDIUM_FONT)
                        .size(22)
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .width(Length::FillPortion(1))
                .height(Length::Units(75))
                .style(ButtonStyle::ServerBrowser)
                .on_press(DefaultViewMessage::Interaction(
                    Interaction::ToggleServerBrowser,
                ));

                container(
                    row![]
                        .push(launch_button)
                        .push(server_browser_button)
                        .spacing(10),
                )
                .width(Length::Fill)
                .align_y(Vertical::Center)
                .into()
            },
        }
    }
}
