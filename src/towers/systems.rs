use bevy::{prelude::*, sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d}};

use crate::{
    enemies::components::Enemy,
    mempool::MempoolPath,
    resources::{GameEconomy, PlacementMode},
    transactions::{components::ImmunitySource, Transaction},
};

use super::components::{AnimationTimer, GhostTower, Projectile, Tower, TowerAssets, TowerType};

/// Tick every tower's cooldown and apply its effect when it fires.
pub fn tick_towers(
    mut commands: Commands,
    mut tower_query: Query<(&mut Tower, &Transform)>,
    mut tx_query: Query<(&mut Transaction, &Transform)>,
    mut enemy_query: Query<(Entity, &mut Enemy, &Transform)>,
    time: Res<Time>,
) {
    for (mut tower, tower_transform) in &mut tower_query {
        tower.cooldown.tick(time.delta());
        if !tower.cooldown.just_finished() {
            continue;
        }

        let tower_pos = tower_transform.translation.truncate();
        let range = tower.range;
        let tower_type = tower.tower_type.clone();

        match tower_type {
            TowerType::CoWMatcher => {
                let in_range: Vec<usize> = tx_query.iter().enumerate()
                    .filter(|(_, (_, t))| tower_pos.distance(t.translation.truncate()) <= range)
                    .map(|(i, _)| i).collect();
                if in_range.is_empty() { continue; }
                for (mut tx, _) in tx_query.iter_mut().take(2) {
                    tx.grant_immunity(6.0, ImmunitySource::CoWMatch);
                }
            }
            TowerType::BatchAuctioneer => {
                let in_range: Vec<usize> = tx_query.iter().enumerate()
                    .filter(|(_, (_, t))| tower_pos.distance(t.translation.truncate()) <= range)
                    .map(|(i, _)| i).collect();
                if in_range.is_empty() { continue; }
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
                let target = enemy_query.iter()
                    .filter(|(_, _, t)| tower_pos.distance(t.translation.truncate()) <= range)
                    .min_by(|a, b| {
                        tower_pos.distance(a.2.translation.truncate())
                            .partial_cmp(&tower_pos.distance(b.2.translation.truncate()))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(e, _, _)| e);

                if let Some(target_entity) = target {
                    commands.spawn((
                        Sprite {
                            color: TowerType::Solver.color(),
                            custom_size: Some(Vec2::splat(6.0)),
                            ..default()
                        },
                        Transform::from_xyz(tower_pos.x, tower_pos.y, 5.0),
                        Projectile { target: target_entity, speed: 280.0, damage: 30.0 },
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
            commands.entity(proj_entity).despawn();
        } else {
            let dir = (target_pos - proj_pos).normalize_or_zero();
            proj_t.translation += (dir * proj.speed * time.delta_secs()).extend(0.0);
        }
    }
}

/// Advance sprite animation frames for all animated entities.
/// Skips entities whose atlas index is currently outside the animation strip
/// (i.e. a status frame has been applied and should not be overwritten).
pub fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite)>,
) {
    for (mut anim, mut sprite) in &mut query {
        anim.timer.tick(time.delta());
        if anim.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                // Don't animate over a status frame that lives beyond our strip.
                if atlas.index >= anim.base + anim.frames {
                    continue;
                }
                let local = atlas.index.saturating_sub(anim.base);
                atlas.index = anim.base + (local + 1) % anim.frames;
            }
        }
    }
}


/// Spawn a starter set of towers for the demo scene.
pub fn spawn_initial_towers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut tower_assets: ResMut<TowerAssets>,
) {
    let layout: &[(TowerType, Vec2)] = &[
        (TowerType::BatchAuctioneer,    Vec2::new(-300.0, -100.0)),
        (TowerType::CoWMatcher,         Vec2::new(  0.0,  140.0)),
        (TowerType::DarkPoolNode,       Vec2::new(250.0, -130.0)),
        (TowerType::Solver,             Vec2::new(-150.0,  170.0)),
        (TowerType::SlippageGuard,      Vec2::new( 180.0,  120.0)),
    ];

    let texture_layout = TextureAtlasLayout::from_grid(UVec2::new(84, 122), 6, 1, None, None);
    let layout_handle = layouts.add(texture_layout);
    tower_assets.layout = Some(layout_handle.clone());

    for (tower_type, pos) in layout {
        let color = tower_type.color();
        let range = tower_type.range();

        let c = color.to_srgba();
        // Fill — very transparent
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(range).mesh().resolution(128))),
            MeshMaterial2d(materials.add(ColorMaterial {
                color: Color::srgba(c.red, c.green, c.blue, 0.04),
                alpha_mode: AlphaMode2d::Blend,
                ..default()
            })),
            Transform::from_xyz(pos.x, pos.y, 0.1),
            Name::new("TowerRangeFill"),
        ));
        // Border ring — more visible
        commands.spawn((
            Mesh2d(meshes.add(Annulus::new(range - 0.75, range + 0.75).mesh().resolution(128))),
            MeshMaterial2d(materials.add(ColorMaterial {
                color: Color::srgba(c.red, c.green, c.blue, 0.55),
                alpha_mode: AlphaMode2d::Blend,
                ..default()
            })),
            Transform::from_xyz(pos.x, pos.y, 0.15),
            Name::new("TowerRangeBorder"),
        ));

        let sprite = if let Some(image_path) = tower_type.sprite_path() {
            let texture = asset_server.load(image_path);
            
            Sprite {
                image: texture,
                texture_atlas: Some(TextureAtlas { layout: layout_handle.clone(), index: 0 }),
                custom_size: Some(Vec2::new(84.0, 122.0)),
                ..default()
            }
        } else {
            Sprite {
                color,
                custom_size: Some(Vec2::splat(26.0)),
                ..default()
            }
        };

        // Tower body
        let mut entity = commands.spawn((
            sprite,
            Transform::from_xyz(pos.x, pos.y, 10.0),
            Tower::new(tower_type.clone()),
            Name::new(format!("Tower::{}", tower_type.label())),
        ));
        // Only add animation if this tower has a spritesheet
        if tower_type.sprite_path().is_some() {
            entity.insert(AnimationTimer::new(3.0, 6));
        }
    }
}

