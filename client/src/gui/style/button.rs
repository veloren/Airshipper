#[cfg(windows)]
use crate::gui::style::TOMATO_RED;
use crate::gui::style::{
    AirshipperTheme, CORNFLOWER_BLUE, DARK_WHITE, DISCORD_BLURPLE, LIGHT_GREY,
    LIME_GREEN, MASTODON_PURPLE, NAVY_BLUE, REDDIT_ORANGE, SLATE, TRANSPARENT_WHITE,
    TWITCH_PURPLE, VERY_DARK_GREY, YOUTUBE_RED,
};
use iced::{
    widget::{button, button::Appearance},
    Background, Color, Vector,
};

#[derive(Debug, Clone, Copy)]
pub enum ButtonStyle {
    Download(DownloadButtonStyle),
    AirshipperDownload,
    ServerListEntry(ServerListEntryButtonState),
    Browser(BrowserButtonStyle),
    NextPrev,
    Transparent,
    Settings,
    ColumnHeading,
    ServerBrowser,
}

#[derive(Debug, Clone, Copy)]
pub enum DownloadButtonStyle {
    Launch(ButtonState),
    Update(ButtonState),
    #[cfg(windows)]
    Skip,
}

#[derive(Debug, Clone, Copy)]
pub enum BrowserButtonStyle {
    Gitlab,
    Discord,
    Mastodon,
    Reddit,
    Youtube,
    Twitch,
    Extra,
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonState {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Copy)]
pub enum ServerListEntryButtonState {
    Selected,
    NotSelected,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        ButtonStyle::Download(DownloadButtonStyle::Launch(ButtonState::Enabled))
    }
}

impl button::StyleSheet for AirshipperTheme {
    type Style = ButtonStyle;

    fn active(&self, style: &Self::Style) -> Appearance {
        match style {
            ButtonStyle::Download(download_button_style) => match download_button_style {
                DownloadButtonStyle::Launch(ButtonState::Enabled) => {
                    active_download_button_style(LIME_GREEN)
                },
                DownloadButtonStyle::Update(ButtonState::Enabled) => {
                    active_download_button_style(CORNFLOWER_BLUE)
                },
                DownloadButtonStyle::Launch(ButtonState::Disabled)
                | DownloadButtonStyle::Update(ButtonState::Disabled) => {
                    disabled_download_button_style()
                },
                #[cfg(windows)]
                DownloadButtonStyle::Skip => active_download_button_style(TOMATO_RED),
            },
            ButtonStyle::AirshipperDownload => airshipper_download_button_appearance(),
            ButtonStyle::ServerListEntry(ServerListEntryButtonState::Selected) => {
                server_list_entry_selected_style_active()
            },
            ButtonStyle::ServerListEntry(ServerListEntryButtonState::NotSelected) => {
                server_list_entry_not_selected_style_active()
            },
            ButtonStyle::Browser(style) => browser_button_style_active(*style),
            ButtonStyle::NextPrev => next_prev_button_style(),
            ButtonStyle::Transparent => transparent_button_style(),
            ButtonStyle::Settings => settings_button_style_active(),
            ButtonStyle::ColumnHeading => column_heading_button_style(),
            ButtonStyle::ServerBrowser => server_browser_button_style_active(),
        }
    }

    fn hovered(&self, style: &Self::Style) -> Appearance {
        match style {
            ButtonStyle::Download(download_button_style) => match download_button_style {
                DownloadButtonStyle::Launch(ButtonState::Enabled) => {
                    hovered_download_button_style(LIME_GREEN)
                },
                DownloadButtonStyle::Update(ButtonState::Enabled) => {
                    hovered_download_button_style(CORNFLOWER_BLUE)
                },
                #[cfg(windows)]
                DownloadButtonStyle::Skip => hovered_download_button_style(TOMATO_RED),
                _ => self.active(style),
            },
            ButtonStyle::ServerListEntry(ServerListEntryButtonState::Selected) => {
                server_list_entry_selected_style_hovered()
            },
            ButtonStyle::ServerListEntry(ServerListEntryButtonState::NotSelected) => {
                server_list_entry_not_selected_style_hovered()
            },
            ButtonStyle::Browser(style) => browser_button_style_hovered(*style),
            ButtonStyle::ServerBrowser => server_browser_button_style_hovered(),
            ButtonStyle::Settings => settings_button_style_hovered(),
            _ => self.active(style), // Fallback to no hover style
        }
    }
}

fn airshipper_download_button_appearance() -> Appearance {
    Appearance {
        background: Some(Background::Color(VERY_DARK_GREY)),
        border_radius: 25.0,
        ..Appearance::default()
    }
}

