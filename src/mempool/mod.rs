use bevy::prelude::*;

use crate::game::GameState;

pub mod path;
pub use path::MempoolPath;

pub struct MempoolPlugin;

impl Plugin for MempoolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MempoolPath>()
            .add_systems(OnEnter(GameState::Playing), setup_scene);
    }
}

fn setup_scene(mut commands: Commands, path: Res<MempoolPath>) {
    // Camera
    commands.spawn((Camera2d, Name::new("MainCamera")));

    // Draw each path segment as a rotated rectangle
    for segment in path.waypoints.windows(2) {
        let start = segment[0];
        let end = segment[1];
        let center = (start + end) * 0.5;
        let diff = end - start;
        let length = diff.length();
        let angle = diff.y.atan2(diff.x);

        commands.spawn((
            Sprite {
                color: Color::srgba(0.10, 0.25, 0.45, 0.70),
                custom_size: Some(Vec2::new(length + 4.0, 44.0)),
                ..default()
            },
            Transform::from_xyz(center.x, center.y, -1.0)
                .with_rotation(Quat::from_rotation_z(angle)),
            Name::new("PathSegment"),
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
