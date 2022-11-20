use crate::gui::style::{
    AirshipperTheme, BACKGROUND_BLUE, BLOG_POST_BACKGROUND_BLUE, BRIGHT_ORANGE,
    DARK_WHITE, TOMATO_RED, VERY_DARK_GREY,
};
use iced::{
    widget::{container, container::Appearance},
    Background, Color,
};

pub enum ContainerStyle {
    Default,
    Dark,
    Announcement,
    LoadingBlogPost,
    BlogPost,
    SidePanel,
    ColumnHeading,
    ChangelogHeader,
    Warning,
}

impl Default for ContainerStyle {
    fn default() -> Self {
        ContainerStyle::Default
    }
}

impl container::StyleSheet for AirshipperTheme {
    type Style = ContainerStyle;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        match style {
            ContainerStyle::Default => Appearance::default(),
            ContainerStyle::Announcement => dark_container_style(),
            ContainerStyle::Dark => dark_container_style(),
            ContainerStyle::LoadingBlogPost => loading_blogpost_container_style(),
            ContainerStyle::BlogPost => blogpost_container_style(),
            ContainerStyle::SidePanel => sidepanel_container_style(),
            ContainerStyle::ColumnHeading => column_heading_container_style(),
            ContainerStyle::ChangelogHeader => changelog_header_container_style(),
            ContainerStyle::Warning => warning_container_style(),
        }
    }
}

fn dark_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(VERY_DARK_GREY)),
        text_color: Some(Color::WHITE),
        ..Appearance::default()
    }
}

fn announcement_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(BRIGHT_ORANGE)),
        text_color: Some(Color::WHITE),
        ..Appearance::default()
    }
}

fn loading_blogpost_container_style() -> Appearance {
    Appearance {
        background: None,
        border_width: 0.7,
        border_color: DARK_WHITE,
        text_color: Some(DARK_WHITE),
        ..Appearance::default()
    }
}

fn blogpost_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(BLOG_POST_BACKGROUND_BLUE)),
        text_color: Some(Color::WHITE),
        ..Appearance::default()
    }
}

fn sidepanel_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(BACKGROUND_BLUE)),
        ..Appearance::default()
    }
}

fn column_heading_container_style() -> Appearance {
    Appearance {
        text_color: Some(Color::WHITE),
        ..Appearance::default()
    }
}

fn changelog_header_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(Color::BLACK)),
        text_color: Some(Color::WHITE),
        ..Appearance::default()
    }
}

fn warning_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(VERY_DARK_GREY)),
        border_color: TOMATO_RED,
        border_width: 2.0,
        text_color: Some(Color::WHITE),
        ..Appearance::default()
    }
}
