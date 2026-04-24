use bevy::prelude::*;
use crate::utils::make_rounded_rect;
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};

use crate::{
    game::GameState,
    resources::{GameEconomy, GameScore, PlacementMode, COW_USD_RATE},
    towers::{Tower, TowerShopButton, TowerType},
};

pub struct UiPlugin;

const BAR_H: f32 = 34.0;
const BAR_Z: f32 = 90.0;
const BTN_W: f32 = 120.0;
const BTN_H: f32 = 32.0;
const BTN_ICON_ZONE: f32 = 28.0;
const BTN_ICON_W: f32 = 20.0;
const BTN_ICON_H: f32 = 26.0;
const BTN_GAP: f32 = 12.0;
const SHOP_TOWERS: [TowerType; 5] = [
    TowerType::BatchAuctioneer,
    TowerType::CoWMatcher,
    TowerType::Solver,
    TowerType::SlippageGuard,
    TowerType::DarkPoolNode,
];
const REMOVE_COST: f32 = 10.0;
// Total buttons = 5 tower + 1 remove; used to center the row.
const TOTAL_BTN_COUNT: usize = SHOP_TOWERS.len() + 1;

#[derive(Component)] struct TopBar;
#[derive(Component)] struct BottomBar;
#[derive(Component)] pub struct StatText(StatKind);
#[derive(Component)] pub struct ShopBtn { tower: TowerType }
#[derive(Component)] pub struct RemoveBtn;
#[derive(Component)] struct TooltipPanel;
#[derive(Component)] struct TooltipContent;

const TOOLTIP_W: f32 = 220.0;
const TOOLTIP_H: f32 = 82.0;

