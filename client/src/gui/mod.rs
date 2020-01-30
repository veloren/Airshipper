mod style;
mod time;
mod update;

use crate::{
    error::ClientError, filesystem, network, profiles::Profile, state::SavedState, Result,
};
use iced::{
    button, scrollable, Align, Application, Button, Column, Command, Container, Element,
    HorizontalAlignment, Image, Length, ProgressBar, Row, Scrollable, Settings, Subscription, Text,
    VerticalAlignment,
};
use indicatif::HumanBytes;
use std::time::Duration;

/// Starts the GUI and won't return
pub fn run() {
    let mut settings = Settings::default();
    settings.window.size = (1050, 620);
    Airshipper::run(settings);
}

#[derive(Debug)]
pub enum LauncherState {
    LoadingSave,
    QueryingChangelogAndNews,
    UpdateAvailable,
    ReadyToPlay,
    Downloading(isahc::Metrics),
    Installing,
    Playing,

    Error(ClientError),
}

#[derive(Debug)]
pub struct Airshipper {
    /// Current state the GUI is in (e.g. Loading up the save file, updating veloren, ...)
    state: LauncherState,
    /// Persistent state which needs to get saved to disk
    saveable_state: SavedState,

    /// Other unrelated state
    changelog_scrollable_state: scrollable::State,
    news_scrollable_state: scrollable::State,
    play_button_state: button::State,

    saving: bool,
}

impl Default for Airshipper {
    fn default() -> Self {
        Self {
            state: LauncherState::LoadingSave,
            saveable_state: SavedState::empty(),
            changelog_scrollable_state: Default::default(),
            news_scrollable_state: Default::default(),
            play_button_state: Default::default(),

            saving: false,
        }
    }
}

impl Airshipper {
    fn into_save(&self) -> SavedState {
        self.saveable_state.clone()
    }
    fn update_from_save(&mut self, save: SavedState) {
        self.saveable_state = save;
    }
}

#[derive(Debug)]
pub enum Message {
    Interaction(Interaction),
    Loaded(Result<SavedState>),
    Saved(Result<()>),
    UpdateCheckDone(Result<(bool, Option<String>, Option<Vec<network::Post>>)>),
    Tick(()), // TODO: Get rid of Tick by implementing download via subscription
    InstallDone(Result<Profile>),
    PlayDone(Result<()>),
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
    type Executor = iced_futures::executor::AsyncStd;
    type Message = Message;

