#![allow(dead_code)]
use crate::gui::AirshipperTheme;

pub type Element<'a, Message> = iced::Element<'a, Message, AirshipperTheme>;
pub type Container<'a, Message> = iced::widget::Container<'a, Message, AirshipperTheme>;
pub type Button<'a, Message> = iced::widget::Button<'a, Message, AirshipperTheme>;
pub type ProgressBar = iced::widget::ProgressBar<AirshipperTheme>;
pub type PickList<'a, T, L, V, Message> =
    iced::widget::PickList<'a, T, L, V, Message, AirshipperTheme>;
pub type TextInput<'a, Message> = iced::widget::TextInput<'a, Message, AirshipperTheme>;
pub type Rule = iced::widget::Rule<AirshipperTheme>;
pub type Text<'a> = iced::widget::Text<'a, AirshipperTheme>;
pub type Tooltip<'a, Message> = iced::widget::Tooltip<'a, Message, AirshipperTheme>;
