use iced::Font;

pub const VELOREN_LOGO: &[u8] = include_bytes!("../assets/images/veloren-logo.png");
pub const VELOREN_ICON: &[u8] = include_bytes!("../assets/icons/logo.ico");
pub const SETTINGS_ICON: &[u8] = include_bytes!("../assets/icons/settings.png");
pub const CHANGELOG_ICON: &[u8] = include_bytes!("../assets/icons/changelog.png");
pub const CHAT_ICON: &[u8] = include_bytes!("../assets/icons/chat.png");
pub const HEART_ICON: &[u8] = include_bytes!("../assets/icons/heart.png");
pub const BOOK_ICON: &[u8] = include_bytes!("../assets/icons/book.png");
pub const USER_ICON: &[u8] = include_bytes!("../assets/icons/user.png");
pub const DOWNLOAD_ICON: &[u8] = include_bytes!("../assets/icons/download.png");
pub const FOLDER_ICON: &[u8] = include_bytes!("../assets/icons/folder.png");
pub const UP_RIGHT_ARROW_ICON: &[u8] =
    include_bytes!("../assets/icons/up_right_arrow.png");
pub const STAR_ICON: &[u8] = include_bytes!("../assets/icons/star.png");

pub const PING1_ICON: &[u8] = include_bytes!("../assets/icons/ping1.png");
pub const PING2_ICON: &[u8] = include_bytes!("../assets/icons/ping2.png");
pub const PING3_ICON: &[u8] = include_bytes!("../assets/icons/ping3.png");
pub const PING4_ICON: &[u8] = include_bytes!("../assets/icons/ping4.png");
pub const PING_ERROR_ICON: &[u8] = include_bytes!("../assets/icons/ping_error.png");
pub const PING_NONE_ICON: &[u8] = include_bytes!("../assets/icons/ping_none.png");
pub const GLOBE_ICON: &[u8] = include_bytes!("../assets/icons/globe.png");
pub const KEY_ICON: &[u8] = include_bytes!("../assets/icons/key.png");

// Fonts
// POPPINS_FONT_BYTES is a slice not a Font as it's used as the default application font
pub const POPPINS_FONT_BYTES: &[u8] =
    include_bytes!("../assets/fonts/Poppins-Regular.ttf");

/// A font to be used for text that can be used to display user provided text such as
/// those within the server browser panel.
#[cfg(not(feature = "bundled_font"))]
pub const UNIVERSAL_FONT: Font = Font::Default;

#[cfg(feature = "bundled_font")]
pub const UNIVERSAL_FONT: Font = Font::External {
    name: "Noto Sans Unified",
    bytes: include_bytes!("../assets/fonts/GoNotoCurrent.ttf"),
};

// Poppins is the font used throughout the rest of the Airshipper client
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
