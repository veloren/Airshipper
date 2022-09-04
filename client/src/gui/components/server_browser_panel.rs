use crate::{
    assets::{CHANGELOG_ICON, POPPINS_BOLD_FONT, POPPINS_MEDIUM_FONT},
    gui::{
        components::GamePanelMessage,
        style::{
            ChangelogHeaderStyle, ColumnHeadingButtonStyle, ColumnHeadingContainerStyle,
            DarkContainerStyle, ServerListEntryButtonStyle, DARK_WHITE,
        },
        views::{default::DefaultViewMessage, update::UpdateViewMessage},
    },
    server_list::{Server, ServerList},
};
use iced::{
    alignment::Vertical,
    pure::{
        button, column, container, horizontal_rule, row, text, widget::Image, Element,
    },
    Length, Padding, Text,
};
use iced_native::{image::Handle, Command};

#[derive(Debug, Clone)]
pub enum ServerBrowserPanelMessage {
    SelectServerEntry(usize),
}

#[derive(Debug, Clone)]
pub struct ServerBrowserPanelComponent {
    server_list: ServerList,
    selected_index: usize,
}

impl Default for ServerBrowserPanelComponent {
    fn default() -> Self {
        Self {
            server_list: ron::de::from_reader::<_, ServerList>(
                &include_bytes!("../../../../server_list.ron")[..],
            )
            .unwrap(),
            selected_index: 0,
        }
    }
}

impl ServerBrowserPanelComponent {
    pub fn selected_server(&self) -> Option<&Server> {
        self.server_list.servers.get(self.selected_index)
    }

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

        let mut server_list = column();

        for (i, server) in self.server_list.servers.iter().enumerate() {
            let row = column()
                .push(
                    row()
                        .width(Length::Fill)
                        .push(text(&server.name).width(Length::FillPortion(3)))
                        .push(
                            text(
                                server
                                    .country
                                    .as_ref()
                                    .map_or("Unknown".to_string(), |country| {
                                        country.short_name.clone()
                                    }),
                            )
                            .width(Length::FillPortion(2)),
                        )
                        .push(text("100ms").width(Length::FillPortion(1)))
                        .padding(8),
                )
                .push(horizontal_rule(5));

            let select_row_button = button(row)
                .on_press(DefaultViewMessage::ServerBrowserPanel(
                    ServerBrowserPanelMessage::SelectServerEntry(i),
                ))
                .style(if self.selected_index == i {
                    ServerListEntryButtonStyle::Selected
                } else {
                    ServerListEntryButtonStyle::NotSelected
                })
                .padding(0);

            server_list = server_list.push(select_row_button);
        }

        let col = column()
            .push(
                container(top_row)
                    .width(Length::Fill)
                    .style(ChangelogHeaderStyle),
            )
            .push(column_headings)
            .push(server_list);

        let server_browser_container = container(col)
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(8)
            .style(DarkContainerStyle);
        server_browser_container.into()
    }

    pub fn update(
        &mut self,
        msg: ServerBrowserPanelMessage,
    ) -> Option<Command<DefaultViewMessage>> {
        return match msg {
            ServerBrowserPanelMessage::SelectServerEntry(index) => {
                self.selected_index = index;
                let selected_server = self
                    .server_list
                    .servers
                    .get(index)
                    .map(|x| x.address.clone())
                    .clone();

                Some(Command::perform(async {}, move |()| {
                    DefaultViewMessage::GamePanel(
                        GamePanelMessage::ServerBrowserServerChanged(
                            selected_server.clone(),
                        ),
                    )
                }))
            },
        };
    }
}
