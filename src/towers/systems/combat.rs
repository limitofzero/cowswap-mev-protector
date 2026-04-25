use bevy::prelude::*;

use crate::{
    enemies::components::Enemy,
    transactions::{Transaction, components::ImmunitySource},
};

use super::super::components::{AnimationTimer, HitEffect, Projectile, Tower, TowerType};
use super::super::resources::TowerAssets;

const COW_IMMUNITY_SECS: f32 = 6.0;
const DARKPOOL_IMMUNITY_SECS: f32 = 4.0;
const SLIPPAGE_SLOW_DURATION: f32 = 3.0;
const PROJECTILE_SPEED: f32 = 280.0;
const PROJECTILE_SIZE: f32 = 24.0;
const PROJECTILE_HIT_RADIUS: f32 = 8.0;
const HIT_EFFECT_SIZE: f32 = 48.0;

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
                let mut granted = 0u32;
                for (mut tx, tx_t) in tx_query.iter_mut() {
                    if tower_pos.distance(tx_t.translation.truncate()) <= range {
                        tx.grant_immunity(COW_IMMUNITY_SECS, ImmunitySource::CoWMatch);
                        granted += 1;
                        if granted >= 2 {
                            break;
                        }
                    }
                }
                if granted == 0 {
                    continue;
                }
            }
            TowerType::BatchAuctioneer => {
                let batch_size = tx_query
                    .iter()
                    .filter(|(_, transform)| tower_pos.distance(transform.translation.truncate()) <= range)
                    .count() as u32;
                if batch_size == 0 {
                    continue;
                }
                let mut batch_idx = 0u32;
                for (mut tx, tx_t) in tx_query.iter_mut() {
                    if tower_pos.distance(tx_t.translation.truncate()) <= range {
                        tx.set_batch(batch_idx, batch_size);
                        batch_idx += 1;
                    }
                }
            }
            TowerType::DarkPoolNode => {
                for (mut tx, tx_t) in tx_query.iter_mut() {
                    if tower_pos.distance(tx_t.translation.truncate()) <= range {
                        tx.grant_immunity(DARKPOOL_IMMUNITY_SECS, ImmunitySource::DarkPool);
                    }
                }
            }
            TowerType::Solver => {
                let target = enemy_query
                    .iter()
                    .filter(|(_, _, transform)| tower_pos.distance(transform.translation.truncate()) <= range)
                    .min_by(|lhs, rhs| {
                        tower_pos
                            .distance(lhs.2.translation.truncate())
                            .partial_cmp(&tower_pos.distance(rhs.2.translation.truncate()))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(entity, _, _)| entity);

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
                            custom_size: Some(Vec2::splat(PROJECTILE_SIZE)),
                            ..default()
                        },
                        Transform::from_xyz(tower_pos.x, tower_pos.y, 5.0),
                        Projectile {
                            target: target_entity,
                            speed: PROJECTILE_SPEED,
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
                        enemy.apply_slow(SLIPPAGE_SLOW_DURATION);
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

        if dist < PROJECTILE_HIT_RADIUS {
            enemy.hp = (enemy.hp - proj.damage).max(0.0);
            if let (Some(sheet), Some(layout)) = (
                tower_assets.hit_sheet.clone(),
                tower_assets.hit_layout.clone(),
            ) {
                commands.spawn((
                    Sprite {
                        image: sheet,
                        texture_atlas: Some(TextureAtlas { layout, index: 0 }),
                        custom_size: Some(Vec2::splat(HIT_EFFECT_SIZE)),
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
