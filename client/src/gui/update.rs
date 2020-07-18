use super::{
    widgets::{Changelog, News},
    Airshipper, Interaction, LauncherState, Message,
};
use crate::{net, profiles::Profile, ProcessUpdate, Result};
use iced::Command;

pub fn handle_message(
    airship: &mut Airshipper,
    message: Message,
) -> Result<Command<Message>> {
    match message {
        Message::Loaded(saved_state) => {
            let saved_state = saved_state.unwrap_or_default();
            airship.update_from_save(saved_state);

            airship.state = LauncherState::QueryingForUpdates;
            return Ok(Command::batch(vec![
                Command::perform(
                    Changelog::update(airship.saveable_state.changelog.etag.clone()),
                    Message::ChangelogUpdate,
                ),
                Command::perform(
                    News::update(airship.saveable_state.news.etag.clone()),
                    Message::NewsUpdate,
                ),
                Command::perform(
                    Profile::update(airship.saveable_state.active_profile.clone()),
                    Message::GameUpdate,
                ),
            ]));
        },
        Message::ChangelogUpdate(update) => {
            if let Some(update) = update? {
                airship.saveable_state.changelog = update;
                airship.needs_save = true;
            }
        },
        Message::NewsUpdate(update) => {
            if let Some(update) = update? {
                airship.saveable_state.news = update;
                airship.needs_save = true;
            }
        },
        Message::ProcessUpdate(update) => match update {
            ProcessUpdate::Line(msg) => {
                log::info!("[Veloren] {}", msg);
            },
            ProcessUpdate::Exit(code) => {
                log::debug!("Veloren exited with {}", code);
                airship.state = LauncherState::QueryingForUpdates;
                return Ok(Command::perform(
                    Profile::update(airship.saveable_state.active_profile.clone()),
                    Message::GameUpdate,
                ));
            },
            ProcessUpdate::Error(err) => return Err(err.into()),
        },
        Message::GameUpdate(update) => match update? {
            Some(version) => {
                airship.state = LauncherState::UpdateAvailable(version);
            },
            None => {
                airship.state = LauncherState::ReadyToPlay;
            },
        },
        Message::Saved(res) => {
            let _ = res?;
            airship.needs_save = false;
            airship.saving = false;
            log::trace!("State saved.");
        },
        Message::Interaction(Interaction::PlayPressed) => {
            if let LauncherState::UpdateAvailable(version) = &airship.state {
                airship.state = LauncherState::Downloading(
                    airship.saveable_state.active_profile.url(),
                    airship.saveable_state.active_profile.download_path(),
                    version.clone(),
                )
            } else if let LauncherState::ReadyToPlay = airship.state {
                airship.state = LauncherState::Playing(Profile::start(
                    airship.saveable_state.active_profile.clone(),
                ));
            }
        },
        Message::Interaction(Interaction::ReadMore(url)) => {
            if let Err(e) = opener::open(&url) {
                return Err(format!("failed to open {} : {}", url, e).into());
            }
        },
        Message::InstallDone(result) => {
            let profile = result?;
            airship.saveable_state.active_profile = profile;
            airship.needs_save = true;
            airship.state = LauncherState::ReadyToPlay;
        },
        Message::DownloadProgress(progress) => match progress {
            net::Progress::Errored(e) => return Err(e.into()),
            net::Progress::Finished => {
                let version = match &airship.state {
                    LauncherState::Downloading(_, _, version) => version.to_string(),
                    _ => panic!(
                        "Reached impossible state: Downloading while not in download \
                         state!"
                    ),
                };
                airship.state = LauncherState::Installing;
                return Ok(Command::perform(
                    Profile::install(
                        airship.saveable_state.active_profile.clone(),
                        version.clone(),
                    ),
                    Message::InstallDone,
                ));
            },
            p => airship.download_progress = Some(p),
        },
        Message::Error(e) => {
            airship.state = LauncherState::Error(e);
        },
        Message::Interaction(Interaction::Disabled) => {},
    }

    if airship.needs_save && !airship.saving {
        airship.saving = true;
        log::trace!("Saving state...");
        return Ok(Command::perform(
            airship.save_state().save(),
            Message::Saved,
        ));
    }

    Ok(Command::none())
}
