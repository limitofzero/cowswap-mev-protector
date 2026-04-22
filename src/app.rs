use bevy::prelude::*;

use crate::game::GamePlugin;

pub fn run_app() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "CoW MEV Defense".into(),
                    resolution: (1280.0, 720.0).into(),
                    // Bind to the <canvas id="bevy-canvas"> element in index.html
                    #[cfg(target_arch = "wasm32")]
                    canvas: Some("#bevy-canvas".to_owned()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.10)))
        .add_plugins(GamePlugin)
        .run();
}
