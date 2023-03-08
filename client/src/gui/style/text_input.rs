use crate::gui::style::{
    AirshipperTheme, CORNFLOWER_BLUE, DARK_WHITE, LIGHT_GREY, MEDIUM_GREY, NAVY_BLUE,
};
use iced::{
    widget::{text_input, text_input::Appearance},
    Background, Color,
};

pub enum TextInputStyle {
    Default,
}

impl Default for TextInputStyle {
    fn default() -> Self {
        Self::Default
    }
}

impl text_input::StyleSheet for AirshipperTheme {
    type Style = TextInputStyle;

    fn active(&self, _: &Self::Style) -> Appearance {
        Appearance {
            background: Background::Color(NAVY_BLUE),
            border_width: 0.0,
            border_radius: 3.0,
            border_color: DARK_WHITE,
        }
    }

    fn focused(&self, style: &Self::Style) -> Appearance {
        self.active(style)
    }

    fn placeholder_color(&self, _: &Self::Style) -> Color {
        MEDIUM_GREY
    }

    fn value_color(&self, _: &Self::Style) -> Color {
        LIGHT_GREY
    }

    fn selection_color(&self, _: &Self::Style) -> Color {
        CORNFLOWER_BLUE
    }
}
