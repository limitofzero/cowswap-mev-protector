use bevy::{prelude::*, sprite_render::ColorMaterial};

use super::super::components::{
    AnimationTimer, Tower, TowerType, TowerVisualLevel, UpgradePreview,
};
use super::super::resources::TowerAssets;
use super::placement::spawn_range_visuals;

/// Load all tower sprite sheet handles into TowerAssets. Runs at Startup so
/// OnEnter(Playing) systems can rely on the handles being populated.
pub fn setup_tower_assets(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut tower_assets: ResMut<TowerAssets>,
) {
    let upgrade_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(74, 110),
        6,
        4,
        None,
        None,
    ));
    let ghost_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(84, 110),
        5,
        1,
        None,
        None,
    ));
    let icon_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(46, 59),
        5,
        1,
        None,
        None,
    ));
    let proj_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(48),
        6,
        1,
        None,
        None,
    ));
    let hit_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(80),
        8,
        1,
        None,
        None,
    ));

    tower_assets.upgrade_layout = Some(upgrade_layout);
    tower_assets.ghost_layout = Some(ghost_layout);
    tower_assets.icon_layout = Some(icon_layout);
    tower_assets.proj_layout = Some(proj_layout);
    tower_assets.hit_layout = Some(hit_layout);
    tower_assets.ghost_sheet = Some(asset_server.load("towers/cowswap_towers_ghost.png"));
    tower_assets.icon_sheet = Some(asset_server.load("towers/cowswap_towers_icons.png"));
    tower_assets.delete_icon = Some(asset_server.load("towers/tower_delete.png"));
    tower_assets.proj_sheet = Some(asset_server.load("towers/solver_projectile.png"));
    tower_assets.hit_sheet = Some(asset_server.load("towers/solver_hit.png"));
    tower_assets.cow_upgrades = Some(asset_server.load("towers/tower_cow_upgrades.png"));
    tower_assets.ba_upgrades = Some(asset_server.load("towers/tower_ba_upgrades.png"));
    tower_assets.slv_upgrades = Some(asset_server.load("towers/tower_slv_upgrades.png"));
    tower_assets.sg_upgrades = Some(asset_server.load("towers/tower_sg_upgrades.png"));
    tower_assets.dp_upgrades = Some(asset_server.load("towers/tower_dp_upgrades.png"));
}

/// Spawn a starter set of towers for the demo scene.
pub fn spawn_initial_towers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tower_assets: Res<TowerAssets>,
) {
    let layout: &[(TowerType, Vec2)] = &[
        (TowerType::CoWMatcher, Vec2::new(-380.0, 90.0)),
        (TowerType::BatchAuctioneer, Vec2::new(-80.0, -65.0)),
        (TowerType::DarkPoolNode, Vec2::new(220.0, 100.0)),
        (TowerType::Solver, Vec2::new(-200.0, 240.0)),
        (TowerType::SlippageGuard, Vec2::new(-200.0, -245.0)),
    ];

    let upgrade_layout = tower_assets.upgrade_layout.clone().unwrap();

    for (tower_type, pos) in layout {
        let color = tower_type.color();
        let range = tower_type.range();
        let srgba = color.to_srgba();
        let sheet = tower_assets.upgrade_sheet(tower_type).unwrap();

        commands
            .spawn((
                Sprite {
                    image: sheet.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: upgrade_layout.clone(),
                        index: 0,
                    }),
                    custom_size: Some(Vec2::new(84.0, 110.0)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, 10.0),
                Tower::new(tower_type.clone()),
                AnimationTimer::new_with_offset(6.0 / tower_type.cooldown_secs(), 6, 0),
                TowerVisualLevel(0),
                Name::new(format!("Tower::{}", tower_type.label())),
            ))
            .with_children(|children| {
                spawn_range_visuals(children, &mut meshes, &mut materials, range, srgba, false);
                children.spawn((
                    Sprite {
                        image: sheet.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: upgrade_layout.clone(),
                            index: 6,
                        }),
                        custom_size: Some(Vec2::new(84.0, 110.0)),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.7),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 1.0),
                    Visibility::Hidden,
                    AnimationTimer::new_with_offset(6.0 / tower_type.cooldown_secs(), 6, 6),
                    UpgradePreview,
                ));
            });
    }
}
