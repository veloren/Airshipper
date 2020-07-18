mod changelog;
mod news;

use super::{style, Interaction, Message};
pub use changelog::Changelog;
use iced::{
    button, Button, Element, HorizontalAlignment, Length, Text, VerticalAlignment,
};
pub use news::News;

pub(super) fn secondary_button(
    state: &mut button::State,
    label: impl Into<String>,
    interaction: Interaction,
) -> Element<Message> {
    let btn: Element<Interaction> = Button::new(
        state,
        Text::new(label)
            .size(16)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center),
    )
    .on_press(interaction)
    .style(style::SecondaryButton)
    .into();

    btn.map(Message::Interaction)
}

pub(super) fn primary_button(
    state: &mut button::State,
    label: impl Into<String>,
    interaction: Interaction,
    style: impl button::StyleSheet + 'static,
) -> Element<Message> {
    let btn: Element<Interaction> = Button::new(
        state,
        Text::new(label)
            .size(30)
            .height(Length::Fill)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center),
    )
    .on_press(interaction)
    .width(Length::FillPortion(1))
    .height(Length::Units(60))
    .style(style)
    .padding(2)
    .into();

    btn.map(Message::Interaction)
}
