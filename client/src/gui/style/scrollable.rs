use crate::gui::style::{ALMOST_BLACK, ALMOST_BLACK2, AirshipperTheme};
use iced::{
    Background, Border, Color,
    widget::{
        container, scrollable,
        scrollable::{Appearance, Scrollbar, Scroller},
    },
};

#[derive(Default)]
pub enum ScrollableStyle {
    #[default]
    Default,
}

impl scrollable::StyleSheet for AirshipperTheme {
    type Style = ScrollableStyle;

    fn active(&self, _: &Self::Style) -> Appearance {
        Appearance {
            container: container::Appearance::default(),
            scrollbar: Scrollbar {
                background: Some(Background::Color(Color::TRANSPARENT)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                scroller: Scroller {
                    border: Border {
                        color: ALMOST_BLACK,
                        width: 0.0,
                        radius: 5.0.into(),
                    },
                    color: ALMOST_BLACK,
                },
            },
            gap: None,
        }
    }

    fn hovered(&self, _: &Self::Style, _: bool) -> Appearance {
        Appearance {
            container: container::Appearance::default(),
            scrollbar: Scrollbar {
                background: Some(Background::Color(ALMOST_BLACK2)),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                scroller: Scroller {
                    border: Border {
                        color: ALMOST_BLACK,
                        width: 0.0,
                        radius: 5.0.into(),
                    },
                    color: ALMOST_BLACK,
                },
            },
            gap: None,
        }
    }
}
