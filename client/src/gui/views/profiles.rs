use crate::gui::style;
use iced::{Command, Container, Element, Length, Text};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfilesView {}

#[derive(Debug)]
pub enum ProfilesViewMessage {
    // Messages

    // Updates

    // User Interactions
    Interaction(Interaction),
}

#[derive(Debug, Clone)]
pub enum Interaction {
    Disabled,
}

impl ProfilesView {
    pub fn view(&mut self) -> Element<ProfilesViewMessage> {
        let Self { .. } = self;

        Container::new(Text::new("Here comes the profile section!"))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(style::Content)
            .into()
    }

    pub fn update(&mut self, msg: ProfilesViewMessage) -> Command<ProfilesViewMessage> {
        match msg {
            // Messages

            // Updates

            // User Interaction
            ProfilesViewMessage::Interaction(_) => {},
        }

        Command::none()
    }
}
