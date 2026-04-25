use bevy::prelude::*;

use super::components::ImmunitySource;
use crate::{
    enemies::components::Enemy,
    mempool::MempoolPath,
    resources::{GameEconomy, GameScore, NetworkLoad},
    towers::AnimationTimer,
};

use super::{components::Transaction, resources::TxSpawner};

static TX_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn setup_tx_spawner(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut spawner: ResMut<TxSpawner>,
) {
    spawner.layout = Some(layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(80, 88),
        8,
        1,
        None,
        None,
    )));
    spawner.textures = super::components::TokenType::ALL
        .iter()
        .map(|t| asset_server.load(t.sprite_path()))
        .collect();
    spawner.fx_layout = Some(layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(80, 88),
        6,
        1,
        None,
        None,
    )));
    spawner.fx_cow = Some(asset_server.load("effects/fx_cow.png"));
    spawner.fx_batch = Some(asset_server.load("effects/fx_batch.png"));
    spawner.fx_darkpool = Some(asset_server.load("effects/fx_darkpool.png"));
}

/// Adjusts the tx spawn interval whenever the network load level changes.
pub fn sync_tx_spawn_rate(network: Res<NetworkLoad>, mut spawner: ResMut<TxSpawner>) {
    if !network.is_changed() {
        return;
    }
    spawner.timer = Timer::from_seconds(network.spawn_interval(), TimerMode::Repeating);
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

    commands
        .spawn((
            Sprite {
                image: texture,
                texture_atlas: Some(TextureAtlas {
                    layout,
                    index: start_frame,
                }),
                custom_size: Some(Vec2::new(40.0, 48.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 1.0),
            Transaction::new(value_cow, speed),
            token,
            AnimationTimer::new(5.0, 4),
            Name::new(format!("Tx{id}")),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2d::new(label),
                TextFont {
                    font_size: 8.0,
                    ..default()
                },
                TextColor(label_color),
                Transform::from_xyz(0.0, 32.0, 0.1),
                super::components::TxAmountLabel,
            ));
            // Fx effect sprite — sits behind the tx sprite, animated, hidden until an effect is active
            parent.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(80.0, 88.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -0.1),
                Visibility::Hidden,
                super::components::TxHighlight,
                AnimationTimer::new(8.0, 6),
            ));
        });
}

/// Keep the child Text2d label in sync with the tx's current remaining value.
pub fn update_tx_labels(
    tx_query: Query<(&Transaction, &super::components::TokenType, &Children)>,
    mut text_query: Query<&mut Text2d, With<super::components::TxAmountLabel>>,
) {
    for (tx, token, children) in &tx_query {
        let native = tx.remaining_value / token.cow_rate();
        let label = format_label(native, token.symbol());
        for &child in children {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = label.clone();
            }
        }
    }
}

/// Show an animated fx sprite behind txs that have an active effect.
pub fn update_tx_highlight(
    tx_query: Query<(&Transaction, &Children)>,
    mut highlight_q: Query<(&mut Sprite, &mut Visibility), With<super::components::TxHighlight>>,
    spawner: Res<TxSpawner>,
) {
    let (Some(layout), Some(fx_cow), Some(fx_batch), Some(fx_darkpool)) = (
        spawner.fx_layout.clone(),
        spawner.fx_cow.clone(),
        spawner.fx_batch.clone(),
        spawner.fx_darkpool.clone(),
    ) else {
        return;
    };

    for (tx, children) in &tx_query {
        let fx_image = fx_image_for(tx, &fx_cow, &fx_batch, &fx_darkpool);
        for &child in children {
            let Ok((mut sprite, mut vis)) = highlight_q.get_mut(child) else {
                continue;
            };
            match fx_image.clone() {
                Some(img) => {
                    sprite.image = img;
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: layout.clone(),
                        index: sprite.texture_atlas.as_ref().map_or(0, |a| a.index),
                    });
                    *vis = Visibility::Visible;
                }
                None => {
                    *vis = Visibility::Hidden;
                }
            }
        }
    }
}

fn fx_image_for(
    tx: &super::components::Transaction,
    fx_cow: &Handle<Image>,
    fx_batch: &Handle<Image>,
    fx_darkpool: &Handle<Image>,
) -> Option<Handle<Image>> {
    if let Some((_, source)) = &tx.immunity {
        return Some(match source {
            ImmunitySource::CoWMatch => fx_cow.clone(),
            ImmunitySource::DarkPool => fx_darkpool.clone(),
        });
    }
    if let Some((_, size)) = tx.batch
        && size > 1
    {
        return Some(fx_batch.clone());
    }
    None
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

/// Update each transaction's sprite to reflect its current status.
///
/// Frame mapping (frames 4-7 are status overlays):
///   4 = safe/idle  (unused here — covered by the 0-3 animation loop)
///   5 = at risk    (targeted by an enemy, but not yet immune or extracted)
///   6 = immune     (protected by a tower)
///   7 = extracted  (enemy is actively draining value)
pub fn update_tx_sprites(
    mut tx_query: Query<(Entity, &Transaction, &mut Sprite)>,
    enemy_query: Query<&Enemy>,
) {
    let targeted: std::collections::HashSet<Entity> =
        enemy_query.iter().filter_map(|e| e.target).collect();

    for (entity, tx, mut sprite) in &mut tx_query {
        let Some(atlas) = &mut sprite.texture_atlas else {
            continue;
        };

        let status_frame = if targeted.contains(&entity) && tx.value_extracted() > 0.0 {
            Some(7) // actively being drained
        } else if tx.is_immune() {
            Some(6) // protected
        } else if targeted.contains(&entity) {
            Some(5) // locked on by enemy
        } else {
            None // let the animation loop run normally (0-3)
        };

        if let Some(frame) = status_frame {
            atlas.index = frame;
        } else if atlas.index >= 4 {
            // status cleared — snap back into the animation strip
            atlas.index = 0;
        }
    }
}

pub fn move_transactions(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transaction, &mut Transform)>,
    path: Res<MempoolPath>,
    mut score: ResMut<GameScore>,
    mut economy: ResMut<GameEconomy>,
    network: Res<NetworkLoad>,
    time: Res<Time>,
) {
    let speed_mult = network.speed_mult();
    for (entity, mut tx, mut transform) in &mut query {
        tx.progress += tx.speed * speed_mult * time.delta_secs();
        tx.tick_immunity(time.delta());

        if tx.is_worthless() {
            score.value_extracted += tx.value;
            commands.entity(entity).despawn_related::<Children>();
            commands.entity(entity).despawn();
            continue;
        }

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
