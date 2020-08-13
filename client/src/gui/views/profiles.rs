use super::{Action, View};
use crate::{
    gui::style,
    profiles::{Channel, Profile, Profiles},
};
use iced::{
    button, pick_list, text_input, Align, Button, Column, Command, Container, Element,
    HorizontalAlignment, Length, PickList, Row, Space, Text, TextInput,
    VerticalAlignment,
};

const TEXT_COLUMN: u16 = 3;
const INPUT_COLUMN: u16 = 16;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfilesView {
    #[serde(skip)]
    profile_view: ProfileView,
    #[serde(skip)]
    profile_select_view: ProfileSelectView,
    #[serde(skip)]
    new_profile_name: String,

    #[serde(skip)]
    back_state: button::State,
}

#[derive(Debug, Clone)]
pub enum ProfilesViewMessage {
    // Messages
    Action(Action),

    // Updates
    /// Name change of currently active profile
    ProfileNameChange(String),
    /// Channel change of currently active profile
    ProfileChannelChange(Channel),
    /// Change of currently active profile
    CurrentProfileChange(Profile),

    NewProfileNameChange(String),

    /* User Interactions */
    Back,
    AddProfile(String),
}

#[derive(Debug, Clone)]
pub enum Interaction {}

impl ProfilesView {
    pub fn view(&mut self, profiles: &Profiles) -> Element<ProfilesViewMessage> {
        let Self {
            profile_view,
            profile_select_view,
            new_profile_name,
            back_state,
            ..
        } = self;

        let left = Container::new(
            Column::new()
                .height(Length::Fill)
                .push(profile_select_view.view(&profiles, &new_profile_name)),
        )
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .style(style::Container::Lighter);

        let right = Column::new()
            .width(Length::FillPortion(4))
            .height(Length::Fill)
            .push(profile_view.view(profiles.current()));

        let middle = Container::new(Row::new().spacing(10).push(left).push(right))
            .height(Length::Fill)
            .style(style::Container::Darker);

        let bottom = Column::new()
            .width(Length::Fill)
            .padding(5)
            .align_items(Align::End)
            .push(primary_button(
                back_state,
                "Back",
                ProfilesViewMessage::Back,
                style::PrimaryButton::Enabled,
            ));

        // Contains everything
        let content = Column::new()
            .padding(2)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(middle)
            .push(bottom);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::Container::Darker)
            .into()
    }

    // TODO: Just pass in &mut profiles instead of having local copy synced!
    pub fn update(
        &mut self,
        msg: ProfilesViewMessage,
        profiles: &mut Profiles,
    ) -> Command<ProfilesViewMessage> {
        match msg {
            // Messages
            ProfilesViewMessage::Action(_) => {},

            // Updates
            ProfilesViewMessage::ProfileNameChange(change) => {
                profiles.current_mut().name = change.trim().to_string();
            },
            ProfilesViewMessage::ProfileChannelChange(change) => {
                profiles.current_mut().channel = change;
            },
            ProfilesViewMessage::CurrentProfileChange(change) => {
                profiles.set_current(change);
            },

            ProfilesViewMessage::NewProfileNameChange(change) => {
                self.new_profile_name = change;
            },

            // User Interaction
            ProfilesViewMessage::Back => {
                return Command::perform(
                    async { Action::SwitchView(View::Default) },
                    ProfilesViewMessage::Action,
                );
            },
            ProfilesViewMessage::AddProfile(name) => {
                profiles.add(name, Channel::default());
                self.new_profile_name.clear();
            },
        }
        Command::none()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ProfileView {
    name_input_state: text_input::State,
    channel_picker: pick_list::State<Channel>,
}

impl ProfileView {
    pub fn view(&mut self, profile: &Profile) -> Element<ProfilesViewMessage> {
        let Self {
            name_input_state,
            channel_picker,
        } = self;

        Column::new()
            .align_items(Align::Center)
            .padding(10)
            .spacing(10)
            .push(Text::new(format!("Adjust '{}':", profile.name)))
            .push(
                Row::new()
                    .align_items(Align::Center)
                    .spacing(25)
                    .push(
                        Text::new("Name:")
                            .horizontal_alignment(HorizontalAlignment::Right)
                            .vertical_alignment(VerticalAlignment::Center)
                            .width(Length::FillPortion(TEXT_COLUMN)),
                    )
                    .push(
                        TextInput::new(
                            name_input_state,
                            "Profile's name",
                            &profile.name,
                            ProfilesViewMessage::ProfileNameChange,
                        )
                        .width(Length::FillPortion(INPUT_COLUMN))
                        .padding(8)
                        .style(style::TextInput),
                    ),
            )
            .push(
                Row::new()
                    .align_items(Align::Center)
                    .spacing(25)
                    .push(
                        Text::new("Channel:")
                            .horizontal_alignment(HorizontalAlignment::Right)
                            .vertical_alignment(VerticalAlignment::Center)
                            .width(Length::FillPortion(TEXT_COLUMN)),
                    )
                    .push(
                        PickList::new(
                            channel_picker,
                            &Channel::ALL[..],
                            Some(profile.channel),
                            ProfilesViewMessage::ProfileChannelChange,
                        )
                        .padding(8)
                        .style(style::PickList),
                    )
                    .push(Space::with_width(Length::FillPortion(INPUT_COLUMN - 3))),
            )
            .into()
    }
}

#[derive(Default, Debug, Clone)]
pub struct ProfileSelectView {
    new_profile_input_state: text_input::State,
    new_profile_state: button::State,
    profile_picker: pick_list::State<Profile>,
}

impl ProfileSelectView {
    pub fn view(
        &mut self,
        profiles: &Profiles,
        new_profile_name: &str,
    ) -> Element<ProfilesViewMessage> {
        let Self {
            new_profile_input_state,
            new_profile_state,
            profile_picker,
        } = self;

        Column::new()
            .align_items(Align::Start)
            .padding(10)
            .spacing(10)
            .push(Text::new("Profiles"))
            .push(
                Row::new()
                    .align_items(Align::Center)
                    .spacing(25)
                    .push(
                        Text::new("Current:")
                            .horizontal_alignment(HorizontalAlignment::Right)
                            .vertical_alignment(VerticalAlignment::Center),
                    )
                    .push(
                        PickList::new(
                            profile_picker,
                            profiles.all(),
                            Some(profiles.current().to_owned()),
                            ProfilesViewMessage::CurrentProfileChange,
                        )
                        .padding(8)
                        .style(style::PickList),
                    ),
            )
            .push(Text::new("Add a new Profile:"))
            .push(
                Row::new()
                    .align_items(Align::Center)
                    .spacing(25)
                    .push(
                        TextInput::new(
                            new_profile_input_state,
                            "New Profile Name",
                            new_profile_name,
                            ProfilesViewMessage::NewProfileNameChange,
                        )
                        .padding(8)
                        .style(style::TextInput),
                    )
                    .push(
                        Button::new(new_profile_state, Text::new("Add"))
                            .style(style::SecondaryButton)
                            .on_press(ProfilesViewMessage::AddProfile(
                                new_profile_name.to_string(),
                            )), // TODO
                    ),
            )
            .into()
    }
}

pub fn primary_button(
    state: &mut button::State,
    label: impl Into<String>,
    msg: ProfilesViewMessage,
    style: impl button::StyleSheet + 'static,
) -> Element<ProfilesViewMessage> {
    Button::new(
        state,
        Text::new(label)
            .size(30)
            .height(Length::Fill)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center),
    )
    .on_press(msg)
    .width(Length::Units(120))
    .height(Length::Units(60))
    .style(style)
    .padding(2)
    .into()
}
