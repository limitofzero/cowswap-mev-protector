use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};

use crate::{
    enemies::components::EnemyType, game::GameState, resources::PauseState,
    utils::make_rounded_rect,
};

pub struct MenuPlugin;

const PANEL_W: f32 = 720.0;
const PANEL_H: f32 = 480.0;
const PANEL_Z: f32 = 200.0;
const BTN_W: f32 = 180.0;
const BTN_H: f32 = 38.0;
const BTN_Y: f32 = -168.0;
const ICON_X: f32 = -330.0;
const TEXT_X: f32 = -300.0;

#[derive(Component)]
struct MenuOverlay;
#[derive(Component)]
struct MenuProceedBtn;
#[derive(Component)]
struct MenuBtnLabel;
/// Marks standalone enemy icon sprites — toggled alongside the overlay.
#[derive(Component)]
struct MenuIcon;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(
                Update,
                (update_menu_visibility, handle_menu_input, update_btn_label),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Name::new("MainCamera")));
}

fn setup_menu(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let border = meshes.add(make_rounded_rect(PANEL_W, PANEL_H, 12.0, 10));
    let fill = meshes.add(make_rounded_rect(PANEL_W - 4.0, PANEL_H - 4.0, 11.0, 10));
    let btn_border = meshes.add(make_rounded_rect(BTN_W, BTN_H, 9.0, 10));
    let btn_fill = meshes.add(make_rounded_rect(BTN_W - 4.0, BTN_H - 4.0, 8.0, 10));

    let col_border = materials.add(ColorMaterial {
        color: Color::srgba(0.50, 0.35, 0.88, 0.95),
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    });
    let col_fill = materials.add(ColorMaterial {
        color: Color::srgba(0.04, 0.02, 0.18, 0.97),
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    });
    let col_btn_bg = materials.add(ColorMaterial {
        color: Color::srgba(0.40, 0.28, 0.80, 0.95),
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    });
    let col_btn_fill = materials.add(ColorMaterial {
        color: Color::srgba(0.22, 0.12, 0.55, 0.97),
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    });

    let gray = Color::srgb(0.72, 0.72, 0.72);
    let muted = Color::srgb(0.50, 0.42, 0.70);
    let highlight = Color::srgb(0.85, 0.75, 1.00);

    let enemies: &[(EnemyType, &str, &str)] = &[
        (
            EnemyType::Frontrunner,
            "Frontrunner",
            "Spots your tx and submits one ahead of it to buy first, profiting from your price impact",
        ),
        (
            EnemyType::Backrunner,
            "Backrunner",
            "Follows your tx and harvests the price movement you already created",
        ),
        (
            EnemyType::SandwichBot,
            "Sandwich",
            "Buys before your tx and sells right after it, squeezing profit from both sides",
        ),
        (
            EnemyType::JitLp,
            "JIT LP",
            "Injects liquidity just before your swap to steal the fees, then removes it instantly",
        ),
    ];

    commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, PANEL_Z),
            Visibility::Visible,
            MenuOverlay,
            Name::new("MenuOverlay"),
        ))
        .with_children(|p| {
            // Panel border + fill
            p.spawn((
                Mesh2d(border),
                MeshMaterial2d(col_border),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
            p.spawn((
                Mesh2d(fill),
                MeshMaterial2d(col_fill),
                Transform::from_xyz(0.0, 0.0, 0.05),
            ));

            // Title
            p.spawn((
                Text2d::new("CoW MEV Defense"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(highlight),
                Transform::from_xyz(0.0, 188.0, 1.0),
            ));

            // Subtitle
            p.spawn((
                Text2d::new("Protect mempool transactions from MEV bots"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(gray),
                Transform::from_xyz(0.0, 160.0, 1.0),
            ));

            // "ENEMIES" section header
            p.spawn((
                Text2d::new("-- ENEMY TYPES --"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(muted),
                Transform::from_xyz(0.0, 148.0, 1.0),
            ));

            // Enemy rows: name (left-aligned) on upper line, description on lower line
            let row_start_y = 118.0_f32;
            let row_step = 52.0_f32;
            for (row_idx, (enemy_type, name, desc)) in enemies.iter().enumerate() {
                let row_y = row_start_y - row_idx as f32 * row_step;
                p.spawn((
                    Text2d::new(*name),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(enemy_type.color()),
                    Transform::from_xyz(TEXT_X, row_y + 9.0, 1.0),
                    Anchor::CENTER_LEFT,
                ));
                p.spawn((
                    Text2d::new(*desc),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(gray),
                    Transform::from_xyz(TEXT_X, row_y - 9.0, 1.0),
                    Anchor::CENTER_LEFT,
                ));
            }

            // Controls section header
            p.spawn((
                Text2d::new("-- CONTROLS --"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(muted),
                Transform::from_xyz(0.0, -100.0, 1.0),
            ));

            // Controls text
            p.spawn((
                Text2d::new("Space - pause      RMB / Esc - cancel placement"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(gray),
                Transform::from_xyz(0.0, -120.0, 1.0),
            ));

            // Towers hint
            p.spawn((
                Text2d::new(
                    "Click shop buttons to place towers. Click Remove to sell a tower for -10 COW",
                ),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(muted),
                Transform::from_xyz(0.0, -140.0, 1.0),
            ));

            // Proceed / Start button
            p.spawn((
                Mesh2d(btn_border),
                MeshMaterial2d(col_btn_bg),
                Transform::from_xyz(0.0, BTN_Y, 1.0),
            ));
            p.spawn((
                Mesh2d(btn_fill),
                MeshMaterial2d(col_btn_fill),
                Transform::from_xyz(0.0, BTN_Y, 1.05),
                MenuProceedBtn,
            ));
            p.spawn((
                Text2d::new("Start"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(highlight),
                Transform::from_xyz(0.0, BTN_Y, 1.1),
                MenuBtnLabel,
            ));
        });

    // Load icon textures directly — don't rely on EnemyAssets which may not be populated yet
    let icon_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(96),
        6,
        1,
        None,
        None,
    ));
    let icon_images: [Handle<Image>; 4] = [
        asset_server.load("enemies/enemy_frontrunner.png"),
        asset_server.load("enemies/enemy_backrunner.png"),
        asset_server.load("enemies/enemy_sandwich.png"),
        asset_server.load("enemies/enemy_jitlp.png"),
    ];

    let row_start_y = 118.0_f32;
    let row_step = 52.0_f32;
    for (i, _) in enemies.iter().enumerate() {
        let world_y = row_start_y - i as f32 * row_step;
        commands.spawn((
            Sprite {
                image: icon_images[i].clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: icon_layout.clone(),
                    index: 0,
                }),
                custom_size: Some(Vec2::splat(32.0)),
                ..default()
            },
            Transform::from_xyz(ICON_X, world_y, PANEL_Z + 2.0),
            Visibility::Visible,
            MenuIcon,
            Name::new("MenuIcon"),
        ));
    }
}

/// Show the overlay when in Menu state or when paused; hide otherwise.
fn update_menu_visibility(
    state: Res<State<GameState>>,
    pause: Res<PauseState>,
    mut overlay_q: Query<&mut Visibility, With<MenuOverlay>>,
    mut icon_q: Query<&mut Visibility, (With<MenuIcon>, Without<MenuOverlay>)>,
) {
    let should_show = *state == GameState::Menu || pause.paused;
    for mut vis in &mut overlay_q {
        *vis = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut vis in &mut icon_q {
        *vis = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Update "Start" ↔ "Proceed" label based on game phase.
fn update_btn_label(
    state: Res<State<GameState>>,
    mut label_q: Query<&mut Text2d, With<MenuBtnLabel>>,
) {
    let label = if *state == GameState::Menu {
        "Start"
    } else {
        "Proceed"
    };
    for mut text in &mut label_q {
        if text.0 != label {
            text.0 = label.to_string();
        }
    }
}

/// Handle Space (pause toggle) and button click (start / proceed).
#[allow(clippy::too_many_arguments)]
fn handle_menu_input(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut pause: ResMut<PauseState>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    overlay_q: Query<&Transform, With<MenuOverlay>>,
) {
    // Space toggles pause only while playing
    if keys.just_pressed(KeyCode::Space) && *state == GameState::Playing {
        pause.paused = !pause.paused;
        return;
    }

    // Button click
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    // Only act when overlay is visible
    if *state != GameState::Menu && !pause.paused {
        return;
    }

    let Ok(win) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else {
        return;
    };
    let Some(cursor) = win
        .cursor_position()
        .and_then(|c| cam.viewport_to_world_2d(cam_t, c).ok())
    else {
        return;
    };
    let Ok(overlay_t) = overlay_q.single() else {
        return;
    };

    // Button world position = overlay world pos + local btn offset
    let btn_world = overlay_t.translation.truncate() + Vec2::new(0.0, BTN_Y);
    if (cursor.x - btn_world.x).abs() <= BTN_W * 0.5
        && (cursor.y - btn_world.y).abs() <= BTN_H * 0.5
    {
        if *state == GameState::Menu {
            next_state.set(GameState::Playing);
        } else {
            pause.paused = false;
        }
    }
}
