use bevy::prelude::*;

use crate::game::GameState;

pub mod components;
pub mod systems;

pub use components::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<components::EnemyAssets>()
            .add_systems(
            OnEnter(GameState::Playing),
            systems::spawn_initial_enemies,
        )
        .add_systems(
            Update,
            (
                systems::find_enemy_targets,
                systems::enemy_movement,
                systems::extract_value,
                systems::tick_enemy_slow,
                systems::check_enemy_deaths,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}
