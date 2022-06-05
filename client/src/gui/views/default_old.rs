// TODO: Remove this, only for reference when re-implementing settings panel
// pub fn view_old(&self, active_profile: &Profile) -> Element<DefaultViewMessage> {
//     let Self {
//         changelog,
//         news,
//         state,
//         //play_button_state,
//         //settings_button_state,
//         //download_progress,
//         ..
//     } = self;
//
//     let logo = container(
//         Image::new(Handle::from_memory(crate::assets::VELOREN_LOGO.to_vec()))
//             .width(Length::FillPortion(10)),
//     );
//
//     let icons = row()
//         .width(Length::Fill)
//         .height(Length::Units(90))
//         .align_items(Alignment::Center)
//         .spacing(10)
//         .padding(15)
//         .push(logo);
//
//     // Contains title, changelog
//     let left = column()
//         .width(Length::FillPortion(3))
//         .height(Length::Fill)
//         .padding(15)
//         .push(icons)
//         .push(changelog.view());
//
//     // Contains the news pane and optionally the settings pane at the bottom
//     let mut right = column()
//         .width(Length::FillPortion(2))
//         .height(Length::Fill)
//         .push(news.view());
//
//     if self.show_settings {
//         let server_picker = tooltip(
//             widget_with_label(
//                 "Server:",
//                 picklist(
//                     Some(active_profile.server),
//                     profiles::SERVERS,
//                     Interaction::ServerChanged,
//                 ),
//             ),
//             "The download server used for game files",
//             Position::Top,
//         )
//         .style(style::Tooltip)
//         .gap(5);
//
//         let wgpu_backend_picker = tooltip(
//             widget_with_label(
//                 "Graphics Mode:",
//                 picklist(
//                     Some(active_profile.wgpu_backend),
//                     profiles::WGPU_BACKENDS,
//                     Interaction::WgpuBackendChanged,
//                 ),
//             ),
//             "The rendering backend that the game will use. \nLeave on Auto unless \
//              you are experiencing issues",
//             Position::Top,
//         )
//         .style(style::Tooltip)
//         .gap(5);
//
//         let log_level_picker = tooltip(
//             widget_with_label(
//                 "Log Level:",
//                 picklist(
//                     Some(active_profile.log_level),
//                     profiles::LOG_LEVELS,
//                     Interaction::LogLevelChanged,
//                 ),
//             ),
//             "Changes the amount of information that the game outputs to its log
// file",             Position::Top,
//         )
//         .style(style::Tooltip)
//         .gap(5);
//
//         let open_logs_button = secondary_button_with_width(
//             "Open Logs",
//             Interaction::OpenLogsPressed,
//             Length::Fill,
//         );
//
//         let env_vars = tooltip(
//             widget_with_label(
//                 "Env vars:",
//                 text_input("FOO=foo, BAR=bar", &active_profile.env_vars, |vars| {
//
// DefaultViewMessage::Interaction(Interaction::EnvVarsChanged(vars))
// })                 .width(Length::Fill)
//                 .into(),
//             ),
//             "Environment variables set when running Voxygen",
//             Position::Top,
//         )
//         .style(style::Tooltip)
//         .gap(5);
//
//         let settings = container(
//             column()
//                 .padding(2)
//                 .align_items(Alignment::End)
//                 .push(
//                     row()
//                         .padding(5)
//                         .align_items(Alignment::Center)
//                         .push(wgpu_backend_picker)
//                         .push(server_picker),
//                 )
//                 .push(
//                     row()
//                         .padding(5)
//                         .align_items(Alignment::Center)
//                         .push(log_level_picker)
//                         .push(open_logs_button),
//                 )
//                 .push(row().padding(5).spacing(10).push(env_vars)),
//         )
//         .padding(10)
//         .width(Length::Fill)
//         .style(gui::style::News);
//
//         right = right.push(settings);
//     }
// }
