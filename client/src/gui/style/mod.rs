use iced::{application, application::Appearance, Color};

pub mod button;
pub mod container;
pub mod menu;
pub mod pick_list;
pub mod progress_bar;
pub mod rule;
pub mod scrollable;
pub mod text;
pub mod text_input;

// Colors
// TODO: Rework into a palette and include in AirshipperTheme rather than a load of consts
pub const LIGHT_GREY: Color = Color::from_rgb(0.93, 0.93, 0.93);
pub const MEDIUM_GREY: Color = Color::from_rgb(0.7, 0.7, 0.7);
pub const VERY_DARK_GREY: Color = Color::from_rgb(0.1, 0.1, 0.1);
const SLATE: Color = Color::from_rgb(0.35, 0.43, 0.46);
const TRANSPARENT_WHITE: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.1);
pub const DARK_WHITE: Color = Color::from_rgb(0.9, 0.9, 0.9);
const BACKGROUND_BLUE: Color = Color::from_rgb(0.14, 0.21, 0.41);
const LIME_GREEN: Color = Color::from_rgb(0.41, 0.64, 0.26);
const CORNFLOWER_BLUE: Color = Color::from_rgb(0.19, 0.4, 0.85);
const BLOG_POST_BACKGROUND_BLUE: Color = Color::from_rgb(0.24, 0.33, 0.58);
pub const LILAC: Color = Color::from_rgb(0.62, 0.66, 0.79);
const NAVY_BLUE: Color = Color::from_rgb(0.07, 0.09, 0.15);
const LIGHT_NAVY_BLUE: Color = Color::from_rgb(0.12, 0.14, 0.20);
pub const BRIGHT_ORANGE: Color = Color::from_rgb(0.94, 0.40, 0.24);
pub const TOMATO_RED: Color = Color::from_rgb(0.91, 0.31, 0.31);
pub const ALMOST_BLACK: Color = Color::from_rgba(0.00, 0.00, 0.00, 0.7);
pub const ALMOST_BLACK2: Color = Color::from_rgba(0.05, 0.05, 0.05, 0.25);

const fn rgb8(red: u8, green: u8, blue: u8) -> Color {
    Color::from_rgb(
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
    )
}

const DISCORD_BLURPLE: Color = rgb8(88, 101, 242);
const MASTODON_PURPLE: Color = rgb8(99, 100, 255);
const REDDIT_ORANGE: Color = rgb8(255, 69, 0);
const YOUTUBE_RED: Color = rgb8(255, 0, 0);
const TWITCH_PURPLE: Color = rgb8(100, 65, 165);

#[derive(Default)]
pub struct AirshipperTheme {}

#[derive(Default)]
pub enum AirshipperThemeStyle {
    #[default]
    Default,
}

impl application::StyleSheet for AirshipperTheme {
    type Style = AirshipperThemeStyle;

    fn appearance(&self, _: &Self::Style) -> Appearance {
        Appearance {
            background_color: Color::BLACK,
            text_color: Color::WHITE,
        }
    }
}
