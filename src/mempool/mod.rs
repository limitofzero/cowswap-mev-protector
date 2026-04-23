use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::game::GameState;

pub mod path;
pub use path::MempoolPath;

#[derive(Component)]
struct PathSegment {
    index: usize,
    total: usize,
}

pub struct MempoolPlugin;

impl Plugin for MempoolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MempoolPath>()
            .add_systems(OnEnter(GameState::Playing), setup_scene)
            .add_systems(Update, animate_path.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_segment(
    commands: &mut Commands,
    center: Vec2,
    length: f32,
    width: f32,
    color: Color,
    angle: f32,
    z: f32,
    seg: PathSegment,
    name: &'static str,
) {
    commands.spawn((
        Sprite { color, custom_size: Some(Vec2::new(length + 1.0, width)), ..default() },
        Transform::from_xyz(center.x, center.y, z).with_rotation(Quat::from_rotation_z(angle)),
        seg,
        Name::new(name),
    ));
}

fn spawn_cap(commands: &mut Commands, pos: Vec2, size: f32, color: Color, z: f32, seg: PathSegment, name: &'static str) {
    commands.spawn((
        Sprite { color, custom_size: Some(Vec2::splat(size)), ..default() },
        Transform::from_xyz(pos.x, pos.y, z),
        seg,
        Name::new(name),
    ));
}

fn setup_scene(mut commands: Commands, path: Res<MempoolPath>) {
    commands.spawn((Camera2d, Name::new("MainCamera")));

    let steps = 80;

    // Layer widths (back → front)
    let glow_w  = 54.0_f32; // outer soft glow
    let border_w = 46.0_f32; // bright neon border
    let fill_w   = 36.0_f32; // dark fill center

    for i in 0..steps {
        let t0 = i as f32 / steps as f32;
        let t1 = (i + 1) as f32 / steps as f32;
        let start = path.position_at(t0);
        let end   = path.position_at(t1);
        let mid   = (start + end) * 0.5;
        let diff  = end - start;
        let len   = diff.length();
        let angle = diff.y.atan2(diff.x);

        // Glow (animated)
        spawn_segment(&mut commands, mid, len, glow_w,
            Color::srgba(0.0, 0.5, 0.8, 0.0), // starts invisible, animate fills it
            angle, -3.0, PathSegment { index: i, total: steps }, "PathGlow");
        spawn_cap(&mut commands, end, glow_w,
            Color::srgba(0.0, 0.5, 0.8, 0.0),
            -3.0, PathSegment { index: i, total: steps }, "PathGlowCap");

        // Neon border (animated)
        spawn_segment(&mut commands, mid, len, border_w,
            Color::srgba(0.0, 0.7, 1.0, 0.0),
            angle, -2.0, PathSegment { index: i, total: steps }, "PathBorder");
        spawn_cap(&mut commands, end, border_w,
            Color::srgba(0.0, 0.7, 1.0, 0.0),
            -2.0, PathSegment { index: i, total: steps }, "PathBorderCap");

        // Dark fill — static, no animation
        commands.spawn((
            Sprite {
                color: Color::srgba(0.03, 0.08, 0.18, 0.95),
                custom_size: Some(Vec2::new(len + 1.0, fill_w)),
                ..default()
            },
            Transform::from_xyz(mid.x, mid.y, -1.0).with_rotation(Quat::from_rotation_z(angle)),
            Name::new("PathFill"),
        ));
        commands.spawn((
            Sprite {
                color: Color::srgba(0.03, 0.08, 0.18, 0.95),
                custom_size: Some(Vec2::splat(fill_w)),
                ..default()
            },
            Transform::from_xyz(end.x, end.y, -1.0),
            Name::new("PathFillCap"),
        ));
    }

    // Dots along the path
    let dot_count = 20;
    for i in 0..=dot_count {
        let t = i as f32 / dot_count as f32;
        let pos = path.position_at(t);
        commands.spawn((
            Sprite {
                color: Color::srgba(0.20, 0.70, 1.0, 0.5),
                custom_size: Some(Vec2::splat(5.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, -0.5),
            Name::new("PathDot"),
        ));
    }

    // Start / end markers
    let start_pos = path.position_at(0.0);
    let end_pos   = path.position_at(1.0);
    for (pos, color) in [(start_pos, Color::srgb(0.20, 0.95, 0.45)),
                          (end_pos,   Color::srgb(0.95, 0.80, 0.10))] {
        commands.spawn((
            Sprite { color, custom_size: Some(Vec2::splat(20.0)), ..default() },
            Transform::from_xyz(pos.x, pos.y, 0.1),
            Name::new("PathMarker"),
        ));
    }

    // Settlement zone overlay
    commands.spawn((
        Sprite {
            color: Color::srgba(0.95, 0.85, 0.15, 0.06),
            custom_size: Some(Vec2::new(100.0, 200.0)),
            ..default()
        },
        Transform::from_xyz(end_pos.x, end_pos.y, -0.5),
        Name::new("SettlementZone"),
    ));
}

fn animate_path(time: Res<Time>, mut query: Query<(&PathSegment, &mut Sprite, &Name)>) {
    let t = time.elapsed_secs();
    for (seg, mut sprite, name) in &mut query {
        let phase = seg.index as f32 / seg.total as f32;
        let wave = (t * 2.0 - phase * TAU * 0.7).sin() * 0.5 + 0.5;

        let name = name.as_str();
        if name == "PathGlow" || name == "PathGlowCap" {
            sprite.color = Color::srgba(0.0, 0.4 + 0.3 * wave, 0.7 + 0.2 * wave, 0.08 + 0.10 * wave);
        } else if name == "PathBorder" || name == "PathBorderCap" {
            sprite.color = Color::srgba(0.05, 0.55 + 0.35 * wave, 0.85 + 0.15 * wave, 0.55 + 0.30 * wave);
        }
    }
}
