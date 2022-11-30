use crate::{
    assets::{
        CHANGELOG_ICON, POPPINS_BOLD_FONT, POPPINS_LIGHT_FONT, POPPINS_MEDIUM_FONT,
        UP_RIGHT_ARROW_ICON,
    },
    channels::Channel,
    consts,
    consts::GITLAB_MERGED_MR_URL,
    gui::{
        style::{
            ChangelogHeaderStyle, DarkContainerStyle, GitlabChangelogButtonStyle,
            RuleStyle, DARK_WHITE,
        },
        views::{
            default::{DefaultViewMessage, Interaction},
            Action,
        },
    },
    net, Result,
};
use iced::{
    alignment::Vertical,
    pure::{button, column, container, image, row, scrollable, text, Element},
    Alignment, Color, Image, Length, Padding, Rule,
};
use iced_native::{image::Handle, widget::Text, Command};
use pulldown_cmark::{Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Clone, Debug)]
pub enum ChangelogPanelMessage {
    ScrollPositionChanged(f32),
    UpdateChangelog(Result<Option<ChangelogPanelComponent>>),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogPanelComponent {
    // TODO: Separate the Changelog data from the Panel data to avoid replacing the whole
    // panel when the changelog is updated
    pub versions: Vec<ChangelogVersion>,
    pub etag: String,
    #[serde(skip, default = "default_display_count")]
    pub display_count: usize,
}

pub fn default_display_count() -> usize {
    2
}

impl ChangelogPanelComponent {
    #[allow(clippy::while_let_on_iterator)]
    async fn fetch(channel: Channel) -> Result<Self> {
        let mut versions: Vec<ChangelogVersion> = Vec::new();

        let changelog =
            net::query(consts::CHANGELOG_URL.replace("{tag}", &channel.0)).await?;
        let etag = net::get_etag(&changelog);

        let changelog_text = changelog.text().await?;
        let options = Options::empty();
        let mut parser = Parser::new_ext(changelog_text.as_str(), options).peekable();

        while let Some(event) = parser.next() {
            // h2 version header
            // starts a new version
            if let Event::Start(Tag::Heading(2)) = event {
                let mut version: String = String::new();
                let mut date: Option<String> = None;

                // h2 version header text
                while let Some(event) = parser.next() {
                    match event {
                        Event::End(Tag::Heading(2)) => break,
                        Event::Text(text) => {
                            if text.contains(" - ") {
                                date = Some(text[3..].trim().to_string());
                            } else {
                                version = text.trim().to_string();
                            }
                        },
                        _ => (),
                    }
                }

                let mut sections: Vec<(String, Vec<String>)> = Vec::new();
                let mut notes: Vec<String> = Vec::new();

                // h3 sections
                // and paragraphs without sections aka notes
                while let Some(event) =
                    parser.next_if(|e| e != &Event::Start(Tag::Heading(2)))
                {
                    match event {
                        // h3 section header
                        // starts a new section
                        Event::Start(Tag::Heading(3)) => {
                            let mut section_name: Option<String> = None;
                            let mut section_lines: Vec<String> = Vec::new();

                            // h3 section header text
                            while let Some(event) = parser.next() {
                                match event {
                                    Event::End(Tag::Heading(3)) => break,
                                    Event::Text(text) => {
                                        section_name = Some(text.trim().to_string());
                                    },
                                    _ => (),
                                }
                            }

                            // section list
                            while let Some(event) = parser.next_if(|e| {
                                e != &Event::Start(Tag::Heading(2))
                                    && e != &Event::Start(Tag::Heading(3))
                            }) {
                                if let Event::Start(Tag::Item) = event {
                                    let mut item_text: String = String::new();

                                    while let Some(event) = parser.next() {
                                        match event {
                                            Event::End(Tag::Item) => break,
                                            Event::Text(text) => {
                                                item_text.push_str(&text);
                                            },
                                            Event::Code(text) => {
                                                item_text.push('"');
                                                item_text.push_str(&text);
                                                item_text.push('"');
                                            },
                                            Event::SoftBreak => {
                                                item_text.push(' ');
                                            },
                                            _ => (),
                                        }
                                    }
                                    section_lines.push(item_text);
                                }
                            }

                            // section done
                            // save if not empty
                            if section_name.is_some() && !section_lines.is_empty() {
                                sections.push((section_name.unwrap(), section_lines));
                            }
                        },
                        // paragraph without section aka note
                        Event::Start(Tag::Paragraph) => {
                            while let Some(event) = parser.next() {
                                match event {
                                    Event::End(Tag::Paragraph) => break,
                                    Event::Text(text) => {
                                        notes.push(text.to_string());
                                    },
                                    _ => (),
                                }
                            }
                        },
                        _ => (),
                    }
                }

                // version done
                // save if not empty
                if !sections.is_empty() || !notes.is_empty() {
                    versions.push(ChangelogVersion {
                        version,
                        date,
                        sections,
                        notes,
                    })
                }
            }
        }

        Ok(ChangelogPanelComponent {
            etag,
            versions,
            display_count: 2,
        })
    }

