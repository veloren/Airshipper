use crate::gui::style::{
    pick_list::PickListStyle, AirshipperTheme, LIGHT_NAVY_BLUE, NAVY_BLUE,
};
use iced::{overlay, overlay::menu::Appearance, Background, Border, Color};

#[derive(Copy, Clone, Debug, Default)]
pub enum MenuStyle {
    #[default]
    Default,
}

impl From<PickListStyle> for MenuStyle {
    fn from(_: PickListStyle) -> Self {
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
            border: Border {
                color: Color::WHITE,
                width: 0.0,
                radius: 0.0.into(),
            },
        }
    }
}
