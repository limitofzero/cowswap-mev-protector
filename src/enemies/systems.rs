use bevy::prelude::*;

use crate::transactions::{MevImmunity, Transaction};

use super::components::{Enemy, EnemyType};

/// Phase 1 — find the nearest un-shielded transaction for each enemy.
pub fn find_enemy_targets(
    mut enemy_query: Query<(&mut Enemy, &Transform)>,
    tx_query: Query<(Entity, &Transform), (With<Transaction>, Without<MevImmunity>)>,
) {
    for (mut enemy, enemy_transform) in &mut enemy_query {
        let pos = enemy_transform.translation.truncate();
        let range = enemy.attack_range;

        enemy.target = tx_query
            .iter()
            .filter_map(|(e, tx_t)| {
                let d = pos.distance(tx_t.translation.truncate());
                if d <= range { Some((e, d)) } else { None }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(e, _)| e);
    }
}

/// Phase 2 — extract value from the targeted transaction.
pub fn extract_value(
    enemy_query: Query<&Enemy>,
    mut tx_query: Query<&mut Transaction>,
    time: Res<Time>,
) {
    for enemy in &enemy_query {
        let Some(target) = enemy.target else { continue };
        let Ok(mut tx) = tx_query.get_mut(target) else { continue };
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
                    (dir * enemy.speed * time.delta_secs()).extend(0.0);
            }
        }
    }
}

/// Spawn the starter enemy roster for testing.
pub fn spawn_initial_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let roster: &[(EnemyType, Vec2)] = &[
        (EnemyType::Frontrunner, Vec2::new(-200.0, 280.0)),
        (EnemyType::Backrunner, Vec2::new(150.0, -280.0)),
        // (EnemyType::SandwichBot, Vec2::new(-50.0, 260.0)),
        // (EnemyType::SandwichBot, Vec2::new(-50.0, -260.0)),
        // (EnemyType::Liquidator, Vec2::new(300.0, 260.0)),
        // (EnemyType::GeneralizedFrontrunner, Vec2::new(-400.0, -260.0)),
        // (EnemyType::JitLp, Vec2::new(480.0, 200.0)),
    ];

    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 1, None, None);
    let layout_handle = layouts.add(layout);


    for (enemy_type, pos) in roster {
        let color = enemy_type.color();
        let size = enemy_type.size();

        let sprite = if let Some(sprite_path) = enemy_type.sprite_path() {
            Sprite {
                image: asset_server.load(sprite_path),
                texture_atlas: Some(
                    TextureAtlas { layout: layout_handle.clone(), index: 0 }
                ),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            }
        } else {
            Sprite {
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            }
        };
        
        commands.spawn((
            sprite,
            Transform::from_xyz(pos.x, pos.y, 1.5),
            Enemy::new(enemy_type.clone()),
            Name::new(format!("{enemy_type:?}")),
        ));
    }
}