    /// Returns new Changelog incase remote one is newer
    pub async fn update_changelog(
        version: String,
        channel: Channel,
    ) -> Result<Option<Self>> {
        match net::query_etag(consts::CHANGELOG_URL.replace("{tag}", &channel.0)).await? {
            Some(remote_version) => {
                if version != remote_version {
                    debug!(
                        "Changelog version different (Local: {} Remote: {}), fetching...",
                        version, remote_version
                    );
                    return Ok(Some(Self::fetch(channel).await?));
                } else {
                    debug!("Changelog up-to-date.");
                    Ok(None)
                }
            },
            // We query the changelog in case there's no etag to be found
            // to make sure the player stays informed.
            None => {
                debug!("Changelog missing, fetching...");
                Ok(Some(Self::fetch(channel).await?))
            },
        }
    }

    pub fn update(
        &mut self,
        msg: ChangelogPanelMessage,
    ) -> Option<Command<DefaultViewMessage>> {
        match msg {
            ChangelogPanelMessage::UpdateChangelog(result) => match result {
                Ok(Some(changelog)) => {
                    *self = changelog;
                    Some(Command::perform(
                        async { Action::Save },
                        DefaultViewMessage::Action,
                    ))
                },
                Ok(None) => None,
                Err(e) => {
                    tracing::trace!("Failed to update changelog: {}", e);
                    None
                },
            },
            ChangelogPanelMessage::ScrollPositionChanged(pos) => {
                if pos > 0.9 && self.display_count < self.versions.len() {
                    self.display_count += 1;
                }
                None
            },
        }
    }

    pub fn view(&self) -> Element<DefaultViewMessage> {
        let mut changelog = column().spacing(10);

        for version in &mut self.versions.iter().take(self.display_count) {
            changelog = changelog.push(version.view());
        }

        let top_row = row().height(Length::Units(50)).push(
            column().push(
                container(
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
                                Text::new("Latest Patch Notes")
                                    .color(DARK_WHITE)
                                    .font(POPPINS_MEDIUM_FONT),
                            )
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_y(Vertical::Center)
                            .padding(Padding::from([1, 0, 0, 8])),
                        )
                        .push(
                            container(
                                button(
                                    row()
                                        .push(
                                            text("Recent Changes")
                                                .color(Color::WHITE)
                                                .size(14),
                                        )
                                        .push(image(Handle::from_memory(
                                            UP_RIGHT_ARROW_ICON.to_vec(),
                                        )))
                                        .spacing(5)
                                        .align_items(Alignment::Center),
                                )
                                .on_press(DefaultViewMessage::Interaction(
                                    Interaction::OpenURL(
                                        GITLAB_MERGED_MR_URL.to_string(),
                                    ),
                                ))
                                .padding(Padding::from([2, 10, 2, 10]))
                                .height(Length::Units(20))
                                .style(GitlabChangelogButtonStyle),
                            )
                            .padding(Padding::from([0, 20, 0, 0]))
                            .height(Length::Fill)
                            .align_y(Vertical::Center)
                            .width(Length::Shrink),
                        )
                        .height(Length::Fill),
                )
                .align_y(Vertical::Center),
            ),
        );

        let col = column()
            .push(
                container(top_row)
                    .width(Length::Fill)
                    .style(ChangelogHeaderStyle),
            )
            .push(
                column().push(
                    container(
                        scrollable(changelog)
                            .on_scroll(|pos| {
                                DefaultViewMessage::ChangelogPanel(
                                    ChangelogPanelMessage::ScrollPositionChanged(pos),
                                )
                            })
                            .height(Length::Fill),
                    )
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .style(DarkContainerStyle),
                ),
            );

        let changelog_container = container(col);
        changelog_container.into()
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogVersion {
    pub version: String,
    pub date: Option<String>,
    pub notes: Vec<String>,
    pub sections: Vec<(String, Vec<String>)>,
}

impl ChangelogVersion {
    pub fn view(&self) -> Element<DefaultViewMessage> {
        let version_string = match &self.date {
            Some(date) => format!("v{} ({})", self.version, date),
            None => match self.version.as_str() {
                "Unreleased" => "Nightly".to_string(),
                _ => format!("v{}", self.version),
            },
        };

        let mut version = column().spacing(10).push(
            column()
                .push(
                    container(text(version_string).font(POPPINS_BOLD_FONT).size(28))
                        .padding(Padding::from([20, 0, 6, 33])),
                )
                .push(Rule::horizontal(8).style(RuleStyle)),
        );

        for note in &self.notes {
            version = version.push(text(note).size(18));
        }

        for (section_name, section_lines) in &self.sections {
            let mut section_col = column().push(text(section_name).size(23));

            for line in section_lines {
                section_col = section_col.push(
                    container(
                        row()
                            .push(text(" •  ").font(POPPINS_LIGHT_FONT).size(17))
                            .push(text(line).font(POPPINS_LIGHT_FONT).size(17)),
                    )
                    .padding(Padding::from([1, 0, 1, 10])),
                );
            }

            version = version
                .push(container(section_col).padding(Padding::from([0, 33, 0, 33])));
        }
        container(version).into()
    }
}
