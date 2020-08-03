pub mod components;
mod style;
mod subscriptions;
mod views;

use crate::{cli::CmdLine, fs, profiles::Profile, Result};
use iced::{Application, Command, Element, Settings, Subscription};
use ron::ser::PrettyConfig;
use tokio::fs::File;
use views::{
    default::{DefaultView, DefaultViewMessage},
    Action, View,
};

/// Starts the GUI and won't return
pub fn run(cmd: CmdLine) {
    Airshipper::run(settings(cmd))
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Airshipper {
    #[serde(skip)]
    view: View,

    default_view: DefaultView,
    pub active_profile: Profile,

    #[serde(skip)]
    cmd: CmdLine,
}

impl Airshipper {
    pub fn new(cmd: &CmdLine) -> Self {
        Self {
            view: View::default(),
            default_view: DefaultView::default(),
            active_profile: Profile::default(),
            cmd: cmd.clone(),
        }
    }

    pub async fn load(flags: CmdLine) -> Self {
        tokio::task::block_in_place(|| {
            if let Ok(file) = std::fs::File::open(fs::savedstate_file()) {
                match ron::de::from_reader(file) {
                    Ok(state) => {
                        // Rust type inference magic
                        let mut state: Airshipper = state;
                        state.cmd = flags;
                        state
                    },
                    Err(e) => {
                        log::info!(
                            "Reading state failed. Falling back to default: {}",
                            e
                        );
                        Self::default()
                    },
                }
            } else {
                log::debug!("Falling back to default state.");
                Self::default()
            }
        })
    }

    pub async fn save(airshipper: Self) -> Result<()> {
        use tokio::prelude::*;

        let data = tokio::task::block_in_place(|| {
            ron::ser::to_string_pretty(&airshipper, PrettyConfig::default())
        })?;
        let mut file = File::create(fs::savedstate_file()).await?;
        file.write_all(&data.as_bytes()).await?;
        file.sync_all().await?;

        Ok(())
    }

    pub async fn save_mut(&mut self) -> Result<()> {
        use tokio::prelude::*;

        let data = tokio::task::block_in_place(|| {
            ron::ser::to_string_pretty(&self, PrettyConfig::default())
        })?;
        let mut file = File::create(fs::savedstate_file()).await?;
        file.write_all(&data.as_bytes()).await?;
        file.sync_all().await?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Message {
    Loaded(Airshipper),
    Saved(Result<()>),

    // Messages

    // Updates

    // Views
    DefaultViewMessage(DefaultViewMessage),
}

impl Application for Airshipper {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = CmdLine;

    fn new(flags: CmdLine) -> (Self, Command<Message>) {
        (
            Airshipper::new(&flags),
            Command::perform(Self::load(flags.clone()), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        format!("Airshipper v{}", env!("CARGO_PKG_VERSION"))
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.view {
            View::Default => self
                .default_view
                .subscription()
                .map(Message::DefaultViewMessage),
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Loaded(state) => {
                *self = state;
                return self
                    .default_view
                    .update(DefaultViewMessage::Query, &self.cmd, &self.active_profile)
                    .map(Message::DefaultViewMessage);
            },
            Message::Saved(_) => {},

            // Messages

            // Updates

            // Views
            Message::DefaultViewMessage(msg) => {
                if let DefaultViewMessage::Action(action) = &msg {
                    match action {
                        Action::Save => {
                            return Command::perform(
                                Self::save(self.clone()),
                                Message::Saved,
                            );
                        },
                        Action::SwitchView(view) => self.view = view.clone(),
                        Action::UpdateProfile(profile) => {
                            self.active_profile = profile.clone();
                            return Command::perform(
                                Self::save(self.clone()),
                                Message::Saved,
                            );
                        },
                    }
                }

                return self
                    .default_view
                    .update(msg, &self.cmd, &self.active_profile)
                    .map(Message::DefaultViewMessage);
            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let Self {
            view, default_view, ..
        } = self;

        match view {
            View::Default => default_view.view().map(Message::DefaultViewMessage),
        }
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
