use crate::profiles::Profile;

pub mod default;

#[derive(Debug, Clone)]
pub enum View {
    Default,
}

impl Default for View {
    fn default() -> Self {
        Self::Default
    }
}

/// An action requested by the current view
#[derive(Debug, Clone)]
pub enum Action {
    // TODO: SwitchView(View),
    Save,

    UpdateProfile(Profile),
}
