use super::Action;
use crate::{
    cli::CmdLine,
    gui::{
        components::{Changelog, News},
        style, subscriptions, Result,
    },
    io, net,
    profiles::Profile,
    ProcessUpdate,
};
use iced::{
    button, image::Handle, Align, Button, Column, Command, Container, Element,
    HorizontalAlignment, Image, Length, ProgressBar, Row, Text, VerticalAlignment,
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
    download_progress: Option<net::Progress>,
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
    /// Profile, verbosity
    Playing(Profile, i32),

    Retry,
    /// bool indicates whether Veloren can be started offline
    Offline(bool),
}

impl Default for State {
    fn default() -> Self {
        Self::QueryingForUpdates(false)
    }
}

#[derive(Debug)]
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
    ReadMore(String),

    Disabled,
}

impl DefaultView {
    pub fn subscription(&self) -> iced::Subscription<DefaultViewMessage> {
        match &self.state {
            State::Downloading(url, location, _) => {
                subscriptions::download::file(&url, &location)
                    .map(DefaultViewMessage::DownloadProgress)
            },
            &State::Playing(ref profile, verbose) => {
                subscriptions::process::stream(Profile::start(profile, verbose))
                    .map(DefaultViewMessage::ProcessUpdate)
            },
            _ => iced::Subscription::none(),
        }
    }

    pub fn view(&mut self) -> Element<DefaultViewMessage> {
        let Self {
            changelog,
            news,
            state,
            play_button_state,
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

        // Contains logo, changelog and news
        let middle = Container::new(Row::new().padding(2).push(left).push(news.view()))
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

        let bottom = Container::new(
            Row::new()
                .align_items(Align::End)
                .spacing(20)
                .padding(10)
                .push(download)
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
        cmd: &CmdLine,
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
                        self.state = State::Playing(active_profile.clone(), cmd.verbose);
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
                                State::Playing(active_profile.clone(), cmd.verbose);
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
            .size(30)
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
