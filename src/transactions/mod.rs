use bevy::prelude::*;

use crate::game::GameState;

pub mod components;
pub mod systems;

pub use components::*;

pub struct TransactionPlugin;

impl Plugin for TransactionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            systems::spawn_initial_transactions,
        )
        .add_systems(
            Update,
            (
                systems::move_transactions,
                systems::tick_mev_immunity,
                systems::tint_transactions,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
