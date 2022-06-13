use crate::{
    assets::COMMUNITY_SHOWCASE_DEMO,
    gui::{custom_widgets::heading_with_rule, views::default::DefaultViewMessage},
};
use iced::{
    pure::{column, container, Element},
    ContentFit, Length, Padding,
};
use iced_native::{image::Handle, widget::Image};

#[derive(Clone, Default, Debug)]
pub struct CommunityShowcaseComponent {}

impl CommunityShowcaseComponent {
    pub fn view(&self) -> Element<DefaultViewMessage> {
        column()
            .push(heading_with_rule("Community Showcase"))
            .push(
                container(
                    Image::new(Handle::from_memory(COMMUNITY_SHOWCASE_DEMO.to_vec()))
                        .content_fit(ContentFit::ScaleDown),
                )
                .padding(Padding::from([10, 20]))
                .height(Length::Units(200)),
            )
            .into()
    }
}
