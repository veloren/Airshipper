use super::Action;
use crate::{
    gui::{
        components::{ChangelogPanelComponent, LogoPanelComponent, NewsPanelComponent},
        style, Result,
    },
    profiles,
    profiles::Profile,
};
use iced::{
    alignment::{Horizontal, Vertical},
    image::Handle,
    pure::{button, column, container, pick_list, row, text, tooltip, Element},
    tooltip::Position,
    Command, Image, Length,
};

use crate::gui::{
    components::{
        ChangelogPanelMessage, CommunityShowcaseComponent, GamePanelComponent,
        GamePanelMessage, NewsPanelMessage, SettingsPanelComponent, SettingsPanelMessage,
    },
    style::SidePanelStyle,
};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DefaultView {
    changelog_panel_component: ChangelogPanelComponent,
    #[serde(skip)]
    logo_panel_component: LogoPanelComponent,
    #[serde(skip)]
    community_showcase_component: CommunityShowcaseComponent,
    #[serde(skip)]
    game_panel_component: GamePanelComponent,
    news_panel_component: NewsPanelComponent,
    #[serde(skip)]
    settings_panel_component: SettingsPanelComponent,
    #[serde(skip)]
    show_settings: bool,
}

#[derive(Clone, Debug)]
pub enum DefaultViewMessage {
    // Messages
    Action(Action),
    Query,

    #[cfg(windows)]
    LauncherUpdate(Result<Option<self_update::update::Release>>),

    // User Interactions
    Interaction(Interaction),

    // Panel-specific messages
    GamePanel(GamePanelMessage),
    ChangelogPanel(ChangelogPanelMessage),
    NewsPanel(NewsPanelMessage),
    SettingsPanel(SettingsPanelMessage),
}

#[derive(Debug, Clone)]
pub enum Interaction {
    SettingsPressed,
    OpenURL(String),
    Disabled,
}

impl DefaultView {
    pub fn subscription(&self) -> iced::Subscription<DefaultViewMessage> {
        self.game_panel_component
            .subscription()
            .map(DefaultViewMessage::GamePanel)
    }

