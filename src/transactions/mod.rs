use bevy::prelude::*;

use crate::{game::GameState, resources::not_paused};

pub mod components;
pub mod resources;
pub mod systems;

pub use components::*;
pub use resources::TxSpawner;

pub struct TransactionPlugin;

impl Plugin for TransactionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TxSpawner::new(3.0))
            .add_systems(OnEnter(GameState::Playing), systems::setup_tx_spawner)
            .add_systems(
                Update,
                (
                    systems::spawn_transactions,
                    systems::move_transactions,
                    systems::update_tx_sprites,
                    systems::update_tx_labels,
                    systems::update_tx_highlight,
                )
                    .run_if(in_state(GameState::Playing).and(not_paused)),
            );
    }
}
