/*
//
// Generic Widget Styles
//


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
        CORNFLOWER_BLUE
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

//
// Specific widget styles
//
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

pub struct SettingsButtonStyle;
impl button::StyleSheet for SettingsButtonStyle {
    fn active(&self) -> Style {
        Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border_radius: 10.0,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> Style {
        Style {
            background: Some(Background::Color(TRANSPARENT_WHITE)),
            ..self.active()
        }
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

pub struct NextPrevTextButtonStyle;
impl button::StyleSheet for NextPrevTextButtonStyle {
    fn active(&self) -> Style {
        Style {
            background: None,
            text_color: DARK_WHITE,
            ..button::Style::default()
        }
    }
}

// Used by ChangelogPanel and Update screen
pub struct DarkContainerStyle;
impl container::StyleSheet for DarkContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(VERY_DARK_GREY)),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}

// Used by AnnouncementPanel


pub struct WarningContainerStyle;
impl container::StyleSheet for WarningContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(VERY_DARK_GREY)),
            border_color: TOMATO_RED,
            border_width: 2.0,
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



pub struct AirshipperDownloadButtonStyle;
impl button::StyleSheet for AirshipperDownloadButtonStyle {
    fn active(&self) -> Style {
        Style {
            background: Some(Background::Color(VERY_DARK_GREY)),
            border_radius: 25.0,
            ..button::Style::default()
        }
    }
}

pub struct LoadingBlogPostContainerStyle;
impl container::StyleSheet for LoadingBlogPostContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: None,
            border_width: 0.7,
            border_color: DARK_WHITE,
            text_color: Some(DARK_WHITE),
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

pub enum ButtonState {
    Enabled,
    Disabled,
}
pub enum DownloadButtonStyle {
    Launch(ButtonState),
    Update(ButtonState),
    #[cfg(windows)]
    Skip,
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
            #[cfg(windows)]
            Self::Skip => active_download_button_style(TOMATO_RED),
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

pub struct ServerBrowserButtonStyle;
impl button::StyleSheet for ServerBrowserButtonStyle {
    fn active(&self) -> Style {
        Style {
            background: Some(Background::Color(CORNFLOWER_BLUE)),
            text_color: Color::WHITE,
            border_radius: 4.0,
            ..button::Style::default()
        }
    }
}

pub struct ColumnHeadingButtonStyle;
impl button::StyleSheet for ColumnHeadingButtonStyle {
    fn active(&self) -> Style {
        Style {
            text_color: Color::WHITE,
            ..button::Style::default()
        }
    }
}

pub struct ColumnHeadingContainerStyle;
impl container::StyleSheet for ColumnHeadingContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(EXTRA_MEDIUM_GREY)),
            ..container::Style::default()
        }
    }
}


*/
