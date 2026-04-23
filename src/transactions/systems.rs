use bevy::prelude::*;

use crate::{mempool::MempoolPath, resources::{GameEconomy, GameScore}, towers::AnimationTimer};

use super::{
    components::Transaction,
    resources::TxSpawner,
};

static TX_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn setup_tx_spawner(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut spawner: ResMut<TxSpawner>,
) {
    spawner.layout = Some(layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(80, 88), 8, 1, None, None,
    )));
    spawner.textures = super::components::TokenType::ALL
        .iter()
        .map(|t| asset_server.load(t.sprite_path()))
        .collect();
}

pub fn spawn_transactions(
    mut commands: Commands,
    path: Res<MempoolPath>,
    mut spawner: ResMut<TxSpawner>,
    time: Res<Time>,
) {
    spawner.timer.tick(time.delta());
    if !spawner.timer.just_finished() {
        return;
    }
    let (Some(layout), true) = (spawner.layout.clone(), !spawner.textures.is_empty()) else {
        return;
    };

    let (token, texture) = spawner.rand_token();
    let start_frame = spawner.rand_usize(4);
    let start_progress = spawner.rand_f32() * 0.25;
    let (min_amt, max_amt) = token.amount_range();
    let amount = min_amt + spawner.rand_f32() * (max_amt - min_amt);
    let value_cow = amount * token.cow_rate();
    let speed = 0.028 + spawner.rand_f32() * 0.008;
    let pos = path.position_at(start_progress);
    let id = TX_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let label = format_label(amount, token.symbol());
    let label_color = token.color();

    commands.spawn((
        Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas { layout, index: start_frame }),
            custom_size: Some(Vec2::new(40.0, 48.0)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
        Transaction::new(value_cow, speed),
        token,
        AnimationTimer::new(5.0, 4),
        Name::new(format!("Tx{id}")),
    )).with_children(|parent| {
        parent.spawn((
            Text2d::new(label),
            TextFont { font_size: 8.0, ..default() },
            TextColor(label_color),
            Transform::from_xyz(0.0, 32.0, 0.1),
        ));
    });
}

fn format_label(amount: f32, symbol: &str) -> String {
    if amount >= 1000.0 {
        format!("{:.0} {}", amount, symbol)
    } else if amount >= 1.0 {
        format!("{:.2} {}", amount, symbol)
    } else {
        format!("{:.4} {}", amount, symbol)
    }
}

pub fn move_transactions(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transaction, &mut Transform)>,
    path: Res<MempoolPath>,
    mut score: ResMut<GameScore>,
    mut economy: ResMut<GameEconomy>,
    time: Res<Time>,
) {
    for (entity, mut tx, mut transform) in &mut query {
        tx.progress += tx.speed * time.delta_secs();
        tx.tick_immunity(time.delta());

        if tx.progress >= 1.0 {
            let fee = tx.remaining_value * economy.fee_rate;
            economy.balance += fee;
            score.txs_settled += 1;
            score.value_protected += tx.remaining_value;
            score.value_extracted += tx.value_extracted();
            commands.entity(entity).despawn_related::<Children>();
            commands.entity(entity).despawn();
            continue;
        }

        let pos = path.position_at(tx.progress);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}
