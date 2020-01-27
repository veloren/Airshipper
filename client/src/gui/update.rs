use {
    super::{Airshipper, DownloadStage, Interaction, Message},
    crate::{network, profiles::Profile, Result},
    iced::Command,
    indicatif::HumanBytes,
    std::path::PathBuf,
};

pub fn handle_message(state: &mut Airshipper, message: Message) -> Result<Command<Message>> {
    let mut needs_save = false;

    match message {
        Message::Loaded(saved_state) => {
            let saved_state = saved_state?;
            state.update_from_save(saved_state);

            return Ok(Command::perform(
                check_for_updates(
                    state.active_profile.clone(),
                    state.changelog_etag.clone(),
                    state.news_etag.clone(),
                ),
                Message::UpdateCheckDone,
            ));
        }
        Message::Saved(_) => {
            state.saving = false;
        }
        Message::Interaction(Interaction::PlayPressed) => {
            if state.active_profile.remote_version.is_some() {
                if let DownloadStage::None = state.download {
                    state.download = state
                        .active_profile
                        .start_download()
                        .map(|(m, p)| DownloadStage::Download(m, p))
                        .unwrap_or(DownloadStage::None);
                    state.play_button_text = "Downloading".to_owned();
                    state.download_text = "Update is being downloaded...".to_owned();
                }
            } else {
                return Ok(Command::perform(start(state.active_profile.clone()), Message::PlayDone));
            }
        }
        Message::Interaction(Interaction::ReadMore(url)) => {
            if let Err(e) = opener::open(&url) {
                return Err(format!("failed to open {} : {}", url, e).into());
            }
        }
        Message::UpdateCheckDone(update) => {
            let (profile, changelog, news) = update?;
            
            state.active_profile = profile;
            if state.active_profile.remote_version.is_some() {
                state.play_button_text = "Update".to_owned();
                state.download_text = "Update available".to_owned();
                state.progress = 0.0;
            } else {
                state.play_button_text = "PLAY".to_owned();
                state.download_text = "Ready to play".to_owned();
                state.progress = 100.0;
            }
            if let Some(changelog) = changelog {
                state.changelog = changelog;
            }
            if let Some(news) = news {
                state.news = news;
            }
            needs_save = true
        }
        Message::InstallDone(result) => {
            let profile = result?;
            state.active_profile = profile;
            state.play_button_text = "PLAY".to_owned();
            state.download_text = "Ready to play".to_owned();
            state.progress = 100.0;
            needs_save = true;
            state.download = DownloadStage::None;
        }
        Message::Tick(_) => {
            match &state.download.clone() {
                DownloadStage::Download(m, p) => {
                    let portion =
                        ((m.download_progress().0 * 100) / m.download_progress().1) as f32;
                    state.progress = portion * 0.8; // Leave some percentages for the install process
                    state.download_speed = HumanBytes(m.download_speed() as u64);

                    if portion == 100.0 {
                        state.play_button_text = "Install".to_owned();
                        state.download_text = "Update is being installed...".to_owned();
                        state.download = DownloadStage::Install;
                        return Ok(Command::perform(
                            install(state.active_profile.clone(), p.clone()),
                            Message::InstallDone,
                        ));
                    }
                }
                _ => {}
            }
        }
        Message::Error(e) | Message::PlayDone(Err(e)) => {
            state.play_button_text = "ERROR".to_owned();
            state.download_text = format!("{}", e);
            state.progress = 0.0;
        }
        // Everything went fine when playing the game :O
        Message::PlayDone(Ok(())) => {}
    }

    if needs_save && !state.saving {
        state.saving = true;
        return Ok(Command::perform(state.into_save().save(), Message::Saved));
    }

    Ok(Command::none())
}

/// Will check for profile updates and updated changelog, news.
async fn check_for_updates(
    profile: Profile,
    changelog_etag: String,
    news_etag: String,
) -> Result<(Profile, Option<String>, Option<Vec<network::Post>>)> {
    let profile = check_for_update(profile).await?;

    let mut changelog = None;
    if network::compare_changelog_etag(&changelog_etag).await? {
        changelog = Some(network::query_changelog().await?);
    }
    let mut news = None;
    if network::compare_news_etag(&news_etag).await? {
        news = Some(network::query_news().await?);
    }

    Ok((profile, changelog, news))
}

async fn check_for_update(mut profile: Profile) -> Result<Profile> {
    profile.check_for_update().await?;
    Ok(profile)
}

async fn install(profile: Profile, zip_path: PathBuf) -> Result<Profile> {
    Ok(profile.install(zip_path).await?)
}

async fn start(profile: Profile) -> Result<()> {
    Ok(profile.start()?)
}
