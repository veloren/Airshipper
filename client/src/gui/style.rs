use iced::{
    button,
    button::Style,
    container, pick_list, progress_bar,
    pure::widget::{rule, text_input},
    Background, Color, Vector,
};

pub enum PrimaryButton {
    Enabled,
    Disabled,
}

// Colors
const LIGHT_BLUE: Color = Color::from_rgb(0.05, 0.44, 0.62);
const BROWN: Color = Color::from_rgb(0.29, 0.19, 0.03);
pub const LIGHT_GREY: Color = Color::from_rgb(0.93, 0.93, 0.93);
pub const MEDIUM_GREY: Color = Color::from_rgb(0.7, 0.7, 0.7);
const DARK_TEAL: Color = Color::from_rgb(0.10, 0.21, 0.25);
const MEDIUM_TEAL: Color = Color::from_rgb(0.09, 0.24, 0.29);
const LIGHT_TEAL: Color = Color::from_rgb(0.14, 0.29, 0.35);
const LIGHT_SEA_GREEN: Color = Color::from_rgb(0.35, 0.82, 0.76);
const SEA_GREEN: Color = Color::from_rgb(0.18, 0.65, 0.59);
const DARKER_SEA_GREEN: Color = Color::from_rgb(0.08, 0.61, 0.65);
const SLATE: Color = Color::from_rgb(0.35, 0.43, 0.46);
const TRANSPARENT_WHITE: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.1);
pub const DARK_WHITE: Color = Color::from_rgb(0.9, 0.9, 0.9);
pub const VERY_DARK_GREY: Color = Color::from_rgb(0.1, 0.1, 0.1);
const BACKGROUND_GREEN: Color = Color::from_rgb(0.21, 0.35, 0.27);
const BACKGROUND_BLUE: Color = Color::from_rgb(0.14, 0.21, 0.41);
const LIME_GREEN: Color = Color::from_rgb(0.41, 0.64, 0.26);
const CORNFLOWER_BLUE: Color = Color::from_rgb(0.19, 0.4, 0.85);
const BLOG_POST_BACKGROUND_BLUE: Color = Color::from_rgb(0.24, 0.33, 0.58);
pub const LILAC: Color = Color::from_rgb(0.62, 0.66, 0.79);
const NAVY_BLUE: Color = Color::from_rgb(0.07, 0.09, 0.15);
const LIGHT_NAVY_BLUE: Color = Color::from_rgb(0.12, 0.14, 0.20);

impl button::StyleSheet for PrimaryButton {
    fn active(&self) -> button::Style {
        match self {
            Self::Enabled => button::Style {
                background: Some(Background::Color(LIGHT_BLUE)),
                border_color: BROWN,
                border_width: 4.0,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: LIGHT_GREY,
                ..button::Style::default()
            },
            Self::Disabled => button::Style {
                background: Some(Background::Color(SLATE)),
                border_color: BROWN,
                border_width: 4.0,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: LIGHT_GREY,
                ..button::Style::default()
            },
        }
    }

    fn hovered(&self) -> button::Style {
        match self {
            Self::Enabled => button::Style {
                background: Some(Background::Color(DARKER_SEA_GREEN)),
                text_color: Color::WHITE,
                shadow_offset: Vector::new(1.0, 2.0),
                ..self.active()
            },
            Self::Disabled => button::Style {
                background: Some(Background::Color(SLATE)),
                text_color: LIGHT_GREY,
                shadow_offset: Vector::new(1.0, 2.0),
                ..self.active()
            },
        }
    }
}

pub struct SecondaryButton;
impl button::StyleSheet for SecondaryButton {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(LIGHT_BLUE)),
            text_color: LIGHT_GREY,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(DARKER_SEA_GREEN)),
            text_color: Color::WHITE,
            ..self.active()
        }
    }
}

pub struct SettingsButton;
impl button::StyleSheet for SettingsButton {
    fn active(&self) -> Style {
        button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border_radius: 10.0,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(TRANSPARENT_WHITE)),
            ..self.active()
        }
    }
}

pub struct ServerPickList;
impl pick_list::StyleSheet for ServerPickList {
    fn menu(&self) -> pick_list::Menu {
        pick_list::Menu {
            text_color: Color::WHITE,
            background: Background::Color(NAVY_BLUE),
            selected_background: Background::Color(LIGHT_NAVY_BLUE),
            selected_text_color: Color::WHITE,
            border_width: 0.0,
            ..pick_list::Menu::default()
        }
    }

    fn active(&self) -> pick_list::Style {
        pick_list::Style {
            text_color: Color::WHITE,
            background: Background::Color(NAVY_BLUE),
            icon_size: 0.5,
            border_width: 0.0,
            border_radius: 3.0,
            ..pick_list::Style::default()
        }
    }

    fn hovered(&self) -> pick_list::Style {
        let active = self.active();

        pick_list::Style { ..active }
    }
}

