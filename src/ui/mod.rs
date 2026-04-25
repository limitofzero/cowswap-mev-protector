use crate::utils::make_rounded_rect;
use bevy::prelude::*;
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};

use crate::{
    enemies::resources::WaveManager,
    game::GameState,
    resources::{COW_USD_RATE, GameEconomy, GameScore, NetworkLoad, PlacementMode},
    towers::{Tower, TowerShopButton, TowerType},
};

pub struct UiPlugin;

pub const TOP_BAR_H: f32 = 34.0;
pub const BOT_BAR_H: f32 = 96.0;
const BAR_H: f32 = TOP_BAR_H; // kept for hit-test compat in other systems
const BAR_Z: f32 = 90.0;
const BTN_W: f32 = 148.0;
const BTN_H: f32 = 74.0;
const BTN_CORNER: f32 = 10.0;
const BTN_ICON_W: f32 = 28.0;
const BTN_ICON_H: f32 = 36.0;
const BTN_GAP: f32 = 14.0;
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

#[derive(Component)]
struct TopBar;
#[derive(Component)]
struct BottomBar;
#[derive(Component)]
pub struct StatText(StatKind);
#[derive(Component)]
pub struct ShopBtn {
    tower: TowerType,
}
#[derive(Component)]
pub struct RemoveBtn;
/// Tooltip shown when hovering a placed tower (6 lines: name/stat/upgrades).
#[derive(Component)]
struct TowerTooltipPanel;
#[derive(Component)]
struct TowerTooltipLine(u8);
/// Compact tooltip shown when hovering a shop button (4 lines).
#[derive(Component)]
struct ShopTooltipPanel;
#[derive(Component)]
struct ShopTooltipLine(u8);
/// Tooltip panel shown when hovering any top-bar stat. One panel per StatKind.
#[derive(Component)]
struct StatTooltipPanel(StatKind);
/// A single text line inside a StatTooltipPanel. (stat, line_index)
#[derive(Component)]
struct StatTooltipLine(StatKind, u8);
/// Drives the press-and-release scale animation; removed when animation completes.
#[derive(Component)]
struct BtnClickEffect(f32);

const TOOLTIP_W: f32 = 260.0;
const TOOLTIP_H: f32 = 196.0;
const TOOLTIP_LINE_Y: [f32; 7] = [78.0, 58.0, 38.0, 18.0, 0.0, -18.0, -44.0];

const SHOP_TT_W: f32 = 240.0;
const SHOP_TT_H: f32 = 106.0;
const SHOP_TT_LINE_Y: [f32; 4] = [38.0, 18.0, 0.0, -20.0];

const STAT_TT_W: f32 = 320.0;
const STAT_TT_H: f32 = 72.0;
const STAT_TT_LINE_Y: [f32; 3] = [22.0, 2.0, -18.0];

