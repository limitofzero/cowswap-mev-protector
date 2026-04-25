pub(super) const TOWER_INTERACT_RADIUS: f32 = 42.0;

mod combat;
mod placement;
mod setup;
mod visuals;

pub use combat::{move_projectiles, tick_hit_effects, tick_towers};
pub use placement::{
    handle_placement_click, handle_remove_tower, manage_ghost_tower, update_delete_cursor,
    update_ghost_tower,
};
pub use setup::{setup_tower_assets, spawn_initial_towers};
pub use visuals::{
    animate_sprites, handle_tower_upgrade_click, sync_tower_upgrade_visuals,
    update_tower_range_visibility, update_upgrade_preview,
};
