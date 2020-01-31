use {
    super::{Airshipper, Interaction, LauncherState, Message, SavedState},
    crate::{network, profiles::Profile, Result},
    iced::Command,
};

pub fn handle_message(airship: &mut Airshipper, message: Message) -> Result<Command<Message>> {
    let mut needs_save = false;

    match message {
        Message::Loaded(saved_state) => {
            let saved_state = saved_state.unwrap_or_default();
            airship.update_from_save(saved_state);

            airship.state = LauncherState::QueryingForUpdates;
            return Ok(Command::perform(
                check_for_updates(airship.saveable_state.clone()),
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
            match update? {
                Some((save, profile_update_available)) => {
                    airship.saveable_state = save;
                    if profile_update_available {
                        airship.state = LauncherState::UpdateAvailable;
                    } else {
                        airship.state = LauncherState::ReadyToPlay;
                    }
                }
                None => airship.state = LauncherState::ReadyToPlay,
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
            // After playing check for an possible update
            // useful if you got kicked from the server due to an update so you can instantly update too
            airship.state = LauncherState::QueryingForUpdates;
            return Ok(Command::perform(
                check_for_updates(airship.saveable_state.clone()),
                Message::UpdateCheckDone,
            ));
        }
        Message::Interaction(Interaction::Disabled) => {}
    }

    if needs_save && !airship.saving {
        airship.saving = true;
        return Ok(Command::perform(airship.into_save().save(), Message::Saved));
    }

    Ok(Command::none())
}

/// Returns new state if updated.
/// the bool signifies whether an profile update is available
async fn check_for_updates(mut saveable_state: SavedState) -> Result<Option<(SavedState, bool)>> {
    let mut modified = false;
    let mut profile_update_available = false;
    
    match network::compare_changelog_etag(&saveable_state.changelog_etag).await? {
        Some(remote_changelog_ver) => {
            saveable_state.changelog_etag = remote_changelog_ver;
            saveable_state.changelog = network::query_changelog().await?;
            modified = true;
            log::debug!("Changelog updated.")
        }
        None => log::debug!("Changelog up-to-date."),
    }
    
    match network::compare_news_etag(&saveable_state.news_etag).await? {
        Some(remote_news_ver) => {
            saveable_state.news_etag = remote_news_ver;
            saveable_state.news = network::query_news().await?;
            modified = true;
            log::debug!("News updated.")
        },
        None => log::debug!("News up-to-date."),
    }

    if saveable_state.active_profile.check_for_update().await?
        != saveable_state.active_profile.version
    {
        modified = true;
        profile_update_available = true;
        log::debug!("Found profile update.")
    }

    Ok(if modified {
        Some((saveable_state, profile_update_available))
    } else {
        None
    })
}

// TODO: call state.install_profile() instead
async fn install(profile: Profile) -> Result<Profile> {
    Ok(profile.install().await?)
}

// TODO: call state.start_profile() instead
async fn start(profile: Profile) -> Result<()> {
    Ok(profile.start()?)
}
