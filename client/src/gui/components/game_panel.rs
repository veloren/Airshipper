use crate::{
    assets::{DOWNLOAD_ICON, POPPINS_BOLD_FONT, POPPINS_MEDIUM_FONT},
    gui::{
        custom_widgets::heading_with_rule,
        subscriptions,
        views::{default::DefaultViewMessage, Action},
    },
    io::ProcessUpdate,
    net::Progress,
    profiles::Profile,
    Result,
};
use iced::{
    alignment::{Horizontal, Vertical},
    pure::{button, column, container, progress_bar, row, text, widget::Image, Element},
    Length, Padding,
};
use iced_native::{image::Handle, Command};
use std::path::PathBuf;

use crate::gui::style::{ButtonState, DownloadButtonStyle};
use lazy_static::lazy_static;
use regex::Regex;

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
}

#[derive(Clone, Debug)]
pub enum GamePanelState {
    QueryingForUpdates(bool),
    UpdateAvailable(String),
    /// Url, Download Path, Version
    Downloading(String, PathBuf, String),
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
}

impl Default for GamePanelComponent {
    fn default() -> Self {
        Self {
            state: GamePanelState::QueryingForUpdates(false),
            download_progress: None,
        }
    }
}

impl GamePanelComponent {
    pub fn subscription(&self) -> iced::Subscription<GamePanelMessage> {
        match &self.state {
            GamePanelState::Downloading(url, location, _) => {
                subscriptions::download::file(url, location)
                    .map(|progress| GamePanelMessage::DownloadProgress(progress))
            },
            &GamePanelState::Playing(ref profile) => {
                subscriptions::process::stream(profile.clone())
                    .map(|process_update| GamePanelMessage::ProcessUpdate(process_update))
            },
            _ => iced::Subscription::none(),
        }
    }

    pub fn update(
        &mut self,
        msg: GamePanelMessage,
        active_profile: &Profile,
    ) -> Option<Command<DefaultViewMessage>> {
        let command = match msg {
            GamePanelMessage::PlayPressed => match &self.state {
                GamePanelState::UpdateAvailable(version) => {
                    self.state = GamePanelState::Downloading(
                        active_profile.url(),
                        active_profile.download_path(),
                        version.clone(),
                    );
                    None
                },
                GamePanelState::ReadyToPlay => {
                    println!("Play pressed");
                    self.state = GamePanelState::Playing(active_profile.clone());
                    None
                },
                GamePanelState::Retry => {
                    self.state = GamePanelState::QueryingForUpdates(true);
                    Some(Command::perform(
                        Profile::update(active_profile.clone()),
                        |update| {
                            DefaultViewMessage::GamePanel(GamePanelMessage::GameUpdate(
                                update,
                            ))
                        },
                    ))
                },
                GamePanelState::Offline(available) => {
                    match available {
                        // Play offline
                        true => {
                            self.state = GamePanelState::Playing(active_profile.clone());
                            None
                        },
                        // Retry
                        false => {
                            self.state = GamePanelState::QueryingForUpdates(true);

                            // The game has never been downloaded so the only option is to
                            // retry the download
                            Some(Command::perform(
                                Profile::update(active_profile.clone()),
                                |update| {
                                    DefaultViewMessage::GamePanel(
                                        GamePanelMessage::GameUpdate(update),
                                    )
                                },
                            ))
                        },
                    }
                },

                GamePanelState::Installing
                | GamePanelState::Downloading(_, _, _)
                | GamePanelState::Playing(..)
                | GamePanelState::QueryingForUpdates(_) => None,
            },
            GamePanelMessage::DownloadProgress(progress) => {
                match progress {
                    Progress::Errored(err) => {
                        tracing::error!("Download failed with: {}", err);
                        self.state = GamePanelState::Retry;
                        let mut profile = active_profile.clone();
                        profile.version = None;
                        return Some(Command::perform(
                            async { Action::UpdateProfile(profile) },
                            DefaultViewMessage::Action,
                        ));
                    },
                    Progress::Finished => {
                        let version = match &self.state {
                            GamePanelState::Downloading(_, _, version) => {
                                version.to_string()
                            },
                            _ => panic!(
                                "Reached impossible state: Downloading while not in \
                                 download state!"
                            ),
                        };
                        self.state = GamePanelState::Installing;
                        return Some(Command::perform(
                            Profile::install(active_profile.clone(), version),
                            |result| {
                                DefaultViewMessage::GamePanel(
                                    GamePanelMessage::InstallDone(result),
                                )
                            },
                        ));
                    },
                    p => {
                        match &self.state {
                            // TODO: Too much cloning figure out a better way
                            GamePanelState::Downloading(_, _, _) => {
                                self.download_progress = Some(p);
                            },
                            _ => panic!(
                                "Received progress update for non-existent download"
                            ),
                        };

                        None
                    },
                }
            },
            GamePanelMessage::InstallDone(profile) => match profile {
                Ok(profile) => {
                    self.state = GamePanelState::ReadyToPlay;
                    return Some(Command::perform(
                        async { Action::UpdateProfile(profile) },
                        DefaultViewMessage::Action,
                    ));
                },
                Err(_e) => {
                    // TODO: Fix
                    // tracing::error!("Installation failed with: {}", e);
                    self.state = GamePanelState::Retry;
                    let mut profile = active_profile.clone();
                    profile.version = None;
                    return Some(Command::perform(
                        async { Action::UpdateProfile(profile) },
                        DefaultViewMessage::Action,
                    ));
                },
            },
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
                    None
                },
                ProcessUpdate::Exit(code) => {
                    tracing::debug!("Veloren exited with {}", code);
                    self.state = GamePanelState::QueryingForUpdates(false);
                    return Some(Command::perform(
                        Profile::update(active_profile.clone()),
                        |update| {
                            DefaultViewMessage::GamePanel(GamePanelMessage::GameUpdate(
                                update,
                            ))
                        },
                    ));
                },
                ProcessUpdate::Error(err) => {
                    tracing::error!(
                        "Failed to receive an update from Veloren process! {}",
                        err
                    );
                    self.state = GamePanelState::Retry;
                    None
                },
            },
            GamePanelMessage::GameUpdate(update) => {
                match update {
                    Ok(Some(version)) => {
                        // Skip asking
                        if let GamePanelState::QueryingForUpdates(true) = self.state {
                            println!("GameUpdate, QueryingForUpdates(true)");
                            self.state = GamePanelState::Downloading(
                                active_profile.url(),
                                active_profile.download_path(),
                                version,
                            );
                        } else {
                            println!("GameUpdate, not querying for updates true");
                            self.state = GamePanelState::UpdateAvailable(version);
                        }
                    },
                    Ok(None) => {
                        println!("GameUpdate, None");
                        self.state = GamePanelState::ReadyToPlay;
                    },
                    Err(_) => {
                        println!("GameUpdate, Err");
                        // Go into offline mode incase game can't be updated.
                        if active_profile.installed() {
                            self.state = GamePanelState::Offline(true);
                        } else {
                            self.state = GamePanelState::Offline(false);
                        }
                    },
                };
                None
            },
        };

        command
    }

    pub fn view(&self) -> Element<GamePanelMessage> {
        column()
            .push(heading_with_rule::<GamePanelMessage>("Game Version"))
            .push(
                container(self.download_area())
                    .width(Length::Fill)
                    .padding(Padding::from([10, 20, 20, 20])),
            )
            .into()
    }
}

