use bevy::prelude::*;

use crate::{towers::AnimationTimer, transactions::Transaction};

use super::components::{Enemy, EnemyAssets, EnemyType, WaveManager, WaveState};

/// Phase 1 — find the nearest non-immune transaction for each enemy (no range limit).
pub fn find_enemy_targets(
    mut enemy_query: Query<(&mut Enemy, &Transform)>,
    tx_query: Query<(Entity, &Transaction, &Transform)>,
) {
    for (mut enemy, enemy_transform) in &mut enemy_query {
        let pos = enemy_transform.translation.truncate();

        enemy.target = tx_query
            .iter()
            .filter_map(|(e, tx, tx_t)| {
                if tx.is_immune() { return None; }
                Some((e, pos.distance(tx_t.translation.truncate())))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(e, _)| e);
    }
}

/// Phase 2 — extract value only when within attack range.
pub fn extract_value(
    enemy_query: Query<(&Enemy, &Transform)>,
    mut tx_query: Query<(&mut Transaction, &Transform)>,
    time: Res<Time>,
) {
    for (enemy, enemy_t) in &enemy_query {
        let Some(target) = enemy.target else { continue };
        let Ok((mut tx, tx_t)) = tx_query.get_mut(target) else { continue };
        let dist = enemy_t.translation.truncate().distance(tx_t.translation.truncate());
        if dist > enemy.attack_range { continue; }
        let extracted = enemy.extract_rate * time.delta_secs();
        tx.remaining_value = (tx.remaining_value - extracted).max(0.0);
    }
}

/// Move each enemy toward its target (or patrol if it has none).
pub fn enemy_movement(
    mut enemy_query: Query<(&Enemy, &mut Transform)>,
    tx_query: Query<&Transform, (With<Transaction>, Without<Enemy>)>,
    time: Res<Time>,
) {
    for (enemy, mut transform) in &mut enemy_query {
        if let Some(target) = enemy.target {
            if let Ok(tx_t) = tx_query.get(target) {
                let dir = (tx_t.translation.truncate() - transform.translation.truncate())
                    .normalize_or_zero();
                transform.translation +=
                    (dir * enemy.effective_speed() * time.delta_secs()).extend(0.0);
            }
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
            commands.entity(entity).despawn();
        }
    }
}

/// Pre-load the shared enemy atlas layout.
pub fn setup_enemy_assets(
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut enemy_assets: ResMut<EnemyAssets>,
) {
    enemy_assets.layout = Some(
        layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(96), 6, 1, None, None))
    );
}

/// Drive the wave state machine: countdown → spawn one-by-one → wait for clear → repeat.
pub fn tick_waves(
    mut commands: Commands,
    mut waves: ResMut<WaveManager>,
    enemy_assets: Res<EnemyAssets>,
    asset_server: Res<AssetServer>,
    enemy_q: Query<&Enemy>,
    time: Res<Time>,
) {
    match waves.state {
        WaveState::Countdown => {
            waves.between_timer.tick(time.delta());
            if waves.between_timer.just_finished() {
                waves.build_wave();
                waves.state = WaveState::Spawning;
                waves.spawn_timer.reset();
            }
        }
        WaveState::Spawning => {
            waves.spawn_timer.tick(time.delta());
            if waves.spawn_timer.just_finished() {
                if let Some(enemy_type) = waves.pending.pop_front() {
                    let pos = waves.rand_spawn_pos();
                    spawn_enemy(&mut commands, &asset_server, &enemy_assets, enemy_type, pos);
                }
                if waves.pending.is_empty() {
                    waves.state = WaveState::WaitForClear;
                }
            }
        }
        WaveState::WaitForClear => {
            if enemy_q.is_empty() {
                waves.state = WaveState::Countdown;
                // Shorten inter-wave pause after the first wave
                let pause = if waves.wave >= 3 { 6.0 } else { 8.0 };
                waves.between_timer = Timer::from_seconds(pause, TimerMode::Once);
            }
        }
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    asset_server: &AssetServer,
    enemy_assets: &EnemyAssets,
    enemy_type: EnemyType,
    pos: Vec2,
) {
    let Some(layout) = enemy_assets.layout.clone() else { return };
    let size = enemy_type.size();
    commands.spawn((
        Sprite {
            image: asset_server.load(enemy_type.sprite_path()),
            texture_atlas: Some(TextureAtlas { layout, index: 0 }),
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.5),
        Enemy::new(enemy_type.clone()),
        AnimationTimer::new(3.0, 6),
        Name::new(format!("{enemy_type:?}")),
    ));
}
