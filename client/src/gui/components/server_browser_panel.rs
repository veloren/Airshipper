use crate::{
    assets::{
        CHANGELOG_ICON, PING1_ICON, PING2_ICON, PING3_ICON, PING4_ICON, PING_ERROR_ICON,
        POPPINS_BOLD_FONT, POPPINS_MEDIUM_FONT, STAR_ICON,
    },
    gui::{
        components::GamePanelMessage,
        style::{
            ChangelogHeaderStyle, ColumnHeadingButtonStyle, ColumnHeadingContainerStyle,
            DarkContainerStyle, ServerListEntryButtonStyle, DARK_WHITE,
        },
        subscriptions,
        subscriptions::ping_servers::PingResult,
        views::default::DefaultViewMessage,
    },
    server_list::{Server, ServerList},
    Result,
};
use iced::{
    alignment::{Horizontal, Vertical},
    pure::{
        button, column, container, image, row, scrollable, text, widget::Image, Element,
    },
    Length, Padding, Text,
};
use iced_native::{image::Handle, Command};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct ServerBrowserEntry {
    server: Server,
    ping: Option<u128>,
}

impl Hash for ServerBrowserEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.server.address.hash(state);
    }
}
impl From<Server> for ServerBrowserEntry {
    fn from(server: Server) -> Self {
        Self { server, ping: None }
    }
}

#[derive(Debug, Clone)]
pub enum ServerBrowserPanelMessage {
    SelectServerEntry(usize),
    UpdateServerList(Result<Option<ServerBrowserPanelComponent>>),
    UpdateServerPing(PingResult),
}

#[derive(Debug, Clone)]
pub struct ServerBrowserPanelComponent {
    servers: Vec<ServerBrowserEntry>,
    selected_index: usize,
    waiting_for_pings: Vec<String>,
}

impl Default for ServerBrowserPanelComponent {
    fn default() -> Self {
        Self {
            servers: vec![],
            selected_index: 0,
            waiting_for_pings: vec![],
        }
    }
}

impl ServerBrowserPanelComponent {
    pub(crate) async fn fetch() -> Result<Option<Self>> {
        let server_list =
            ServerList::fetch("https://serverlist.veloren.net/v1/servers".to_owned())
                .await?;
        let waiting_for_pings = server_list
            .servers
            .iter()
            .map(|x| x.address.clone())
            .collect();
        Ok(Some(Self {
            servers: server_list
                .servers
                .into_iter()
                .map(|server| ServerBrowserEntry::from(server))
                .collect(),
            selected_index: 0,
            waiting_for_pings,
        }))
    }

    pub fn subscription(&self) -> iced::Subscription<ServerBrowserPanelMessage> {
        if !self.waiting_for_pings.is_empty() {
            subscriptions::ping_servers::ping_servers(self.waiting_for_pings.clone())
                .map(ServerBrowserPanelMessage::UpdateServerPing)
        } else {
            iced::Subscription::none()
        }
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

        let heading_button = |button_text: &str| {
            button(
                text(button_text)
                    .font(POPPINS_BOLD_FONT)
                    .vertical_alignment(Vertical::Center),
            )
            .padding(0)
            .style(ColumnHeadingButtonStyle)
        };

        let column_headings = container(
            row()
                .width(Length::Fill)
                .height(Length::Units(30))
                .push(heading_button("").width(Length::Units(30)))
                .push(heading_button("Server").width(Length::FillPortion(3)))
                .push(heading_button("Location").width(Length::FillPortion(2)))
                .push(heading_button("Ping").width(Length::FillPortion(1))),
        )
        .style(ColumnHeadingContainerStyle)
        .padding(Padding::from([0, 8]))
        .width(Length::Fill);

        let mut server_list = column();

        let column_cell = |content: &str| {
            text(content)
                .width(Length::FillPortion(3))
                .height(Length::Fill)
                .vertical_alignment(Vertical::Center)
        };

        for (i, server_entry) in self.servers.iter().enumerate() {
            let ping_icon = match server_entry.ping {
                Some(0..=50) => image(Handle::from_memory(PING1_ICON.to_vec())),
                Some(50..=150) => image(Handle::from_memory(PING2_ICON.to_vec())),
                Some(150..=300) => image(Handle::from_memory(PING3_ICON.to_vec())),
                Some(_) => image(Handle::from_memory(PING4_ICON.to_vec())),
                _ => image(Handle::from_memory(PING_ERROR_ICON.to_vec())),
            };

            let row = row()
                .width(Length::Fill)
                .push(
                    if server_entry.server.official {
                        container(
                            image(Handle::from_memory(STAR_ICON.to_vec()))
                                .height(Length::Units(16))
                                .width(Length::Units(16)),
                        )
                        .height(Length::Fill)
                        .align_y(Vertical::Center)
                        .align_x(Horizontal::Center)
                    } else {
                        container(text(""))
                    }
                    .width(Length::Units(30)),
                )
                .push(
                    column_cell(&server_entry.server.name).width(Length::FillPortion(3)),
                )
                .push(
                    column_cell(
                        &server_entry
                            .server
                            .location
                            .as_ref()
                            .map_or("Unknown".to_owned(), |country| {
                                country.short_name.clone()
                            }),
                    )
                    .width(Length::FillPortion(2)),
                )
                .push(
                    row()
                        .spacing(5)
                        .push(
                            container(ping_icon)
                                .height(Length::Fill)
                                .align_y(Vertical::Center),
                        )
                        .push(column_cell(
                            &server_entry
                                .ping
                                .map_or("Error".to_owned(), |x| format!("{}", x)),
                        ))
                        .width(Length::FillPortion(1)),
                )
                .padding(0);

            let select_row_button = button(container(row).padding(Padding::from([0, 8])))
                .on_press(DefaultViewMessage::ServerBrowserPanel(
                    ServerBrowserPanelMessage::SelectServerEntry(i),
                ))
                .style(if self.selected_index == i {
                    ServerListEntryButtonStyle::Selected
                } else {
                    ServerListEntryButtonStyle::NotSelected
                })
                .height(Length::Units(30))
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
            .push(scrollable(server_list));

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
            ServerBrowserPanelMessage::UpdateServerList(result) => match result {
                Ok(Some(server_browser)) => {
                    *self = server_browser;
                    None
                },
                Ok(None) => None,
                Err(e) => {
                    tracing::trace!("Failed to update server list: {}", e);
                    None
                },
            },
            ServerBrowserPanelMessage::UpdateServerPing(ping_result) => {
                println!("ping result: {:?}", ping_result);
                self.servers
                    .iter_mut()
                    .find(|x| x.server.address == ping_result.server_address)
                    .map(|server| server.ping = ping_result.ping);
                None
            },
            ServerBrowserPanelMessage::SelectServerEntry(index) => {
                self.selected_index = index;
                let selected_server = self
                    .servers
                    .get(index)
                    .map(|x| x.server.address.clone())
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
