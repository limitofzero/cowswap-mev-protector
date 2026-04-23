use bevy::prelude::*;

use crate::{
    game::GameState,
    mempool::MempoolPath,
    resources::{GameEconomy, GameScore, PlacementMode},
    towers::{TowerShopButton, TowerType},
};

pub struct UiPlugin;

const BAR_H: f32 = 34.0;
const BAR_Z: f32 = 90.0;
const BTN_W: f32 = 88.0;
const BTN_H: f32 = 24.0;
const BTN_GAP: f32 = 12.0;
const SHOP_TOWERS: [TowerType; 5] = [
    TowerType::BatchAuctioneer,
    TowerType::CoWMatcher,
    TowerType::Solver,
    TowerType::SlippageGuard,
    TowerType::DarkPoolNode,
];

#[derive(Component)] struct TopBar;
#[derive(Component)] struct BottomBar;
#[derive(Component)] pub struct StatText(StatKind);
#[derive(Component)] pub struct ShopBtn { tower: TowerType }

#[derive(Clone, Copy)]
pub enum StatKind { Settled, Protected, Extracted, Balance }

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_world_ui)
            .add_systems(
                Update,
                (reposition_ui, update_stats, handle_shop_click)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_world_ui(mut commands: Commands) {
    // ── top bar background ────────────────────────────────────────────
    commands.spawn((
        Sprite {
            color: Color::srgba(0.04, 0.02, 0.12, 0.88),
            custom_size: Some(Vec2::new(9999.0, BAR_H)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, BAR_Z),
        TopBar,
        Name::new("TopBarBg"),
    ));

    let stats = [
        (StatKind::Settled,   "Settled: 0",       Color::WHITE,                      -420.0_f32),
        (StatKind::Protected, "Protected: 0 COW", Color::srgb(0.30, 1.00, 0.45),    -180.0),
        (StatKind::Extracted, "Extracted: 0 COW", Color::srgb(1.00, 0.35, 0.35),      80.0),
        (StatKind::Balance,   "Balance: 300 COW", Color::srgb(0.80, 0.65, 1.00),     340.0),
    ];
    for (kind, text, color, x) in stats {
        commands.spawn((
            Text2d::new(text),
            TextFont { font_size: 13.0, ..default() },
            TextColor(color),
            Transform::from_xyz(x, 0.0, BAR_Z + 1.0),
            StatText(kind),
        ));
    }

    // ── bottom bar background ─────────────────────────────────────────
    commands.spawn((
        Sprite {
            color: Color::srgba(0.04, 0.02, 0.12, 0.92),
            custom_size: Some(Vec2::new(9999.0, BAR_H)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, BAR_Z),
        BottomBar,
        Name::new("BottomBarBg"),
    ));

    // "BUILD" label — left of the first button
    let first_btn_x = btn_x(0);
    commands.spawn((
        Text2d::new("BUILD"),
        TextFont { font_size: 11.0, ..default() },
        TextColor(Color::srgb(0.5, 0.5, 0.5)),
        Transform::from_xyz(first_btn_x - BTN_W * 0.5 - 36.0, 0.0, BAR_Z + 1.0),
        BottomBar,
    ));

    // Shop buttons — one sprite + one label per tower
    for (idx, tower) in SHOP_TOWERS.iter().enumerate() {
        let x = btn_x(idx);
        let color = tower.color();
        let c = color.to_srgba();

        // Button background
        commands.spawn((
            Sprite {
                color: Color::srgba(c.red * 0.2, c.green * 0.2, c.blue * 0.2, 0.95),
                custom_size: Some(Vec2::new(BTN_W, BTN_H)),
                ..default()
            },
            Transform::from_xyz(x, 0.0, BAR_Z + 1.0),
            BottomBar,
            ShopBtn { tower: tower.clone() },
            TowerShopButton(tower.clone()),
            Name::new(format!("ShopBtn::{}", tower.label())),
        ));

        // Button label
        commands.spawn((
            Text2d::new(format!("{}  {:.0}c", tower.label(), tower.cost())),
            TextFont { font_size: 11.0, ..default() },
            TextColor(color),
            Transform::from_xyz(x, 0.0, BAR_Z + 2.0),
            BottomBar,
        ));
    }

    // Cancel hint — right of the last button
    let last_btn_x = btn_x(SHOP_TOWERS.len() - 1);
    commands.spawn((
        Text2d::new("RMB/Esc: cancel"),
        TextFont { font_size: 10.0, ..default() },
        TextColor(Color::srgb(0.4, 0.4, 0.4)),
        Transform::from_xyz(last_btn_x + BTN_W * 0.5 + 42.0, 0.0, BAR_Z + 1.0),
        BottomBar,
    ));
}

fn btn_x(idx: usize) -> f32 {
    let n = SHOP_TOWERS.len() as f32;
    let total = n * BTN_W + (n - 1.0) * BTN_GAP;
    -total * 0.5 + idx as f32 * (BTN_W + BTN_GAP) + BTN_W * 0.5
}

/// Reposition both bars to the top/bottom of the current window every frame.
fn reposition_ui(
    windows: Query<&Window>,
    mut top_q: Query<&mut Transform, (With<TopBar>, Without<BottomBar>)>,
    mut bot_q: Query<&mut Transform, (With<BottomBar>, Without<TopBar>)>,
) {
    let Ok(win) = windows.single() else { return };
    let half_h = win.height() * 0.5;
    let top_y = half_h - BAR_H * 0.5;
    let bot_y = -half_h + BAR_H * 0.5;

    for mut t in &mut top_q { t.translation.y = top_y; }
    for mut t in &mut bot_q { t.translation.y = bot_y; }
}

fn update_stats(
    score: Res<GameScore>,
    economy: Res<GameEconomy>,
    mut q: Query<(&StatText, &mut Text2d)>,
) {
    if !score.is_changed() && !economy.is_changed() { return; }
    for (stat, mut text) in &mut q {
        text.0 = match stat.0 {
            StatKind::Settled   => format!("Settled: {}", score.txs_settled),
            StatKind::Protected => format!("Protected: {:.0} COW", score.value_protected),
            StatKind::Extracted => format!("Extracted: {:.0} COW", score.value_extracted),
            StatKind::Balance   => format!("Balance: {:.0} COW", economy.balance),
        };
    }
}

/// Click on a shop button in world space → enter placement mode.
pub fn handle_shop_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    btn_q: Query<(&ShopBtn, &Transform)>,
    mut placement_mode: ResMut<PlacementMode>,
    economy: Res<GameEconomy>,
    path: Res<MempoolPath>,
) {
    if !mouse.just_pressed(MouseButton::Left) { return; }
    // Only handle clicks inside the bottom bar area (to avoid conflict with placement)
    let Ok(win) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else { return };
    let Some(cursor) = win.cursor_position() else { return };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_t, cursor) else { return };

    let half_h = win.height() * 0.5;
    let bot_y = -half_h + BAR_H * 0.5;

    // Ignore click if not near the bottom bar
    if (world_pos.y - bot_y).abs() > BAR_H * 0.5 + 4.0 { return; }

    for (btn, btn_t) in &btn_q {
        let bx = btn_t.translation.x;
        if (world_pos.x - bx).abs() <= BTN_W * 0.5 {
            if economy.balance >= btn.tower.cost() {
                *placement_mode = PlacementMode::Placing(btn.tower.clone());
            }
            return;
        }
    }

    // Also cancel placement if clicking bottom bar but not on a button
    if let PlacementMode::Placing(_) = *placement_mode {
        *placement_mode = PlacementMode::Idle;
    }
    let _ = path; // suppress unused warning
}
