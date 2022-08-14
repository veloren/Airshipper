mod changelog_panel;
mod community_showcase_panel;
mod game_panel;
mod logo_panel;
mod news_panel;
mod settings_panel;

pub use changelog_panel::{ChangelogPanelComponent, ChangelogPanelMessage};
pub use community_showcase_panel::{
    CommunityShowcaseComponent, CommunityShowcasePanelMessage,
};
pub use game_panel::{GamePanelComponent, GamePanelMessage};
pub use logo_panel::LogoPanelComponent;
pub use news_panel::{NewsPanelComponent, NewsPanelMessage};
pub use settings_panel::{SettingsPanelComponent, SettingsPanelMessage};
