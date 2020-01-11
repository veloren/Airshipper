mod style;
mod update;

use {
    crate::{profiles::Profile, saved_state::SavedState},
    iced::{
        button, scrollable, Align, Application, Button, Column, Command, Container, Element,
        HorizontalAlignment, Image, Length, ProgressBar, Row, Scrollable, Settings, Space, Svg,
        Text, VerticalAlignment,
    },
};

pub fn run() {
    let mut settings = Settings::default();
    settings.window.size = (800, 460);
    settings.window.resizable = false;
    Airshipper::run(settings);
}

#[derive(Debug, Clone)]
pub struct Airshipper {
    changelog_scrollable_state: scrollable::State,
    news_scrollable_state: scrollable::State,
    play_button_state: button::State,

    play_button_text: String,

    changelog: String,
    news: String,
    active_profile: Profile,

    saving: bool,
    downloading: bool,
}

impl Default for Airshipper {
    fn default() -> Self {
        Self {
            changelog_scrollable_state: Default::default(),
            news_scrollable_state: Default::default(),
            play_button_state: Default::default(),

            play_button_text: "PLAY".to_owned(),

            changelog: Default::default(),
            news: Default::default(),
            active_profile: Default::default(),

            saving: false,
            downloading: false,
        }
    }
}

impl From<SavedState> for Airshipper {
    fn from(saved: SavedState) -> Self {
        Self {
            changelog: saved.changelog,
            news: saved.news,
            active_profile: saved.active_profile,
            ..Default::default()
        }
    }
}

impl From<Airshipper> for SavedState {
    fn from(state: Airshipper) -> Self {
        SavedState {
            changelog: state.changelog,
            news: state.news,
            active_profile: state.active_profile,
        }
    }
}

impl Airshipper {
    fn update_from_save(&mut self, save: SavedState) {
        self.changelog = save.changelog;
        self.news = save.news;
        self.active_profile = save.active_profile;
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<SavedState, crate::saved_state::LoadError>),
    Saved(Result<(), crate::saved_state::SaveError>),
    DownloadDone(Profile),
    UpdateCheckDone(Profile),
    PlayPressed,
}

impl Application for Airshipper {
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

    fn update(&mut self, message: Message) -> Command<Message> {
        update::handle_message(self, message)
    }

    fn view(&mut self) -> Element<Message> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR").to_owned();
        let title = Container::new(Image::new(
            manifest_dir.clone() + "/assets/veloren-logo.png",
        ))
        .width(Length::FillPortion(10));
        let discord = Svg::new(manifest_dir.clone() + "/assets/discord.svg").width(Length::Fill);
        let gitlab = Svg::new(manifest_dir.clone() + "/assets/gitlab.svg").width(Length::Fill);
        let youtube = Svg::new(manifest_dir.clone() + "/assets/youtube.svg").width(Length::Fill);
        let reddit = Svg::new(manifest_dir.clone() + "/assets/reddit.svg").width(Length::Fill);
        let twitter = Svg::new(manifest_dir.clone() + "/assets/twitter.svg").width(Length::Fill);

        let icons = Row::new()
            .width(Length::Fill)
            .height(Length::Units(80))
            .align_items(Align::Center)
            .spacing(10)
            .padding(15)
            .push(title)
            .push(Space::with_width(Length::FillPortion(5)))
            .push(discord)
            .push(gitlab)
            .push(youtube)
            .push(reddit)
            .push(twitter);

        let changelog_text = Text::new(&self.changelog).size(14);
        let changelog = Scrollable::new(&mut self.changelog_scrollable_state)
            .height(Length::Fill)
            .padding(15)
            .spacing(20)
            .push(changelog_text);

        // Contains title, changelog
        let left = Column::new()
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .padding(15)
            .push(icons)
            .push(changelog);

        let news_test = Text::new(&self.news).size(14);
        let news = Scrollable::new(&mut self.news_scrollable_state)
            .spacing(20)
            .padding(25)
            .push(news_test);
        let news_container = Container::new(news)
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .style(style::News);

        // Contains logo, changelog and news
        let middle = Row::new().padding(2).push(left).push(news_container);
        let middle_container = Container::new(middle)
            .height(Length::FillPortion(6))
            .style(style::Middle);

        let download_speed = Text::new("8 kb / s").size(12);
        let download_progressbar = ProgressBar::new(0.0..=100.0, 20.0).style(style::Progress);
        let download = Column::new()
            .width(Length::FillPortion(4))
            .spacing(5)
            .push(download_speed)
            .push(download_progressbar);

        let play = Button::new(
            &mut self.play_button_state,
            Text::new(self.play_button_text.clone())
                .size(30)
                .height(Length::Fill)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center),
        )
        .on_press(Message::PlayPressed)
        .width(Length::Fill)
        .height(Length::Units(60))
        .padding(2)
        .style(style::PlayButton);

        let bottom = Row::new()
            .align_items(Align::End)
            .spacing(20)
            .padding(10)
            .push(download)
            .push(play);
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
