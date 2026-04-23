use bevy::prelude::*;

use crate::game::GameState;

pub mod components;
pub mod resources;
pub mod systems;

pub use components::*;
pub use resources::TxSpawner;

pub struct TransactionPlugin;

impl Plugin for TransactionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TxSpawner::new(1.0))
            .add_systems(OnEnter(GameState::Playing), systems::setup_tx_spawner)
            .add_systems(
                Update,
                (
                    systems::spawn_transactions,
                    systems::move_transactions,
                    systems::tick_mev_immunity,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
