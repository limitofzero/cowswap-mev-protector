use bevy::prelude::*;

use crate::game::GameState;

pub mod components;
pub mod systems;

pub use components::*;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            systems::spawn_initial_towers,
        )
        .add_systems(
            Update,
            (
                systems::tick_towers,
                systems::animate_sprites,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }

}
