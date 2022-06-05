use iced::Font;

pub const COMMUNITY_SHOWCASE_DEMO: &[u8] =
    include_bytes!("../assets/images/community_showcase_demo.png");

pub const VELOREN_LOGO: &[u8] = include_bytes!("../assets/images/veloren-logo.png");
pub const VELOREN_ICON: &[u8] = include_bytes!("../assets/icons/logo.ico");
pub const SETTINGS_ICON: &[u8] = include_bytes!("../assets/icons/settings.png");
pub const CHANGELOG_ICON: &[u8] = include_bytes!("../assets/icons/changelog.png");
pub const CHAT_ICON: &[u8] = include_bytes!("../assets/icons/chat.png");
pub const HEART_ICON: &[u8] = include_bytes!("../assets/icons/heart.png");
pub const BOOK_ICON: &[u8] = include_bytes!("../assets/icons/book.png");
pub const USER_ICON: &[u8] = include_bytes!("../assets/icons/user.png");
pub const DOWNLOAD_ICON: &[u8] = include_bytes!("../assets/icons/download.png");
pub const UP_RIGHT_ARROW_ICON: &[u8] =
    include_bytes!("../assets/icons/up_right_arrow.png");

// Fonts
// POPPINS_FONT_BYTES is a slice not a Font as it's used as the default application font
pub const POPPINS_FONT_BYTES: &[u8] =
    include_bytes!("../assets/fonts/Poppins-Regular.ttf");

pub const POPPINS_BOLD_FONT: Font = Font::External {
    name: "Poppins Bold",
    bytes: include_bytes!("../assets/fonts/Poppins-Bold.ttf"),
};
pub const POPPINS_LIGHT_FONT: Font = Font::External {
    name: "Poppins Light",
    bytes: include_bytes!("../assets/fonts/Poppins-Light.ttf"),
};
pub const POPPINS_MEDIUM_FONT: Font = Font::External {
    name: "Poppins Medium",
    bytes: include_bytes!("../assets/fonts/Poppins-Medium.ttf"),
};
