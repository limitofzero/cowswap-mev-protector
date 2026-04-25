use crate::towers::AnimationTimer;
use bevy::prelude::*;
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};

use crate::{
    game::GameState,
    resources::{NetworkLoad, not_paused},
};

pub mod resources;
pub use resources::MempoolPath;

// Colors per network load level: [free, busy, very busy]
const PATH_BASE_COLORS: [Color; 3] = [
    Color::srgb(0.098, 0.063, 0.314), // free — dark purple
    Color::srgb(0.22, 0.09, 0.06),    // busy — dark amber
    Color::srgb(0.32, 0.04, 0.04),    // high — dark red
];
const PATH_SHIM_COLORS: [Color; 3] = [
    Color::srgb(0.051, 0.188, 0.376), // free — dark blue
    Color::srgb(0.30, 0.13, 0.04),    // busy — amber
    Color::srgb(0.42, 0.06, 0.05),    // high — red
];
const DOT_COLORS: [Color; 3] = [
    Color::srgba(0.176, 0.376, 0.753, 0.55), // free — blue
    Color::srgba(0.80, 0.45, 0.10, 0.65),    // busy — orange
    Color::srgba(0.90, 0.15, 0.12, 0.75),    // high — red
];

const BASE_W: f32 = 46.0;
const SHIM_W: f32 = 36.0;

#[derive(Component)]
struct MempoolDot {
    progress: f32,
    speed: f32,
}
#[derive(Component)]
struct MempoolPathBase;
#[derive(Component)]
struct MempoolPathShim;

/// Stores the shared dot material handle so we can recolor all dots at once.
#[derive(Resource, Default)]
struct MempoolDotMat(Option<Handle<ColorMaterial>>);

pub struct MempoolPlugin;

impl Plugin for MempoolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MempoolPath>()
            .init_resource::<MempoolDotMat>()
            .add_systems(OnEnter(GameState::Playing), setup_scene)
            .add_systems(
                Update,
                (
                    animate_dots.run_if(in_state(GameState::Playing).and(not_paused)),
                    update_mempool_colors.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

fn setup_scene(
    mut commands: Commands,
    path: Res<MempoolPath>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut dot_mat_res: ResMut<MempoolDotMat>,
) {
    let waypoints = &path.waypoints;

    // --- Static path: 2 layers per segment + circle caps at each joint ---
    let base_cap = meshes.add(Circle::new(BASE_W * 0.5));
    let shim_cap = meshes.add(Circle::new(SHIM_W * 0.5));

    for seg_idx in 0..waypoints.len() - 1 {
        let seg_start = waypoints[seg_idx];
        let seg_end = waypoints[seg_idx + 1];
        let mid = (seg_start + seg_end) * 0.5;
        let len = seg_start.distance(seg_end);
        let angle = (seg_end - seg_start).to_angle();
        let rot = Quat::from_rotation_z(angle);

        // Base layer
        commands.spawn((
            Sprite {
                color: PATH_BASE_COLORS[0],
                custom_size: Some(Vec2::new(len, BASE_W)),
                ..default()
            },
            Transform::from_xyz(mid.x, mid.y, -3.0).with_rotation(rot),
            MempoolPathBase,
            Name::new("PathBase"),
        ));
        // Shim layer
        commands.spawn((
            Sprite {
                color: PATH_SHIM_COLORS[0],
                custom_size: Some(Vec2::new(len, SHIM_W)),
                ..default()
            },
            Transform::from_xyz(mid.x, mid.y, -2.0).with_rotation(rot),
            MempoolPathShim,
            Name::new("PathShim"),
        ));
    }

    // Circle caps at every waypoint so joins are round
    for &pt in waypoints {
        commands.spawn((
            Mesh2d(base_cap.clone()),
            MeshMaterial2d(materials.add(ColorMaterial {
                color: PATH_BASE_COLORS[0],
                ..default()
            })),
            Transform::from_xyz(pt.x, pt.y, -3.0),
        ));
        commands.spawn((
            Mesh2d(shim_cap.clone()),
            MeshMaterial2d(materials.add(ColorMaterial {
                color: PATH_SHIM_COLORS[0],
                ..default()
            })),
            Transform::from_xyz(pt.x, pt.y, -2.0),
        ));
    }

    // --- Moving glowing dots (8 total, evenly staggered) ---
    let dot_mesh = meshes.add(Circle::new(3.5));
    let dot_mat_handle = materials.add(ColorMaterial {
        color: DOT_COLORS[0],
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    });
    dot_mat_res.0 = Some(dot_mat_handle.clone());
    let dot_mat = dot_mat_handle;
    for dot_idx in 0..8 {
        let progress = dot_idx as f32 / 8.0;
        let pos = path.position_at(progress);
        commands.spawn((
            Mesh2d(dot_mesh.clone()),
            MeshMaterial2d(dot_mat.clone()),
            Transform::from_xyz(pos.x, pos.y, -0.5),
            MempoolDot {
                progress,
                speed: 0.04,
            },
            Name::new("PathDot"),
        ));
    }

    // Portal animated sprites (6 frames, 52×52 each) at path entry and exit.
    let portal_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(52, 52),
        6,
        1,
        None,
        None,
    ));
    let entry_pos = path.position_at(0.0);
    let exit_pos = path.position_at(1.0);

    commands.spawn((
        Sprite {
            image: asset_server.load("effects/portal_mempool.png"),
            texture_atlas: Some(TextureAtlas {
                layout: portal_layout.clone(),
                index: 0,
            }),
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
            texture_atlas: Some(TextureAtlas {
                layout: portal_layout,
                index: 0,
            }),
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

fn update_mempool_colors(
    network: Res<NetworkLoad>,
    dot_mat_res: Res<MempoolDotMat>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut base_q: Query<&mut Sprite, (With<MempoolPathBase>, Without<MempoolPathShim>)>,
    mut shim_q: Query<&mut Sprite, (With<MempoolPathShim>, Without<MempoolPathBase>)>,
) {
    if !network.is_changed() {
        return;
    }
    let lv = network.level as usize;
    for mut s in &mut base_q {
        s.color = PATH_BASE_COLORS[lv];
    }
    for mut s in &mut shim_q {
        s.color = PATH_SHIM_COLORS[lv];
    }
    if let Some(handle) = &dot_mat_res.0
        && let Some(mat) = materials.get_mut(handle)
    {
        mat.color = DOT_COLORS[lv];
    }
}
