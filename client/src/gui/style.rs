use iced::{button, container, progress_bar, Background, Color, Vector};

pub enum PrimaryButton {
    Enabled,
    Disabled,
}
impl button::StyleSheet for PrimaryButton {
    fn active(&self) -> button::Style {
        match self {
            Self::Enabled => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.05, 0.44, 0.62))),
                border_color: Color::from_rgb(0.29, 0.19, 0.03),
                border_width: 4.0,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
                ..button::Style::default()
            },
            Self::Disabled => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.35, 0.43, 0.46))),
                border_color: Color::from_rgb(0.29, 0.19, 0.03),
                border_width: 4.0,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
                ..button::Style::default()
            },
        }
    }

    fn hovered(&self) -> button::Style {
        match self {
            Self::Enabled => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.08, 0.61, 0.65))),
                text_color: Color::WHITE,
                shadow_offset: Vector::new(1.0, 2.0),
                ..self.active()
            },
            Self::Disabled => button::Style {
                background: Some(Background::Color(Color::from_rgb8(91, 110, 117))),
                text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
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
            background: Some(Background::Color(Color::from_rgb(0.05, 0.44, 0.62))),
            text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(Color::from_rgb(0.08, 0.61, 0.65))),
            text_color: Color::WHITE,
            ..self.active()
        }
    }
}

pub struct News;
impl container::StyleSheet for News {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb(0.09, 0.24, 0.29))),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}

pub struct Middle;
impl container::StyleSheet for Middle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb(0.10, 0.21, 0.25))),
            text_color: Some(Color::WHITE),
            border_width: 2.0,
            border_color: Color::from_rgb(0.14, 0.29, 0.35),
            ..container::Style::default()
        }
    }
}

pub struct Bottom;
impl container::StyleSheet for Bottom {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb(0.10, 0.21, 0.25))),
            text_color: Some(Color::WHITE),
            border_width: 2.0,
            border_color: Color::from_rgb(0.14, 0.29, 0.35),
            ..container::Style::default()
        }
    }
}

pub struct Progress;
impl progress_bar::StyleSheet for Progress {
    fn style(&self) -> progress_bar::Style {
        progress_bar::Style {
            background: Background::Color(Color::from_rgb(0.35, 0.43, 0.46)),
            bar: Background::Color(Color::from_rgb(0.35, 0.82, 0.76)),
            border_radius: 5.0,
        }
    }
}

pub struct Content;
impl container::StyleSheet for Content {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb(0.10, 0.21, 0.25))),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}
