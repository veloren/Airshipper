use crate::{
    assets::{BOOK_ICON, FOLDER_ICON},
    channels::{Channel, Channels},
    gui::{
        components::GamePanelMessage,
        custom_widgets::heading_with_rule,
        style::{button::ButtonStyle, container::ContainerStyle, text::TextStyle},
        views::{
            default::{DefaultViewMessage, Interaction},
            Action,
        },
        widget::*,
    },
    profiles,
    profiles::Profile,
    Result,
};
use iced::{
    alignment::Horizontal,
    widget::{
        button, column, container, image, image::Handle, pick_list, row, text,
        text_input, tooltip, tooltip::Position, Image,
    },
    Alignment, Command, Length, Padding,
};
use tracing::debug;

#[derive(Clone, Debug)]
pub enum SettingsPanelMessage {
    LogLevelChanged(profiles::LogLevel),
    ServerChanged(profiles::Server),
    ChannelChanged(Channel),
    WgpuBackendChanged(profiles::WgpuBackend),
    EnvVarsChanged(String),
    AssetsOverrideChanged(String),
    OpenLogsPressed,
    ChannelsLoaded(Result<Channels>),
}

#[derive(Clone, Debug, Default)]
pub struct SettingsPanelComponent {
    channels: Channels,
}

impl SettingsPanelComponent {
    pub fn update(
        &mut self,
        msg: SettingsPanelMessage,
        active_profile: &Profile,
    ) -> Option<Command<DefaultViewMessage>> {
        match msg {
            SettingsPanelMessage::ServerChanged(new_server) => {
                tracing::debug!("new server selected {}", new_server);
                let mut profile = active_profile.clone();
                profile.server = new_server;
                let profile2 = profile.clone();
                Some(Command::batch(vec![
                    Command::perform(
                        async { Action::UpdateProfile(profile2) },
                        DefaultViewMessage::Action,
                    ),
                    Command::perform(async {}, |_| {
                        DefaultViewMessage::GamePanel(GamePanelMessage::StartUpdate)
                    }),
                ]))
            },
            SettingsPanelMessage::ChannelChanged(new_channel) => {
                tracing::debug!("new channel selected {}", new_channel);
                let mut profile = active_profile.clone();
                profile.channel = new_channel;
                let profile2 = profile.clone();
                Some(Command::batch(vec![
                    Command::perform(
                        async { Action::UpdateProfile(profile2) },
                        DefaultViewMessage::Action,
                    ),
                    Command::perform(async {}, |_| {
                        DefaultViewMessage::GamePanel(GamePanelMessage::StartUpdate)
                    }),
                ]))
            },
            SettingsPanelMessage::WgpuBackendChanged(wgpu_backend) => {
                let mut profile = active_profile.clone();
                profile.wgpu_backend = wgpu_backend;
                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
            SettingsPanelMessage::LogLevelChanged(log_level) => {
                let mut profile = active_profile.clone();
                profile.log_level = log_level;
                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
            SettingsPanelMessage::OpenLogsPressed => {
                if let Err(e) = opener::open(active_profile.voxygen_logs_path()) {
                    tracing::error!("Failed to open logs dir: {:?}", e);
                }
                None
            },
            SettingsPanelMessage::EnvVarsChanged(vars) => {
                let mut profile = active_profile.clone();
                profile.env_vars = vars;
                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
            SettingsPanelMessage::AssetsOverrideChanged(assets) => {
                let mut profile = active_profile.clone();
                profile.assets_override = Some(assets);
                Some(Command::perform(
                    async { Action::UpdateProfile(profile) },
                    DefaultViewMessage::Action,
                ))
            },
            SettingsPanelMessage::ChannelsLoaded(result) => {
                if let Ok(channels) = result {
                    debug!(?channels, "Fetched available channels:");
                    self.channels = channels;
                }

                None
            },
        }
    }

    pub fn view<'a>(
        &self,
        active_profile: &'a Profile,
    ) -> Element<'a, DefaultViewMessage> {
        const PICK_LIST_PADDING: u16 = 7;
        const FONT_SIZE: u16 = 12;

        let graphics_mode = tooltip(
            column![]
                .spacing(5)
                .push(text("GRAPHICS MODE").size(10).style(TextStyle::LightGrey))
                .push(
                    container(
                        pick_list(
                            active_profile.supported_wgpu_backends.as_slice(),
                            Some(active_profile.wgpu_backend),
                            |x| {
                                DefaultViewMessage::SettingsPanel(
                                    SettingsPanelMessage::WgpuBackendChanged(x),
                                )
                            },
                        )
                        .text_size(FONT_SIZE)
                        .padding(PICK_LIST_PADDING)
                        .width(Length::Fill),
                    )
                    .height(Length::Fixed(30.0)),
                )
                .width(Length::FillPortion(1)),
            "The rendering backend that the game will use. \nLeave on Auto unless you \
             are experiencing issues",
            Position::Top,
        )
        .style(ContainerStyle::Tooltip)
        .gap(5);

        let log_level = tooltip(
            column![]
                .spacing(5)
                .push(
                    row![]
                        .spacing(5)
                        .push(text("LOG LEVEL").size(10).style(TextStyle::LightGrey))
                        .push(
                            container(
                                button(image(Handle::from_memory(FOLDER_ICON.to_vec())))
                                    .on_press(DefaultViewMessage::SettingsPanel(
                                        SettingsPanelMessage::OpenLogsPressed,
                                    ))
                                    .padding(Padding::new(0.0))
                                    .style(ButtonStyle::Transparent),
                            )
                            .align_x(Horizontal::Right),
                        )
                        .align_items(Alignment::Center),
                )
                .push(
                    container(
                        pick_list(
                            profiles::LOG_LEVELS,
                            Some(active_profile.log_level),
                            |x| {
                                DefaultViewMessage::SettingsPanel(
                                    SettingsPanelMessage::LogLevelChanged(x),
                                )
                            },
                        )
                        .text_size(FONT_SIZE)
                        .padding(PICK_LIST_PADDING)
                        .width(Length::Fill),
                    )
                    .height(Length::Fixed(30.0)),
                )
                .width(Length::FillPortion(1)),
            "Changes the amount of information that the game outputs to its log file",
            Position::Top,
        )
        .style(ContainerStyle::Tooltip)
        .gap(5);

        let server_picker = tooltip(
            column![]
                .spacing(5)
                .push(text("SERVER").size(10).style(TextStyle::LightGrey))
                .push(
                    container(
                        pick_list(profiles::SERVERS, Some(active_profile.server), |x| {
                            DefaultViewMessage::SettingsPanel(
                                SettingsPanelMessage::ServerChanged(x),
                            )
                        })
                        .text_size(FONT_SIZE)
                        .padding(PICK_LIST_PADDING)
                        .width(Length::Fill),
                    )
                    .height(Length::Fixed(30.0)),
                )
                .width(Length::FillPortion(1)),
            "The download server used for game downloads",
            Position::Top,
        )
        .style(ContainerStyle::Tooltip)
        .gap(5);

        let help_link =
            "https://book.veloren.net/players/env-vars.html#veloren_assets_override"
                .to_owned();
        let assets_override = tooltip(
            column![]
                .spacing(5)
                .push(
                    row![]
                        .spacing(5)
                        .push(
                            text("ASSETS OVERRIDE").size(15).style(TextStyle::LightGrey),
                        )
                        .push(help_link_button(help_link)),
                )
                .push(
                    container(
                        text_input(
                            "/path/to/asset/folder/with/overrides",
                            active_profile
                                .assets_override
                                .as_deref()
                                .unwrap_or_default(),
                        )
                        .on_input(|path| {
                            DefaultViewMessage::SettingsPanel(
                                SettingsPanelMessage::AssetsOverrideChanged(path),
                            )
                        })
                        .padding(PICK_LIST_PADDING)
                        .size(FONT_SIZE),
                    )
                    .height(Length::Fixed(50.0))
                    .width(Length::Fixed(260.0)),
                ),
            "Folder where you can put modified assets for testing or fun!",
            Position::Top,
        )
        .style(
            // TODO: this and env_vars should probably scream at you for putting
            // invalid data in
            ContainerStyle::Tooltip,
        )
        .gap(5);

        let help_link = "https://book.veloren.net/players/env-vars.html".to_owned();
        let env_vars = tooltip(
            column![]
                .spacing(5)
                .push(
                    row![]
                        .spacing(5)
                        .push(
                            text("ENVIRONMENT VARIABLES")
                                .size(10)
                                .style(TextStyle::LightGrey),
                        )
                        .push(help_link_button(help_link)),
                )
                .push(
                    container(
                        text_input("FOO=foo, BAR=bar", &active_profile.env_vars)
                            .on_input(|vars| {
                                DefaultViewMessage::SettingsPanel(
                                    SettingsPanelMessage::EnvVarsChanged(vars),
                                )
                            })
                            .padding(PICK_LIST_PADDING)
                            .size(FONT_SIZE)
                            .width(Length::Fill),
                    )
                    .height(Length::Fixed(50.0)),
                )
                .width(Length::FillPortion(2)),
            "Environment variables set when running Voxygen",
            Position::Top,
        )
        .style(ContainerStyle::Tooltip)
        .gap(5);

        let channel_picker = tooltip(
            column![]
                .spacing(5)
                .push(text("CHANNEL").size(10).style(TextStyle::LightGrey))
                .push(
                    container(
                        pick_list(
                            self.channels.names.clone(),
                            Some(active_profile.channel.clone()),
                            |x| {
                                DefaultViewMessage::SettingsPanel(
                                    SettingsPanelMessage::ChannelChanged(x),
                                )
                            },
                        )
                        .width(Length::Fill)
                        .text_size(FONT_SIZE)
                        .padding(PICK_LIST_PADDING),
                    )
                    .height(Length::Fixed(30.0)),
                )
                .width(Length::FillPortion(1)),
            "The download channel used for game downloads",
            Position::Top,
        )
        .style(ContainerStyle::Tooltip)
        .gap(5);

        let first_row = container(
            row![]
                .spacing(5)
                .align_items(Alignment::End)
                .push(graphics_mode)
                .push(log_level)
                .push(server_picker),
        );

        let second_row = container(row![].spacing(5).push(env_vars).push(channel_picker));

        let third_row = container(
            row![]
                .spacing(10)
                .align_items(Alignment::End)
                .push(assets_override),
        );

        let col = column![]
            .spacing(5)
            .push(first_row)
            .push(second_row)
            .push(third_row);

        column![]
            .push(heading_with_rule("Settings"))
            .push(
                container(col)
                    .padding(Padding::from([10, 20]))
                    .height(Length::Shrink),
            )
            .into()
    }
}

fn help_link_button(url: String) -> Element<'static, DefaultViewMessage> {
    button(
        Image::new(Handle::from_memory(BOOK_ICON.to_vec()))
            .height(Length::Fixed(10.0))
            .width(Length::Fixed(10.0)),
    )
    .on_press(DefaultViewMessage::Interaction(Interaction::OpenURL(url)))
    .padding(Padding::new(0.0))
    .style(ButtonStyle::Transparent)
    .into()
}
