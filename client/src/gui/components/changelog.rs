use crate::{
    assets::{HAXRCORP_4089_FONT, HAXRCORP_4089_FONT_SIZE_2},
    consts,
    gui::views::default::{secondary_button, DefaultViewMessage, Interaction},
    net, Result,
};
use iced::{button, scrollable, Column, Element, Length, Row, Rule, Scrollable, Text};
use pulldown_cmark::{Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Changelog {
    pub versions: Vec<ChangelogVersion>,
    pub etag: String,

    #[serde(skip)]
    changelog_scrollable_state: scrollable::State,
    #[serde(skip)]
    read_on_gitlab_btn: button::State,
    #[serde(skip)]
    show_more_btn: button::State,
    #[serde(skip)]
    show_less_btn: button::State,
    #[serde(skip)]
    show_all_btn: button::State,
    #[serde(skip)]
    show_latest_only_btn: button::State,
    #[serde(skip)]
    pub display_count: usize,
}

impl Changelog {
    #[allow(clippy::while_let_on_iterator)]
    async fn fetch() -> Result<Self> {
        let mut versions: Vec<ChangelogVersion> = Vec::new();

        let changelog = net::query(consts::CHANGELOG_URL).await?;
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

        Ok(Changelog {
            etag,
            versions,
            display_count: 2,
            ..Default::default()
        })
    }

    /// Returns new Changelog incase remote one is newer
    pub async fn update(version: String) -> Result<Option<Self>> {
        match net::query_etag(consts::CHANGELOG_URL).await? {
            Some(remote_version) => {
                if version != remote_version {
                    return Ok(Some(Self::fetch().await?));
                } else {
                    tracing::debug!("Changelog up-to-date.");
                    Ok(None)
                }
            },
            // We query the changelog incase there's no etag to be found
            // to make sure the player stays informed.
            None => Ok(Some(Self::fetch().await?)),
        }
    }

    pub fn view(&mut self) -> Element<DefaultViewMessage> {
        let mut changelog = Scrollable::new(&mut self.changelog_scrollable_state)
            .height(Length::Fill)
            .padding(15)
            .spacing(20);

        for version in &mut self.versions.iter().take(self.display_count as usize) {
            changelog = changelog.push(version.view());
        }

        changelog
            .push(
                Row::new()
                    .spacing(10)
                    .push(secondary_button(
                        &mut self.show_more_btn,
                        "Show More",
                        Interaction::SetChangelogDisplayCount(
                            self.display_count.saturating_add(1),
                        ),
                    ))
                    .push(secondary_button(
                        &mut self.show_less_btn,
                        "Show Less",
                        Interaction::SetChangelogDisplayCount(
                            self.display_count.saturating_sub(1),
                        ),
                    ))
                    .push(secondary_button(
                        &mut self.show_all_btn,
                        "Show All",
                        Interaction::SetChangelogDisplayCount(self.versions.len()),
                    ))
                    .push(secondary_button(
                        &mut self.show_latest_only_btn,
                        "Show Latest Only",
                        Interaction::SetChangelogDisplayCount(1),
                    )),
            )
            .push(secondary_button(
                &mut self.read_on_gitlab_btn,
                "Read Changelog on Gitlab",
                Interaction::ReadMore(consts::CHANGELOG_URL_LINK.to_owned()),
            ))
            .into()
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

        let mut version = Column::new().spacing(10).push(
            Column::new()
                .push(
                    Text::new(version_string)
                        .font(HAXRCORP_4089_FONT)
                        .size(HAXRCORP_4089_FONT_SIZE_2),
                )
                .push(Rule::horizontal(8)),
        );

        for note in &self.notes {
            version = version.push(Text::new(note).size(18));
        }

        for (section_name, section_lines) in &self.sections {
            let mut section = Column::new().push(Text::new(section_name).size(22));

            for line in section_lines {
                section = section.push(
                    Row::new()
                        .push(Text::new(" • ").size(18))
                        .push(Text::new(line).size(18)),
                );
            }

            version = version.push(section);
        }

        version.into()
    }
}
