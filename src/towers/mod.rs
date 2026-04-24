use bevy::prelude::*;

use crate::game::GameState;

pub mod components;
pub mod systems;

pub use components::*;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<components::TowerAssets>()
            .add_systems(
            OnEnter(GameState::Playing),
            systems::spawn_initial_towers,
        )
        .add_systems(
            Update,
            (
                systems::manage_ghost_tower,
                systems::update_ghost_tower,
                systems::update_delete_cursor,
                systems::handle_placement_click,
                systems::handle_remove_tower,
                systems::tick_towers,
                systems::move_projectiles,
                systems::animate_sprites,
                systems::update_tower_range_visibility,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }

}
