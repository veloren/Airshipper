use crate::gui::style::{
    AirshipperTheme, BACKGROUND_BLUE, BLOG_POST_BACKGROUND_BLUE, BRIGHT_ORANGE,
    DARK_WHITE, LIGHT_GREY, LIME_GREEN, MEDIUM_GREY, NAVY_BLUE, VERY_DARK_GREY,
};
use iced::{
    widget::{container, container::Appearance},
    Background, Border, Color,
};

#[derive(Default)]
pub enum ContainerStyle {
    #[default]
    Default,
    Dark,
    Announcement,
    LoadingBlogPost,
    BlogPost,
    SidePanel,
    ColumnHeading,
    ChangelogHeader,
    Tooltip,
    ExtraBrowser,
}

impl container::StyleSheet for AirshipperTheme {
    type Style = ContainerStyle;

    fn appearance(&self, style: &Self::Style) -> Appearance {
        match style {
            ContainerStyle::Default => Appearance::default(),
            ContainerStyle::Announcement => announcement_container_style(),
            ContainerStyle::Dark => dark_container_style(),
            ContainerStyle::LoadingBlogPost => loading_blogpost_container_style(),
            ContainerStyle::BlogPost => blogpost_container_style(),
            ContainerStyle::SidePanel => sidepanel_container_style(),
            ContainerStyle::ColumnHeading => column_heading_container_style(),
            ContainerStyle::ChangelogHeader => changelog_header_container_style(),
            ContainerStyle::Tooltip => tooltip_container_style(),
            ContainerStyle::ExtraBrowser => extra_browser_container_style(),
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
        border: Border {
            color: DARK_WHITE,
            width: 0.7,
            ..Default::default()
        },
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

fn extra_browser_container_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(LIME_GREEN)),
        border: Border::with_radius(25.0),
        ..Appearance::default()
    }
}

fn tooltip_container_style() -> Appearance {
    Appearance {
        text_color: Some(LIGHT_GREY),
        background: Some(Background::Color(NAVY_BLUE)),
        border: Border {
            color: MEDIUM_GREY,
            width: 1.0,
            ..Default::default()
        },
        ..Appearance::default()
    }
}
