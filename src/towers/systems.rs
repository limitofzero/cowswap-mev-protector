use bevy::{
    prelude::*,
    sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d},
};

use crate::{
    enemies::components::Enemy,
    mempool::MempoolPath,
    resources::{GameEconomy, PlacementMode},
    transactions::{Transaction, components::ImmunitySource},
};

const REMOVE_COST: f32 = 10.0;

use super::components::{
    AnimationTimer, DeleteCursor, GhostTower, HitEffect, Projectile, Tower, TowerRangeVisual,
    TowerType, TowerVisualLevel, UpgradePreview,
};
use super::resources::TowerAssets;

/// Tick every tower's cooldown and apply its effect when it fires.
pub fn tick_towers(
    mut commands: Commands,
    mut tower_query: Query<(&mut Tower, &Transform)>,
    mut tx_query: Query<(&mut Transaction, &Transform)>,
    mut enemy_query: Query<(Entity, &mut Enemy, &Transform)>,
    tower_assets: Res<TowerAssets>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut tower, tower_transform) in &mut tower_query {
        tower.upgrade_cooldown = (tower.upgrade_cooldown - dt).max(0.0);
        tower.cooldown.tick(time.delta());
        if !tower.cooldown.just_finished() {
            continue;
        }

        let tower_pos = tower_transform.translation.truncate();
        let range = tower.range;
        let tower_type = tower.tower_type.clone();

        match tower_type {
            TowerType::CoWMatcher => {
                let in_range: Vec<usize> = tx_query
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, t))| tower_pos.distance(t.translation.truncate()) <= range)
                    .map(|(i, _)| i)
                    .collect();
                if in_range.is_empty() {
                    continue;
                }
                for (mut tx, _) in tx_query.iter_mut().take(2) {
                    tx.grant_immunity(6.0, ImmunitySource::CoWMatch);
                }
            }
            TowerType::BatchAuctioneer => {
                let in_range: Vec<usize> = tx_query
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, t))| tower_pos.distance(t.translation.truncate()) <= range)
                    .map(|(i, _)| i)
                    .collect();
                if in_range.is_empty() {
                    continue;
                }
                let batch_size = in_range.len() as u32;
                for (i, (mut tx, _)) in tx_query.iter_mut().enumerate() {
                    if in_range.contains(&i) {
                        tx.set_batch(i as u32, batch_size);
                    }
                }
            }
            TowerType::DarkPoolNode => {
                for (mut tx, tx_t) in tx_query.iter_mut() {
                    if tower_pos.distance(tx_t.translation.truncate()) <= range {
                        tx.grant_immunity(4.0, ImmunitySource::DarkPool);
                    }
                }
            }
            TowerType::Solver => {
                let target = enemy_query
                    .iter()
                    .filter(|(_, _, t)| tower_pos.distance(t.translation.truncate()) <= range)
                    .min_by(|a, b| {
                        tower_pos
                            .distance(a.2.translation.truncate())
                            .partial_cmp(&tower_pos.distance(b.2.translation.truncate()))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(e, _, _)| e);

                if let Some(target_entity) = target {
                    let (Some(sheet), Some(layout)) = (
                        tower_assets.proj_sheet.clone(),
                        tower_assets.proj_layout.clone(),
                    ) else {
                        continue;
                    };
                    commands.spawn((
                        Sprite {
                            image: sheet,
                            texture_atlas: Some(TextureAtlas { layout, index: 0 }),
                            custom_size: Some(Vec2::splat(24.0)),
                            ..default()
                        },
                        Transform::from_xyz(tower_pos.x, tower_pos.y, 5.0),
                        Projectile {
                            target: target_entity,
                            speed: 280.0,
                            damage: tower.tower_type.solver_damage(tower.upgrade_level),
                        },
                        AnimationTimer::new(12.0, 6),
                        Name::new("Projectile"),
                    ));
                }
            }
            TowerType::SlippageGuard => {
                for (_, mut enemy, enemy_t) in enemy_query.iter_mut() {
                    if tower_pos.distance(enemy_t.translation.truncate()) <= range {
                        enemy.apply_slow(3.0);
                    }
                }
            }
        }
    }
}