#[derive(Clone, Copy)]
pub enum StatKind {
    Block,
    Settled,
    Protected,
    Extracted,
    Balance,
    BaseFee,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_world_ui)
            .add_systems(
                Update,
                (
                    reposition_ui,
                    update_stats,
                    handle_shop_click,
                    update_tooltip,
                    update_stat_tooltips,
                    animate_btn_click,
                )
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
        (
            StatKind::Block,
            "Block #0",
            Color::srgb(0.55, 0.85, 1.00),
            -560.0_f32,
        ),
        (StatKind::Settled, "Settled: 0", Color::WHITE, -380.0),
        (
            StatKind::Protected,
            "Protected: 0 COW",
            Color::srgb(0.30, 1.00, 0.45),
            -140.0,
        ),
        (
            StatKind::Extracted,
            "Extracted: 0 COW",
            Color::srgb(1.00, 0.35, 0.35),
            110.0,
        ),
        (
            StatKind::Balance,
            "Balance: 300 COW",
            Color::srgb(0.80, 0.65, 1.00),
            360.0,
        ),
        (
            StatKind::BaseFee,
            "BaseFee: LOW",
            Color::srgb(0.40, 0.90, 0.55),
            0.0_f32,
        ),
    ];
    for (kind, text, color, x) in stats {
        commands.spawn((
            Text2d::new(text),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(color),
            Transform::from_xyz(x, 0.0, BAR_Z + 1.0),
            StatText(kind),
            TopBar,
        ));
    }

    // ── Stat tooltips (one per top-bar stat, hidden until hover) ──────
    for kind in [
        StatKind::Block,
        StatKind::Settled,
        StatKind::Protected,
        StatKind::Extracted,
        StatKind::Balance,
        StatKind::BaseFee,
    ] {
        commands
            .spawn((
                Sprite {
                    color: Color::srgba(0.04, 0.02, 0.12, 0.94),
                    custom_size: Some(Vec2::new(STAT_TT_W, STAT_TT_H)),
                    ..default()
                },
                Transform::from_xyz(-9999.0, -9999.0, BAR_Z + 3.0),
                Visibility::Hidden,
                StatTooltipPanel(kind),
            ))
            .with_children(|p| {
                for line_idx in 0u8..3 {
                    p.spawn((
                        Text2d::new(""),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Transform::from_xyz(0.0, STAT_TT_LINE_Y[line_idx as usize], 0.1),
                        StatTooltipLine(kind, line_idx),
                    ));
                }
            });
    }

    // ── bottom bar background ─────────────────────────────────────────
    commands.spawn((
        Sprite {
            color: Color::srgba(0.04, 0.02, 0.12, 0.92),
            custom_size: Some(Vec2::new(9999.0, BOT_BAR_H)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, BAR_Z),
        BottomBar,
        Name::new("BottomBarBg"),
    ));

    // Pre-build shared mesh handles (one set per button, reused across loop)
    let border_mesh = meshes.add(make_rounded_rect(BTN_W, BTN_H, BTN_CORNER, 10));
    let fill_mesh = meshes.add(make_rounded_rect(
        BTN_W - 4.0,
        BTN_H - 4.0,
        BTN_CORNER - 1.0,
        10,
    ));

    // Shop buttons
    for (idx, tower) in SHOP_TOWERS.iter().enumerate() {
        let x = btn_x(idx);
        let color = tower.color();
        let srgba = color.to_srgba();

        let col_border = materials.add(ColorMaterial {
            color: Color::srgba(srgba.red, srgba.green, srgba.blue, 0.90),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        });
        let col_fill = materials.add(ColorMaterial {
            color: Color::srgba(srgba.red * 0.12, srgba.green * 0.12, srgba.blue * 0.12, 0.97),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        });

        let mut btn = commands.spawn((
            Sprite {
                color: Color::NONE,
                custom_size: Some(Vec2::new(BTN_W, BTN_H)),
                ..default()
            },
            Transform::from_xyz(x, 0.0, BAR_Z + 1.0),
            BottomBar,
            ShopBtn {
                tower: tower.clone(),
            },
            TowerShopButton(tower.clone()),
            Name::new(format!("ShopBtn::{}", tower.label())),
        ));

        btn.with_children(|p| {
            p.spawn((
                Mesh2d(border_mesh.clone()),
                MeshMaterial2d(col_border),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
            p.spawn((
                Mesh2d(fill_mesh.clone()),
                MeshMaterial2d(col_fill),
                Transform::from_xyz(0.0, 0.0, 0.05),
            ));

            // Icon — left side, vertically centered
            if let (Some(sheet), Some(layout)) = (
                tower_assets.icon_sheet.clone(),
                tower_assets.icon_layout.clone(),
            ) {
                p.spawn((
                    Sprite {
                        image: sheet,
                        texture_atlas: Some(TextureAtlas {
                            layout,
                            index: tower.atlas_index(),
                        }),
                        custom_size: Some(Vec2::new(BTN_ICON_W, BTN_ICON_H)),
                        ..default()
                    },
                    Transform::from_xyz(-BTN_W * 0.5 + BTN_ICON_W * 0.5 + 10.0, 0.0, 1.0),
                ));
            }

            // Short name — centered in button
            p.spawn((
                Text2d::new(tower.short_label()),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(color),
                Transform::from_xyz(0.0, 10.0, 1.0),
            ));
            // Cost — centered below name
            p.spawn((
                Text2d::new(format!("{:.0} CoW", tower.cost())),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.70, 0.70, 0.70)),
                Transform::from_xyz(0.0, -10.0, 1.0),
            ));
        });
    }

    // Remove button
    let remove_x = btn_x(SHOP_TOWERS.len());
    let col_rm_border = materials.add(ColorMaterial {
        color: Color::srgba(0.85, 0.25, 0.25, 0.90),
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    });
    let col_rm_fill = materials.add(ColorMaterial {
        color: Color::srgba(0.15, 0.03, 0.03, 0.97),
        alpha_mode: AlphaMode2d::Blend,
        ..default()
    });
    let mut remove_btn = commands.spawn((
        Sprite {
            color: Color::NONE,
            custom_size: Some(Vec2::new(BTN_W, BTN_H)),
            ..default()
        },
        Transform::from_xyz(remove_x, 0.0, BAR_Z + 1.0),
        BottomBar,
        RemoveBtn,
        Name::new("RemoveBtn"),
    ));
    remove_btn.with_children(|p| {
        p.spawn((
            Mesh2d(border_mesh.clone()),
            MeshMaterial2d(col_rm_border),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        p.spawn((
            Mesh2d(fill_mesh.clone()),
            MeshMaterial2d(col_rm_fill),
            Transform::from_xyz(0.0, 0.0, 0.05),
        ));
        if let Some(icon) = tower_assets.delete_icon.clone() {
            p.spawn((
                Sprite {
                    image: icon,
                    custom_size: Some(Vec2::new(BTN_ICON_W, BTN_ICON_H)),
                    ..default()
                },
                Transform::from_xyz(-BTN_W * 0.5 + BTN_ICON_W * 0.5 + 10.0, 0.0, 1.0),
            ));
        }
        p.spawn((
            Text2d::new("RMV"),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.55, 0.55)),
            Transform::from_xyz(0.0, 10.0, 1.0),
        ));
        p.spawn((
            Text2d::new(format!("-{:.0} CoW", REMOVE_COST)),
            TextFont {
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::srgb(0.70, 0.70, 0.70)),
            Transform::from_xyz(0.0, -10.0, 1.0),
        ));
    });

    // ── Tower tooltip (placed-tower hover) — 6 lines ────────────────────
    let spawn_tooltip = |_commands: &mut Commands,
                         meshes: &mut Assets<Mesh>,
                         materials: &mut Assets<ColorMaterial>,
                         w: f32,
                         h: f32,
                         cr: f32| {
        let border_mesh = meshes.add(make_rounded_rect(w, h, cr, 8));
        let fill_mesh = meshes.add(make_rounded_rect(w - 4.0, h - 4.0, cr - 1.0, 8));
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
        (border_mesh, fill_mesh, col_border, col_fill)
    };

    let (border_mesh, fill_mesh, cb, cf) = spawn_tooltip(
        &mut commands,
        &mut meshes,
        &mut materials,
        TOOLTIP_W,
        TOOLTIP_H,
        7.0,
    );
    commands
        .spawn((
            Sprite {
                color: Color::NONE,
                custom_size: Some(Vec2::new(TOOLTIP_W, TOOLTIP_H)),
                ..default()
            },
            Transform::from_xyz(0.0, -9999.0, BAR_Z + 3.0),
            Visibility::Hidden,
            TowerTooltipPanel,
            Name::new("TowerTooltip"),
        ))
        .with_children(|p| {
            p.spawn((
                Mesh2d(border_mesh),
                MeshMaterial2d(cb),
                Transform::from_xyz(0.0, 0.0, -0.05),
            ));
            p.spawn((
                Mesh2d(fill_mesh),
                MeshMaterial2d(cf),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
            for line_idx in 0..7u8 {
                p.spawn((
                    Text2d::new(""),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, TOOLTIP_LINE_Y[line_idx as usize], 0.5),
                    TowerTooltipLine(line_idx),
                    Visibility::Hidden,
                ));
            }
        });

    // ── Shop tooltip (button hover) — 4 lines, compact ──────────────────
    let (border_mesh, fill_mesh, cb, cf) = spawn_tooltip(
        &mut commands,
        &mut meshes,
        &mut materials,
        SHOP_TT_W,
        SHOP_TT_H,
        7.0,
    );
    commands
        .spawn((
            Sprite {
                color: Color::NONE,
                custom_size: Some(Vec2::new(SHOP_TT_W, SHOP_TT_H)),
                ..default()
            },
            Transform::from_xyz(0.0, -9999.0, BAR_Z + 3.0),
            Visibility::Hidden,
            ShopTooltipPanel,
            Name::new("ShopTooltip"),
        ))
        .with_children(|p| {
            p.spawn((
                Mesh2d(border_mesh),
                MeshMaterial2d(cb),
                Transform::from_xyz(0.0, 0.0, -0.05),
            ));
            p.spawn((
                Mesh2d(fill_mesh),
                MeshMaterial2d(cf),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
            for line_idx in 0..4u8 {
                p.spawn((
                    Text2d::new(""),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, SHOP_TT_LINE_Y[line_idx as usize], 0.5),
                    ShopTooltipLine(line_idx),
                    Visibility::Hidden,
                ));
            }
        });
}

fn btn_x(idx: usize) -> f32 {
    let btn_count = TOTAL_BTN_COUNT as f32;
    let total = btn_count * BTN_W + (btn_count - 1.0) * BTN_GAP;
    -total * 0.5 + idx as f32 * (BTN_W + BTN_GAP) + BTN_W * 0.5
}

/// Reposition both bars to the top/bottom of the current window every frame.
/// Also distributes stat labels into equal-width columns across the top bar.
#[allow(clippy::type_complexity)]
fn reposition_ui(
    windows: Query<&Window>,
    mut top_bg_q: Query<&mut Transform, (With<TopBar>, Without<BottomBar>, Without<StatText>)>,
    mut bot_q: Query<&mut Transform, (With<BottomBar>, Without<TopBar>)>,
    mut stat_q: Query<(&StatText, &mut Transform), (With<TopBar>, Without<BottomBar>)>,
) {
    let Ok(win) = windows.single() else { return };
    let half_w = win.width() * 0.5;
    let half_h = win.height() * 0.5;
    let top_y = half_h - TOP_BAR_H * 0.5;
    let bot_y = -half_h + BOT_BAR_H * 0.5;

    for mut t in &mut top_bg_q {
        t.translation.y = top_y;
    }
    for mut t in &mut bot_q {
        t.translation.y = bot_y;
    }

    const N: f32 = 6.0;
    let col_w = win.width() / N;
    for (stat, mut t) in &mut stat_q {
        let col = match stat.0 {
            StatKind::Block => 0,
            StatKind::Settled => 1,
            StatKind::Protected => 2,
            StatKind::Extracted => 3,
            StatKind::Balance => 4,
            StatKind::BaseFee => 5,
        };
        t.translation.x = -half_w + col_w * (col as f32 + 0.5);
        t.translation.y = top_y;
    }
}

fn update_stats(
    score: Res<GameScore>,
    economy: Res<GameEconomy>,
    waves: Res<WaveManager>,
    network: Res<NetworkLoad>,
    mut q: Query<(&StatText, &mut Text2d, &mut TextColor)>,
) {
    if !score.is_changed() && !economy.is_changed() && !waves.is_changed() && !network.is_changed()
    {
        return;
    }
    for (stat, mut text, mut color) in &mut q {
        match stat.0 {
            StatKind::Block => {
                text.0 = format!("Block #{}", waves.wave);
            }
            StatKind::Settled => {
                text.0 = format!("Settled: {}", score.txs_settled);
            }
            StatKind::Protected => {
                text.0 = format!(
                    "Protected: {}",
                    fmt_usd(score.value_protected * COW_USD_RATE)
                );
            }
            StatKind::Extracted => {
                text.0 = format!(
                    "Extracted: {}",
                    fmt_usd(score.value_extracted * COW_USD_RATE)
                );
            }
            StatKind::Balance => {
                text.0 = format!("Balance: {:.0} COW", economy.balance);
            }
            StatKind::BaseFee => {
                text.0 = format!(
                    "BaseFee: {}  {}tx/s  -{}%",
                    network.label(),
                    network.txs_per_sec_str(),
                    network.speed_loss_pct()
                );
                color.0 = match network.level {
                    0 => Color::srgb(0.40, 0.90, 0.55),
                    1 => Color::srgb(1.00, 0.75, 0.20),
                    _ => Color::srgb(1.00, 0.35, 0.35),
                };
            }
        }
    }
}

/// Click on a shop button in world space → enter placement/remove mode.
#[allow(clippy::too_many_arguments)]
pub fn handle_shop_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    btn_q: Query<(Entity, &ShopBtn, &Transform)>,
    remove_btn_q: Query<(Entity, &Transform), With<RemoveBtn>>,
    mut placement_mode: ResMut<PlacementMode>,
    economy: Res<GameEconomy>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(win) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else {
        return;
    };
    let Some(cursor) = win.cursor_position() else {
        return;
    };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_t, cursor) else {
        return;
    };

    let half_h = win.height() * 0.5;
    let bot_y = -half_h + BOT_BAR_H * 0.5;

    if (world_pos.y - bot_y).abs() > BOT_BAR_H * 0.5 + 4.0 {
        return;
    }

    // Tower buttons
    for (entity, btn, btn_t) in &btn_q {
        if (world_pos.x - btn_t.translation.x).abs() <= BTN_W * 0.5
            && (world_pos.y - bot_y).abs() <= BTN_H * 0.5
        {
            commands.entity(entity).insert(BtnClickEffect(0.0));
            if economy.balance >= btn.tower.cost() {
                *placement_mode = PlacementMode::Placing(btn.tower.clone());
            }
            return;
        }
    }

    // Remove button — toggle
    if let Ok((entity, t)) = remove_btn_q.single()
        && (world_pos.x - t.translation.x).abs() <= BTN_W * 0.5
        && (world_pos.y - bot_y).abs() <= BTN_H * 0.5
    {
        commands.entity(entity).insert(BtnClickEffect(0.0));
        *placement_mode = if *placement_mode == PlacementMode::Removing {
            PlacementMode::Idle
        } else {
            PlacementMode::Removing
        };
        return;
    }

    *placement_mode = PlacementMode::Idle;
}

