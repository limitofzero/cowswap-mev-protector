use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    /// Main game loop — transactions flowing, enemies attacking, towers defending
    #[default]
    Playing,
    Paused,
    GameOver,
}
