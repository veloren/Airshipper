use crate::gui::style::{AirshipperTheme, LIME_GREEN, VERY_DARK_GREY};
use iced::{
    widget::{progress_bar, progress_bar::Appearance},
    Background,
};

pub enum ProgressBarStyle {
    Default,
}

impl Default for ProgressBarStyle {
    fn default() -> Self {
        Self::Default
    }
}

impl progress_bar::StyleSheet for AirshipperTheme {
    type Style = ProgressBarStyle;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        match style {
            ProgressBarStyle::Default => default_progress_bar_style(),
        }
    }
}

fn default_progress_bar_style() -> Appearance {
    Appearance {
        background: Background::Color(VERY_DARK_GREY),
        bar: Background::Color(LIME_GREEN),
        border_radius: 3.0,
    }
}
