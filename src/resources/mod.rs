use bevy::prelude::*;

/// Running totals for the current game session.
#[derive(Resource, Default)]
pub struct GameScore {
    /// Transactions that reached the settlement layer intact.
    pub txs_settled: u32,
    /// Total ETH value that made it through.
    pub value_protected: f32,
    /// Total ETH value stolen by MEV bots.
    pub value_extracted: f32,
}

/// Tracks which wave of enemies we're on and their spawn timing.
#[derive(Resource)]
pub struct WaveState {
    pub wave: u32,
    pub spawn_timer: Timer,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            wave: 1,
            spawn_timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        }
    }
}

pub struct GameResourcesPlugin;

impl Plugin for GameResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameScore>()
            .init_resource::<WaveState>();
    }
}