/// Move homing projectiles toward their targets; deal damage on contact.
pub fn move_projectiles(
    mut commands: Commands,
    mut proj_query: Query<(Entity, &Projectile, &mut Transform)>,
    mut enemy_query: Query<(&mut Enemy, &Transform), Without<Projectile>>,
    tower_assets: Res<TowerAssets>,
    time: Res<Time>,
) {
    for (proj_entity, proj, mut proj_t) in &mut proj_query {
        let Ok((mut enemy, enemy_t)) = enemy_query.get_mut(proj.target) else {
            commands.entity(proj_entity).despawn();
            continue;
        };

        let target_pos = enemy_t.translation.truncate();
        let proj_pos = proj_t.translation.truncate();
        let dist = proj_pos.distance(target_pos);

        if dist < 8.0 {
            enemy.hp = (enemy.hp - proj.damage).max(0.0);
            if let (Some(sheet), Some(layout)) = (
                tower_assets.hit_sheet.clone(),
                tower_assets.hit_layout.clone(),
            ) {
                commands.spawn((
                    Sprite {
                        image: sheet,
                        texture_atlas: Some(TextureAtlas { layout, index: 0 }),
                        custom_size: Some(Vec2::splat(48.0)),
                        ..default()
                    },
                    Transform::from_xyz(proj_t.translation.x, proj_t.translation.y, 5.0),
                    HitEffect {
                        timer: Timer::from_seconds(1.0 / 12.0, TimerMode::Repeating),
                        frames: 8,
                        frame: 0,
                    },
                    Name::new("HitEffect"),
                ));
            }
            commands.entity(proj_entity).despawn();
        } else {
            let dir = (target_pos - proj_pos).normalize_or_zero();
            proj_t.translation += (dir * proj.speed * time.delta_secs()).extend(0.0);
        }
    }
}

/// Advance one-shot hit animations frame-by-frame; despawn after the last frame.
pub fn tick_hit_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HitEffect, &mut Sprite)>,
) {
    for (entity, mut hit, mut sprite) in &mut query {
        hit.timer.tick(time.delta());
        if !hit.timer.just_finished() {
            continue;
        }
        hit.frame += 1;
        if hit.frame >= hit.frames {
            commands.entity(entity).despawn();
        } else if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = hit.frame;
        }
    }
}

