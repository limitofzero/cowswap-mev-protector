use bevy::{
    ecs::query::QueryFilter,
    prelude::*,
    sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d},
};

use crate::{
    mempool::MempoolPath,
    resources::{GameEconomy, PlacementMode},
};

use super::super::components::{
    AnimationTimer, DeleteCursor, GhostTower, Tower, TowerRangeVisual, TowerVisualLevel,
    UpgradePreview,
};
use super::super::resources::TowerAssets;
use super::TOWER_INTERACT_RADIUS;

const REMOVE_COST: f32 = 10.0;
const MIN_PATH_DISTANCE: f32 = 46.0;
const MIN_TOWER_SPACING: f32 = 40.0;

pub(super) fn cursor_world_pos(
    window: &Window,
    camera: &Camera,
    cam_t: &GlobalTransform,
) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    camera.viewport_to_world_2d(cam_t, cursor).ok()
}

pub(super) fn is_valid_placement<F: QueryFilter>(
    pos: Vec2,
    path: &MempoolPath,
    tower_q: &Query<&Transform, F>,
) -> bool {
    if path.is_near_path(pos, MIN_PATH_DISTANCE) {
        return false;
    }
    tower_q
        .iter()
        .all(|t| t.translation.truncate().distance(pos) >= MIN_TOWER_SPACING)
}

pub(super) fn spawn_range_visuals(
    p: &mut bevy::ecs::relationship::RelatedSpawnerCommands<'_, bevy::ecs::hierarchy::ChildOf>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    range: f32,
    c: bevy::color::Srgba,
    ghost: bool,
) {
    let fill_alpha = if ghost { 0.07 } else { 0.04 };
    let ring_alpha = if ghost { 0.70 } else { 0.55 };

    let mut fill = p.spawn((
        Mesh2d(meshes.add(Circle::new(range).mesh().resolution(128))),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: Color::srgba(c.red, c.green, c.blue, fill_alpha),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, -9.9),
    ));
    if !ghost {
        fill.insert((Visibility::Hidden, TowerRangeVisual));
    }

    let mut ring = p.spawn((
        Mesh2d(
            meshes.add(
                Annulus::new(range - 0.75, range + 0.75)
                    .mesh()
                    .resolution(128),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: Color::srgba(c.red, c.green, c.blue, ring_alpha),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, -9.85),
    ));
    if !ghost {
        ring.insert((Visibility::Hidden, TowerRangeVisual));
    }
}

/// Spawn/despawn the ghost tower when placement mode changes.
pub fn manage_ghost_tower(
    mut commands: Commands,
    placement_mode: Res<PlacementMode>,
    ghost_q: Query<(Entity, &GhostTower)>,
    delete_cursor_q: Query<Entity, With<DeleteCursor>>,
    tower_assets: Res<TowerAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !placement_mode.is_changed() {
        return;
    }
    // Always despawn whichever cursor is active before potentially spawning a new one
    for e in &delete_cursor_q {
        commands.entity(e).despawn();
    }
    match &*placement_mode {
        PlacementMode::Placing(tower_type) => {
            for (e, _) in &ghost_q {
                commands.entity(e).despawn();
            }
            let (Some(sheet), Some(layout)) = (
                tower_assets.ghost_sheet.clone(),
                tower_assets.ghost_layout.clone(),
            ) else {
                return;
            };
            let color = tower_type.color();
            let range = tower_type.range();
            let c = color.to_srgba();
            commands
                .spawn((
                    Sprite {
                        image: sheet,
                        texture_atlas: Some(TextureAtlas {
                            layout,
                            index: tower_type.atlas_index(),
                        }),
                        custom_size: Some(Vec2::new(84.0, 110.0)),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.65),
                        ..default()
                    },
                    Transform::from_xyz(0.0, -9999.0, 20.0),
                    GhostTower(tower_type.clone()),
                    Name::new("GhostTower"),
                ))
                .with_children(|p| {
                    spawn_range_visuals(p, &mut meshes, &mut materials, range, c, true);
                });
        }
        PlacementMode::Removing => {
            for (e, _) in &ghost_q {
                commands.entity(e).despawn();
            }
            if let Some(icon) = tower_assets.delete_icon.clone() {
                commands.spawn((
                    Sprite {
                        image: icon,
                        custom_size: Some(Vec2::new(37.0, 55.0)),
                        color: Color::srgba(1.0, 0.4, 0.4, 0.85),
                        ..default()
                    },
                    Transform::from_xyz(0.0, -9999.0, 20.0),
                    DeleteCursor,
                    Name::new("DeleteCursor"),
                ));
            }
        }
        PlacementMode::Idle => {
            for (e, _) in &ghost_q {
                commands.entity(e).despawn();
            }
        }
    }
}

