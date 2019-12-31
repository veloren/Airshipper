use {
crate::{profiles::{Profile, Channel}, saved_state::{self, SavedState}},
    iced::{
    button, scrollable, slider, Align, Application, Button, Color, Column, Command, Element,
    HorizontalAlignment, Image, Length, Row, Scrollable, Settings, Slider, Space, Svg, Text,
    VerticalAlignment,
}};

pub fn run() {
    let mut settings = Settings::default();
    settings.window.size = (800, 460);
    settings.window.resizable = false;
    Airshipper::run(settings);
}

/// Represents the state of Airshipper.
/// Like loading the changelog, news or downloading/extracting the game.
#[derive(Debug)]
enum Airshipper {
    Loading,
    Loaded(State),
}

#[derive(Default, Debug, Clone)]
struct State {
    download_slider_state: slider::State,
    changelog_scrollable_state: scrollable::State,
    news_scrollable_state: scrollable::State,
    play_button_state: button::State,

    changelog: String,
    news: String,
    profiles: Vec<Profile>,

    downloading: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<SavedState, crate::saved_state::LoadError>),
    DownloadDone(Vec<Profile>),
    PlayPressed,
    SliderChanged(f32),
}

async fn download_stuff(mut state: State) -> Vec<Profile> {
    // Default to checking for updates and starting the game.
    if state.profiles.is_empty() {
        log::info!("Downloading from a new profile...");
        state.profiles.push(Profile::from_download("default".to_owned(), Channel::Nightly, "https://download.veloren.net".to_owned(), saved_state::get_profiles_path()).unwrap());
    }
    log::info!("Done checking for updates...");
    state.profiles
}

impl Application for Airshipper {
    type Message = Message;

    fn new() -> (Self, Command<Message>) {
        (
            Airshipper::Loading,
            Command::perform(SavedState::load(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        format!("Airshipper v{}", env!("CARGO_PKG_VERSION"))
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Loaded(saved_state) => {
                let state = saved_state
                    .map(|state| State {
                        changelog: state.changelog,
                        news: state.news,
                        profiles: state.profiles,
                        ..Default::default()
                    })
                    .unwrap_or_default();

                *self = Airshipper::Loaded(state);

                Command::none()
            }
            Message::PlayPressed => {
                // TODO: do this asynchronously and feed back information about to the UI.
                match self {
                    Self::Loaded(state) => {
                        state.downloading = true;
                        let mut state = state.clone();
                        Command::perform(download_stuff(state), Message::DownloadDone)
                    }
                    _ => {
                        log::info!("Not ready yet :o");
                        Command::none()
                    }
                }
            }
            Message::DownloadDone(profiles) => {
                if let Self::Loaded(state) = self {
                    state.profiles = profiles;
                }
                Command::none()
            }
            Message::SliderChanged(_) => Command::none(),
            _ => Command::none()
        }
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            Airshipper::Loading => Text::new("").into(),
            Airshipper::Loaded(state) => {
                let manifest_dir = env!("CARGO_MANIFEST_DIR").to_owned();
                let title = Image::new(manifest_dir.clone() + "/assets/veloren-logo.png")
                    .width(Length::FillPortion(10));
                let discord =
                    Svg::new(manifest_dir.clone() + "/assets/discord.svg").width(Length::Fill);
                let gitlab =
                    Svg::new(manifest_dir.clone() + "/assets/gitlab.svg").width(Length::Fill);
                let youtube =
                    Svg::new(manifest_dir.clone() + "/assets/youtube.svg").width(Length::Fill);
                let reddit =
                    Svg::new(manifest_dir.clone() + "/assets/reddit.svg").width(Length::Fill);
                let twitter =
                    Svg::new(manifest_dir.clone() + "/assets/twitter.svg").width(Length::Fill);

                let icons = Row::new()
                    .align_items(Align::Center)
                    .spacing(10)
                    .push(title)
                    .push(Space::with_width(Length::FillPortion(5)))
                    .push(Column::new())
                    .push(discord)
                    .push(gitlab)
                    .push(youtube)
                    .push(reddit)
                    .push(twitter);

                let changelog_text = Text::new(&state.changelog).size(18);
                let changelog = Scrollable::new(&mut state.changelog_scrollable_state)
                    .height(Length::Fill)
                    .spacing(20)
                    .push(changelog_text);

                // Contains title, changelog
                let left = Column::new()
                    .width(Length::FillPortion(2))
                    .height(Length::Fill)
                    .spacing(20)
                    .push(icons)
                    .push(changelog);

                let news_test = Text::new(&state.news).size(16);
                let news = Scrollable::new(&mut state.news_scrollable_state)
                    .width(Length::Fill)
                    .spacing(20)
                    .push(news_test);

                // Contains logo, changelog and news
                let middle = Row::new()
                    .height(Length::FillPortion(5))
                    .padding(25)
                    .spacing(60)
                    .push(left)
                    .push(news);

                let download_speed = Text::new("8 kb / s").size(12);
                let download_slider = Slider::new(
                    &mut state.download_slider_state,
                    0.0..=100.0,
                    20.0,
                    Message::SliderChanged,
                );
                let download = Column::new()
                    .width(Length::FillPortion(4))
                    .spacing(5)
                    .push(download_speed)
                    .push(download_slider);

                let play = Button::new(
                    &mut state.play_button_state,
                    Text::new(if state.downloading { "Loading" } else { "PLAY" })
                        .size(60)
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center)
                        .color(Color::WHITE),
                )
                .on_press(Message::PlayPressed)
                .width(Length::Fill)
                .height(Length::Fill)
                .background(Color::from_rgb(0.2, 0.2, 0.7));

                let bottom = Row::new()
                    .align_items(Align::End)
                    .height(Length::Fill)
                    .spacing(20)
                    .push(download)
                    .push(play);

                // Contains everything
                let content = Column::new()
                    .spacing(20)
                    .padding(20)
                    .push(middle)
                    .push(bottom);

                content.into()
            }
        }
    }
}
