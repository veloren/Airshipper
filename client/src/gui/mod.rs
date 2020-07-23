mod style;
mod subscriptions;
mod update;
pub mod widgets;

use crate::{
    cli::CmdLine, error::ClientError, io, net, profiles::Profile, state::SavedState,
    CommandBuilder, Result,
};
use iced::{
    button, image::Handle, Align, Application, Column, Command, Container, Element,
    Image, Length, ProgressBar, Row, Settings, Subscription, Text,
};
use std::path::PathBuf;
use subscriptions::{download, process};
use widgets::{Changelog, News};

/// Starts the GUI and won't return
pub fn run(cmd: CmdLine) {
    Airshipper::run(settings(cmd))
}

#[derive(Debug)]
pub enum LauncherState {
    LoadingSave,
    // do not ask, used for retry.
    QueryingForUpdates(bool),
    UpdateAvailable(String),
    /// Url, Download Path, Version
    Downloading(String, PathBuf, String),
    Installing,
    ReadyToPlay,
    Playing(CommandBuilder),

    Error(ClientError),
}

#[derive(Debug)]
pub struct Airshipper {
    /// Current state the GUI is in (e.g. Loading up the save file, updating veloren,
    /// ...)
    /// TODO: Use a state machine to make impossible states impossible
    state: LauncherState,
    /// Persistent state which needs to get saved to disk
    saveable_state: SavedState,

    /// Other unrelated state
    play_button_state: button::State,
    download_progress: Option<net::Progress>,

    needs_save: bool,
    saving: bool,
}

impl Default for Airshipper {
    fn default() -> Self {
        Self {
            state: LauncherState::LoadingSave,
            saveable_state: SavedState::empty(),
            play_button_state: Default::default(),
            download_progress: None,

            needs_save: false,
            saving: false,
        }
    }
}

impl Airshipper {
    fn save_state(&self) -> SavedState {
        self.saveable_state.clone()
    }
    fn update_from_save(&mut self, save: SavedState) {
        self.saveable_state = save;
    }

    /// Resets everything **except** current LauncherState.
    ///
    /// It's recommended after calling it to set the state manually.
    pub fn reset(&mut self) {
        self.download_progress = None;
        self.needs_save = false;
        self.saving = false;
    }
}

#[derive(Debug)]
pub enum Message {
    Interaction(Interaction),
    Loaded(Result<SavedState>),
    Saved(Result<()>),
    ChangelogUpdate(Result<Option<Changelog>>),
    NewsUpdate(Result<Option<News>>),
    GameUpdate(Result<Option<String>>),
    ProcessUpdate(io::ProcessUpdate),
    DownloadProgress(net::Progress),
    InstallDone(Result<Profile>),
    Error(ClientError),
}

#[derive(Debug, Clone)]
pub enum Interaction {
    PlayPressed,
    ReadMore(String),
    // Interaction won't do anything
    Disabled,
}

impl Application for Airshipper {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = CmdLine;

    fn new(_flags: CmdLine) -> (Self, Command<Message>) {
        (
            Airshipper::default(),
            Command::perform(SavedState::load(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        format!(
            "Airshipper{} v{}",
            if self.needs_save { "*" } else { "" },
            env!("CARGO_PKG_VERSION")
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.state {
            LauncherState::Downloading(url, location, _) => {
                download::file(&url, &location).map(Message::DownloadProgress)
            },
            LauncherState::Playing(cmd) => {
                process::stream(cmd).map(Message::ProcessUpdate)
            },
            _ => Subscription::none(),
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match update::handle_message(self, message) {
            Ok(x) => x,
            Err(e) => Command::perform(async { e }, Message::Error),
        }
    }

    fn view(&mut self) -> Element<Message> {
        let Airshipper {
            saveable_state,
            play_button_state,
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
            .push(saveable_state.changelog.view());

        // Contains logo, changelog and news
        let middle = Container::new(
            Row::new()
                .padding(2)
                .push(left)
                .push(saveable_state.news.view()),
        )
        .height(Length::FillPortion(6))
        .style(style::Middle);

        let download_progress = match &self.state {
            LauncherState::Downloading(_, _, _) => {
                if let Some(prog) = &self.download_progress {
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
        let play_button_text = match &self.state {
            LauncherState::Downloading(_, _, _) => "Downloading".to_string(),
            LauncherState::Installing => "Installing".into(),
            LauncherState::LoadingSave => "Loading".into(),
            LauncherState::QueryingForUpdates(_) => "Loading".into(),
            LauncherState::ReadyToPlay => "Play".into(),
            LauncherState::UpdateAvailable(_) => "Update".into(),
            LauncherState::Playing(_) => "Playing".into(),
            LauncherState::Error(_) => "Retry".into(),
        };

        let download_text = match &self.state {
            LauncherState::Downloading(_, _, _) => self
                .download_progress
                .as_ref()
                .map(|x| x.to_string())
                .unwrap_or_else(|| "Downloading...".to_string()),
            LauncherState::Installing => "Installing...".to_string(),
            LauncherState::LoadingSave => "Loading...".to_string(),
            LauncherState::QueryingForUpdates(_) => "Checking for updates...".to_string(),
            LauncherState::ReadyToPlay => "Ready to play...".to_string(),
            LauncherState::UpdateAvailable(_) => "Update available!".to_string(),
            LauncherState::Playing(_) => "Much fun playing!".to_string(),
            LauncherState::Error(e) => e.to_string(),
        };
        let download_speed = Text::new(&download_text).size(16);
        let download_progressbar =
            ProgressBar::new(0.0..=100.0, download_progress).style(style::Progress);
        let download = Column::new()
            .width(Length::FillPortion(4))
            .spacing(5)
            .push(download_speed)
            .push(download_progressbar);

        let play = widgets::primary_button(
            play_button_state,
            play_button_text,
            match self.state {
                LauncherState::ReadyToPlay
                | LauncherState::UpdateAvailable(_)
                | LauncherState::Error(_) => Interaction::PlayPressed,
                _ => Interaction::Disabled,
            },
            match self.state {
                LauncherState::ReadyToPlay
                | LauncherState::UpdateAvailable(_)
                | LauncherState::Error(_) => style::PrimaryButton::Enabled,
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
}

fn settings(cmd: CmdLine) -> Settings<CmdLine> {
    use iced::window::{Icon, Settings as Window};
    use image::GenericImageView;
    let icon = image::load_from_memory(crate::assets::VELOREN_ICON).unwrap();

    Settings {
        window: Window {
            size: (1050, 620),
            resizable: true,
            decorations: true,
            icon: Some(
                Icon::from_rgba(icon.to_rgba().into_raw(), icon.width(), icon.height())
                    .unwrap(),
            ),
            min_size: Some((1050, 620)),
            ..Default::default()
        },
        flags: cmd,
        default_font: Some(crate::assets::FONT),
        default_text_size: 20,
        // Enforce the usage of dedicated gpu if available
        antialiasing: true,
    }
}
