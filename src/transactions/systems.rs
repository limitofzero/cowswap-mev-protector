use bevy::prelude::*;

use crate::{mempool::MempoolPath, resources::GameScore, towers::AnimationTimer};

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

/// Tint transactions based on remaining value — only for non-sprite transactions.
pub fn tint_transactions(mut query: Query<(&Transaction, &mut Sprite)>) {
    for (tx, mut sprite) in &mut query {
        if sprite.texture_atlas.is_some() {
            // Sprite transactions: stay white (full color) unless handled by tint_shielded
            sprite.color = Color::WHITE;
            continue;
        }
        let ratio = if tx.value > 0.0 { tx.remaining_value / tx.value } else { 0.0 };
        sprite.color = Color::srgb(0.9, 0.65 * ratio + 0.05, 0.05);
    }
}

const TOKEN_SPRITES: [&str; 6] = [
    "tx_eth.png", "tx_usdt.png", "tx_usdc.png",
    "tx_cow.png", "tx_dai.png",  "tx_wbtc.png",
];

/// Spawn the initial set of test transactions at the path origin.
pub fn spawn_initial_transactions(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    path: Res<MempoolPath>,
) {
    let start = path.position_at(0.0);
    // Each tx_*.png is a single-row spritesheet: 8 frames × 80×88px
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(80, 88), 8, 1, None, None,
    ));

    let txs = [
        (1.5_f32, 0.06_f32),
        (0.8, 0.08),
        (3.0, 0.04),
        (1.0, 0.07),
        (2.2, 0.05),
    ];

    for (i, (value, speed)) in txs.iter().enumerate() {
        let sprite_path = TOKEN_SPRITES[i % TOKEN_SPRITES.len()];
        let texture = asset_server.load(sprite_path);
        let offset_y = (i as f32 - 2.0) * 8.0;
        commands.spawn((
            Sprite {
                image: texture,
                texture_atlas: Some(TextureAtlas { layout: layout.clone(), index: 0 }),
                custom_size: Some(Vec2::new(40.0, 48.0)),
                ..default()
            },
            Transform::from_xyz(start.x, start.y + offset_y, 1.0),
            Transaction::new(*value, *speed),
            AnimationTimer::new(5.0, 8),
            Name::new(format!("Tx{i}")),
        ));
    }
}
