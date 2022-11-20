use crate::{assets::POPPINS_BOLD_FONT, gui::style::AirshipperTheme};
use iced::{
    widget::{container, horizontal_rule, row, text},
    Alignment, Element, Length, Padding,
};

pub(crate) fn heading_with_rule<'a, T: 'a>(
    heading_text: &'a str,
) -> Element<T, AirshipperTheme> {
    container(
        row![]
            .align_items(Alignment::Center)
            .push(container(horizontal_rule(8)).width(Length::Units(13)))
            .push(
                container(text(heading_text).font(POPPINS_BOLD_FONT).size(20))
                    .padding(Padding::from([0, 7])),
            )
            .push(container(horizontal_rule(8)).width(Length::Fill)),
    )
    .into()
}
