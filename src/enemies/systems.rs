use bevy::prelude::*;

use crate::{towers::AnimationTimer, transactions::Transaction};

use super::components::{Enemy, EnemyType};

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

/// Spawn the starter enemy roster.
pub fn spawn_initial_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let roster: &[(EnemyType, Vec2)] = &[
        (EnemyType::Frontrunner, Vec2::new(-400.0,  60.0)),
        (EnemyType::Backrunner,  Vec2::new( 200.0, -100.0)),
        (EnemyType::Frontrunner, Vec2::new( -50.0, -100.0)),
    ];

    let layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(96), 6, 1, None, None));

    for (enemy_type, pos) in roster {
        let size = enemy_type.size();
        let sprite = if let Some(path) = enemy_type.sprite_path() {
            Sprite {
                image: asset_server.load(path),
                texture_atlas: Some(TextureAtlas { layout: layout.clone(), index: 0 }),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            }
        } else {
            Sprite { color: enemy_type.color(), custom_size: Some(Vec2::splat(size)), ..default() }
        };

        let mut entity = commands.spawn((
            sprite,
            Transform::from_xyz(pos.x, pos.y, 1.5),
            Enemy::new(enemy_type.clone()),
            Name::new(format!("{enemy_type:?}")),
        ));
        if enemy_type.sprite_path().is_some() {
            entity.insert(AnimationTimer::new(3.0, 6));
        }
    }
}
