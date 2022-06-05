use crate::{assets::POPPINS_BOLD_FONT, gui::style::RuleStyle};
use iced::{
    pure::{container, horizontal_rule, row, text, Element},
    Alignment, Color, Length, Padding,
};

pub(crate) fn heading_with_rule<'a, T: 'a>(heading_text: &'a str) -> Element<T> {
    container(
        row()
            .align_items(Alignment::Center)
            .push(container(horizontal_rule(8).style(RuleStyle)).width(Length::Units(13)))
            .push(
                container(
                    text(heading_text)
                        .font(POPPINS_BOLD_FONT)
                        .color(Color::WHITE)
                        .size(20),
                )
                .padding(Padding::from([0, 7])),
            )
            .push(container(horizontal_rule(8).style(RuleStyle)).width(Length::Fill)),
    )
    .into()
}
