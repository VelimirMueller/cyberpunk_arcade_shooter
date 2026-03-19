use bevy::prelude::States;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    RoundAnnounce,
    RoundActive,
    Paused,
    GameOver,
    Won,
}