/// Move the ghost to the cursor and tint it green/red based on placement validity.
pub fn update_ghost_tower(
    mut ghost_q: Query<(&mut Transform, &mut Sprite), With<GhostTower>>,
    tower_q: Query<&Transform, (With<Tower>, Without<GhostTower>)>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    path: Res<MempoolPath>,
    placement_mode: Res<PlacementMode>,
) {
    let PlacementMode::Placing(_) = &*placement_mode else {
        return;
    };
    let Ok((mut ghost_t, mut ghost_s)) = ghost_q.single_mut() else {
        return;
    };
    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else {
        return;
    };
    let Some(pos) = cursor_world_pos(window, cam, cam_t) else {
        return;
    };

    ghost_t.translation.x = pos.x;
    ghost_t.translation.y = pos.y;

    let valid = is_valid_placement(pos, &path, &tower_q);
    let a = ghost_s.color.alpha();
    ghost_s.color = if valid {
        Color::srgba(0.75, 1.0, 0.75, a)
    } else {
        Color::srgba(1.0, 0.45, 0.45, a)
    };
}

/// Move the delete cursor icon to follow the mouse during remove mode.
pub fn update_delete_cursor(
    mut cursor_q: Query<&mut Transform, With<DeleteCursor>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let Ok(mut t) = cursor_q.single_mut() else {
        return;
    };
    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else {
        return;
    };
    let Some(pos) = cursor_world_pos(window, cam, cam_t) else {
        return;
    };
    t.translation.x = pos.x;
    t.translation.y = pos.y;
}

/// Left-click to place, right-click / Escape to cancel.
#[allow(clippy::too_many_arguments)]
pub fn handle_placement_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut placement_mode: ResMut<PlacementMode>,
    path: Res<MempoolPath>,
    mut economy: ResMut<GameEconomy>,
    tower_q: Query<&Transform, With<Tower>>,
    ui_buttons: Query<&Interaction, With<Button>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tower_assets: Res<TowerAssets>,
) {
    let PlacementMode::Placing(ref tower_type) = *placement_mode else {
        return;
    };

    if mouse.just_pressed(MouseButton::Right) || keys.just_pressed(KeyCode::Escape) {
        *placement_mode = PlacementMode::Idle;
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // The same click that activated placement mode must not also place a tower
    if placement_mode.is_changed() {
        return;
    }

    // Don't place when clicking a UI button
    if ui_buttons.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else {
        return;
    };
    let Some(pos) = cursor_world_pos(window, cam, cam_t) else {
        return;
    };

    if !is_valid_placement(pos, &path, &tower_q) {
        return;
    }

    let cost = tower_type.cost();
    if economy.balance < cost {
        return;
    }
    economy.balance -= cost;

    let tower_type = tower_type.clone();
    *placement_mode = PlacementMode::Idle;

    let color = tower_type.color();
    let range = tower_type.range();
    let c = color.to_srgba();
    let (Some(sheet), Some(layout)) = (
        tower_assets.upgrade_sheet(&tower_type),
        tower_assets.upgrade_layout.clone(),
    ) else {
        return;
    };
    commands
        .spawn((
            Sprite {
                image: sheet.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: layout.clone(),
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
        .with_children(|p| {
            spawn_range_visuals(p, &mut meshes, &mut materials, range, c, false);
            p.spawn((
                Sprite {
                    image: sheet.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: layout.clone(),
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

/// When in Removing mode: left-click a tower to demolish it for REMOVE_COST COW.
/// RMB / Escape cancels the mode.
#[allow(clippy::too_many_arguments)]
pub fn handle_remove_tower(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut placement_mode: ResMut<PlacementMode>,
    tower_q: Query<(Entity, &Transform), With<Tower>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut economy: ResMut<GameEconomy>,
) {
    if *placement_mode != PlacementMode::Removing {
        return;
    }

    if mouse.just_pressed(MouseButton::Right) || keys.just_pressed(KeyCode::Escape) {
        *placement_mode = PlacementMode::Idle;
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else {
        return;
    };
    let Some(pos) = cursor_world_pos(window, cam, cam_t) else {
        return;
    };

    for (entity, transform) in &tower_q {
        if transform.translation.truncate().distance(pos) < TOWER_INTERACT_RADIUS {
            if economy.balance >= REMOVE_COST {
                economy.balance -= REMOVE_COST;
                commands.entity(entity).despawn_related::<Children>();
                commands.entity(entity).despawn();
                *placement_mode = PlacementMode::Idle;
            }
            return;
        }
    }
}
