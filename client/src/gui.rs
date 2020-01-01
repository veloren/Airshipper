use {
    crate::{profiles::Profile, saved_state::SavedState},
    iced::{
        button, scrollable, slider, Align, Application, Button, Color, Column, Command, Element,
        HorizontalAlignment, Image, Length, Row, Scrollable, Settings, Slider, Space, Svg, Text,
        VerticalAlignment,
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
    download_slider_state: slider::State,
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
        let title = Image::new(manifest_dir.clone() + "/assets/veloren-logo.png")
            .width(Length::FillPortion(10));
        let discord = Svg::new(manifest_dir.clone() + "/assets/discord.svg").width(Length::Fill);
        let gitlab = Svg::new(manifest_dir.clone() + "/assets/gitlab.svg").width(Length::Fill);
        let youtube = Svg::new(manifest_dir.clone() + "/assets/youtube.svg").width(Length::Fill);
        let reddit = Svg::new(manifest_dir.clone() + "/assets/reddit.svg").width(Length::Fill);
        let twitter = Svg::new(manifest_dir.clone() + "/assets/twitter.svg").width(Length::Fill);

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

        let changelog_text = Text::new(&self.changelog).size(18);
        let changelog = Scrollable::new(&mut self.changelog_scrollable_state)
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

        let news_test = Text::new(&self.news).size(16);
        let news = Scrollable::new(&mut self.news_scrollable_state)
            .width(Length::Fill)
            .spacing(20)
            .push(news_test);

        // Contains logo, changelog and news
        let middle = Row::new()
            .height(Length::FillPortion(6))
            .padding(25)
            .spacing(60)
            .push(left)
            .push(news);

        let download_speed = Text::new("8 kb / s").size(12);
        let download_slider = Slider::new(
            &mut self.download_slider_state,
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
            .size(40)
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