// ─── Placement ────────────────────────────────────────────────────────────────

fn cursor_world_pos(
    window: &Window,
    camera: &Camera,
    cam_t: &GlobalTransform,
) -> Option<Vec2> {
    let cursor = window.cursor_position()?;
    camera.viewport_to_world_2d(cam_t, cursor).ok()
}

fn is_valid_placement<F: bevy::ecs::query::QueryFilter>(
    pos: Vec2,
    path: &MempoolPath,
    tower_q: &Query<&Transform, F>,
) -> bool {
    if path.is_near_path(pos, 46.0) { return false; }
    tower_q.iter().all(|t| t.translation.truncate().distance(pos) >= 40.0)
}

/// Spawn/despawn the ghost tower when placement mode changes.
pub fn manage_ghost_tower(
    mut commands: Commands,
    placement_mode: Res<PlacementMode>,
    ghost_q: Query<(Entity, &GhostTower)>,
    asset_server: Res<AssetServer>,
    tower_assets: Res<TowerAssets>,
) {
    if !placement_mode.is_changed() { return; }
    match &*placement_mode {
        PlacementMode::Placing(tower_type) => {
            for (e, _) in &ghost_q { commands.entity(e).despawn(); }
            let sprite = if let (Some(path), Some(layout)) = (tower_type.sprite_path(), &tower_assets.layout) {
                Sprite {
                    image: asset_server.load(path),
                    texture_atlas: Some(TextureAtlas { layout: layout.clone(), index: 0 }),
                    custom_size: Some(Vec2::new(84.0, 122.0)),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.65),
                    ..default()
                }
            } else {
                let c = tower_type.color().to_srgba();
                Sprite {
                    color: Color::srgba(c.red, c.green, c.blue, 0.65),
                    custom_size: Some(Vec2::splat(28.0)),
                    ..default()
                }
            };
            commands.spawn((
                sprite,
                Transform::from_xyz(0.0, -9999.0, 20.0),
                GhostTower(tower_type.clone()),
                Name::new("GhostTower"),
            ));
        }
        PlacementMode::Idle => {
            for (e, _) in &ghost_q { commands.entity(e).despawn(); }
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
    let PlacementMode::Placing(_) = &*placement_mode else { return };
    let Ok((mut ghost_t, mut ghost_s)) = ghost_q.single_mut() else { return };
    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else { return };
    let Some(pos) = cursor_world_pos(window, cam, cam_t) else { return };

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

/// Left-click to place, right-click / Escape to cancel.
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
    asset_server: Res<AssetServer>,
    tower_assets: Res<TowerAssets>,
) {
    let PlacementMode::Placing(ref tower_type) = *placement_mode else { return };

    if mouse.just_pressed(MouseButton::Right) || keys.just_pressed(KeyCode::Escape) {
        *placement_mode = PlacementMode::Idle;
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) { return; }

    // The same click that activated placement mode must not also place a tower
    if placement_mode.is_changed() { return; }

    // Don't place when clicking a UI button
    if ui_buttons.iter().any(|i| *i == Interaction::Pressed) { return; }

    let Ok(window) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else { return };
    let Some(pos) = cursor_world_pos(window, cam, cam_t) else { return };

    if !is_valid_placement(pos, &path, &tower_q) { return; }

    let cost = tower_type.cost();
    if economy.balance < cost { return; }
    economy.balance -= cost;

    let tower_type = tower_type.clone();
    *placement_mode = PlacementMode::Idle;

    // Spawn range visuals
    let color = tower_type.color();
    let range = tower_type.range();
    let c = color.to_srgba();
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(range).mesh().resolution(128))),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: Color::srgba(c.red, c.green, c.blue, 0.04),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })),
        Transform::from_xyz(pos.x, pos.y, 0.1),
    ));
    commands.spawn((
        Mesh2d(meshes.add(Annulus::new(range - 0.75, range + 0.75).mesh().resolution(128))),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: Color::srgba(c.red, c.green, c.blue, 0.55),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })),
        Transform::from_xyz(pos.x, pos.y, 0.15),
    ));
    let sprite = if let (Some(image_path), Some(layout)) = (tower_type.sprite_path(), &tower_assets.layout) {
        Sprite {
            image: asset_server.load(image_path),
            texture_atlas: Some(TextureAtlas { layout: layout.clone(), index: 0 }),
            custom_size: Some(Vec2::new(84.0, 122.0)),
            ..default()
        }
    } else {
        Sprite { color, custom_size: Some(Vec2::splat(26.0)), ..default() }
    };

    let mut entity = commands.spawn((
        sprite,
        Transform::from_xyz(pos.x, pos.y, 10.0),
        Tower::new(tower_type.clone()),
        Name::new(format!("Tower::{}", tower_type.label())),
    ));
    if tower_type.sprite_path().is_some() {
        entity.insert(AnimationTimer::new(3.0, 6));
    }
}
