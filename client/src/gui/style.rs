use iced::{
    button, button::Style, container, pick_list, progress_bar, Background, Color, Vector,
};

pub enum PrimaryButton {
    Enabled,
    Disabled,
}

// Colors
const LIGHT_BLUE: Color = Color::from_rgb(0.05, 0.44, 0.62);
const BROWN: Color = Color::from_rgb(0.29, 0.19, 0.03);
const LIGHT_GREY: Color = Color::from_rgb(0.93, 0.93, 0.93);
const MEDIUM_GREY: Color = Color::from_rgb(0.7, 0.7, 0.7);
const DARK_TEAL: Color = Color::from_rgb(0.10, 0.21, 0.25);
const MEDIUM_TEAL: Color = Color::from_rgb(0.09, 0.24, 0.29);
const LIGHT_TEAL: Color = Color::from_rgb(0.14, 0.29, 0.35);
const LIGHT_SEA_GREEN: Color = Color::from_rgb(0.35, 0.82, 0.76);
const SEA_GREEN: Color = Color::from_rgb(0.18, 0.65, 0.59);
const DARKER_SEA_GREEN: Color = Color::from_rgb(0.08, 0.61, 0.65);
const SLATE: Color = Color::from_rgb(0.35, 0.43, 0.46);
const TRANSPARENT_WHITE: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.1);

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
            background: Background::Color(DARK_TEAL),
            selected_background: Background::Color(SEA_GREEN),
            selected_text_color: Color::WHITE,
            ..pick_list::Menu::default()
        }
    }

    fn active(&self) -> pick_list::Style {
        pick_list::Style {
            text_color: Color::WHITE,
            background: Background::Color(DARK_TEAL),
            icon_size: 0.5,
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

pub struct Progress;
impl progress_bar::StyleSheet for Progress {
    fn style(&self) -> progress_bar::Style {
        progress_bar::Style {
            background: Background::Color(SLATE),
            bar: Background::Color(LIGHT_SEA_GREEN),
            border_radius: 5.0,
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

pub struct Tooltip;

impl container::StyleSheet for Tooltip {
    fn style(&self) -> container::Style {
        container::Style {
            text_color: Some(LIGHT_GREY),
            background: Some(Background::Color(DARK_TEAL)),
            border_color: MEDIUM_GREY,
            border_width: 1.0,
            ..container::Style::default()
        }
    }
}
