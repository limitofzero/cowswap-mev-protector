use bevy::prelude::*;

use crate::transactions::{Batched, ImmunitySource, MevImmunity, Transaction};

use super::components::{Tower, TowerType};

/// Tick every tower's cooldown and apply its effect when it fires.
pub fn tick_towers(
    mut commands: Commands,
    mut tower_query: Query<(&mut Tower, &Transform)>,
    tx_query: Query<(Entity, &Transform), With<Transaction>>,
    time: Res<Time>,
) {
    for (mut tower, tower_transform) in &mut tower_query {
        tower.cooldown.tick(time.delta());
        if !tower.cooldown.just_finished() {
            continue;
        }

        let tower_pos = tower_transform.translation.truncate();
        let range = tower.range;

        // Collect in-range transaction entities (immutable pass — iterator consumed here)
        let in_range: Vec<Entity> = tx_query
            .iter()
            .filter(|(_, tx_t)| tower_pos.distance(tx_t.translation.truncate()) <= range)
            .map(|(e, _)| e)
            .collect();

        if in_range.is_empty() {
            continue;
        }

        match &tower.tower_type {
            TowerType::CoWMatcher => {
                // Pair the first two transactions found — grant MEV immunity to both.
                for &entity in in_range.iter().take(2) {
                    commands.entity(entity).insert(MevImmunity {
                        duration: Timer::from_seconds(6.0, TimerMode::Once),
                        source: ImmunitySource::CoWMatch,
                    });
                }
            }
            TowerType::BatchAuctioneer => {
                let batch_size = in_range.len() as u32;
                for (i, &entity) in in_range.iter().enumerate() {
                    commands.entity(entity).insert(Batched {
                        batch_id: i as u32, // simple id for now
                        batch_size,
                    });
                }
            }
            TowerType::DarkPoolNode => {
                for &entity in &in_range {
                    commands.entity(entity).insert(MevImmunity {
                        duration: Timer::from_seconds(4.0, TimerMode::Once),
                        source: ImmunitySource::DarkPool,
                    });
                }
            }
            TowerType::CommitRevealBeacon => {
                for &entity in &in_range {
                    commands.entity(entity).insert(MevImmunity {
                        duration: Timer::from_seconds(3.0, TimerMode::Once),
                        source: ImmunitySource::CommitReveal,
                    });
                }
            }
            // SlippageGuard and Solver effects will reduce sandwich/route profitability
            // — placeholder, to be implemented in the next step.
            _ => {}
        }
    }
}

/// Tint shielded transactions bright cyan so players can see the immunity.
pub fn tint_shielded_transactions(
    mut query: Query<(&mut Sprite, Option<&MevImmunity>), With<Transaction>>,
) {
    for (mut sprite, immunity) in &mut query {
        if immunity.is_some() {
            sprite.color = Color::srgb(0.2, 0.9, 0.9);
        }
        // Non-immune tinting is handled by transactions::systems::tint_transactions
    }
}

/// Spawn a starter set of towers for the demo scene.
pub fn spawn_initial_towers(mut commands: Commands) {
    let layout: &[(TowerType, Vec2)] = &[
        (TowerType::BatchAuctioneer, Vec2::new(-300.0, -100.0)),
        (TowerType::CoWMatcher, Vec2::new(0.0, 140.0)),
        (TowerType::DarkPoolNode, Vec2::new(250.0, -130.0)),
        (TowerType::SlippageGuard, Vec2::new(-120.0, -150.0)),
    ];

    for (tower_type, pos) in layout {
        let color = tower_type.color();
        let range = tower_type.range();

        // Range indicator (translucent disc approximated as a large square for now)
        commands.spawn((
            Sprite {
                color: Color::srgba(
                    color.to_srgba().red,
                    color.to_srgba().green,
                    color.to_srgba().blue,
                    0.07,
                ),
                custom_size: Some(Vec2::splat(range * 2.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 0.1),
            Name::new("TowerRange"),
        ));

        // Tower body
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(26.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 0.5),
            Tower::new(tower_type.clone()),
            Name::new(format!("Tower::{}", tower_type.label())),
        ));
    }
}
