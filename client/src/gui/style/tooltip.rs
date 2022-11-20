use crate::gui::style::{LIGHT_GREY, MEDIUM_GREY, NAVY_BLUE};
use iced::{
    widget::{container, container::Appearance},
    Background,
};

pub enum TooltipStyle {
    Default,
}

impl Default for TooltipStyle {
    fn default() -> Self {
        TooltipStyle::Default
    }
}

impl container::StyleSheet for TooltipStyle {
    type Style = TooltipStyle;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        Appearance {
            text_color: Some(LIGHT_GREY),
            background: Some(Background::Color(NAVY_BLUE)),
            border_color: MEDIUM_GREY,
            border_width: 1.0,
            ..Appearance::default()
        }
    }
}