impl GamePanelComponent {
    fn download_area(&self) -> Element<GamePanelMessage> {
        println!("{:?}", self.state);
        match &self.state {
            GamePanelState::ReadyToPlay
            | GamePanelState::UpdateAvailable(_)
            | GamePanelState::Playing(_)
            | GamePanelState::Offline(_)
            | GamePanelState::Installing
            | GamePanelState::Retry => {
                let (button_text, button_style, enabled) = match self.state {
                    GamePanelState::ReadyToPlay => (
                        "Launch",
                        DownloadButtonStyle::Launch(ButtonState::Enabled),
                        true,
                    ),
                    GamePanelState::Offline(true) => (
                        "Play Offline",
                        DownloadButtonStyle::Launch(ButtonState::Enabled),
                        true,
                    ),
                    GamePanelState::Offline(false) => (
                        "Try Again",
                        DownloadButtonStyle::Update(ButtonState::Enabled),
                        true,
                    ),
                    GamePanelState::Retry => (
                        "Retry",
                        DownloadButtonStyle::Update(ButtonState::Enabled),
                        true,
                    ),
                    GamePanelState::Installing => (
                        "Installing...",
                        DownloadButtonStyle::Launch(ButtonState::Disabled),
                        false,
                    ),
                    GamePanelState::QueryingForUpdates(_) => (
                        "Loading...",
                        DownloadButtonStyle::Update(ButtonState::Disabled),
                        false,
                    ),
                    GamePanelState::Playing(_) => (
                        "Playing",
                        DownloadButtonStyle::Launch(ButtonState::Disabled),
                        false,
                    ),
                    GamePanelState::UpdateAvailable(_) => (
                        "Update",
                        DownloadButtonStyle::Update(ButtonState::Enabled),
                        true,
                    ),
                    _ => unreachable!(),
                };

                let mut button = button(
                    text(button_text)
                        .font(POPPINS_BOLD_FONT)
                        .size(45)
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center)
                        .width(Length::Fill),
                )
                .style(button_style)
                .width(Length::Fill)
                .height(Length::Units(75));

                if enabled {
                    button = button.on_press(GamePanelMessage::PlayPressed);
                }

                container(button)
                    .width(Length::Fill)
                    .align_y(Vertical::Center)
                    .into()
            },
            GamePanelState::Downloading(_, _, _) => {
                let download_progress = match self
                    .download_progress
                    .as_ref()
                    .unwrap_or(&Progress::Started)
                {
                    Progress::Advanced(_msg, percentage) => *percentage as f32,
                    Progress::Finished => 100.0,
                    _ => 0.0,
                };

                container(
                    column()
                        .push(text("Downloading").font(POPPINS_MEDIUM_FONT))
                        .push(
                            row()
                                .push(Image::new(Handle::from_memory(
                                    DOWNLOAD_ICON.to_vec(),
                                )))
                                .push(progress_bar(0.0..=100.0f32, download_progress)),
                        ),
                )
                .into()
            },
            GamePanelState::QueryingForUpdates(skip) => {
                println!("querying for updates");
                text(format!("Querying for Updates ({})", skip)).into()
            },
        }
    }
}
