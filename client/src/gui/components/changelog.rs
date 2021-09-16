use crate::{
    assets::{HAXRCORP_4089_FONT, HAXRCORP_4089_FONT_SIZE_2},
    consts,
    gui::views::default::{secondary_button, DefaultViewMessage, Interaction},
    net, Result,
};
use iced::{button, scrollable, Column, Element, Length, Row, Rule, Scrollable, Text};
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
    pub display_count: i32,
}

impl Changelog {
    async fn fetch() -> Result<Self> {
        let changelog = net::query(consts::CHANGELOG_URL).await?;
        let etag = net::get_etag(&changelog);

        let mut versions: Vec<ChangelogVersion> = Vec::new();
        let mut version: Option<&str> = None;
        let mut date: Option<&str> = None;

        let mut sections: Vec<(String, Vec<String>)> = Vec::new();
        let mut section_name: Option<&str> = None;
        let mut section_lines: Vec<String> = Vec::new();

        let save_version =
            |versions: &mut Vec<ChangelogVersion>,
             version: Option<&str>,
             date: Option<&str>,
             sections: Vec<(String, Vec<String>)>| {
                if sections.len() > 0 {
                    match version {
                        Some(version) => versions.push(ChangelogVersion {
                            version: version.to_string(),
                            date: match date {
                                Some(date) => Some(date.to_string()),
                                None => None,
                            },
                            sections: sections.clone(),
                        }),
                        None => (),
                    }
                }
            };

        let save_section = |sections: &mut Vec<(String, Vec<String>)>,
                            section_name: Option<&str>,
                            section_lines: Vec<String>| {
            if section_lines.len() > 0 {
                match section_name {
                    Some(section_name) => {
                        sections.push((section_name.to_string(), section_lines))
                    },
                    None => sections.push(("Info".to_string(), section_lines)),
                };
            }
        };

        let changelog_text = changelog.text().await?;
        for line in changelog_text
            .lines()
            .skip_while(|x| !x.contains(&"## [Unreleased]"))
        {
            // h3 - section heading
            if line.starts_with("###") {
                // save previous section
                save_section(&mut sections, section_name, section_lines);

                // initialize new section
                section_name = Some(line[3..].trim());
                section_lines = Vec::new();

                continue;
            }

            // h2 - version heading
            if line.starts_with("##") {
                // save previous section and version
                save_section(&mut sections, section_name, section_lines);
                save_version(&mut versions, version, date, sections);

                // initialize new version
                let seperator = line.match_indices(" - ").next();
                match seperator {
                    Some((index, _seperator)) => {
                        version = Some(line[2..index].trim());
                        date = Some(line[index + 3..].trim());
                    },
                    None => {
                        version = Some(line[2..].trim());
                        date = None;
                    },
                }
                version = match version {
                    Some(version) => {
                        let mut start = 0;
                        let mut end = version.len();
                        if version.starts_with("[") {
                            start = 1;
                        }
                        if version.ends_with("]") {
                            end -= 1;
                        }
                        if &version[end - 2..end] == ".0" {
                            end -= 2;
                        }
                        Some(&version[start..end])
                    },
                    None => None,
                };
                sections = Vec::new();
                section_name = None;
                section_lines = Vec::new();

                continue;
            }

            // line with text that isn't a link url
            if !line.trim().is_empty() && !line.trim().starts_with("[") {
                let mut trimmed_line = line.trim();

                if trimmed_line.starts_with("-") {
                    trimmed_line = trimmed_line[1..].trim();
                }

                if trimmed_line.starts_with("_") && trimmed_line.ends_with("_") {
                    trimmed_line = trimmed_line[1..trimmed_line.len() - 1].trim();
                }

                section_lines.push(trimmed_line.to_string())
            }
        }
        // save last section and version
        save_section(&mut sections, section_name, section_lines);
        save_version(&mut versions, version, date, sections);

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
                    log::debug!("Changelog up-to-date.");
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
                        Interaction::SetChangelogDisplayCount(self.display_count + 1),
                    ))
                    .push(secondary_button(
                        &mut self.show_less_btn,
                        "Show Less",
                        Interaction::SetChangelogDisplayCount(self.display_count - 1),
                    ))
                    .push(secondary_button(
                        &mut self.show_all_btn,
                        "Show All",
                        Interaction::SetChangelogDisplayCount(self.versions.len() as i32),
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
    pub sections: Vec<(String, Vec<String>)>,
}

impl ChangelogVersion {
    pub fn view(&self) -> Element<DefaultViewMessage> {
        let version_string = match &self.date {
            Some(date) => format!("v{} ({})", &self.version, date),
            None => match &self.version[..] {
                "Unreleased" => "Nightly".to_string(),
                _ => format!("v{}", &self.version),
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
