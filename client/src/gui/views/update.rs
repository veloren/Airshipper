//! Display an update dialog (windows only) to ask whether to update airshipper

use super::{Action, View};
use crate::gui::style;
use iced::{
    button, Align, Button, Column, Command, Container, Element, HorizontalAlignment,
    Length, Row, Text, VerticalAlignment,
};
use self_update::update::Release;

#[derive(Debug, Clone)]
pub struct UpdateView {
    message: String,
    update_button_state: button::State,
    skip_button_state: button::State,
}

impl Default for UpdateView {
    fn default() -> Self {
        Self {
            message: "Update for Airshipper available. Do you want to update?"
                .to_string(),
            update_button_state: Default::default(),
            skip_button_state: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UpdateViewMessage {
    // Messages
    Action(Action),

    // Updates
    LauncherUpdateFailed(String),

    // User Interactions
    UpdatePressed,
    SkipPressed,
}

impl UpdateView {
    pub fn view(&mut self) -> Element<UpdateViewMessage> {
        let Self {
            update_button_state,
            skip_button_state,
            ..
        } = self;

        // Contains everything
        let content = Column::new()
            .align_items(Align::Center)
            .spacing(10)
            .push(Text::new(&self.message))
            .push(
                Row::new()
                    .align_items(Align::Center)
                    .spacing(100)
                    .padding(10)
                    .push(
                        Button::new(
                            skip_button_state,
                            Text::new("Skip")
                                .size(30)
                                .height(Length::Fill)
                                .horizontal_alignment(HorizontalAlignment::Center)
                                .vertical_alignment(VerticalAlignment::Center),
                        )
                        .on_press(UpdateViewMessage::SkipPressed)
                        .style(style::SecondaryButton)
                        .padding(7),
                    )
                    .push(
                        Button::new(
                            update_button_state,
                            Text::new("Update")
                                .size(30)
                                .height(Length::Fill)
                                .width(Length::Units(90))
                                .horizontal_alignment(HorizontalAlignment::Center)
                                .vertical_alignment(VerticalAlignment::Center),
                        )
                        .on_press(UpdateViewMessage::UpdatePressed)
                        .style(style::PrimaryButton::Enabled)
                        .padding(7),
                    ),
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::Content)
            .center_x()
            .center_y()
            .into()
    }

    pub fn update(
        &mut self,
        msg: UpdateViewMessage,
        release: &Option<Release>,
    ) -> Command<UpdateViewMessage> {
        match msg {
            // Will be handled by main view
            UpdateViewMessage::Action(_) => {},

            UpdateViewMessage::UpdatePressed => {
                log::info!("Updating Airshipper...");
                self.message = "Updating Airshipper...".to_string();
                let release = release.as_ref().unwrap().clone();
                return Command::perform(
                    async {
                        tokio::task::block_in_place(move || {
                            if let Err(e) = crate::windows::update(&release) {
                                log::error!("Failed to update Airshipper: {}", e);
                                return e.to_string();
                            }
                            String::new()
                        })
                    },
                    UpdateViewMessage::LauncherUpdateFailed, /* Update won't return
                                                              * except if update
                                                              * failed */
                );
            },

            UpdateViewMessage::LauncherUpdateFailed(reason) => {
                self.message = format!("Error: {}", reason);
            },

            UpdateViewMessage::SkipPressed => {
                return Command::perform(
                    async { Action::SwitchView(View::Default) },
                    UpdateViewMessage::Action,
                );
            },
        }

        Command::none()
    }
}
