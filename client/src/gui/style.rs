use iced::{
    button, container, pick_list, progress_bar, text_input, Background, Color, Vector,
};

struct Colors;

impl Colors {
    pub const SURFACE_DARK: Color = Color::from_rgb(
        0x1A as f32 / 255.0,
        0x36 as f32 / 255.0,
        0x40 as f32 / 255.0,
    );

    pub const SURFACE: Color = Color::from_rgb(
        0x24 as f32 / 255.0,
        0x4A as f32 / 255.0,
        0x59 as f32 / 255.0,
    );

    pub const SURFACE_LIGHT: Color = Color::from_rgb(
        0x17 as f32 / 255.0,
        0x3D as f32 / 255.0,
        0x4A as f32 / 255.0,
    );

    // Brown border
    pub const BORDER: Color = Color::from_rgb(
        0x4A as f32 / 255.0,
        0x31 as f32 / 255.0,
        0x08 as f32 / 255.0,
    );

    pub const TEXT: Color = Color::from_rgb(
        0xEE as f32 / 255.0,
        0xEE as f32 / 255.0,
        0xEE as f32 / 255.0,
    );

    // Light gray
    pub const DISABLED: Color = Color::from_rgb(
        0x59 as f32 / 255.0,
        0x6E as f32 / 255.0,
        0x75 as f32 / 255.0,
    );

    // Blue
    pub const ENABLED: Color = Color::from_rgb(
        0x0D as f32 / 255.0,
        0x70 as f32 / 255.0,
        0x9E as f32 / 255.0,
    );

    // Light blue
    pub const ENABLED_HOVER: Color = Color::from_rgb(
        0x14 as f32 / 255.0,
        0x9C as f32 / 255.0,
        0xA6 as f32 / 255.0,
    );

    // Same gray tone for now.
    pub const DISABLED_HOVER: Color = Self::DISABLED;
}

pub enum PrimaryButton {
    Enabled,
    Disabled,
}
impl button::StyleSheet for PrimaryButton {
    fn active(&self) -> button::Style {
        match self {
            Self::Enabled => button::Style {
                background: Some(Background::Color(Colors::ENABLED)),
                border_color: Colors::BORDER,
                border_width: 4,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: Colors::TEXT,
                ..button::Style::default()
            },
            Self::Disabled => button::Style {
                background: Some(Background::Color(Colors::DISABLED)),
                border_color: Colors::BORDER,
                border_width: 4,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: Colors::TEXT,
                ..button::Style::default()
            },
        }
    }

    fn hovered(&self) -> button::Style {
        match self {
            Self::Enabled => button::Style {
                background: Some(Background::Color(Colors::ENABLED_HOVER)),
                text_color: Colors::TEXT,
                shadow_offset: Vector::new(1.0, 2.0),
                ..self.active()
            },
            Self::Disabled => button::Style {
                background: Some(Background::Color(Colors::DISABLED_HOVER)),
                text_color: Colors::TEXT,
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
            background: Some(Background::Color(Colors::ENABLED)),
            text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(Colors::ENABLED_HOVER)),
            text_color: Color::WHITE,
            ..self.active()
        }
    }
}

pub enum Container {
    /// Changelog, Progressbar
    Darker,
    /// General Container
    Middle,
    /// News
    Lighter,
}

impl container::StyleSheet for Container {
    fn style(&self) -> container::Style {
        match &self {
            Self::Darker => container::Style {
                background: Some(Background::Color(Colors::SURFACE_DARK)),
                text_color: Some(Colors::TEXT),
                ..container::Style::default()
            },
            Self::Middle => container::Style {
                background: Some(Background::Color(Colors::SURFACE)),
                text_color: Some(Colors::TEXT),
                ..container::Style::default()
            },
            Self::Lighter => container::Style {
                background: Some(Background::Color(Colors::SURFACE_LIGHT)),
                text_color: Some(Colors::TEXT),
                ..container::Style::default()
            },
        }
    }
}

pub struct Progress;
impl progress_bar::StyleSheet for Progress {
    fn style(&self) -> progress_bar::Style {
        progress_bar::Style {
            background: Background::Color(Colors::DISABLED),
            bar: Background::Color(Colors::ENABLED_HOVER),
            border_radius: 5,
        }
    }
}

pub struct TextInput;
impl text_input::StyleSheet for TextInput {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: Background::Color(Colors::DISABLED),
            border_color: Colors::BORDER,
            border_width: 3,
            ..text_input::Style::default()
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_color: Colors::ENABLED,
            border_width: 2,
            ..self.active()
        }
    }

    fn hovered(&self) -> text_input::Style {
        text_input::Style {
            border_color: Color {
                a: 0.9,
                ..Colors::BORDER
            },
            ..self.focused()
        }
    }

    fn placeholder_color(&self) -> Color {
        Colors::SURFACE_DARK
    }

    fn value_color(&self) -> Color {
        Colors::TEXT
    }

    fn selection_color(&self) -> Color {
        Colors::ENABLED
    }
}

pub struct PickList;
impl pick_list::StyleSheet for PickList {
    fn menu(&self) -> pick_list::Menu {
        pick_list::Menu {
            background: Background::Color(Colors::ENABLED),
            text_color: Colors::TEXT,
            selected_text_color: Colors::TEXT,
            selected_background: Background::Color(Colors::ENABLED_HOVER),
            border_color: Colors::BORDER,
            border_width: 3,
        }
    }
    fn active(&self) -> pick_list::Style {
        pick_list::Style {
            background: Background::Color(Colors::ENABLED),
            text_color: Colors::TEXT,
            border_color: Colors::BORDER,
            border_width: 3,
            ..pick_list::Style::default()
        }
    }
    fn hovered(&self) -> pick_list::Style {
        pick_list::Style {
            background: Background::Color(Colors::ENABLED_HOVER),
            ..self.active()
        }
    }
}
