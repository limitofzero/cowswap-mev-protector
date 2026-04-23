use bevy::{prelude::*, sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d}};

use crate::transactions::{components::ImmunitySource, Transaction};

use super::components::{AnimationTimer, Tower, TowerType};

/// Tick every tower's cooldown and apply its effect when it fires.
pub fn tick_towers(
    mut tower_query: Query<(&mut Tower, &Transform)>,
    mut tx_query: Query<(&mut Transaction, &Transform)>,
    time: Res<Time>,
) {
    for (mut tower, tower_transform) in &mut tower_query {
        tower.cooldown.tick(time.delta());
        if !tower.cooldown.just_finished() {
            continue;
        }

        let tower_pos = tower_transform.translation.truncate();
        let range = tower.range;

        let in_range: Vec<usize> = tx_query
            .iter()
            .enumerate()
            .filter(|(_, (_, tx_t))| tower_pos.distance(tx_t.translation.truncate()) <= range)
            .map(|(i, _)| i)
            .collect();

        if in_range.is_empty() { continue; }

        // Collect mutable refs by iterating again — safe since we don't hold borrows
        match &tower.tower_type {
            TowerType::CoWMatcher => {
                for (mut tx, _) in tx_query.iter_mut().take(2) {
                    tx.grant_immunity(6.0, ImmunitySource::CoWMatch);
                }
            }
            TowerType::BatchAuctioneer => {
                let batch_size = in_range.len() as u32;
                for (i, (mut tx, _)) in tx_query.iter_mut().enumerate() {
                    if in_range.contains(&i) {
                        tx.set_batch(i as u32, batch_size);
                    }
                }
            }
            TowerType::DarkPoolNode => {
                for (mut tx, tx_t) in tx_query.iter_mut() {
                    if tower_pos.distance(tx_t.translation.truncate()) <= range {
                        tx.grant_immunity(4.0, ImmunitySource::DarkPool);
                    }
                }
            }
            TowerType::CommitRevealBeacon => {
                for (mut tx, tx_t) in tx_query.iter_mut() {
                    if tower_pos.distance(tx_t.translation.truncate()) <= range {
                        tx.grant_immunity(3.0, ImmunitySource::CommitReveal);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Advance sprite animation frames for all animated entities.
pub fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite)>,
) {
    for (mut anim, mut sprite) in &mut query {
        anim.timer.tick(time.delta());
        if anim.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                let local = atlas.index.saturating_sub(anim.base);
                atlas.index = anim.base + (local + 1) % anim.frames;
            }
        }
    }
}


/// Spawn a starter set of towers for the demo scene.
pub fn spawn_initial_towers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let layout: &[(TowerType, Vec2)] = &[
        (TowerType::BatchAuctioneer,    Vec2::new(-300.0, -100.0)),
        (TowerType::CoWMatcher,         Vec2::new(  0.0,  140.0)),
        (TowerType::DarkPoolNode,       Vec2::new(250.0, -130.0)),
        (TowerType::Solver,             Vec2::new(-150.0,  170.0)),
        (TowerType::SlippageGuard,      Vec2::new( 180.0,  120.0)),
    ];

    let texture_layout = TextureAtlasLayout::from_grid(UVec2::new(84, 122), 6, 1, None, None);
    let layout_handle = layouts.add(texture_layout);

    for (tower_type, pos) in layout {
        let color = tower_type.color();
        let range = tower_type.range();

        let c = color.to_srgba();
        // Fill — very transparent
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(range).mesh().resolution(128))),
            MeshMaterial2d(materials.add(ColorMaterial {
                color: Color::srgba(c.red, c.green, c.blue, 0.04),
                alpha_mode: AlphaMode2d::Blend,
                ..default()
            })),
            Transform::from_xyz(pos.x, pos.y, 0.1),
            Name::new("TowerRangeFill"),
        ));
        // Border ring — more visible
        commands.spawn((
            Mesh2d(meshes.add(Annulus::new(range - 0.75, range + 0.75).mesh().resolution(128))),
            MeshMaterial2d(materials.add(ColorMaterial {
                color: Color::srgba(c.red, c.green, c.blue, 0.55),
                alpha_mode: AlphaMode2d::Blend,
                ..default()
            })),
            Transform::from_xyz(pos.x, pos.y, 0.15),
            Name::new("TowerRangeBorder"),
        ));

        let sprite = if let Some(image_path) = tower_type.sprite_path() {
            let texture = asset_server.load(image_path);
            
            Sprite {
                image: texture,
                texture_atlas: Some(TextureAtlas { layout: layout_handle.clone(), index: 0 }),
                custom_size: Some(Vec2::new(84.0, 122.0)),
                ..default()
            }
        } else {
            Sprite {
                color,
                custom_size: Some(Vec2::splat(26.0)),
                ..default()
            }
        };

        // Tower body
        let mut entity = commands.spawn((
            sprite,
            Transform::from_xyz(pos.x, pos.y, 10.0),
            Tower::new(tower_type.clone()),
            Name::new(format!("Tower::{}", tower_type.label())),
        ));
        // Only add animation if this tower has a spritesheet
        if tower_type.sprite_path().is_some() {
            entity.insert(AnimationTimer::new(3.0, 6));
        }
    }
}
