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

fn setup_scene(mut commands: Commands, path: Res<MempoolPath>) {
    // Camera
    commands.spawn((Camera2d, Name::new("MainCamera")));

    // Draw smooth curved path by sampling the Catmull-Rom spline
    let steps = 60;
    let path_color = Color::srgba(0.10, 0.25, 0.45, 0.70);
    let path_width = 44.0_f32;

    for i in 0..steps {
        let t0 = i as f32 / steps as f32;
        let t1 = (i + 1) as f32 / steps as f32;
        let start = path.position_at(t0);
        let end = path.position_at(t1);
        let center = (start + end) * 0.5;
        let diff = end - start;
        let length = diff.length();
        let angle = diff.y.atan2(diff.x);

        // Segment
        commands.spawn((
            Sprite {
                color: path_color,
                custom_size: Some(Vec2::new(length + 1.0, path_width)),
                ..default()
            },
            Transform::from_xyz(center.x, center.y, -1.0)
                .with_rotation(Quat::from_rotation_z(angle)),
            PathSegment { index: i, total: steps },
            Name::new("PathSegment"),
        ));

        // Square cap at joint to fill the seam between segments
        commands.spawn((
            Sprite {
                color: path_color,
                custom_size: Some(Vec2::splat(path_width)),
                ..default()
            },
            Transform::from_xyz(end.x, end.y, -1.0),
            PathSegment { index: i, total: steps },
            Name::new("PathCap"),
        ));
    }

    // Draw waypoint nodes
    for (i, &wp) in path.waypoints.iter().enumerate() {
        let (color, size) = if i == 0 {
            (Color::srgb(0.20, 0.85, 0.35), 18.0) // spawn — green
        } else if i == path.waypoints.len() - 1 {
            (Color::srgb(0.95, 0.85, 0.15), 22.0) // settlement — gold
        } else {
            (Color::srgb(0.20, 0.45, 0.75), 12.0) // junction — blue
        };

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(wp.x, wp.y, 0.1),
            Name::new(format!("Waypoint{i}")),
        ));
    }

    // First cap at path start
    let start = path.position_at(0.0);
    commands.spawn((
        Sprite {
            color: path_color,
            custom_size: Some(Vec2::splat(path_width)),
            ..default()
        },
        Transform::from_xyz(start.x, start.y, -1.0),
        PathSegment { index: 0, total: steps },
        Name::new("PathCap"),
    ));

    // Settlement zone label area (large translucent rect on the right)
    let last = *path.waypoints.last().unwrap();
    commands.spawn((
        Sprite {
            color: Color::srgba(0.95, 0.85, 0.15, 0.08),
            custom_size: Some(Vec2::new(100.0, 200.0)),
            ..default()
        },
        Transform::from_xyz(last.x, last.y, -0.5),
        Name::new("SettlementZone"),
    ));
}

fn animate_path(time: Res<Time>, mut query: Query<(&PathSegment, &mut Sprite)>) {
    let t = time.elapsed_secs();
    for (seg, mut sprite) in &mut query {
        let phase = seg.index as f32 / seg.total as f32;
        // Wave travels left→right along the path
        let wave = ((t * 1.5 - phase * std::f32::consts::TAU * 0.8).sin() * 0.5 + 0.5) as f32;
        sprite.color = Color::srgba(
            0.08 + 0.12 * wave,
            0.20 + 0.20 * wave,
            0.45 + 0.25 * wave,
            0.55 + 0.25 * wave,
        );
    }
}
