
#![allow(dead_code)]
use crate::gui::AirshipperTheme;

pub type Renderer = iced::Renderer<AirshipperTheme>;

pub type Element<'a, Message> =
    iced::Element<'a, Message, iced::Renderer<AirshipperTheme>>;
pub type Container<'a, Message> =
    iced::widget::Container<'a, Message, iced::Renderer<AirshipperTheme>>;
pub type Button<'a, Message> =
    iced::widget::Button<'a, Message, iced::Renderer<AirshipperTheme>>;
pub type ProgressBar = iced::widget::ProgressBar<iced::Renderer<AirshipperTheme>>;
pub type PickList<'a, T, Message> =
    iced::widget::PickList<'a, T, Message, iced::Renderer<AirshipperTheme>>;
pub type TextInput<'a, Message> =
    iced::widget::TextInput<'a, Message, iced::Renderer<AirshipperTheme>>;
pub type Rule = iced::widget::Rule<iced::Renderer<AirshipperTheme>>;
pub type Text<'a> = iced::widget::Text<'a, iced::Renderer<AirshipperTheme>>;
pub type Tooltip<'a, Message> =
    iced::widget::Tooltip<'a, Message, iced::Renderer<AirshipperTheme>>;
