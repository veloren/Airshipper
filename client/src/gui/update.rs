use {
    super::{Airshipper, Message},
    crate::{profiles::Profile, saved_state::SavedState},
    iced::Command,
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
            if !state.downloading {
                state.downloading = true;
                state.play_button_text = "Loading...".to_owned();
                return Command::perform(
                    download_or_run(state.active_profile.clone()),
                    Message::DownloadDone,
                );
            }
        }
        Message::UpdateCheckDone(profile) => {
            state.active_profile = profile;
            if state.active_profile.newer_version.is_some() {
                state.play_button_text = "Update".to_owned();
            }
            needs_save = true
        }
        Message::DownloadDone(profile) => {
            state.active_profile = profile;
            state.downloading = false;
            state.play_button_text = "PLAY".to_owned();
            needs_save = true
        }
    }

    if needs_save && !state.saving {
        state.saving = true;
        return Command::perform(SavedState::from(state.clone()).save(), Message::Saved);
    }

    Command::none()
}

async fn download_or_run(mut profile: Profile) -> Profile {
    if profile.is_ready() && profile.newer_version.is_none() {
        profile.start().await;
        profile
    } else {
        profile.download().await;
        profile
    }
}

async fn check_for_update(mut profile: Profile) -> Profile {
    profile.check_for_update().await;
    profile
}
