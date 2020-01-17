use {
    super::{Airshipper, DownloadStage, Message},
    crate::{profiles::Profile, saved_state::SavedState},
    iced::Command,
    std::path::PathBuf,
    indicatif::HumanBytes,
};

pub fn handle_message(state: &mut Airshipper, message: Message) -> Command<Message> {
    let mut needs_save = false;

    match message {
        Message::Loaded(saved_state) => {
            if let Ok(saved) = saved_state {
                state.update_from_save(saved);
            }

            return Command::perform(
                check_for_update(state.active_profile.clone()),
                Message::UpdateCheckDone,
            );
        }
        Message::Saved(_) => {
            state.saving = false;
        }
        Message::PlayPressed => {
            if state.active_profile.newer_version.is_some() {
                if let DownloadStage::None = state.download {
                    state.download = state
                        .active_profile
                        .start_download()
                        .map(|(m, p)| DownloadStage::Download(m, p))
                        .unwrap_or(DownloadStage::None);
                    state.play_button_text = "Download".to_owned();
                }
            } else {
                return Command::perform(start(state.active_profile.clone()), Message::PlayDone);
            }
        }
        Message::UpdateCheckDone(profile) => {
            state.active_profile = profile;
            if state.active_profile.newer_version.is_some() {
                state.play_button_text = "Update".to_owned();
                state.progress = 0.0;
            }
            needs_save = true
        }
        Message::InstallDone(result) => {
            if let Ok(profile) = result {
                state.active_profile = profile;
                state.play_button_text = "PLAY".to_owned();
                state.progress = 100.0;
                needs_save = true;
            } else {
                state.play_button_text = "ERROR".to_owned();
                state.progress = 0.0;
            }

            state.download = DownloadStage::None;
        }
        Message::Tick(_) => {
            match &state.download.clone() {
                DownloadStage::Download(m, p) => {
                    let portion = ((m.download_progress().0 * 100) / m.download_progress().1) as f32;
                    state.progress = portion * 0.8; // Leave some percentages for the install process
                    state.download_speed = HumanBytes(m.download_speed() as u64);

                    if portion == 100.0 {
                        state.play_button_text = "Install".to_owned();
                        state.download = DownloadStage::Install;
                        return Command::perform(install(state.active_profile.clone(), p.clone()), Message::InstallDone);
                    }
                }
                _ => {}
            }
        }
        Message::PlayDone(_) => {}
    }

    if needs_save && !state.saving {
        state.saving = true;
        return Command::perform(state.into_save().save(), Message::Saved);
    }

    Command::none()
}

async fn check_for_update(mut profile: Profile) -> Profile {
    profile.check_for_update().await;
    profile
}

async fn install(profile: Profile, zip_path: PathBuf) -> Result<Profile, ()> {
    profile.install(zip_path).await.map_err(|_| ())
}

async fn start(profile: Profile) {
    profile.start().await;
}