pub struct News;
impl container::StyleSheet for News {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(MEDIUM_TEAL)),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}

pub struct Middle;
impl container::StyleSheet for Middle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(DARK_TEAL)),
            text_color: Some(Color::WHITE),
            border_width: 2.0,
            border_color: LIGHT_TEAL,
            ..container::Style::default()
        }
    }
}

pub struct Bottom;
impl container::StyleSheet for Bottom {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(DARK_TEAL)),
            text_color: Some(Color::WHITE),
            border_width: 2.0,
            border_color: LIGHT_TEAL,
            ..container::Style::default()
        }
    }
}

pub struct ProgressBarStyle;
impl progress_bar::StyleSheet for ProgressBarStyle {
    fn style(&self) -> progress_bar::Style {
        progress_bar::Style {
            background: Background::Color(VERY_DARK_GREY),
            bar: Background::Color(LIME_GREEN),
            border_radius: 3.0,
        }
    }
}

pub struct Content;
impl container::StyleSheet for Content {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(DARK_TEAL)),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}

pub struct TooltipStyle;
impl container::StyleSheet for TooltipStyle {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: Some(LIGHT_GREY),
            background: Some(Background::Color(NAVY_BLUE)),
            border_color: MEDIUM_GREY,
            border_width: 1.0,
            ..container::Style::default()
        }
    }
}

pub struct TextInputStyle;
impl text_input::StyleSheet for TextInputStyle {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: Background::Color(NAVY_BLUE),
            border_width: 0.0,
            border_radius: 3.0,
            ..text_input::Style::default()
        }
    }

    fn focused(&self) -> text_input::Style {
        let active = self.active();

        text_input::Style { ..active }
    }

    fn placeholder_color(&self) -> Color {
        MEDIUM_GREY
    }

    fn value_color(&self) -> Color {
        LIGHT_GREY
    }

    fn selection_color(&self) -> Color {
        LIGHT_NAVY_BLUE
    }
}

pub struct SidePanelStyle;
impl container::StyleSheet for SidePanelStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(BACKGROUND_BLUE)),
            ..container::Style::default()
        }
    }
}

pub struct TestStyle2;
impl container::StyleSheet for TestStyle2 {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(DARKER_SEA_GREEN)),
            ..container::Style::default()
        }
    }
}

pub struct TestStyle3;
impl container::StyleSheet for TestStyle3 {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(LIME_GREEN)),
            ..container::Style::default()
        }
    }
}

pub struct ChangelogHeaderStyle;
impl container::StyleSheet for ChangelogHeaderStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::BLACK)),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}
pub struct ChangelogContainerStyle;
impl container::StyleSheet for ChangelogContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(VERY_DARK_GREY)),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}

pub struct BlogPostContainerStyle;
impl container::StyleSheet for BlogPostContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(BLOG_POST_BACKGROUND_BLUE)),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}

pub struct RuleStyle;
impl rule::StyleSheet for RuleStyle {
    fn style(&self) -> rule::Style {
        rule::Style {
            width: 1,
            color: Color::WHITE,
            ..rule::Style::default()
        }
    }
}

pub struct GitlabChangelogButtonStyle;
impl button::StyleSheet for GitlabChangelogButtonStyle {
    fn active(&self) -> Style {
        Style {
            background: Some(Background::Color(LIME_GREEN)),
            border_radius: 25.0,
            ..button::Style::default()
        }
    }
}

pub struct TransparentButtonStyle;
impl button::StyleSheet for TransparentButtonStyle {
    fn active(&self) -> Style {
        Style {
            background: None,
            ..button::Style::default()
        }
    }
}

pub enum ButtonState {
    Enabled,
    Disabled,
}
pub enum DownloadButtonStyle {
    Launch(ButtonState),
    Update(ButtonState),
}

impl button::StyleSheet for DownloadButtonStyle {
    fn active(&self) -> Style {
        match self {
            Self::Launch(ButtonState::Enabled) => {
                active_download_button_style(LIME_GREEN)
            },
            Self::Update(ButtonState::Enabled) => {
                active_download_button_style(CORNFLOWER_BLUE)
            },
            Self::Launch(ButtonState::Disabled) | Self::Update(ButtonState::Disabled) => {
                disabled_download_button_style()
            },
        }
    }
}

fn active_download_button_style(background_color: Color) -> Style {
    Style {
        background: Some(Background::Color(background_color)),
        text_color: Color::WHITE,
        border_radius: 4.0,
        ..button::Style::default()
    }
}

fn disabled_download_button_style() -> Style {
    Style {
        background: Some(Background::Color(SLATE)),
        shadow_offset: Vector::new(1.0, 1.0),
        text_color: LIGHT_GREY,
        border_radius: 4.0,
        ..button::Style::default()
    }
}
