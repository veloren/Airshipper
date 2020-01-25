mod style;
mod time;
mod update;

use {
    crate::{network, profiles::Profile, saved_state::SavedState, Result},
    iced::{
        button, scrollable, Align, Application, Button, Column, Command, Container, Element,
        HorizontalAlignment, Image, Length, ProgressBar, Row, Scrollable, Settings, Subscription,
        Text, VerticalAlignment,
    },
    indicatif::HumanBytes,
    std::{path::PathBuf, time::Duration},
};

pub fn run() {
    let mut settings = Settings::default();
    settings.window.size = (1050, 620);
    Airshipper::run(settings);
}

#[derive(Debug, Clone)]
pub enum DownloadStage {
    None,
    Download(isahc::Metrics, PathBuf),
    Install,
}

#[derive(Debug)]
pub struct Airshipper {
    changelog_scrollable_state: scrollable::State,
    news_scrollable_state: scrollable::State,
    play_button_state: button::State,
    progress: f32,

    play_button_text: String,

    changelog: String,
    changelog_etag: String,
    news: Vec<network::Post>,
    news_etag: String,
    active_profile: Profile,

    saving: bool,
    download: DownloadStage,
    download_speed: HumanBytes,
}

impl Default for Airshipper {
    fn default() -> Self {
        Self {
            changelog_scrollable_state: Default::default(),
            news_scrollable_state: Default::default(),
            play_button_state: Default::default(),
            progress: 100.0,

            play_button_text: "PLAY".to_owned(),

            changelog: Default::default(),
            changelog_etag: Default::default(),
            news: Default::default(),
            news_etag: Default::default(),
            active_profile: Default::default(),

            saving: false,
            download: DownloadStage::None,
            download_speed: HumanBytes(0),
        }
    }
}

impl Airshipper {
    fn into_save(&self) -> SavedState {
        SavedState {
            changelog: self.changelog.clone(),
            changelog_etag: self.changelog_etag.clone(),
            news: self.news.clone(),
            news_etag: self.news_etag.clone(),
            active_profile: self.active_profile.clone(),
        }
    }
    fn update_from_save(&mut self, save: SavedState) {
        self.changelog = save.changelog;
        self.news = save.news;
        self.active_profile = save.active_profile;
    }
}

#[derive(Debug)]
pub enum Message {
    Interaction(Interaction),
    Loaded(Result<SavedState>),
    Saved(Result<()>),
    UpdateCheckDone(Result<(Profile, Option<String>, Option<Vec<network::Post>>)>),
    Tick(()), // TODO: Get rid of Tick by implementing download via subscription
    InstallDone(Result<Profile>),
    PlayDone(()),
}
#[derive(Debug, Clone)]
pub enum Interaction {
    PlayPressed,
    ReadMore(String),
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
        match self.download {
            DownloadStage::None => Subscription::none(),
            _ => time::every(Duration::from_millis(300)).map(Message::Tick),
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        update::handle_message(self, message)
    }

    fn view(&mut self) -> Element<Message> {
        // TODO: Use correct path
        let manifest_dir = env!("CARGO_MANIFEST_DIR").to_owned();
        let title = Container::new(Image::new(
            manifest_dir.clone() + "/assets/veloren-logo.png",
        ))
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
            .push(Text::new(&self.changelog).size(18));

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

        for post in &mut self.news {
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

        let download_text = match self.download {
            DownloadStage::None => "Ready to play...".into(),
            DownloadStage::Download(_, _) => self.download_speed.to_string(),
            DownloadStage::Install => "Installing...".into(),
        };

        let download_speed = Text::new(download_text).size(16);
        let download_progressbar =
            ProgressBar::new(0.0..=100.0, self.progress).style(style::Progress);
        let download = Column::new()
            .width(Length::FillPortion(4))
            .spacing(5)
            .push(download_speed)
            .push(download_progressbar);

        let play: Element<Interaction> = Button::new(
            &mut self.play_button_state,
            Text::new(self.play_button_text.clone())
                .size(30)
                .height(Length::Fill)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center),
        )
        .on_press(Interaction::PlayPressed)
        .width(Length::Fill)
        .height(Length::Units(60))
        .padding(2)
        .style(style::PlayButton)
        .into();

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
