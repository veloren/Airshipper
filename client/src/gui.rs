use {
    crate::{profiles::Profile, saved_state::SavedState},
    iced::{
        Container, container, button, scrollable, Align, Application, Background, Button, Color, Column, Command,
        Element, HorizontalAlignment, Image, Length, progress_bar, ProgressBar, Row, Scrollable, Settings, Space,
        Svg, Text, Vector, VerticalAlignment,
    },
};

pub fn run() {
    let mut settings = Settings::default();
    settings.window.size = (800, 460);
    settings.window.resizable = false;
    Airshipper::run(settings);
}

#[derive(Default, Debug, Clone)]
struct Airshipper {
    changelog_scrollable_state: scrollable::State,
    news_scrollable_state: scrollable::State,
    play_button_state: button::State,

    changelog: String,
    news: String,
    active_profile: Profile,

    saving: bool,
    downloading: bool,
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
enum Message {
    Loaded(Result<SavedState, crate::saved_state::LoadError>),
    Saved(Result<(), crate::saved_state::SaveError>),
    DownloadDone(Profile),
    UpdateCheckDone(Profile),
    PlayPressed,
    SliderChanged(f32),
}

async fn download_or_run(mut profile: Profile) -> Profile {
    if profile.is_ready() && profile.newer_version.is_none() {
        profile.start().await;
        profile
    } else {
        profile.download().await;
        profile
    }
}

async fn check_for_update(mut profile: Profile) -> Profile {
    profile.check_for_update().await;
    profile
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
        let mut needs_save = false;

        match message {
            Message::Loaded(saved_state) => {
                if let Ok(saved) = saved_state {
                    self.update_from_save(saved);
                } else {
                    needs_save = true;
                }

                return Command::perform(
                    check_for_update(self.active_profile.clone()),
                    Message::UpdateCheckDone,
                );
            }
            Message::Saved(_) => {
                self.saving = false;
            }
            Message::PlayPressed => {
                // TODO: do this asynchronously and feed back information about to the UI.
                if !self.downloading {
                    self.downloading = true;
                    return Command::perform(
                        download_or_run(self.active_profile.clone()),
                        Message::DownloadDone,
                    );
                }
            }
            Message::UpdateCheckDone(profile) => {
                self.active_profile = profile;
                needs_save = true
            }
            Message::DownloadDone(profile) => {
                self.active_profile = profile;
                self.downloading = false;
                needs_save = true
            }
            _ => {}
        }

        if needs_save && !self.saving {
            self.saving = true;
            return Command::perform(SavedState::from(self.clone()).save(), Message::Saved);
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR").to_owned();
        let title = Container::new(Image::new(manifest_dir.clone() + "/assets/veloren-logo.png"))
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
            .style(NewsStyle);

        // Contains logo, changelog and news
        let middle = Row::new()
            .padding(2)
            .push(left)
            .push(news_container);
        let middle_container = Container::new(middle)
            .height(Length::FillPortion(6))
            .style(MiddleStyle);

        let download_speed = Text::new("8 kb / s").size(12);
        let download_progressbar = ProgressBar::new(0.0..=100.0, 20.0).style(ProgressStyle);
        let download = Column::new()
            .width(Length::FillPortion(4))
            .spacing(5)
            .push(download_speed)
            .push(download_progressbar);

        let play = Button::new(
            &mut self.play_button_state,
            Text::new(if self.downloading {
                "Loading"
            } else {
                if self.active_profile.is_ready() && self.active_profile.newer_version.is_none() {
                    "PLAY"
                } else {
                    "Download"
                }
            })
            .size(30)
            .height(Length::Fill)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center)
        )
        .on_press(Message::PlayPressed)
        .width(Length::Fill)
        .height(Length::Units(60))
        .padding(2)
        .style(PlayButtonStyle);

        let bottom = Row::new()
            .align_items(Align::End)
            .spacing(20)
            .padding(10)
            .push(download)
            .push(play);
        let bottom_container = Container::new(bottom)
            .style(BottomStyle);

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
            .style(ContentStyle)
            .into()
    }
}

struct PlayButtonStyle;
impl button::StyleSheet for PlayButtonStyle {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(Color::from_rgb(0.05, 0.44, 0.62))),
            border_color: Color::from_rgb(0.29, 0.19, 0.03),
            border_width: 4,
            shadow_offset: Vector::new(1.0, 1.0),
            text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(Color::from_rgb(0.08, 0.61, 0.65))),
            text_color: Color::WHITE,
            shadow_offset: Vector::new(1.0, 2.0),
            ..self.active()
        }
    }
}

pub struct NewsStyle;
impl container::StyleSheet for NewsStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb(
                0.09, 0.24, 0.29
            ))),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}

pub struct MiddleStyle;
impl container::StyleSheet for MiddleStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb(
                0.10, 0.21, 0.25
            ))),
            text_color: Some(Color::WHITE),
            border_width: 2,
            border_color: Color::from_rgb(
                0.14, 0.29, 0.35
            ),
            ..container::Style::default()
        }
    }
}

pub struct BottomStyle;
impl container::StyleSheet for BottomStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb(
                0.10, 0.21, 0.25
            ))),
            text_color: Some(Color::WHITE),
            border_width: 2,
            border_color: Color::from_rgb(
                0.14, 0.29, 0.35
            ),
            ..container::Style::default()
        }
    }
}

pub struct ProgressStyle;
impl progress_bar::StyleSheet for ProgressStyle {
    fn style(&self) -> progress_bar::Style {
        progress_bar::Style {
            background: Background::Color(Color::from_rgb(0.35, 0.43, 0.46)),
            bar: Background::Color(Color::from_rgb(
                0.35, 0.82, 0.76
            )),
            border_radius: 5,
        }
    }
}

pub struct ContentStyle;
impl container::StyleSheet for ContentStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb(
                0.10, 0.21, 0.25
            ))),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}
