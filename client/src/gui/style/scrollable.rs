use crate::gui::style::{AirshipperTheme, ALMOST_BLACK, ALMOST_BLACK2};
use iced::{
    widget::{
        scrollable,
        scrollable::{Scrollbar, Scroller},
    },
    Background, Color,
};

pub enum ScrollableStyle {
    Default,
}

impl Default for ScrollableStyle {
    fn default() -> Self {
        ScrollableStyle::Default
    }
}

impl scrollable::StyleSheet for AirshipperTheme {
    type Style = ScrollableStyle;

    fn active(&self, _: &Self::Style) -> Scrollbar {
        Scrollbar {
            border_color: Color::TRANSPARENT,
            background: Some(Background::Color(Color::TRANSPARENT)),
            border_radius: 0.0,
            border_width: 0.0,
            scroller: Scroller {
                border_width: 0.0,
                border_radius: 5.0,
                border_color: ALMOST_BLACK,
                color: ALMOST_BLACK,
            },
        }
    }

    fn hovered(&self, _: &Self::Style) -> Scrollbar {
        Scrollbar {
            border_color: Color::TRANSPARENT,
            background: Some(Background::Color(ALMOST_BLACK2)),
            border_radius: 0.0,
            border_width: 0.0,
            scroller: Scroller {
                border_width: 0.0,
                border_radius: 5.0,
                border_color: ALMOST_BLACK,
                color: ALMOST_BLACK,
            },
        }
    }
}
