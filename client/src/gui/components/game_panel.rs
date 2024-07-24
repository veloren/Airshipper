use crate::{
    assets::{DOWNLOAD_ICON, POPPINS_BOLD_FONT, POPPINS_MEDIUM_FONT, SETTINGS_ICON},
    error::ClientError,
    gui::{
        custom_widgets::heading_with_rule,
        style::{
            button::{ButtonState, ButtonStyle, DownloadButtonStyle},
            text::TextStyle,
        },
        subscriptions,
        views::{
            default::{
                DefaultViewMessage,
                Interaction::{self, SettingsPressed},
            },
            Action,
        },
        widget::*,
    },
    io::ProcessUpdate,
    logger::{pretty_bytes, redirect_voxygen_log},
    profiles::Profile,
    update::{Progress, State, UpdateParameters},
};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{
        button, column, container, image, image::Handle, progress_bar, row, text,
        tooltip, tooltip::Position, Image,
    },
    Alignment, Command, Length, Padding,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

use crate::gui::style::container::ContainerStyle;
use tracing::debug;

#[derive(Clone, Debug)]
pub enum GamePanelMessage {
    ProcessUpdate(ProcessUpdate),
    DownloadProgress(Option<Progress>),
    PlayPressed,
    ServerBrowserServerChanged(Option<String>),
    StartUpdate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DownloadButtonState {
    Checking,
    WaitForConfirm,
    InProgress,
}

#[derive(Clone)]
pub enum GamePanelState {
    Updating {
        astate: Arc<Mutex<Option<State>>>,
        btnstate: DownloadButtonState,
    },
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

impl std::fmt::Debug for GamePanelState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GamePanelState::Updating { .. } => write!(f, "GamePanelState::Updating"),
            GamePanelState::ReadyToPlay => write!(f, "GamePanelState::ReadyToPlay"),
            GamePanelState::Playing(_) => write!(f, "GamePanelState::Playing"),
            GamePanelState::Offline(_) => write!(f, "GamePanelState::Offline"),
            GamePanelState::Retry => write!(f, "GamePanelState::Retry"),
        }
    }
}

impl Default for GamePanelComponent {
    fn default() -> Self {
        Self {
            state: GamePanelState::ReadyToPlay,
            download_progress: None,
            selected_server_browser_address: None,
        }
    }
}

impl GamePanelComponent {
    pub fn subscription(&self) -> iced::Subscription<GamePanelMessage> {
        match &self.state {
            GamePanelState::Playing(profile) => subscriptions::process::stream(
                profile.clone(),
                self.selected_server_browser_address.clone(),
            )
            .map(GamePanelMessage::ProcessUpdate),
            _ => iced::Subscription::none(),
        }
    }

    fn trigger_next_state(
        state: State,
        empty_arc_state: Arc<Mutex<Option<State>>>,
        dstate: DownloadButtonState,
    ) -> (Option<GamePanelState>, Option<Command<DefaultViewMessage>>) {
        (
            Some(GamePanelState::Updating {
                astate: empty_arc_state.clone(),
                btnstate: dstate.clone(),
            }),
            Some(Command::perform(
                async move {
                    let start_time = Instant::now();
                    let mut last_progress = None;
                    let mut lstate = state;
                    // ICED is really slow, so we have to do multiple steps
                    while start_time.elapsed() < Duration::from_millis(30) {
                        match lstate.progress().await {
                            Some((progress, state)) => {
                                lstate = state;
                                last_progress = Some(progress);
                                if matches!(
                                    last_progress,
                                    Some(Progress::ReadyToDownload)
                                ) {
                                    // wait for user input!
                                    break;
                                }
                            },
                            None => {
                                return last_progress;
                            },
                        }
                    }
                    *empty_arc_state.lock().await = Some(lstate);
                    last_progress
                },
                |progress| {
                    DefaultViewMessage::GamePanel(GamePanelMessage::DownloadProgress(
                        progress,
                    ))
                },
            )),
        )
    }

