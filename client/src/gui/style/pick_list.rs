use crate::gui::style::{AirshipperTheme, NAVY_BLUE, VERY_DARK_GREY};
use iced::{
    widget::{
        pick_list,
        pick_list::{Appearance, StyleSheet},
    },
    Background, Color,
};

#[derive(Copy, Clone, Debug)]
pub enum PickListStyle {
    Default,
}

impl Default for PickListStyle {
    fn default() -> Self {
        Self::Default
    }
}

impl pick_list::StyleSheet for AirshipperTheme {
    type Style = PickListStyle;

    // TODO: menu from old picklist style?
    fn active(&self, _: &<Self as StyleSheet>::Style) -> Appearance {
        Appearance {
            text_color: Color::WHITE,
            background: Background::Color(NAVY_BLUE),
            icon_size: 0.5,
            border_width: 0.0,
            border_radius: 3.0,
            border_color: Color::WHITE,        // TODO
            placeholder_color: VERY_DARK_GREY, // TODO
        }
    }

    fn hovered(&self, style: &<Self as StyleSheet>::Style) -> Appearance {
        self.active(style)
    }
}
