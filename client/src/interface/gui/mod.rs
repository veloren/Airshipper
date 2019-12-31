use crate::config::ClientConfig;
use iced::{
    button, scrollable, slider, Align, Application, Button, Color, Column, Command, Element,
    HorizontalAlignment, Image, Length, Row, Scrollable, Settings, Slider, Space, Svg, Text,
    VerticalAlignment,
};

pub fn run(_config: &mut ClientConfig) {
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
    Loaded(AirshipperState),
    //Downloading(AirshipperState),
    //Extracting(AirshipperState),
    //Playing(AirshipperState),
}

#[derive(Debug)]
struct AirshipperState {
    download_slider_state: slider::State,
    changelog_scrollable_state: scrollable::State,
    news_scrollable_state: scrollable::State,
    play_button_state: button::State,
    
    changelog: String,
    news: String,
    config: crate::config::ClientConfig,
}

#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<AirshipperData, LoadError>),
    PlayPressed,
    /// Do nothing.
    SliderChanged(f32),
}

/// Cached/persistent values like the changelog, news and profile information.
#[derive(Debug, Clone)]
struct AirshipperData {
    changelog: String,
    news: String,
    config: crate::config::ClientConfig,
}

impl AirshipperData {
    /// Loads up data like profile information, changelog, news
    /// either by a local cache or online.
    pub(crate) async fn load() -> Result<Self, LoadError> {
        // TODO: Cache it and don't request it every single time!
        let changelog = match reqwest::get("https://gitlab.com/veloren/veloren/raw/master/CHANGELOG.md") {
            Ok(mut resp) => {
                match resp.text() {
                    Ok(txt) => txt,
                    Err(_) => return Err(LoadError),
                }  
            },
            Err(_) => return Err(LoadError),
        };
        
        let config = crate::config::ClientConfig::load();
                
        Ok(Self {
            changelog,
            news: "To be done".into(),
            config,
        })
    }
}

/// TODO: Actually handle the error and display
/// an error message.
#[derive(Debug, Clone)]
struct LoadError;

impl Application for Airshipper {
    type Message = Message;

    fn new() -> (Self, Command<Message>) {
        (
            Airshipper::Loading,
            Command::perform(AirshipperData::load(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        format!("Airshipper v{}", env!("CARGO_PKG_VERSION"))
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Loaded(Ok(state)) => {
                *self = Airshipper::Loaded(AirshipperState {
                        download_slider_state: slider::State::default(),
                        changelog_scrollable_state: scrollable::State::default(),
                        news_scrollable_state: scrollable::State::default(),
                        play_button_state: button::State::default(),
                        
                        changelog: state.changelog,
                        news: state.news,
                        config: state.config,
                });
                
                Command::none()
            },
            Message::Loaded(Err(_e)) => {
                // TODO: Display network error.
                panic!("Still livin in the 80s? I need a working internet connection!");
            },
            Message::PlayPressed => {
                
                // TODO: do this asynchronously and feed back information about to the UI. 
                match self {
                    Self::Loaded(state) => {
                        // Default to checking for updates and starting the game.
                        log::info!("Checking for updates...");
                        state.config.update().expect("Failed updating the game.");
                        log::info!("Starting...");
                        state.config.start().expect("Failed to start the game");
                    },
                    _ => log::info!("Not ready yet :o"),
                }
                
                Command::none()
            },
            Message::SliderChanged(_) => Command::none(),
        }
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            Airshipper::Loading => Text::new("Loading...").into(),
            Airshipper::Loaded(state) => {
                let title = Image::new("client/assets/veloren-logo.png").width(Length::FillPortion(10));
                let discord = Svg::new("client/assets/discord.svg").width(Length::Fill);
                let gitlab = Svg::new("client/assets/gitlab.svg").width(Length::Fill);
                let youtube = Svg::new("client/assets/youtube.svg").width(Length::Fill);
                let reddit = Svg::new("client/assets/reddit.svg").width(Length::Fill);
                let twitter = Svg::new("client/assets/twitter.svg").width(Length::Fill);

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

                let news_test = Text::new(&state.news)

                .size(16);
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
                    Text::new("PLAY")
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
