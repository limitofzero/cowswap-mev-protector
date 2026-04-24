use bevy::prelude::*;

/// Shared sprite sheet assets for towers.
#[derive(Resource, Default)]
pub struct TowerAssets {
    /// cowswap_towers_anim.png — 6 cols × 5 rows, 84×110 per frame
    pub anim_layout: Option<Handle<TextureAtlasLayout>>,
    /// cowswap_towers_ghost.png — 5 cols × 1 row, 84×110 per frame
    pub ghost_layout: Option<Handle<TextureAtlasLayout>>,
    /// cowswap_towers_icons.png — 5 cols × 1 row, 46×59 per frame
    pub icon_layout: Option<Handle<TextureAtlasLayout>>,
    pub anim_sheet: Option<Handle<Image>>,
    pub ghost_sheet: Option<Handle<Image>>,
    pub icon_sheet: Option<Handle<Image>>,
    /// towers/tower_delete.png — icon for the remove button
    pub delete_icon: Option<Handle<Image>>,
    /// solver_projectile.png — 6 cols × 1 row, 48×48 per frame
    pub proj_layout: Option<Handle<TextureAtlasLayout>>,
    pub proj_sheet: Option<Handle<Image>>,
    /// solver_hit.png — 8 cols × 1 row, 80×80 per frame
    pub hit_layout: Option<Handle<TextureAtlasLayout>>,
    pub hit_sheet: Option<Handle<Image>>,
}
