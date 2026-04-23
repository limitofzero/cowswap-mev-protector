use bevy::prelude::*;

use crate::{mempool::MempoolPath, resources::GameScore, towers::AnimationTimer};

use super::{
    components::{MevImmunity, TokenType, Transaction},
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
    spawner.textures = TokenType::ALL
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
    let start_frame = spawner.rand_usize(8);
    let start_progress = spawner.rand_f32() * 0.25;
    let value = 0.5 + spawner.rand_f32() * 3.0;
    let speed = 0.04 + spawner.rand_f32() * 0.06;
    let pos = path.position_at(start_progress);
    let id = TX_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    commands.spawn((
        Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas { layout, index: start_frame }),
            custom_size: Some(Vec2::new(40.0, 48.0)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 1.0),
        Transaction::new(value, speed),
        token,
        AnimationTimer::new(5.0, 8),
        Name::new(format!("Tx{id}")),
    ));
}

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