fn active_download_button_style(background_color: Color) -> Appearance {
    Appearance {
        background: Some(Background::Color(background_color)),
        text_color: Color::WHITE,
        border_radius: 4.0,
        ..Appearance::default()
    }
}

fn hovered_download_button_style(background_color: Color) -> Appearance {
    Appearance {
        background: Some(Background::Color(color_multiply(background_color, 1.1))),
        text_color: Color::WHITE,
        border_radius: 4.0,
        ..Appearance::default()
    }
}

fn disabled_download_button_style() -> Appearance {
    Appearance {
        background: Some(Background::Color(SLATE)),
        shadow_offset: Vector::new(1.0, 1.0),
        text_color: LIGHT_GREY,
        border_radius: 4.0,
        ..Appearance::default()
    }
}

fn server_list_entry_selected_style_active() -> Appearance {
    Appearance {
        background: Some(Background::Color(NAVY_BLUE)),
        text_color: Color::WHITE,
        ..Appearance::default()
    }
}

fn server_list_entry_selected_style_hovered() -> Appearance {
    Appearance {
        background: Some(Background::Color(NAVY_BLUE)),
        text_color: Color::WHITE,
        shadow_offset: Vector::new(0.0, 0.0),
        ..Appearance::default()
    }
}

fn server_list_entry_not_selected_style_active() -> Appearance {
    Appearance {
        background: Some(Background::Color(VERY_DARK_GREY)),
        text_color: Color::WHITE,
        ..Appearance::default()
    }
}

fn server_list_entry_not_selected_style_hovered() -> Appearance {
    Appearance {
        background: Some(Background::Color(color_multiply(VERY_DARK_GREY, 1.2))),
        text_color: Color::WHITE,
        shadow_offset: Vector::new(0.0, 0.0),
        ..Appearance::default()
    }
}

fn browser_button_style_to_color(style: BrowserButtonStyle) -> Color {
    match style {
        BrowserButtonStyle::Discord => *DISCORD_BLURPLE,
        BrowserButtonStyle::Gitlab => LIME_GREEN,
        BrowserButtonStyle::Extra => LIME_GREEN,
        BrowserButtonStyle::Youtube => *YOUTUBE_RED,
        BrowserButtonStyle::Mastodon => *MASTODON_PURPLE,
        BrowserButtonStyle::Reddit => *REDDIT_ORANGE,
        BrowserButtonStyle::Twitch => *TWITCH_PURPLE,
    }
}

fn browser_button_style_active(style: BrowserButtonStyle) -> Appearance {
    Appearance {
        background: Some(Background::Color(browser_button_style_to_color(style))),
        border_radius: 25.0,
        ..Appearance::default()
    }
}

fn browser_button_style_hovered(style: BrowserButtonStyle) -> Appearance {
    let color = color_multiply(browser_button_style_to_color(style), 1.1);
    Appearance {
        background: Some(Background::Color(color)),
        ..browser_button_style_active(style)
    }
}

fn next_prev_button_style() -> Appearance {
    Appearance {
        background: None,
        text_color: DARK_WHITE,
        ..Appearance::default()
    }
}

fn transparent_button_style() -> Appearance {
    Appearance {
        background: None,
        ..Appearance::default()
    }
}

fn settings_button_style_active() -> Appearance {
    Appearance {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border_radius: 10.0,
        ..Appearance::default()
    }
}

fn settings_button_style_hovered() -> Appearance {
    Appearance {
        background: Some(Background::Color(TRANSPARENT_WHITE)),
        border_radius: 10.0,
        ..Appearance::default()
    }
}

fn column_heading_button_style() -> Appearance {
    Appearance {
        text_color: Color::WHITE,
        ..Appearance::default()
    }
}

fn server_browser_button_style_active() -> Appearance {
    Appearance {
        background: Some(Background::Color(CORNFLOWER_BLUE)),
        text_color: Color::WHITE,
        border_radius: 4.0,
        ..Appearance::default()
    }
}

fn server_browser_button_style_hovered() -> Appearance {
    Appearance {
        background: Some(Background::Color(color_multiply(CORNFLOWER_BLUE, 1.1))),
        ..server_browser_button_style_active()
    }
}

fn color_multiply(color: Color, multiplier: f32) -> Color {
    Color::new(
        (color.r * multiplier).clamp(0.0, 1.0),
        (color.g * multiplier).clamp(0.0, 1.0),
        (color.b * multiplier).clamp(0.0, 1.0),
        color.a,
    )
}
