use iced::Font;

pub const VELOREN_LOGO: &[u8] = include_bytes!("../assets/images/veloren-logo.png");
pub const VELOREN_ICON: &[u8] = include_bytes!("../assets/icons/logo.ico");
pub const SETTINGS_ICON: &[u8] = include_bytes!("../assets/icons/settings.png");
pub const CHANGELOG_ICON: &[u8] = include_bytes!("../assets/icons/changelog.png");
pub const UP_RIGHT_ARROW_ICON: &[u8] =
    include_bytes!("../assets/icons/up_right_arrow.png");
pub const OPEN_SANS_BYTES: &[u8] = include_bytes!("../assets/fonts/OpenSans-Regular.ttf");

/// haxcorp 4089 is designed to be used at 12px without antialiasing, with integer
/// scaling, so that the glyph's vector will rasterize and align with the physical pixels
/// for sharp font edges when displayed on digital displays.
pub const HAXRCORP_4089_FONT: Font = Font::External {
    name: "Haxrcorp 4089",
    bytes: include_bytes!("../assets/fonts/haxrcorp_4089_cyrillic_altgr_extended.ttf"),
};

// TODO: Remove unused variants and associated assets
pub const POPPINS_FONT: Font = Font::External {
    name: "Poppins",
    bytes: include_bytes!("../assets/fonts/Poppins-Regular.ttf"),
};

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

pub const POPPINS_EXTRA_BOLD_FONT: Font = Font::External {
    name: "Poppins Extra Bold",
    bytes: include_bytes!("../assets/fonts/Poppins-ExtraBold.ttf"),
};

pub const POPPINS_BLACK_FONT: Font = Font::External {
    name: "Poppins Black",
    bytes: include_bytes!("../assets/fonts/Poppins-Black.ttf"),
};

// 12px is generally too small for accessibility purposes.
// ~16px is minimum on 96ppi (pixels per inch) display.
// pub const HAXRCORP_4089_FONT_SIZE_1: u16 = HAXRCORP_4089_BASE_FONT_SIZE * 1; // 12
pub const HAXRCORP_4089_FONT_SIZE_2: u16 = HAXRCORP_4089_BASE_FONT_SIZE * 2; // 24
pub const HAXRCORP_4089_FONT_SIZE_3: u16 = HAXRCORP_4089_BASE_FONT_SIZE * 3; // 36
// pub const HAXRCORP_4089_FONT_SIZE_4: u16 = HAXRCORP_4089_BASE_FONT_SIZE * 4; // 48
// pub const HAXRCORP_4089_FONT_SIZE_5: u16 = HAXRCORP_4089_BASE_FONT_SIZE * 5; // 60

const HAXRCORP_4089_BASE_FONT_SIZE: u16 = 12;
