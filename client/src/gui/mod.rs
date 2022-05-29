pub mod components;
mod style;
mod subscriptions;
mod views;

use crate::{cli::CmdLine, fs, profiles::Profile, Result};
use iced::{
    pure::{Application, Element},
    Command, Settings, Subscription,
};
use ron::ser::PrettyConfig;
use tokio::{fs::File, io::AsyncWriteExt};
#[cfg(windows)]
use views::update::{UpdateView, UpdateViewMessage};
use views::{
    default::{DefaultView, DefaultViewMessage},
    Action, View,
};

/// Starts the GUI and won't return unless an error occurs
pub fn run(cmd: CmdLine) -> Result<()> {
    Ok(Airshipper::run(settings(cmd))?)
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Airshipper {
    #[serde(skip)]
    view: View,

    pub default_view: DefaultView,
    #[cfg(windows)]
    #[serde(skip)]
    update_view: UpdateView,
    pub active_profile: Profile,

    // Airshipper update
    #[cfg(windows)]
    #[serde(skip)]
    update: Option<self_update::update::Release>,

    #[serde(skip)]
    cmd: CmdLine,
}

impl Airshipper {
    pub fn new(cmd: &CmdLine) -> Self {
        Self {
            view: View::default(),
            default_view: DefaultView::default(),
            #[cfg(windows)]
            update_view: UpdateView::default(),
            active_profile: Profile::default(),
            #[cfg(windows)]
            update: None,
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
                        tracing::debug!(
                            "Reading state failed. Falling back to default: {}",
                            e
                        );
                        Self::default()
                    },
                }
            } else {
                tracing::debug!("Falling back to default state.");
                Self::default()
            }
        })
    }

    pub async fn save(airshipper: Self) -> Result<()> {
        let data = tokio::task::block_in_place(|| {
            ron::ser::to_string_pretty(&airshipper, PrettyConfig::default())
        })?;
        let mut file = File::create(fs::savedstate_file()).await?;
        file.write_all(data.as_bytes()).await?;
        file.sync_all().await?;

        Ok(())
    }

    pub async fn save_mut(&mut self) -> Result<()> {
        let data = tokio::task::block_in_place(|| {
            ron::ser::to_string_pretty(&self, PrettyConfig::default())
        })?;
        let mut file = File::create(fs::savedstate_file()).await?;
        file.write_all(data.as_bytes()).await?;
        file.sync_all().await?;

        Ok(())
    }
}

#[allow(clippy::enum_variant_names, clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum Message {
    Loaded(Airshipper), // Todo: put in Box<>
    Saved(Result<()>),

    // Messages

    // Updates

    // Views
    DefaultViewMessage(DefaultViewMessage),
    #[cfg(windows)]
    UpdateViewMessage(UpdateViewMessage),
}

impl Application for Airshipper {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = CmdLine;

    fn new(flags: CmdLine) -> (Self, Command<Message>) {
        #[cfg(windows)]
        crate::windows::hide_non_inherited_console();

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
            #[cfg(windows)]
            View::Update => iced::Subscription::none(),
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Loaded(state) => {
                *self = state;
                return self
                    .default_view
                    .update(DefaultViewMessage::Query, &self.active_profile)
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
                        Action::UpdateProfile(profile) => {
                            self.active_profile = profile.clone();
                            return Command::perform(
                                Self::save(self.clone()),
                                Message::Saved,
                            );
                        },
                        #[cfg(windows)] // for now
                        Action::SwitchView(view) => self.view = *view,
                        #[cfg(windows)]
                        Action::LauncherUpdate(release) => {
                            self.update = Some(release.clone());
                            self.view = View::Update
                        },
                    }
                }

                return self
                    .default_view
                    .update(msg, &self.active_profile)
                    .map(Message::DefaultViewMessage);
            },
            #[cfg(windows)]
            Message::UpdateViewMessage(msg) => {
                if let UpdateViewMessage::Action(action) = &msg {
                    match action {
                        Action::Save => {
                            return Command::perform(
                                Self::save(self.clone()),
                                Message::Saved,
                            );
                        },
                        Action::UpdateProfile(profile) => {
                            self.active_profile = profile.clone();
                            return Command::perform(
                                Self::save(self.clone()),
                                Message::Saved,
                            );
                        },
                        Action::SwitchView(view) => self.view = *view,
                        Action::LauncherUpdate(_) => {},
                    }
                }

                return self
                    .update_view
                    .update(msg, &self.update)
                    .map(Message::UpdateViewMessage);
            },
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let Self {
            view, default_view, ..
        } = self;

        match view {
            View::Default => default_view
                .view(&self.active_profile)
                .map(Message::DefaultViewMessage),
            #[cfg(windows)]
            View::Update => self.update_view.view().map(Message::UpdateViewMessage),
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
                Icon::from_rgba(icon.to_rgba8().into_raw(), icon.width(), icon.height())
                    .unwrap(),
            ),
            min_size: Some((400, 250)),
            ..Default::default()
        },
        flags: cmd,
        default_font: Some(crate::assets::OPEN_SANS_BYTES),
        default_text_size: 20,
        // https://github.com/hecrj/iced/issues/537
        antialiasing: false,
        exit_on_close_request: true,
        id: Some("airshipper".to_string()),
        text_multithreading: false,
        try_opengles_first: false, // Only used with glow backend
    }
}