#[derive(Clone, Copy)]
pub enum StatKind { Settled, Protected, Extracted, Balance }

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_world_ui)
            .add_systems(
                Update,
                (reposition_ui, update_stats, handle_shop_click, update_tooltip)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_world_ui(
    mut commands: Commands,
    tower_assets: Res<crate::towers::TowerAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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
            TopBar,
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

    // Shop buttons — parent entity holds background; icon + label are children
    for (idx, tower) in SHOP_TOWERS.iter().enumerate() {
        let x = btn_x(idx);
        let color = tower.color();
        let c = color.to_srgba();
        // All buttons have icons now (one sheet, one index per tower)
        let label_local_x = -BTN_W * 0.5 + BTN_ICON_ZONE + (BTN_W - BTN_ICON_ZONE) * 0.5;

        let mut btn = commands.spawn((
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

        btn.with_children(|p| {
            // Icon from shared sheet
            if let (Some(sheet), Some(layout)) = (tower_assets.icon_sheet.clone(), tower_assets.icon_layout.clone()) {
                p.spawn((
                    Sprite {
                        image: sheet,
                        texture_atlas: Some(TextureAtlas { layout, index: tower.atlas_index() }),
                        custom_size: Some(Vec2::new(BTN_ICON_W, BTN_ICON_H)),
                        ..default()
                    },
                    Transform::from_xyz(-BTN_W * 0.5 + BTN_ICON_ZONE * 0.5, 0.0, 1.0),
                ));
            }

            // Label — centered in text zone
            p.spawn((
                Text2d::new(format!("{} {:.0}c", tower.label(), tower.cost())),
                TextFont { font_size: 10.0, ..default() },
                TextColor(color),
                Transform::from_xyz(label_local_x, 0.0, 1.0),
            ));
        });
    }

    // Remove button (index = last slot) — same layout as tower buttons
    let remove_x = btn_x(SHOP_TOWERS.len());
    let label_local_x = -BTN_W * 0.5 + BTN_ICON_ZONE + (BTN_W - BTN_ICON_ZONE) * 0.5;
    let mut remove_btn = commands.spawn((
        Sprite {
            color: Color::srgba(0.55, 0.08, 0.08, 0.95),
            custom_size: Some(Vec2::new(BTN_W, BTN_H)),
            ..default()
        },
        Transform::from_xyz(remove_x, 0.0, BAR_Z + 1.0),
        BottomBar,
        RemoveBtn,
        Name::new("RemoveBtn"),
    ));
    remove_btn.with_children(|p| {
        if let Some(icon) = tower_assets.delete_icon.clone() {
            p.spawn((
                Sprite {
                    image: icon,
                    custom_size: Some(Vec2::new(BTN_ICON_W, BTN_ICON_H)),
                    ..default()
                },
                Transform::from_xyz(-BTN_W * 0.5 + BTN_ICON_ZONE * 0.5, 0.0, 1.0),
            ));
        }
        p.spawn((
            Text2d::new(format!("Remove -{:.0}c", REMOVE_COST)),
            TextFont { font_size: 10.0, ..default() },
            TextColor(Color::srgb(1.0, 0.65, 0.65)),
            Transform::from_xyz(label_local_x, 0.0, 1.0),
        ));
    });

    // Tooltip panel — transparent Sprite for visibility propagation; mesh children for visuals
    const CR: f32 = 7.0; // corner radius
    let border_mesh = meshes.add(make_rounded_rect(TOOLTIP_W, TOOLTIP_H, CR, 8));
    let fill_mesh   = meshes.add(make_rounded_rect(TOOLTIP_W - 4.0, TOOLTIP_H - 4.0, CR - 1.0, 8));
    commands.spawn((
        Sprite { color: Color::NONE, custom_size: Some(Vec2::new(TOOLTIP_W, TOOLTIP_H)), ..default() },
        Transform::from_xyz(0.0, -9999.0, 88.0),
        Visibility::Hidden,
        TooltipPanel,
        Name::new("Tooltip"),
    )).with_children(|p| {
        p.spawn((
            Mesh2d(border_mesh),
            MeshMaterial2d(materials.add(ColorMaterial {
                color: Color::srgba(0.50, 0.35, 0.88, 0.95),
                alpha_mode: AlphaMode2d::Blend,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, -0.05),
        ));
        p.spawn((
            Mesh2d(fill_mesh),
            MeshMaterial2d(materials.add(ColorMaterial {
                color: Color::srgba(0.04, 0.02, 0.18, 0.97),
                alpha_mode: AlphaMode2d::Blend,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        p.spawn((
            Text2d::new(""),
            TextFont { font_size: 10.0, ..default() },
            TextColor(Color::srgb(0.90, 0.90, 0.90)),
            Transform::from_xyz(0.0, 0.0, 0.5),
            TooltipContent,
        ));
    });

    // Cancel hint — right of the last button
    let last_btn_x = btn_x(TOTAL_BTN_COUNT - 1);
    commands.spawn((
        Text2d::new("RMB/Esc: cancel"),
        TextFont { font_size: 10.0, ..default() },
        TextColor(Color::srgb(0.4, 0.4, 0.4)),
        Transform::from_xyz(last_btn_x + BTN_W * 0.5 + 42.0, 0.0, BAR_Z + 1.0),
        BottomBar,
    ));
}

fn btn_x(idx: usize) -> f32 {
    let n = TOTAL_BTN_COUNT as f32;
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
            StatKind::Protected => format!("Protected: {}", fmt_usd(score.value_protected * COW_USD_RATE)),
            StatKind::Extracted => format!("Extracted: {}", fmt_usd(score.value_extracted * COW_USD_RATE)),
            StatKind::Balance   => format!("Balance: {:.0} COW", economy.balance),
        };
    }
}

/// Click on a shop button in world space → enter placement/remove mode.
pub fn handle_shop_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    btn_q: Query<(&ShopBtn, &Transform)>,
    remove_btn_q: Query<&Transform, With<RemoveBtn>>,
    mut placement_mode: ResMut<PlacementMode>,
    economy: Res<GameEconomy>,
) {
    if !mouse.just_pressed(MouseButton::Left) { return; }
    let Ok(win) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else { return };
    let Some(cursor) = win.cursor_position() else { return };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_t, cursor) else { return };

    let half_h = win.height() * 0.5;
    let bot_y = -half_h + BAR_H * 0.5;

    if (world_pos.y - bot_y).abs() > BAR_H * 0.5 + 4.0 { return; }

    // Tower buttons
    for (btn, btn_t) in &btn_q {
        if (world_pos.x - btn_t.translation.x).abs() <= BTN_W * 0.5 {
            if economy.balance >= btn.tower.cost() {
                *placement_mode = PlacementMode::Placing(btn.tower.clone());
            }
            return;
        }
    }

    // Remove button — toggle
    if let Ok(t) = remove_btn_q.single() {
        if (world_pos.x - t.translation.x).abs() <= BTN_W * 0.5 {
            *placement_mode = if *placement_mode == PlacementMode::Removing {
                PlacementMode::Idle
            } else {
                PlacementMode::Removing
            };
            return;
        }
    }

    // Click on bar but not any button — cancel current mode
    *placement_mode = PlacementMode::Idle;
}

fn update_tooltip(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    tower_q: Query<(&Tower, &Transform), Without<TooltipPanel>>,
    btn_q: Query<(&ShopBtn, &Transform), Without<TooltipPanel>>,
    mut panel_q: Query<(&mut Transform, &mut Visibility), With<TooltipPanel>>,
    mut content_q: Query<&mut Text2d, With<TooltipContent>>,
) {
    let Ok(win) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else { return };
    let Ok((mut panel_t, mut panel_vis)) = panel_q.single_mut() else { return };

    let Some(cursor) = win.cursor_position()
        .and_then(|c| cam.viewport_to_world_2d(cam_t, c).ok())
    else {
        *panel_vis = Visibility::Hidden;
        return;
    };

    let half_w = win.width()  * 0.5;
    let half_h = win.height() * 0.5;
    let bot_y  = -half_h + BAR_H * 0.5;

    let tower_hit = tower_q.iter()
        .find(|(_, t)| t.translation.truncate().distance(cursor) < 42.0)
        .map(|(tw, t)| (tw.tower_type.clone(), t.translation.truncate()));

    let btn_hit = btn_q.iter()
        .find(|(_, t)| {
            (cursor.x - t.translation.x).abs() <= BTN_W * 0.5
                && (cursor.y - bot_y).abs() <= BAR_H * 0.5 + 4.0
        })
        .map(|(btn, t)| (btn.tower.clone(), Vec2::new(t.translation.x, bot_y)));

    let Some((tower_type, anchor)) = tower_hit.or(btn_hit) else {
        *panel_vis = Visibility::Hidden;
        return;
    };

    // Buttons sit on the bottom bar; towers are mid-screen sprites (110 px tall, half = 55).
    let v_offset = if (anchor.y - bot_y).abs() < BAR_H {
        BAR_H * 0.5 + 6.0 + TOOLTIP_H * 0.5   // just above the bar
    } else {
        55.0 + 6.0 + TOOLTIP_H * 0.5           // just above the tower sprite
    };
    let tx = anchor.x.clamp(-half_w + TOOLTIP_W * 0.5 + 4.0, half_w - TOOLTIP_W * 0.5 - 4.0);
    let ty = (anchor.y + v_offset)
        .clamp(-half_h + TOOLTIP_H * 0.5 + 4.0, half_h - TOOLTIP_H * 0.5 - 4.0);
    panel_t.translation.x = tx;
    panel_t.translation.y = ty;
    *panel_vis = Visibility::Visible;

    if let Ok(mut text) = content_q.single_mut() {
        text.0 = format!(
            "{}\n{}\n{}",
            tower_type.label(),
            tower_type.description(),
            tower_type.stats_line(),
        );
    }
}


fn fmt_usd(usd: f32) -> String {
    if usd >= 1_000_000.0 {
        format!("${:.1}M", usd / 1_000_000.0)
    } else if usd >= 1_000.0 {
        format!("${:.1}k", usd / 1_000.0)
    } else {
        format!("${:.2}", usd)
    }
}
