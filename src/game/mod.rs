use bevy::prelude::*;

use crate::{
    enemies::EnemyPlugin, mempool::MempoolPlugin, resources::GameResourcesPlugin,
    towers::TowerPlugin, transactions::TransactionPlugin, ui::UiPlugin,
};

pub mod state;
pub use state::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>().add_plugins((
            GameResourcesPlugin,
            MempoolPlugin,
            TransactionPlugin,
            EnemyPlugin,
            TowerPlugin,
            UiPlugin,
        ));
    }
}
