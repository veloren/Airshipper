use iced::Font;

pub const VELOREN_LOGO: &[u8] = include_bytes!("../assets/veloren-logo.png");
pub const VELOREN_ICON: &[u8] = include_bytes!("../assets/logo.ico");

pub const OPEN_SANS_BYTES: &[u8] = include_bytes!("../assets/OpenSans-Regular.ttf");

/// haxcorp 4089 is designed to be used at 12px without antialiasing, with integer
/// scaling, so that the glyph's vector will rasterize and align with the physical pixels
/// for sharp font edges when displayed on digital displays.
pub const HAXRCORP_4089_FONT: Font = Font::External {
    name: "Haxrcorp 4089",
    bytes: include_bytes!("../assets/haxrcorp_4089_cyrillic_altgr_extended.ttf"),
};

// 12px is generally too small for accessibility purposes.
// ~16px is minimum on 96ppi (pixels per inch) display.
// pub const HAXRCORP_4089_FONT_SIZE_1: u16 = HAXRCORP_4089_BASE_FONT_SIZE * 1; // 12
pub const HAXRCORP_4089_FONT_SIZE_2: u16 = HAXRCORP_4089_BASE_FONT_SIZE * 2; // 24
pub const HAXRCORP_4089_FONT_SIZE_3: u16 = HAXRCORP_4089_BASE_FONT_SIZE * 3; // 36
// pub const HAXRCORP_4089_FONT_SIZE_4: u16 = HAXRCORP_4089_BASE_FONT_SIZE * 4; // 48
// pub const HAXRCORP_4089_FONT_SIZE_5: u16 = HAXRCORP_4089_BASE_FONT_SIZE * 5; // 60

const HAXRCORP_4089_BASE_FONT_SIZE: u16 = 12;
