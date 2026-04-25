use bevy::prelude::*;

use super::components::TowerType;

/// Shared sprite sheet assets for towers.
#[derive(Resource, Default)]
pub struct TowerAssets {
    /// tower_XX_upgrades.png — 6 cols × 4 rows, 74×110 per frame (row = upgrade level 0-3)
    pub upgrade_layout: Option<Handle<TextureAtlasLayout>>,
    /// cowswap_towers_ghost.png — 5 cols × 1 row, 84×110 per frame
    pub ghost_layout: Option<Handle<TextureAtlasLayout>>,
    /// cowswap_towers_icons.png — 5 cols × 1 row, 46×59 per frame
    pub icon_layout: Option<Handle<TextureAtlasLayout>>,
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
    /// Per-tower upgrade sheets (tower_XX_upgrades.png)
    pub cow_upgrades: Option<Handle<Image>>,
    pub ba_upgrades:  Option<Handle<Image>>,
    pub slv_upgrades: Option<Handle<Image>>,
    pub sg_upgrades:  Option<Handle<Image>>,
    pub dp_upgrades:  Option<Handle<Image>>,
}

impl TowerAssets {
    pub fn upgrade_sheet(&self, tower_type: &TowerType) -> Option<Handle<Image>> {
        match tower_type {
            TowerType::CoWMatcher      => self.cow_upgrades.clone(),
            TowerType::BatchAuctioneer => self.ba_upgrades.clone(),
            TowerType::Solver          => self.slv_upgrades.clone(),
            TowerType::SlippageGuard   => self.sg_upgrades.clone(),
            TowerType::DarkPoolNode    => self.dp_upgrades.clone(),
        }
    }
}