/// Advance sprite animation frames for all animated entities.
/// Skips entities whose atlas index is currently outside the animation strip
/// (i.e. a status frame has been applied and should not be overwritten).
pub fn animate_sprites(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut Sprite)>) {
    for (mut anim, mut sprite) in &mut query {
        anim.timer.tick(time.delta());
        if anim.timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            // Don't animate over a status frame that lives beyond our strip.
            if atlas.index >= anim.base + anim.frames {
                continue;
            }
            let local = atlas.index.saturating_sub(anim.base);
            atlas.index = anim.base + (local + 1) % anim.frames;
        }
    }
}

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
        let c = color.to_srgba();
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
            .with_children(|p| {
                spawn_range_visuals(p, &mut meshes, &mut materials, range, c);
                p.spawn((
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

// ─── Placement ────────────────────────────────────────────────────────────────

fn cursor_world_pos(window: &Window, camera: &Camera, cam_t: &GlobalTransform) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    camera.viewport_to_world_2d(cam_t, cursor).ok()
}

fn is_valid_placement<F: bevy::ecs::query::QueryFilter>(
    pos: Vec2,
    path: &MempoolPath,
    tower_q: &Query<&Transform, F>,
) -> bool {
    if path.is_near_path(pos, 46.0) {
        return false;
    }
    tower_q
        .iter()
        .all(|t| t.translation.truncate().distance(pos) >= 40.0)
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
                    spawn_ghost_range_visuals(p, &mut meshes, &mut materials, range, c);
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

    // Spawn range visuals
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
            spawn_range_visuals(p, &mut meshes, &mut materials, range, c);
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

fn spawn_range_visuals(
    p: &mut bevy::ecs::relationship::RelatedSpawnerCommands<'_, bevy::ecs::hierarchy::ChildOf>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    range: f32,
    c: bevy::color::Srgba,
) {
    p.spawn((
        Mesh2d(meshes.add(Circle::new(range).mesh().resolution(128))),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: Color::srgba(c.red, c.green, c.blue, 0.04),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, -9.9),
        Visibility::Hidden,
        TowerRangeVisual,
    ));
    p.spawn((
        Mesh2d(
            meshes.add(
                Annulus::new(range - 0.75, range + 0.75)
                    .mesh()
                    .resolution(128),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: Color::srgba(c.red, c.green, c.blue, 0.55),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, -9.85),
        Visibility::Hidden,
        TowerRangeVisual,
    ));
}

fn spawn_ghost_range_visuals(
    p: &mut bevy::ecs::relationship::RelatedSpawnerCommands<'_, bevy::ecs::hierarchy::ChildOf>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    range: f32,
    c: bevy::color::Srgba,
) {
    p.spawn((
        Mesh2d(meshes.add(Circle::new(range).mesh().resolution(128))),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: Color::srgba(c.red, c.green, c.blue, 0.07),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, -9.9),
    ));
    p.spawn((
        Mesh2d(
            meshes.add(
                Annulus::new(range - 0.75, range + 0.75)
                    .mesh()
                    .resolution(128),
            ),
        ),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: Color::srgba(c.red, c.green, c.blue, 0.70),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, -9.85),
    ));
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
        if transform.translation.truncate().distance(pos) < 42.0 {
            if economy.balance >= REMOVE_COST {
                economy.balance -= REMOVE_COST;
                commands.entity(entity).despawn_related::<Children>();
                commands.entity(entity).despawn();
            }
            *placement_mode = PlacementMode::Idle;
            return;
        }
    }
}

/// Show range circles only for the tower the cursor is currently over.
pub fn update_tower_range_visibility(
    tower_q: Query<(&Transform, &Children), With<Tower>>,
    mut visual_q: Query<&mut Visibility, With<TowerRangeVisual>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let cursor = windows
        .single()
        .ok()
        .and_then(|w| w.cursor_position())
        .and_then(|c| {
            camera_q
                .single()
                .ok()
                .and_then(|(cam, cam_t)| cam.viewport_to_world_2d(cam_t, c).ok())
        });

    for (tower_t, children) in &tower_q {
        let hovered = cursor.is_some_and(|c| c.distance(tower_t.translation.truncate()) < 42.0);
        for &child in children {
            if let Ok(mut vis) = visual_q.get_mut(child) {
                *vis = if hovered {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

/// Sync the tower sprite to the correct upgrade row when upgrade_level changes.
/// Uses TowerVisualLevel to detect changes without firing every frame.
pub fn sync_tower_upgrade_visuals(
    tower_assets: Res<TowerAssets>,
    mut tower_q: Query<
        (
            &Tower,
            &mut TowerVisualLevel,
            &mut Sprite,
            &mut AnimationTimer,
        ),
        Without<UpgradePreview>,
    >,
) {
    let Some(layout) = tower_assets.upgrade_layout.clone() else {
        return;
    };
    for (tower, mut vis_level, mut sprite, mut anim) in &mut tower_q {
        if vis_level.0 == tower.upgrade_level {
            continue;
        }
        let level = tower.upgrade_level;
        vis_level.0 = level;
        let base = level as usize * 6;
        anim.base = base;
        anim.timer.reset();
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = base;
            atlas.layout = layout.clone();
        }
        if let Some(sheet) = tower_assets.upgrade_sheet(&tower.tower_type) {
            sprite.image = sheet;
        }
    }
}

type UpgradePreviewQ<'w, 's> = Query<
    'w,
    's,
    (&'static mut Visibility, &'static mut AnimationTimer),
    (With<UpgradePreview>, Without<TowerVisualLevel>),
>;

/// Show the next-level upgrade preview sprite on the hovered tower when the player can afford it.
/// The preview uses the same AnimationTimer fps as the tower so it animates at the same speed.
pub fn update_upgrade_preview(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    economy: Res<crate::resources::GameEconomy>,
    tower_assets: Res<TowerAssets>,
    tower_q: Query<(&Tower, &Transform, &Children)>,
    mut preview_q: UpgradePreviewQ,
) {
    let cursor = windows
        .single()
        .ok()
        .and_then(|w| w.cursor_position())
        .and_then(|c| {
            camera_q
                .single()
                .ok()
                .and_then(|(cam, cam_t)| cam.viewport_to_world_2d(cam_t, c).ok())
        });

    let _layout = tower_assets.upgrade_layout.clone();

    for (tower, tower_t, children) in &tower_q {
        let hovered = cursor.is_some_and(|c| c.distance(tower_t.translation.truncate()) < 42.0);
        let can_afford = tower.can_upgrade()
            && economy.balance >= tower.tower_type.upgrade_cost(tower.upgrade_level);

        for &child in children {
            let Ok((mut vis, mut anim)) = preview_q.get_mut(child) else {
                continue;
            };
            if hovered && can_afford {
                let next_base = (tower.upgrade_level as usize + 1) * 6;
                if anim.base != next_base {
                    anim.base = next_base;
                    // Reset to start of the new level row
                    anim.timer.reset();
                }
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

/// Left-click a hovered tower in Idle mode to purchase the next upgrade level.
pub fn handle_tower_upgrade_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    placement_mode: Res<PlacementMode>,
    mut economy: ResMut<crate::resources::GameEconomy>,
    mut tower_q: Query<(&mut Tower, &Transform)>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if *placement_mode != PlacementMode::Idle {
        return;
    }

    let Ok(win) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else {
        return;
    };
    let Some(cursor) = win
        .cursor_position()
        .and_then(|c| cam.viewport_to_world_2d(cam_t, c).ok())
    else {
        return;
    };

    // Ignore clicks in the bottom bar UI area
    let bot_edge = -win.height() * 0.5 + crate::ui::BOT_BAR_H;
    if cursor.y < bot_edge {
        return;
    }

    for (mut tower, tower_t) in &mut tower_q {
        if tower_t.translation.truncate().distance(cursor) < 42.0 {
            if !tower.can_upgrade() {
                return;
            }
            let cost = tower.tower_type.upgrade_cost(tower.upgrade_level);
            if economy.balance < cost {
                return;
            }
            economy.balance -= cost;
            tower.apply_upgrade();
            return;
        }
    }
}