fn animate_btn_click(
    time: Res<Time>,
    mut commands: Commands,
    mut btn_q: Query<(Entity, &mut Transform, &mut BtnClickEffect)>,
) {
    for (entity, mut transform, mut effect) in &mut btn_q {
        effect.0 += time.delta_secs() / 0.14;
        // Press down (0→0.5) then spring back (0.5→1.0)
        let scale = if effect.0 < 0.5 {
            1.0 - effect.0 * 0.18
        } else {
            0.91 + (effect.0 - 0.5) * 0.18
        };
        transform.scale = Vec3::splat(scale.clamp(0.91, 1.0));
        if effect.0 >= 1.0 {
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<BtnClickEffect>();
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_tooltip(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    economy: Res<GameEconomy>,
    tower_q: Query<(&Tower, &Transform), (Without<TowerTooltipPanel>, Without<ShopTooltipPanel>)>,
    btn_q: Query<(&ShopBtn, &Transform), (Without<TowerTooltipPanel>, Without<ShopTooltipPanel>)>,
    mut tower_panel_q: Query<
        (&mut Transform, &mut Visibility),
        (With<TowerTooltipPanel>, Without<ShopTooltipPanel>),
    >,
    mut shop_panel_q: Query<
        (&mut Transform, &mut Visibility),
        (With<ShopTooltipPanel>, Without<TowerTooltipPanel>),
    >,
    mut tower_lines_q: Query<
        (
            &TowerTooltipLine,
            &mut Text2d,
            &mut TextColor,
            &mut Visibility,
        ),
        (
            Without<TowerTooltipPanel>,
            Without<ShopTooltipPanel>,
            Without<ShopTooltipLine>,
        ),
    >,
    mut shop_lines_q: Query<
        (
            &ShopTooltipLine,
            &mut Text2d,
            &mut TextColor,
            &mut Visibility,
        ),
        (
            Without<TowerTooltipPanel>,
            Without<ShopTooltipPanel>,
            Without<TowerTooltipLine>,
        ),
    >,
) {
    let Ok(win) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else {
        return;
    };
    let Ok((mut tower_pt, mut tower_pv)) = tower_panel_q.single_mut() else {
        return;
    };
    let Ok((mut shop_pt, mut shop_pv)) = shop_panel_q.single_mut() else {
        return;
    };

    let Some(cursor) = win
        .cursor_position()
        .and_then(|c| cam.viewport_to_world_2d(cam_t, c).ok())
    else {
        *tower_pv = Visibility::Hidden;
        *shop_pv = Visibility::Hidden;
        return;
    };

    let half_w = win.width() * 0.5;
    let half_h = win.height() * 0.5;
    let bot_y = -half_h + BOT_BAR_H * 0.5;

    let placed_hit = tower_q
        .iter()
        .find(|(_, transform)| transform.translation.truncate().distance(cursor) < 42.0)
        .map(|(tw, transform)| {
            (
                tw.tower_type.clone(),
                tw.upgrade_level,
                tw.upgrade_cooldown,
                transform.translation.truncate(),
            )
        });

    let btn_hit = btn_q
        .iter()
        .find(|(_, transform)| {
            (cursor.x - transform.translation.x).abs() <= BTN_W * 0.5
                && (cursor.y - bot_y).abs() <= BOT_BAR_H * 0.5 + 4.0
        })
        .map(|(btn, transform)| (btn.tower.clone(), Vec2::new(transform.translation.x, bot_y)));

    // ── Tower tooltip ────────────────────────────────────────────────────
    if let Some((tt, lvl, upg_cd, pos)) = placed_hit {
        *shop_pv = Visibility::Hidden;

        let col_done = Color::srgb(1.00, 0.85, 0.20);
        let col_stat = Color::WHITE;
        let col_locked = Color::srgb(0.45, 0.45, 0.45);
        let col_muted = Color::srgb(0.50, 0.42, 0.70);
        let col_hi = Color::srgb(0.85, 0.75, 1.00);

        let mut rows: [(String, Color, bool); 7] =
            std::array::from_fn(|_| (String::new(), Color::WHITE, true));
        rows[0] = (format!("{}  Lv {}", tt.label(), lvl), col_hi, true);
        rows[1] = (tower_stat_text(&tt, lvl), col_stat, true);
        rows[2] = ("-- UPGRADES --".into(), col_muted, true);
        for row in 0..3u8 {
            let lv = row + 1;
            let effect = tt.upgrade_effect_desc(lv);
            let (text, color) = if lv <= lvl {
                (format!("[+] Lv{}  {}", lv, effect), col_done)
            } else {
                let cost = tt.upgrade_cost(row);
                (
                    format!("[ ] Lv{}  {}  {:.0} CoW", lv, effect, cost),
                    col_locked,
                )
            };
            rows[3 + row as usize] = (text, color, true);
        }
        // Upgrade action line
        rows[6] = if lvl < crate::towers::MAX_UPGRADE_LEVEL {
            if upg_cd > 0.0 {
                (
                    format!("Can be upgraded in {:.1}s", upg_cd),
                    col_muted,
                    true,
                )
            } else {
                let cost = tt.upgrade_cost(lvl);
                if economy.balance >= cost {
                    (
                        format!("Upgrade to Lv{}  {:.0} CoW", lvl + 1, cost),
                        Color::srgb(0.45, 1.00, 0.55),
                        true,
                    )
                } else {
                    (
                        format!("Need {:.0} CoW to upgrade", cost),
                        Color::srgb(1.00, 0.40, 0.40),
                        true,
                    )
                }
            }
        } else {
            ("[ MAX LEVEL ]".into(), col_muted, true)
        };

        let tooltip_offset_y = 55.0 + 8.0 + TOOLTIP_H * 0.5;
        let tx = pos.x.clamp(
            -half_w + TOOLTIP_W * 0.5 + 4.0,
            half_w - TOOLTIP_W * 0.5 - 4.0,
        );
        let ty = (pos.y + tooltip_offset_y).clamp(
            -half_h + TOOLTIP_H * 0.5 + 4.0,
            half_h - TOOLTIP_H * 0.5 - 4.0,
        );
        tower_pt.translation = Vec3::new(tx, ty, tower_pt.translation.z);
        *tower_pv = Visibility::Visible;

        for (line, mut text, mut color, mut vis) in &mut tower_lines_q {
            let (text_str, text_color, show) = &rows[line.0 as usize];
            text.0 = text_str.clone();
            *color = TextColor(*text_color);
            *vis = if *show {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }

    // ── Shop tooltip ─────────────────────────────────────────────────────
    } else if let Some((tt, anchor)) = btn_hit {
        *tower_pv = Visibility::Hidden;

        let col_hi = Color::srgb(0.85, 0.75, 1.00);
        let col_gray = Color::srgb(0.72, 0.72, 0.72);
        let parts: Vec<&str> = tt.description().splitn(2, '\n').collect();
        let line1 = parts.first().copied().unwrap_or("").to_string();
        let line2 = parts.get(1).copied().unwrap_or("").to_string();
        let rows: [(String, Color, bool); 4] = [
            (tt.label().to_string(), col_hi, true),
            (line1, col_gray, true),
            (line2.clone(), col_gray, !line2.is_empty()),
            (tt.stats_line(), col_gray, true),
        ];

        let shop_offset_y = BOT_BAR_H * 0.5 + 8.0 + SHOP_TT_H * 0.5;
        let tx = anchor.x.clamp(
            -half_w + SHOP_TT_W * 0.5 + 4.0,
            half_w - SHOP_TT_W * 0.5 - 4.0,
        );
        let ty = (anchor.y + shop_offset_y).clamp(
            -half_h + SHOP_TT_H * 0.5 + 4.0,
            half_h - SHOP_TT_H * 0.5 - 4.0,
        );
        shop_pt.translation = Vec3::new(tx, ty, shop_pt.translation.z);
        *shop_pv = Visibility::Visible;

        for (line, mut text, mut color, mut vis) in &mut shop_lines_q {
            let (text_str, text_color, show) = &rows[line.0 as usize];
            text.0 = text_str.clone();
            *color = TextColor(*text_color);
            *vis = if *show {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    } else {
        *tower_pv = Visibility::Hidden;
        *shop_pv = Visibility::Hidden;
    }
}

fn tower_stat_text(tt: &TowerType, level: u8) -> String {
    let cd = tt.cooldown_secs_upgraded(level);
    match tt {
        TowerType::Solver => format!(
            "Attacks bots  {:.0} HP dmg  every {:.1}s",
            tt.solver_damage(level),
            cd
        ),
        TowerType::CoWMatcher => format!("Grants MEV immunity for 6s  every {:.1}s", cd),
        TowerType::BatchAuctioneer => format!("Groups txs into batches  every {:.1}s", cd),
        TowerType::SlippageGuard => format!(
            "Slows bots to {}% speed  every {:.1}s",
            tt.slow_pct(level),
            cd
        ),
        TowerType::DarkPoolNode => format!("Hides txs for 4s  every {:.1}s", cd),
    }
}

fn update_stat_tooltips(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    economy: Res<GameEconomy>,
    network: Res<NetworkLoad>,
    waves: Res<WaveManager>,
    mut panel_q: Query<(&StatTooltipPanel, &mut Transform, &mut Visibility)>,
    mut line_q: Query<(&StatTooltipLine, &mut Text2d, &mut TextColor)>,
) {
    let Ok(win) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else {
        return;
    };

    let half_w = win.width() * 0.5;
    let half_h = win.height() * 0.5;
    let top_y = half_h - TOP_BAR_H * 0.5;
    let col_w = win.width() / 6.0;

    // Which column (0-5) is the cursor over, if any?
    let hovered_col: Option<usize> = win
        .cursor_position()
        .and_then(|screen_pos| cam.viewport_to_world_2d(cam_t, screen_pos).ok())
        .and_then(|world_pos| {
            if (world_pos.y - top_y).abs() > TOP_BAR_H * 0.5 {
                return None;
            }
            let col = ((world_pos.x + half_w) / col_w) as usize;
            (col < 6).then_some(col)
        });

    let hovered_kind = hovered_col.map(|col_idx| match col_idx {
        0 => StatKind::Block,
        1 => StatKind::Settled,
        2 => StatKind::Protected,
        3 => StatKind::Extracted,
        4 => StatKind::Balance,
        _ => StatKind::BaseFee,
    });

    // Position / show-hide each panel
    for (panel, mut transform, mut vis) in &mut panel_q {
        let kind = panel.0;
        let active = matches!(hovered_kind, Some(k) if std::mem::discriminant(&k) == std::mem::discriminant(&kind));
        if !active {
            *vis = Visibility::Hidden;
            continue;
        }
        let col = match kind {
            StatKind::Block => 0,
            StatKind::Settled => 1,
            StatKind::Protected => 2,
            StatKind::Extracted => 3,
            StatKind::Balance => 4,
            StatKind::BaseFee => 5,
        };
        let panel_x = -half_w + col_w * (col as f32 + 0.5);
        // Clamp so tooltip stays on screen
        let panel_x = panel_x.clamp(
            -half_w + STAT_TT_W * 0.5 + 4.0,
            half_w - STAT_TT_W * 0.5 - 4.0,
        );
        *vis = Visibility::Visible;
        transform.translation.x = panel_x;
        transform.translation.y = top_y - TOP_BAR_H * 0.5 - STAT_TT_H * 0.5 - 4.0;
    }

    // Fill line text for the hovered stat
    let Some(active_kind) = hovered_kind else {
        return;
    };
    let grey = Color::srgb(0.70, 0.70, 0.70);
    let white = Color::WHITE;
    let green = Color::srgb(0.35, 1.00, 0.50);
    let red = Color::srgb(1.00, 0.40, 0.40);
    let cyan = Color::srgb(0.45, 0.85, 1.00);

    let fee_pct = (economy.fee_rate * 100.0).round() as u32;

    let lines: [(String, Color); 3] = match active_kind {
        StatKind::Block => [
            ("Each block = 15s of simulated Ethereum time".into(), cyan),
            (
                format!(
                    "Wave {} — higher blocks spawn stronger, more frequent bots",
                    waves.wave
                ),
                grey,
            ),
            ("".into(), white),
        ],
        StatKind::Settled => [
            (
                "Transactions confirmed to the settlement contract".into(),
                green,
            ),
            (
                format!("Each settled tx pays you a {}% COW fee", fee_pct),
                grey,
            ),
            (
                "Build more towers to protect and settle txs faster".into(),
                grey,
            ),
        ],
        StatKind::Protected => [
            ("Total COW value shielded from MEV extraction".into(), green),
            (
                "Saved by CoW mechanisms: batching, matching, dark pools".into(),
                grey,
            ),
            ("".into(), white),
        ],
        StatKind::Extracted => [
            (
                "COW value drained by MEV bots from your transactions".into(),
                red,
            ),
            (
                "Reach zero by intercepting every bot before it drains a tx".into(),
                grey,
            ),
            ("".into(), white),
        ],
        StatKind::Balance => [
            (
                "Your spendable COW — build and upgrade defense towers".into(),
                white,
            ),
            (
                format!("Income: {}% settlement fee on each confirmed tx", fee_pct),
                green,
            ),
            (
                "Spend: tower placements (130–220 COW) & upgrades".into(),
                grey,
            ),
        ],
        StatKind::BaseFee => {
            let net_color = match network.level {
                0 => Color::srgb(0.40, 0.90, 0.55),
                1 => Color::srgb(1.00, 0.75, 0.20),
                _ => red,
            };
            [
                (network.status_line().to_string(), net_color),
                (
                    format!(
                        "Txs per second: {}   Tx speed: {}%",
                        network.txs_per_sec_str(),
                        network.speed_pct()
                    ),
                    grey,
                ),
                (
                    "Shifts ±1 level each block — affects tx flow & speed".into(),
                    grey,
                ),
            ]
        }
    };

    for (line, mut text, mut color) in &mut line_q {
        if std::mem::discriminant(&line.0) != std::mem::discriminant(&active_kind) {
            continue;
        }
        let (t, c) = &lines[line.1 as usize];
        text.0 = t.clone();
        color.0 = *c;
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
