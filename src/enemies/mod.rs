use bevy::prelude::*;

use crate::{game::GameState, resources::not_paused};

pub mod components;
pub mod resources;
pub mod systems;

pub use components::*;
pub use resources::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<resources::EnemyAssets>()
            .init_resource::<resources::WaveManager>()
            .add_systems(Startup, systems::setup_enemy_assets)
            .add_systems(
                Update,
                (
                    systems::tick_waves,
                    systems::find_enemy_targets,
                    systems::enemy_movement,
                    systems::extract_value,
                    systems::tick_enemy_slow,
                    systems::check_enemy_deaths,
                    systems::update_enemy_hp_bars,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing).and(not_paused)),
            );
    }
}
