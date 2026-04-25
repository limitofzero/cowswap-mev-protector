use bevy::prelude::*;

use crate::{resources::NetworkLoad, towers::AnimationTimer, transactions::Transaction};

use super::components::{Enemy, EnemyHpBarFg, EnemyType};
use super::resources::{EnemyAssets, WaveManager};

const BAR_H: f32 = 5.0;
/// HP bar width as a fraction of the enemy's sprite size.
const HP_BAR_WIDTH_SCALE: f32 = 0.85;
/// Gap (px) between the bottom of the enemy sprite and the top of the HP bar.
const HP_BAR_Y_GAP: f32 = 7.0;
/// Animation frames per level row in the enemy sprite sheet.
const ENEMY_ANIM_FRAMES: usize = 6;
/// Enemy sprite animation speed in frames-per-second.
const ENEMY_ANIM_FPS: f32 = 3.0;
/// Tile size (px, square) of each cell in the enemy upgrade atlas.
const ENEMY_ATLAS_TILE: u32 = 96;
/// Columns in the enemy upgrade atlas (one per animation frame).
const ENEMY_ATLAS_COLS: u32 = 6;
/// Rows in the enemy upgrade atlas (one per upgrade level).
const ENEMY_ATLAS_ROWS: u32 = 4;

/// Phase 1 — assign each enemy its nearest unclaimed tx (one enemy per tx).
/// Enemies with a valid existing target keep it; only bots whose tx is gone/immune re-target.
pub fn find_enemy_targets(
    mut enemy_query: Query<(&mut Enemy, &Transform)>,
    tx_query: Query<(Entity, &Transaction, &Transform)>,
) {
    // Lock in targets that are still valid so no other bot can steal them.
    let mut claimed: std::collections::HashSet<Entity> = enemy_query
        .iter()
        .filter_map(|(enemy, _)| {
            let target_entity = enemy.target?;
            let still_valid = tx_query
                .get(target_entity)
                .is_ok_and(|(_, tx, _)| !tx.is_immune());
            still_valid.then_some(target_entity)
        })
        .collect();

    for (mut enemy, enemy_transform) in &mut enemy_query {
        // Keep valid existing target.
        if let Some(target_entity) = enemy.target
            && claimed.contains(&target_entity)
        {
            continue;
        }

        // Lost target or had none — find nearest free tx.
        let pos = enemy_transform.translation.truncate();
        enemy.target = tx_query
            .iter()
            .filter_map(|(entity, tx, tx_transform)| {
                if tx.is_immune() || claimed.contains(&entity) {
                    return None;
                }
                Some((entity, pos.distance(tx_transform.translation.truncate())))
            })
            .min_by(|lhs, rhs| {
                lhs.1
                    .partial_cmp(&rhs.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(entity, _)| entity);

        if let Some(target_entity) = enemy.target {
            claimed.insert(target_entity);
        }
    }
}

/// Phase 2 — extract value only when within attack range.
/// Multiple enemies can target the same tx; drains are accumulated then applied once.
pub fn extract_value(
    enemy_query: Query<(&Enemy, &Transform)>,
    mut tx_query: Query<(&mut Transaction, &Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let mut drains: std::collections::HashMap<Entity, f32> = std::collections::HashMap::new();

    // Read pass — .get() is immutable, no aliasing issue
    for (enemy, enemy_t) in &enemy_query {
        let Some(target) = enemy.target else { continue };
        let Ok((tx, tx_t)) = tx_query.get(target) else {
            continue;
        };
        let dist = enemy_t
            .translation
            .truncate()
            .distance(tx_t.translation.truncate());
        if dist > enemy.attack_range {
            continue;
        }
        // Batched txs dilute the drain across all members — attacker can't isolate one tx
        let batch_scale = tx.batch.map_or(1.0, |(_, size)| 1.0 / size.max(1) as f32);
        *drains.entry(target).or_default() += tx.value * enemy.drain_rate * batch_scale * dt;
    }

    // Write pass — each entity touched exactly once
    for (target, total) in drains {
        if let Ok((mut tx, _)) = tx_query.get_mut(target) {
            tx.remaining_value = (tx.remaining_value - total).max(0.0);
        }
    }
}

/// Move each enemy toward its target tx, or toward the nearest path point when idle.
pub fn enemy_movement(
    mut enemy_query: Query<(&Enemy, &mut Transform)>,
    tx_query: Query<&Transform, (With<Transaction>, Without<Enemy>)>,
    path: Res<crate::mempool::MempoolPath>,
    time: Res<Time>,
) {
    for (enemy, mut transform) in &mut enemy_query {
        let pos = transform.translation.truncate();
        let dest = if let Some(target) = enemy.target {
            tx_query.get(target).ok().map(|t| t.translation.truncate())
        } else {
            // No target — walk toward the nearest point on the path
            Some(path.nearest_point(pos))
        };
        if let Some(d) = dest {
            let dir = (d - pos).normalize_or_zero();
            transform.translation +=
                (dir * enemy.effective_speed() * time.delta_secs()).extend(0.0);
        }
    }
}

pub fn tick_enemy_slow(mut query: Query<&mut Enemy>, time: Res<Time>) {
    for mut enemy in &mut query {
        enemy.tick_slow(time.delta());
    }
}

pub fn check_enemy_deaths(mut commands: Commands, query: Query<(Entity, &Enemy)>) {
    for (entity, enemy) in &query {
        if enemy.hp <= 0.0 {
            commands.entity(entity).despawn_related::<Children>();
            commands.entity(entity).despawn();
        }
    }
}

/// Update HP bar width and color each frame to reflect current health.
pub fn update_enemy_hp_bars(
    enemy_q: Query<(&Enemy, &Children)>,
    mut bar_q: Query<(&mut Sprite, &mut Transform), With<EnemyHpBarFg>>,
) {
    for (enemy, children) in &enemy_q {
        let ratio = (enemy.hp / enemy.max_hp).clamp(0.0, 1.0);
        let full_w = enemy.sprite_size() * HP_BAR_WIDTH_SCALE;
        for &child in children {
            let Ok((mut sprite, mut t)) = bar_q.get_mut(child) else {
                continue;
            };
            let new_w = full_w * ratio;
            sprite.custom_size = Some(Vec2::new(new_w, BAR_H));
            t.translation.x = -full_w * 0.5 + new_w * 0.5;
            sprite.color = hp_color(ratio);
        }
    }
}

fn hp_color(ratio: f32) -> Color {
    if ratio > 0.5 {
        // green → yellow
        let blend = (1.0 - ratio) * 2.0;
        Color::srgb(blend, 1.0, 0.0)
    } else {
        // yellow → red
        let blend = ratio * 2.0;
        Color::srgb(1.0, blend, 0.0)
    }
}

/// Pre-load the shared enemy atlas layout and all enemy textures.
pub fn setup_enemy_assets(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut enemy_assets: ResMut<EnemyAssets>,
) {
    enemy_assets.upgrade_layout = Some(layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(ENEMY_ATLAS_TILE),
        ENEMY_ATLAS_COLS,
        ENEMY_ATLAS_ROWS,
        None,
        None,
    )));
    enemy_assets.frontrunner_upgrades =
        Some(asset_server.load("enemies/enemy_frontrunner_upgrades.png"));
    enemy_assets.backrunner_upgrades =
        Some(asset_server.load("enemies/enemy_backrunner_upgrades.png"));
    enemy_assets.sandwich_upgrades = Some(asset_server.load("enemies/enemy_sandwich_upgrades.png"));
    enemy_assets.jitlp_upgrades = Some(asset_server.load("enemies/enemy_jitlp_upgrades.png"));
}

/// Spawning rule: active_enemies ≤ wave_target at all times.
/// Every 15 s a new block raises the target. When enemies die, new ones fill the gap.
pub fn tick_waves(
    mut commands: Commands,
    mut waves: ResMut<WaveManager>,
    mut network: ResMut<NetworkLoad>,
    enemy_assets: Res<EnemyAssets>,
    enemy_q: Query<&Enemy>,
    time: Res<Time>,
) {
    let delta = time.delta();

    // Advance wave on block boundary
    if !waves.first_block_done {
        waves.first_block_timer.tick(delta);
        if waves.first_block_timer.just_finished() {
            waves.first_block_done = true;
            waves.next_wave();
            network.tick_block(waves.wave);
            waves.block_timer.reset();
        }
    } else {
        waves.block_timer.tick(delta);
        if waves.block_timer.just_finished() {
            waves.next_wave();
            network.tick_block(waves.wave);
        }
    }

    // Fill up to wave_target; spawn batch grows every 8 waves
    if waves.wave_target == 0 {
        return;
    }
    waves.spawn_timer.tick(delta);
    if waves.spawn_timer.just_finished() {
        let per_tick = 1 + waves.wave / 8;
        let active = enemy_q.iter().count() as u32;
        let to_spawn = (waves.wave_target.saturating_sub(active)).min(per_tick);
        for _ in 0..to_spawn {
            let (enemy_type, level) = waves.pick_spawn();
            let pos = waves.rand_spawn_pos();
            spawn_enemy(&mut commands, &enemy_assets, enemy_type, level, pos);
        }
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    enemy_assets: &EnemyAssets,
    enemy_type: EnemyType,
    level: u8,
    pos: Vec2,
) {
    let enemy = Enemy::new_leveled(enemy_type.clone(), level);
    let size = enemy.sprite_size();
    let bar_w = size * HP_BAR_WIDTH_SCALE;
    let bar_y = -(size * 0.5 + HP_BAR_Y_GAP);
    let (Some(layout), Some(image)) = (
        enemy_assets.upgrade_layout.clone(),
        enemy_assets.upgrade_texture(&enemy_type),
    ) else {
        return;
    };
    let anim_base = level as usize * ENEMY_ANIM_FRAMES;
    commands
        .spawn((
            Sprite {
                image,
                texture_atlas: Some(TextureAtlas {
                    layout,
                    index: anim_base,
                }),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 1.5),
            enemy,
            AnimationTimer::new_with_offset(ENEMY_ANIM_FPS, ENEMY_ANIM_FRAMES, anim_base),
            Name::new(format!("{enemy_type:?} Lv{}", level + 1)),
        ))
        .with_children(|p| {
            p.spawn((
                Sprite {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.7),
                    custom_size: Some(Vec2::new(bar_w, BAR_H)),
                    ..default()
                },
                Transform::from_xyz(0.0, bar_y, 0.1),
            ));
            p.spawn((
                Sprite {
                    color: Color::srgb(0.2, 1.0, 0.0),
                    custom_size: Some(Vec2::new(bar_w, BAR_H)),
                    ..default()
                },
                Transform::from_xyz(0.0, bar_y, 0.2),
                EnemyHpBarFg,
            ));
        });
}
