use crate::{
    assets::{
        GLOBE_ICON, KEY_ICON, PING1_ICON, PING2_ICON, PING3_ICON, PING4_ICON,
        PING_ERROR_ICON, POPPINS_BOLD_FONT, POPPINS_MEDIUM_FONT, STAR_ICON,
    },
    gui::{
        components::GamePanelMessage,
        style::{
            ChangelogHeaderStyle, ColumnHeadingButtonStyle, ColumnHeadingContainerStyle,
            DarkContainerStyle, RuleStyle, ServerListEntryButtonStyle, TooltipStyle,
            BRIGHT_ORANGE, DARK_WHITE,
        },
        views::default::DefaultViewMessage,
    },
    net,
    ping::PingResult,
    server_list::{Server, ServerList},
    Result,
};
use iced::{
    alignment::{Horizontal, Vertical},
    pure::{
        button, column, container, horizontal_rule, image, row, scrollable, text,
        tooltip, widget::Image, Element,
    },
    Alignment, Length, Padding, Text,
};
use iced_native::{image::Handle, widget::tooltip::Position, Command};
use std::{
    cmp::min,
    hash::{Hash, Hasher},
    sync::Arc,
};
use tracing::debug;

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
    SortServers(ServerSortOrder),
}

#[derive(Debug, Default, Clone)]
pub struct ServerBrowserPanelComponent {
    servers: Vec<ServerBrowserEntry>,
    selected_index: Option<usize>,
}

impl ServerBrowserPanelComponent {
    pub(crate) async fn fetch() -> Result<Option<Self>> {
        let server_list =
            ServerList::fetch("https://serverlist.veloren.net/v1/servers".to_owned())
                .await?;
        Ok(Some(Self {
            servers: server_list
                .servers
                .into_iter()
                .map(ServerBrowserEntry::from)
                .collect(),
            selected_index: None,
        }))
    }

