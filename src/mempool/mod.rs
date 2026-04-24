use bevy::prelude::*;
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};
use crate::towers::AnimationTimer;

use crate::{game::GameState, resources::not_paused};

pub mod resources;
pub use resources::MempoolPath;

// Colors matching the mockup palette
const PATH_BASE:  Color = Color::srgb(0.098, 0.063, 0.314); // #1a1050
const PATH_SHIM:  Color = Color::srgb(0.051, 0.188, 0.376); // #0d3060
const DOT_COLOR:  Color = Color::srgba(0.176, 0.376, 0.753, 0.55); // #2d60c0 50%

const BASE_W: f32 = 46.0;
const SHIM_W: f32 = 36.0;

#[derive(Component)]
struct MempoolDot {
    progress: f32,
    speed: f32,
}

pub struct MempoolPlugin;

impl Plugin for MempoolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MempoolPath>()
            .add_systems(OnEnter(GameState::Playing), setup_scene)
            .add_systems(Update, animate_dots.run_if(in_state(GameState::Playing).and(not_paused)));
    }
}

fn setup_scene(
    mut commands: Commands,
    path: Res<MempoolPath>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {


    let waypoints = &path.waypoints;

    // --- Static path: 2 layers per segment + circle caps at each joint ---
    let base_cap  = meshes.add(Circle::new(BASE_W * 0.5));
    let shim_cap  = meshes.add(Circle::new(SHIM_W * 0.5));

    for i in 0..waypoints.len() - 1 {
        let a = waypoints[i];
        let b = waypoints[i + 1];
        let mid   = (a + b) * 0.5;
        let len   = a.distance(b);
        let angle = (b - a).to_angle();
        let rot   = Quat::from_rotation_z(angle);

        // Base layer
        commands.spawn((
            Sprite { color: PATH_BASE, custom_size: Some(Vec2::new(len, BASE_W)), ..default() },
            Transform::from_xyz(mid.x, mid.y, -3.0).with_rotation(rot),
            Name::new("PathBase"),
        ));
        // Shim layer
        commands.spawn((
            Sprite { color: PATH_SHIM, custom_size: Some(Vec2::new(len, SHIM_W)), ..default() },
            Transform::from_xyz(mid.x, mid.y, -2.0).with_rotation(rot),
            Name::new("PathShim"),
        ));
    }

    // Circle caps at every waypoint so joins are round
    for &pt in waypoints {
        commands.spawn((
            Mesh2d(base_cap.clone()),
            MeshMaterial2d(materials.add(ColorMaterial { color: PATH_BASE, ..default() })),
            Transform::from_xyz(pt.x, pt.y, -3.0),
        ));
        commands.spawn((
            Mesh2d(shim_cap.clone()),
            MeshMaterial2d(materials.add(ColorMaterial { color: PATH_SHIM, ..default() })),
            Transform::from_xyz(pt.x, pt.y, -2.0),
        ));
    }

    // --- Moving glowing dots (8 total, evenly staggered) ---
    let dot_mesh = meshes.add(Circle::new(3.5));
    let dot_mat  = materials.add(ColorMaterial {
        color: DOT_COLOR,
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    });
    for i in 0..8 {
        let progress = i as f32 / 8.0;
        let pos = path.position_at(progress);
        commands.spawn((
            Mesh2d(dot_mesh.clone()),
            MeshMaterial2d(dot_mat.clone()),
            Transform::from_xyz(pos.x, pos.y, -0.5),
            MempoolDot { progress, speed: 0.04 },
            Name::new("PathDot"),
        ));
    }

    // Portal animated sprites (6 frames, 52×52 each) at path entry and exit.
    let portal_layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::new(52, 52), 6, 1, None, None));
    let entry_pos = path.position_at(0.0);
    let exit_pos  = path.position_at(1.0);

    commands.spawn((
        Sprite {
            image: asset_server.load("effects/portal_mempool.png"),
            texture_atlas: Some(TextureAtlas { layout: portal_layout.clone(), index: 0 }),
            custom_size: Some(Vec2::new(52.0, 52.0)),
            ..default()
        },
        Transform::from_xyz(entry_pos.x, entry_pos.y, 2.0),
        AnimationTimer::new(8.0, 6),
        Name::new("PortalMempool"),
    ));
    commands.spawn((
        Sprite {
            image: asset_server.load("effects/portal_settlement.png"),
            texture_atlas: Some(TextureAtlas { layout: portal_layout, index: 0 }),
            custom_size: Some(Vec2::new(52.0, 52.0)),
            ..default()
        },
        Transform::from_xyz(exit_pos.x, exit_pos.y, 2.0),
        AnimationTimer::new(8.0, 6),
        Name::new("PortalSettlement"),
    ));
}

fn animate_dots(
    time: Res<Time>,
    path: Res<MempoolPath>,
    mut query: Query<(&mut MempoolDot, &mut Transform)>,
) {
    for (mut dot, mut transform) in &mut query {
        dot.progress = (dot.progress + dot.speed * time.delta_secs()) % 1.0;
        let pos = path.position_at(dot.progress);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}
