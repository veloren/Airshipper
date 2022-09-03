use crate::{
    assets::{CHANGELOG_ICON, POPPINS_BOLD_FONT, POPPINS_MEDIUM_FONT},
    gui::{
        style::{
            ChangelogHeaderStyle, ColumnHeadingButtonStyle, ColumnHeadingContainerStyle,
            DarkContainerStyle, DARK_WHITE, MEDIUM_GREY,
        },
        views::default::DefaultViewMessage,
    },
};
use iced::{
    alignment::Vertical,
    pure::{button, column, container, row, text, widget::Image, Element},
    Application, Length, Padding, Text,
};
use iced_native::{image::Handle, Command};

#[derive(Debug, Clone)]
pub enum ServerBrowserPanelMessage {}

#[derive(Debug, Clone)]
pub struct ServerBrowserPanelComponent {}

impl Default for ServerBrowserPanelComponent {
    fn default() -> Self {
        Self {}
    }
}

impl ServerBrowserPanelComponent {
    pub fn view(&self) -> Element<DefaultViewMessage> {
        let top_row = row().height(Length::Units(50)).push(
            column().push(container(
                row()
                    .push(
                        container(Image::new(Handle::from_memory(
                            CHANGELOG_ICON.to_vec(),
                        )))
                        .height(Length::Fill)
                        .width(Length::Shrink)
                        .align_y(Vertical::Center)
                        .padding(Padding::from([0, 0, 0, 12])),
                    )
                    .push(
                        container(
                            Text::new("Server Browser")
                                .color(DARK_WHITE)
                                .font(POPPINS_MEDIUM_FONT),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_y(Vertical::Center)
                        .padding(Padding::from([1, 0, 0, 8])),
                    ),
            )),
        );

        let column_headings = container(
            row()
                .width(Length::Fill)
                .height(Length::Units(30))
                .push(
                    button(text("Server").font(POPPINS_BOLD_FONT))
                        .style(ColumnHeadingButtonStyle)
                        .width(Length::FillPortion(3)),
                )
                .push(
                    button(text("Location").font(POPPINS_BOLD_FONT))
                        .style(ColumnHeadingButtonStyle)
                        .width(Length::FillPortion(2)),
                )
                .push(
                    button(text("Ping").font(POPPINS_BOLD_FONT))
                        .style(ColumnHeadingButtonStyle)
                        .width(Length::FillPortion(1)),
                ),
        )
        .style(ColumnHeadingContainerStyle)
        .padding(3)
        .width(Length::Fill);

        let col = column()
            .push(
                container(top_row)
                    .width(Length::Fill)
                    .style(ChangelogHeaderStyle),
            )
            .push(column_headings);

        let server_browser_container = container(col)
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(2)
            .style(DarkContainerStyle);
        server_browser_container.into()
    }

    pub fn update(
        &mut self,
        _msg: ServerBrowserPanelMessage,
    ) -> Option<Command<DefaultViewMessage>> {
        None
    }
}