    pub fn view(&self) -> Element<DefaultViewMessage> {
        let top_row = row().height(Length::Units(50)).push(
            column().push(container(
                row()
                    .push(
                        container(Image::new(Handle::from_memory(GLOBE_ICON.to_vec())))
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

        let heading_button = |button_text: &str, sort_order: Option<ServerSortOrder>| {
            let mut button = button(
                text(button_text)
                    .font(POPPINS_BOLD_FONT)
                    .vertical_alignment(Vertical::Center),
            )
            .padding(0)
            .style(ColumnHeadingButtonStyle);
            if let Some(order) = sort_order {
                button = button.on_press(DefaultViewMessage::ServerBrowserPanel(
                    ServerBrowserPanelMessage::SortServers(order),
                ))
            }
            button
        };

        let column_headings = container(
            row()
                .width(Length::Fill)
                .height(Length::Units(30))
                // Spacer heading for icons column
                .push(heading_button("", None).width(Length::Units(65)))
                .push(
                    heading_button("Server", Some(ServerSortOrder::ServerName))
                        .width(Length::FillPortion(3)),
                )
                .push(
                    heading_button("Location", Some(ServerSortOrder::Location))
                        .width(Length::FillPortion(2)),
                )
                .push(
                    heading_button("Ping (ms)", Some(ServerSortOrder::Ping))
                        .width(Length::FillPortion(1)),
                ),
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
                Some(51..=150) => image(Handle::from_memory(PING2_ICON.to_vec())),
                Some(151..=300) => image(Handle::from_memory(PING3_ICON.to_vec())),
                Some(_) => image(Handle::from_memory(PING4_ICON.to_vec())),
                _ => image(Handle::from_memory(PING_ERROR_ICON.to_vec())),
            };

            let mut status_icons = row()
                .spacing(5)
                .height(Length::Fill)
                .align_items(Alignment::Center);

            if !matches!(
                server_entry.server.auth_server.as_deref(),
                Some("https://auth.veloren.net")
            ) {
                status_icons = status_icons.push(
                    tooltip(
                        image(Handle::from_memory(KEY_ICON.to_vec()))
                            .height(Length::Units(16))
                            .width(Length::Units(16)),
                        "This server is using a custom auth server. Do not log into \
                         this server unless you trust the owner.",
                        Position::Right,
                    )
                    .gap(5)
                    .style(TooltipStyle),
                );
            }

            if server_entry.server.official {
                status_icons = status_icons.push(
                    tooltip(
                        image(Handle::from_memory(STAR_ICON.to_vec()))
                            .height(Length::Units(16))
                            .width(Length::Units(16)),
                        "This server is operated by the Veloren development team",
                        Position::Right,
                    )
                    .gap(5)
                    .style(TooltipStyle),
                );
            }

            let row = row()
                .width(Length::Fill)
                .align_items(Alignment::Center)
                .push(
                    container(status_icons)
                        .padding(Padding::from([0, 8]))
                        .width(Length::Units(45))
                        .align_x(Horizontal::Right),
                )
                .push(
                    column_cell(
                        // Iced currently doesn't support truncating text widgets to
                        // prevent multi-line overflow so for now we truncate the server
                        // name to a length which doesn't wrap when the Airshipper window
                        // is at its default size
                        &server_entry.server.name
                            [..min(server_entry.server.name.len(), 40)],
                    )
                    .height(Length::Fill)
                    .width(Length::FillPortion(3)),
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

            let row_style = if let Some(selected_index) = self.selected_index && selected_index == i {
                ServerListEntryButtonStyle::Selected
            } else {
                ServerListEntryButtonStyle::NotSelected
            };
            let select_row_button = button(container(row).padding(Padding::from([0, 8])))
                .on_press(DefaultViewMessage::ServerBrowserPanel(
                    ServerBrowserPanelMessage::SelectServerEntry(i),
                ))
                .style(row_style)
                .height(Length::Units(30))
                .padding(0);

            server_list = server_list.push(select_row_button);
        }

        let mut col = column()
            .push(
                container(top_row)
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .style(ChangelogHeaderStyle),
            )
            .push(column_headings.height(Length::Shrink))
            .push(scrollable(server_list).height(Length::Fill));

        // If there's a selected server (which there should always be, unless the server
        // list API returned no servers) show details for that server.
        let selected_server = self
            .selected_index
            .and_then(|x| self.servers.get(x).map(|y| &y.server));

        if let Some(server) = selected_server {
            col = col
                .push(
                    container(horizontal_rule(8).style(RuleStyle))
                        .width(Length::Fill)
                        .padding(Padding::from([5, 0])),
                )
                .push(
                    container(scrollable(
                        column().push(
                            row().push(
                                column()
                                    .spacing(5)
                                    .push(
                                        row().spacing(10).push(text(&server.name)).push(
                                            text(&server.address).color(BRIGHT_ORANGE),
                                        ),
                                    )
                                    .push(text("Description: "))
                                    .push(text(&server.description)),
                            ),
                        ),
                    ))
                    .height(Length::Units(128)),
                );
        }

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
                    if !self.servers.is_empty() {
                        let client_v4 = Arc::new(
                            surge_ping::Client::new(&surge_ping::Config::default())
                                .unwrap(),
                        );
                        let client_v6 = Arc::new(
                            surge_ping::Client::new(
                                &surge_ping::Config::builder()
                                    .kind(surge_ping::ICMP::V6)
                                    .build(),
                            )
                            .unwrap(),
                        );

                        Some(Command::batch(self.servers.iter().enumerate().map(
                            |(i, server)| {
                                Command::perform(
                                    net::ping::ping(
                                        (client_v4.clone(), client_v6.clone()),
                                        server.server.address.clone(),
                                        i as u16,
                                    ),
                                    |result| {
                                        DefaultViewMessage::ServerBrowserPanel(
                                            ServerBrowserPanelMessage::UpdateServerPing(
                                                result,
                                            ),
                                        )
                                    },
                                )
                            },
                        )))
                    } else {
                        None
                    }
                },
                Ok(None) => None,
                Err(e) => {
                    tracing::trace!("Failed to update server list: {}", e);
                    None
                },
            },
            ServerBrowserPanelMessage::UpdateServerPing(ping_result) => {
                debug!(?ping_result, "Received ping result for server");

                if let Some(server) = self
                    .servers
                    .iter_mut()
                    .find(|x| x.server.address == ping_result.server_address)
                {
                    server.ping = ping_result.ping
                };

                // Currently there is no way to refresh pings, so it is OK to sort the
                // list using the default sort every time a ping is
                // returned since pings are only requested on application
                // startup.
                self.sort_servers(ServerSortOrder::Default);

                None
            },
            ServerBrowserPanelMessage::SelectServerEntry(index) => {
                self.selected_index = Some(index);
                let selected_server =
                    self.servers.get(index).map(|x| x.server.address.clone());

                Some(Command::perform(async {}, move |()| {
                    DefaultViewMessage::GamePanel(
                        GamePanelMessage::ServerBrowserServerChanged(
                            selected_server.clone(),
                        ),
                    )
                }))
            },
            ServerBrowserPanelMessage::SortServers(order) => {
                self.sort_servers(order);
                None
            },
        };
    }

    fn sort_servers(&mut self, order: ServerSortOrder) {
        match order {
            ServerSortOrder::Default => self.servers.sort_unstable_by_key(|x| {
                (
                    !x.server.official,
                    x.ping.or(Some(99999)),
                    x.server.name.clone(),
                )
            }),
            ServerSortOrder::Ping => self
                .servers
                .sort_unstable_by_key(|x| x.ping.or(Some(99999))),
            ServerSortOrder::ServerName => {
                self.servers.sort_unstable_by_key(|x| x.server.name.clone())
            },
            ServerSortOrder::Location => self.servers.sort_unstable_by_key(|x| {
                x.server
                    .location
                    .as_ref()
                    .map_or("".to_owned(), |country| country.short_name.clone())
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ServerSortOrder {
    Default,
    ServerName,
    Location,
    Ping,
}
