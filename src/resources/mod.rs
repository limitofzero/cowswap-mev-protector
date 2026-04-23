use bevy::prelude::*;

/// Running totals for the current game session.
#[derive(Resource, Default)]
pub struct GameScore {
    pub txs_settled: u32,
    pub value_protected: f32,
    pub value_extracted: f32,
}

/// Player economy: balance in COW tokens, fee earned per settled tx.
#[derive(Resource)]
pub struct GameEconomy {
    /// Current COW balance (spendable on towers).
    pub balance: f32,
    /// Fraction of a tx's COW value paid as fee on settlement (e.g. 0.01 = 1%).
    pub fee_rate: f32,
}

impl Default for GameEconomy {
    fn default() -> Self {
        Self { balance: 300.0, fee_rate: 0.01 }
    }
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
            .init_resource::<GameEconomy>()
            .init_resource::<WaveState>();
    }
}
