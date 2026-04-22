use bevy::prelude::*;

use crate::{mempool::MempoolPath, resources::GameScore};

use super::components::{MevImmunity, Transaction};

/// Advance every transaction along the path; despawn those that reach settlement.
pub fn move_transactions(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transaction, &mut Transform)>,
    path: Res<MempoolPath>,
    mut score: ResMut<GameScore>,
    time: Res<Time>,
) {
    for (entity, mut tx, mut transform) in &mut query {
        tx.progress += tx.speed * time.delta_secs();

        if tx.progress >= 1.0 {
            score.txs_settled += 1;
            score.value_protected += tx.remaining_value;
            score.value_extracted += tx.value_extracted();
            commands.entity(entity).despawn();
            continue;
        }

        let pos = path.position_at(tx.progress);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

/// Tick immunity timers and remove expired shields.
pub fn tick_mev_immunity(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MevImmunity)>,
    time: Res<Time>,
) {
    for (entity, mut immunity) in &mut query {
        immunity.duration.tick(time.delta());
        if immunity.duration.just_finished() {
            commands.entity(entity).remove::<MevImmunity>();
        }
    }
}

/// Tint transactions based on their remaining value (yellow → red as value is drained).
pub fn tint_transactions(mut query: Query<(&Transaction, &mut Sprite)>) {
    for (tx, mut sprite) in &mut query {
        let ratio = if tx.value > 0.0 {
            tx.remaining_value / tx.value
        } else {
            0.0
        };
        // Full value: bright gold. Drained: dark red.
        sprite.color = Color::srgb(0.9, 0.65 * ratio + 0.05, 0.05);
    }
}

/// Spawn the initial set of test transactions at the path origin.
pub fn spawn_initial_transactions(mut commands: Commands, path: Res<MempoolPath>) {
    let start = path.position_at(0.0);

    let txs = [
        (1.5_f32, 0.06_f32), // (value ETH, speed)
        (0.8, 0.08),
        (3.0, 0.04),
        (1.0, 0.07),
        (2.2, 0.05),
    ];

    for (i, (value, speed)) in txs.iter().enumerate() {
        // Stagger start positions slightly so they don't all overlap
        let offset_y = (i as f32 - 2.0) * 8.0;
        commands.spawn((
            Sprite {
                color: Color::srgb(0.9, 0.65, 0.05),
                custom_size: Some(Vec2::splat(14.0)),
                ..default()
            },
            Transform::from_xyz(start.x, start.y + offset_y, 1.0),
            Transaction::new(*value, *speed),
            Name::new(format!("Tx{i}")),
        ));
    }
}
