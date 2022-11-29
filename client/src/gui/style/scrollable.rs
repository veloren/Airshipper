use crate::gui::style::{AirshipperTheme, DARK_WHITE, TOMATO_RED};
use iced::widget::{
    scrollable,
    scrollable::{Scrollbar, Scroller},
};

pub enum ScrollableStyle {
    Default,
}

impl Default for ScrollableStyle {
    fn default() -> Self {
        ScrollableStyle::Default
    }
}

// TODO: Fix styles
impl scrollable::StyleSheet for AirshipperTheme {
    type Style = ScrollableStyle;

    fn active(&self, _: &Self::Style) -> Scrollbar {
        Scrollbar {
            border_color: DARK_WHITE,
            background: None,
            border_radius: 2.0,
            border_width: 1.0,
            scroller: Scroller {
                border_width: 1.0,
                border_radius: 2.0,
                border_color: DARK_WHITE,
                color: TOMATO_RED,
            },
        }
    }

    fn hovered(&self, _: &Self::Style) -> Scrollbar {
        Scrollbar {
            border_color: DARK_WHITE,
            background: None,
            border_radius: 2.0,
            border_width: 1.0,
            scroller: Scroller {
                border_width: 1.0,
                border_radius: 2.0,
                border_color: DARK_WHITE,
                color: TOMATO_RED,
            },
        }
    }
}
