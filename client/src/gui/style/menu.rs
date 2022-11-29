use crate::gui::style::{
    pick_list::PickListStyle, AirshipperTheme, LIGHT_NAVY_BLUE, NAVY_BLUE,
};
use iced::{overlay, overlay::menu::Appearance, Background, Color};

#[derive(Copy, Clone, Debug)]
pub enum MenuStyle {
    Default,
}

impl From<PickListStyle> for MenuStyle {
    fn from(_: PickListStyle) -> Self {
        MenuStyle::Default
    }
}

impl Default for MenuStyle {
    fn default() -> Self {
        MenuStyle::Default
    }
}

impl overlay::menu::StyleSheet for AirshipperTheme {
    type Style = MenuStyle;

    fn appearance(&self, _: &Self::Style) -> Appearance {
        Appearance {
            text_color: Color::WHITE,
            background: Background::Color(NAVY_BLUE),
            selected_background: Background::Color(LIGHT_NAVY_BLUE),
            selected_text_color: Color::WHITE,
            border_width: 0.0,
            border_radius: 0.0,
            border_color: Color::WHITE,
        }
    }
}
