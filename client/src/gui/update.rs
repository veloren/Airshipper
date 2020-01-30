use {
    super::{Airshipper, Interaction, LauncherState, Message},
    crate::{network, profiles::Profile, Result},
    iced::Command,
};

pub fn handle_message(airship: &mut Airshipper, message: Message) -> Result<Command<Message>> {
    let mut needs_save = false;

    match message {
        Message::Loaded(saved_state) => {
            let saved_state = saved_state.unwrap_or_default();
            airship.update_from_save(saved_state);
            
            airship.state = LauncherState::QueryingChangelogAndNews;
            return Ok(Command::perform(
                check_for_updates(
                    airship.saveable_state.active_profile.clone(),
                    airship.saveable_state.changelog_etag.clone(),
                    airship.saveable_state.news_etag.clone(),
                ),
                Message::UpdateCheckDone,
            ));
        }
        Message::Saved(_) => {
            airship.saving = false;
        }
        Message::Interaction(Interaction::PlayPressed) => {
            if let LauncherState::UpdateAvailable = airship.state {
                airship.state = LauncherState::Downloading(
                    airship.saveable_state.active_profile.start_download()?,
                )
            } else {
                match airship.state {
                    LauncherState::ReadyToPlay => {
                        airship.state = LauncherState::Playing;
                        return Ok(Command::perform(
                            start(airship.saveable_state.active_profile.clone()),
                            Message::PlayDone,
                        ));
                    }
                    _ => {}
                }
            }
        }
        Message::Interaction(Interaction::ReadMore(url)) => {
            if let Err(e) = opener::open(&url) {
                return Err(format!("failed to open {} : {}", url, e).into());
            }
        }
        Message::UpdateCheckDone(update) => {
            let (update_available, changelog, news) = update?;

            if update_available {
                airship.state = LauncherState::UpdateAvailable;
            } else {
                airship.state = LauncherState::ReadyToPlay;
            }

            if let Some(changelog) = changelog {
                airship.saveable_state.changelog = changelog;
            }
            if let Some(news) = news {
                airship.saveable_state.news = news;
            }
            needs_save = true;
        }
        Message::InstallDone(result) => {
            let profile = result?;
            airship.saveable_state.active_profile = profile;
            needs_save = true;
            airship.state = LauncherState::ReadyToPlay;
        }
        Message::Tick(_) => match &airship.state {
            LauncherState::Downloading(m) => {
                let percentage = ((m.download_progress().0 * 100) / m.download_progress().1) as f32;
                if percentage == 100.0 {
                    airship.state = LauncherState::Installing;
                    return Ok(Command::perform(
                        install(airship.saveable_state.active_profile.clone()),
                        Message::InstallDone,
                    ));
                }
            }
            _ => {}
        },
        Message::Error(e) | Message::PlayDone(Err(e)) => {
            airship.state = LauncherState::Error(e);
        }
        // Everything went fine when playing the game :O
        Message::PlayDone(Ok(())) => {
            airship.state = LauncherState::ReadyToPlay;
        }
        Message::Interaction(Interaction::Disabled) => {}
    }

    if needs_save && !airship.saving {
        airship.saving = true;
        return Ok(Command::perform(airship.into_save().save(), Message::Saved));
    }

    Ok(Command::none())
}

/// Will return whether an update is available, updated changelog and news.
async fn check_for_updates(
    profile: Profile,
    changelog_etag: String,
    news_etag: String,
) -> Result<(bool, Option<String>, Option<Vec<network::Post>>)> {
    let update_available = profile.check_for_update().await? != profile.version;

    let mut changelog = None;
    if network::compare_changelog_etag(&changelog_etag).await? {
        changelog = Some(network::query_changelog().await?);
    }
    let mut news = None;
    if network::compare_news_etag(&news_etag).await? {
        news = Some(network::query_news().await?);
    }

    Ok((update_available, changelog, news))
}

// TODO: call state.install_profile() instead
async fn install(profile: Profile) -> Result<Profile> {
    Ok(profile.install().await?)
}

// TODO: call state.start_profile() instead
async fn start(profile: Profile) -> Result<()> {
    Ok(profile.start()?)
}
