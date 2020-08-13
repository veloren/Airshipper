use crate::profiles::{Profile, Profiles};

pub mod default;
pub mod profiles;

#[derive(Debug, Clone)]
pub enum View {
    Default,
    Profiles,
}

impl Default for View {
    fn default() -> Self {
        Self::Profiles // TODO
    }
}

/// An action requested by the current view
#[derive(Debug, Clone)]
pub enum Action {
    SwitchView(View),
    Save,

    UpdateProfile(Profile),
    UpdateProfiles(Profiles),
}
