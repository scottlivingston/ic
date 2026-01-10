use bevy::prelude::*;

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppMode {
    #[default]
    Checking,
    Onboarding,
    Normal,
}