    fn new() -> (Self, Command<Message>) {
        (
            Airshipper::default(),
            Command::perform(SavedState::load(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        format!("Airshipper v{}", env!("CARGO_PKG_VERSION"))
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.state {
            LauncherState::Downloading(_) => {
                time::every(Duration::from_millis(300)).map(Message::Tick)
            }
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
        let title = Container::new(Image::new(filesystem::get_assets_path("veloren-logo.png")))
            .width(Length::FillPortion(10));
        // Will be reenabled once finished
        //let discord = Svg::new(manifest_dir.clone() + "/assets/discord.svg").width(Length::Fill);
        //let gitlab = Svg::new(manifest_dir.clone() + "/assets/gitlab.svg").width(Length::Fill);
        //let youtube = Svg::new(manifest_dir.clone() + "/assets/youtube.svg").width(Length::Fill);
        //let reddit = Svg::new(manifest_dir.clone() + "/assets/reddit.svg").width(Length::Fill);
        //let twitter = Svg::new(manifest_dir.clone() + "/assets/twitter.svg").width(Length::Fill);

        let icons = Row::new()
            .width(Length::Fill)
            .height(Length::Units(90))
            .align_items(Align::Center)
            .spacing(10)
            .padding(15)
            .push(title);
        //.push(Space::with_width(Length::FillPortion(5)))
        //.push(discord)
        //.push(gitlab)
        //.push(youtube)
        //.push(reddit)
        //.push(twitter);

        let changelog = Scrollable::new(&mut self.changelog_scrollable_state)
            .height(Length::Fill)
            .padding(15)
            .spacing(20)
            .push(Text::new(&self.saveable_state.changelog).size(18));

        // Contains title, changelog
        let left = Column::new()
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .padding(15)
            .push(icons)
            .push(changelog);

        let mut news = Scrollable::new(&mut self.news_scrollable_state)
            .spacing(20)
            .padding(25);

        for post in &mut self.saveable_state.news {
            news = news.push(Text::new(post.title.clone()).size(20));
            news = news.push(Text::new(post.description.clone()).size(16));
            let read_more_btn: Element<Interaction> = Button::new(
                &mut post.btn_state,
                Text::new("Read More")
                    .size(14)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .vertical_alignment(VerticalAlignment::Center),
            )
            .on_press(Interaction::ReadMore(post.button_url.clone()))
            .width(Length::Units(80))
            .height(Length::Units(25))
            .padding(2)
            .style(style::ReadMoreButton)
            .into();
            news = news.push(read_more_btn.map(Message::Interaction));
        }

        let news_container = Container::new(news)
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .style(style::News);

        // Contains logo, changelog and news
        let middle = Row::new().padding(2).push(left).push(news_container);
        let middle_container = Container::new(middle)
            .height(Length::FillPortion(6))
            .style(style::Middle);

        let download_text = match &self.state {
            LauncherState::Downloading(m) => format!(
                "Downloading... {}/sec",
                HumanBytes(m.download_speed() as u64)
            ),
            LauncherState::Installing => "Installing...".into(),
            LauncherState::LoadingSave => "Loading...".into(),
            LauncherState::QueryingChangelogAndNews => "Checking for updates...".into(),
            LauncherState::ReadyToPlay => "Ready to play...".into(),
            LauncherState::UpdateAvailable => "Update available!".into(),
            LauncherState::Playing => "Much fun playing!".into(),
            LauncherState::Error(e) => e.to_string(),
        };
        let download_progress = match &self.state {
            LauncherState::Downloading(m) => {
                // Percentage of completed download
                ((m.download_progress().0 * 100) / m.download_progress().1) as f32
            }
            _ => 0.0,
        };
        let play_button_text = match &self.state {
            LauncherState::Downloading(_) => format!("Downloading"),
            LauncherState::Installing => "Installing".into(),
            LauncherState::LoadingSave => "Loading".into(),
            LauncherState::QueryingChangelogAndNews => "Loading".into(),
            LauncherState::ReadyToPlay => "Play".into(),
            LauncherState::UpdateAvailable => "Update".into(),
            LauncherState::Playing => "Playing".into(),
            LauncherState::Error(_) => "ERROR".into(),
        };

        let download_speed = Text::new(&download_text).size(16);
        let download_progressbar =
            ProgressBar::new(0.0..=100.0, download_progress).style(style::Progress);
        let download = Column::new()
            .width(Length::FillPortion(4))
            .spacing(5)
            .push(download_speed)
            .push(download_progressbar);

        let mut play = Button::new(
            &mut self.play_button_state,
            Text::new(&play_button_text)
                .size(30)
                .height(Length::Fill)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center),
        )
        .on_press(Interaction::PlayPressed)
        .width(Length::Fill)
        .height(Length::Units(60))
        .style(style::PlayButton)
        .padding(2);

        // Disable button if loading, playing or downloading the game.
        match self.state {
            LauncherState::LoadingSave | LauncherState::Playing | LauncherState::Downloading(_) | LauncherState::QueryingChangelogAndNews => {
                play = play.style(style::PlayButtonDisabled);
                play = play.on_press(Interaction::Disabled);
            }
            _ => {}
        }
        let play: Element<Interaction> = play.into();

        let bottom = Row::new()
            .align_items(Align::End)
            .spacing(20)
            .padding(10)
            .push(download)
            .push(play.map(Message::Interaction));
        let bottom_container = Container::new(bottom).style(style::Bottom);

        // Contains everything
        let content = Column::new()
            .padding(2)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(middle_container)
            .push(bottom_container);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::Content)
            .into()
    }
}