    pub fn view(&self, active_profile: &Profile) -> Element<DefaultViewMessage> {
        let Self {
            changelog_panel_component,
            news_panel_component,
            logo_panel_component,
            community_showcase_component,
            game_panel_component,
            settings_panel_component,
            ..
        } = self;

        let left_middle_contents = if self.show_settings {
            settings_panel_component.view(active_profile)
        } else {
            community_showcase_component.view()
        };

        let left = container(
            column()
                .push(container(logo_panel_component.view()).height(Length::Fill))
                .push(container(left_middle_contents).height(Length::Shrink))
                .push(container(game_panel_component.view()).height(Length::Shrink)),
        )
        .height(Length::Fill)
        .width(Length::Units(347))
        .style(SidePanelStyle);
        let middle = container(changelog_panel_component.view())
            .height(Length::Fill)
            .width(Length::Fill);
        let right = container(news_panel_component.view())
            .height(Length::Fill)
            .width(Length::Units(237))
            .style(SidePanelStyle);

        let main_row = row().push(left).push(middle).push(right);

        container(main_row)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn update(
        &mut self,
        msg: DefaultViewMessage,
        active_profile: &Profile,
    ) -> Command<DefaultViewMessage> {
        match msg {
            // Messages
            // Will be handled by main view
            DefaultViewMessage::Action(_) => {},
            DefaultViewMessage::Query => {
                return Command::batch(vec![
                    Command::perform(
                        ChangelogPanelComponent::update_changelog(
                            self.changelog_panel_component.etag.clone(),
                        ),
                        |update| {
                            DefaultViewMessage::ChangelogPanel(
                                ChangelogPanelMessage::UpdateChangelog(update),
                            )
                        },
                    ),
                    Command::perform(
                        NewsPanelComponent::update_news(
                            self.news_panel_component.etag().to_owned(),
                        ),
                        |update| {
                            DefaultViewMessage::NewsPanel(NewsPanelMessage::UpdateNews(
                                update,
                            ))
                        },
                    ),
                    Command::perform(Profile::update(active_profile.clone()), |update| {
                        DefaultViewMessage::GamePanel(GamePanelMessage::GameUpdate(
                            update,
                        ))
                    }),
                    #[cfg(windows)]
                    Command::perform(
                        async { tokio::task::block_in_place(crate::windows::query) },
                        DefaultViewMessage::LauncherUpdate,
                    ),
                ]);
            },

            DefaultViewMessage::GamePanel(msg) => {
                if let Some(command) =
                    self.game_panel_component.update(msg, active_profile)
                {
                    return command;
                }
            },
            DefaultViewMessage::ChangelogPanel(msg) => {
                if let Some(command) = self.changelog_panel_component.update(msg) {
                    return command;
                }
            },
            DefaultViewMessage::NewsPanel(msg) => {
                if let Some(command) = self.news_panel_component.update(msg) {
                    return command;
                }
            },

            DefaultViewMessage::SettingsPanel(msg) => {
                if let Some(command) =
                    self.settings_panel_component.update(msg, active_profile)
                {
                    return command;
                }
            },

            #[cfg(windows)]
            DefaultViewMessage::LauncherUpdate(update) => {
                if let Ok(Some(release)) = update {
                    return Command::perform(
                        async { Action::LauncherUpdate(release) },
                        DefaultViewMessage::Action,
                    );
                }
            },

            // User Interaction
            DefaultViewMessage::Interaction(interaction) => match interaction {
                Interaction::SettingsPressed => {
                    self.show_settings = !self.show_settings;
                },
                Interaction::Disabled => {},
                Interaction::OpenURL(url) => {
                    if let Err(e) = opener::open(url) {
                        tracing::error!(
                            "Failed to open gitlab changelog website: {:?}",
                            e
                        );
                    }
                },
            },
        }

        Command::none()
    }
}

pub fn settings_button<'a>(
    interaction: Interaction,
    style: impl iced::button::StyleSheet + 'static,
) -> Element<'a, DefaultViewMessage> {
    let btn: Element<Interaction> = button(Image::new(Handle::from_memory(
        crate::assets::SETTINGS_ICON.to_vec(),
    )))
    .on_press(interaction)
    .width(Length::Units(30))
    .height(Length::Units(30))
    .style(style)
    .padding(2)
    .into();

    let element = btn.map(DefaultViewMessage::Interaction);

    tooltip(element, "Settings", Position::Top)
        .style(style::TooltipStyle)
        .gap(5)
        .into()
}

pub fn secondary_button_with_width<'a>(
    label: impl Into<String>,
    on_click_msg: DefaultViewMessage,
    width: Length,
) -> Element<'a, DefaultViewMessage> {
    button(
        text(label)
            .size(16)
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center),
    )
    .on_press(on_click_msg)
    .width(width)
    .style(style::SecondaryButton)
    .into()
}

pub fn picklist<'a, T: Clone + Eq + std::fmt::Display + 'static>(
    selected: Option<T>,
    values: &'a [T],
    on_selected_msg: impl Fn(T) -> DefaultViewMessage + 'static,
) -> Element<'a, DefaultViewMessage> {
    let selected = Some(selected.unwrap_or_else(|| values[0].clone()));

    pick_list(values, selected, on_selected_msg)
        .style(style::ServerPickList)
        .padding(10)
        .into()
}

pub fn widget_with_label<'a>(
    label_text: &'a str,
    widget: Element<'a, DefaultViewMessage>,
) -> Element<'a, DefaultViewMessage> {
    row()
        .spacing(10)
        .push(text(label_text).horizontal_alignment(Horizontal::Right))
        .push(widget)
        .into()
}