    pub fn update(
        &mut self,
        msg: GamePanelMessage,
        active_profile: &Profile,
    ) -> Option<Command<DefaultViewMessage>> {
        let (next_state, command) = match msg {
            GamePanelMessage::PlayPressed => match &self.state {
                GamePanelState::ReadyToPlay => {
                    (Some(GamePanelState::Playing(active_profile.clone())), None)
                },
                GamePanelState::Retry => (
                    None,
                    Some(Command::perform(async {}, |_| {
                        DefaultViewMessage::GamePanel(GamePanelMessage::StartUpdate)
                    })),
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
                                None,
                                Some(Command::perform(async {}, |_| {
                                    DefaultViewMessage::GamePanel(
                                        GamePanelMessage::StartUpdate,
                                    )
                                })),
                            )
                        },
                    }
                },
                GamePanelState::Updating { btnstate, astate }
                    if *btnstate == DownloadButtonState::WaitForConfirm =>
                {
                    let state = {
                        let mut l = astate.blocking_lock();
                        l.take().expect("impossible, should always be filled")
                    };
                    Self::trigger_next_state(
                        state,
                        astate.clone(),
                        DownloadButtonState::InProgress,
                    )
                },
                GamePanelState::Updating { .. } | GamePanelState::Playing(..) => {
                    (None, None)
                },
            },
            GamePanelMessage::StartUpdate => {
                let state = State::ToBeEvaluated(UpdateParameters {
                    profile: active_profile.clone(),
                    force_complete_redownload: false,
                });

                let astate = Arc::new(Mutex::new(None));
                Self::trigger_next_state(state, astate, DownloadButtonState::Checking)
            },
            GamePanelMessage::DownloadProgress(progress) => {
                self.download_progress = progress.clone();

                match progress {
                    Some(Progress::Errored(err)) => {
                        tracing::error!("Download failed with: {}", err);
                        let mut profile = active_profile.clone();
                        profile.version = None;
                        let next_state = if matches!(err, ClientError::NetworkError) {
                            if active_profile.installed() {
                                GamePanelState::Offline(true)
                            } else {
                                GamePanelState::Offline(false)
                            }
                        } else {
                            GamePanelState::Retry
                        };
                        (
                            Some(next_state),
                            Some(Command::perform(
                                async { Action::UpdateProfile(profile) },
                                DefaultViewMessage::Action,
                            )),
                        )
                    },
                    Some(Progress::Successful(profile)) => (
                        Some(GamePanelState::ReadyToPlay),
                        Some(Command::perform(
                            async { Action::UpdateProfile(profile) },
                            DefaultViewMessage::Action,
                        )),
                    ),
                    Some(Progress::InProgress(_)) | Some(Progress::Evaluating) => {
                        if let GamePanelState::Updating { astate, btnstate } = &self.state
                        {
                            let state = {
                                let mut l = astate.blocking_lock();
                                l.take().expect("impossible, should always be filled")
                            };
                            Self::trigger_next_state(
                                state,
                                astate.clone(),
                                btnstate.clone(),
                            )
                        } else {
                            tracing::warn!("Wrong State");
                            (None, None)
                        }
                    },
                    Some(Progress::ReadyToDownload) => {
                        tracing::info!("Need to confirm the update");
                        (
                            if let GamePanelState::Updating { astate, .. } = &self.state {
                                Some(GamePanelState::Updating {
                                    astate: astate.clone(),
                                    btnstate: DownloadButtonState::WaitForConfirm,
                                })
                            } else {
                                None
                            },
                            None,
                        )
                    },
                    None => (None, None),
                }
            },
            // TODO: Move this out of GamePanelComponent? This code handles redirecting
            // voxygen output to Airshipper's log output
            GamePanelMessage::ProcessUpdate(update) => match update {
                ProcessUpdate::Line(msg) => {
                    redirect_voxygen_log(&msg);
                    (None, None)
                },
                ProcessUpdate::Exit(code) => {
                    debug!("Veloren exited with {}", code);
                    (
                        Some(GamePanelState::Retry),
                        Some(Command::perform(async {}, |_| {
                            DefaultViewMessage::GamePanel(GamePanelMessage::StartUpdate)
                        })),
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
                        .height(Length::Fixed(30.0))
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
                            .style(ContainerStyle::Tooltip)
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
        use GamePanelState::*;
        let same = match &self.state {
            Updating { .. } => matches!(state, Updating { .. }),
            ReadyToPlay => matches!(state, ReadyToPlay),
            Playing(_) => matches!(state, Playing(_)),
            Offline(_) => matches!(state, Offline(_)),
            Retry => matches!(state, Retry),
        };
        if !same {
            debug!("GamePanel state: {:?} -> {:?}", self.state, state);
        }
        self.state = state;
    }

    fn download_area(&self) -> Element<DefaultViewMessage> {
        match &self.state {
            GamePanelState::Updating { btnstate, .. }
                if *btnstate == DownloadButtonState::InProgress =>
            {
                // When the game is downloading, the download progress bar and related
                // stats replace the Launch / Update button
                let (percent, total, downloaded, bytes_per_sec, remaining) = match self
                    .download_progress
                    .as_ref()
                    .unwrap_or(&Progress::Evaluating)
                {
                    Progress::InProgress(progress_data) => (
                        progress_data.percent_complete() as f32,
                        progress_data.total_bytes,
                        progress_data.downloaded_bytes,
                        progress_data.bytes_per_sec,
                        progress_data.remaining(),
                    ),
                    Progress::Successful(_) => (100.0, 0, 0, 0, Duration::from_secs(0)),
                    _ => (0.0, 0, 0, 0, Duration::from_secs(0)),
                };

                let download_rate = bytes_per_sec as f32 / 1_000_000.0;

                let progress_text =
                    format!("{} / {}", pretty_bytes(downloaded), pretty_bytes(total));

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
                                .height(Length::Fixed(28.0)),
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
                        GamePanelState::Updating {
                            btnstate: dstate, ..
                        } => match *dstate {
                            DownloadButtonState::Checking => (
                                "Checking...",
                                ButtonStyle::Download(DownloadButtonStyle::Update(
                                    ButtonState::Disabled,
                                )),
                                false,
                                None,
                            ),
                            DownloadButtonState::WaitForConfirm => (
                                "Download",
                                ButtonStyle::Download(DownloadButtonStyle::Update(
                                    ButtonState::Enabled,
                                )),
                                true,
                                None,
                            ),
                            _ => unreachable!(),
                        },
                        GamePanelState::Retry => (
                            "Retry",
                            ButtonStyle::Download(DownloadButtonStyle::Update(
                                ButtonState::Enabled,
                            )),
                            true,
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
                .height(Length::Fixed(75.0));

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
                .height(Length::Fixed(75.0))
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
