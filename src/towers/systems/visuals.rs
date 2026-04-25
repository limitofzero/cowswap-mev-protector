use bevy::prelude::*;

use crate::resources::{GameEconomy, PlacementMode};

use super::super::components::{
    AnimationTimer, Tower, TowerRangeVisual, TowerVisualLevel, UpgradePreview,
};
use super::super::resources::TowerAssets;
use super::TOWER_INTERACT_RADIUS;

type UpgradePreviewQ<'w, 's> = Query<
    'w,
    's,
    (&'static mut Visibility, &'static mut AnimationTimer),
    (With<UpgradePreview>, Without<TowerVisualLevel>),
>;

/// Advance sprite animation frames for all animated entities.
/// Skips entities whose atlas index is currently outside the animation strip
/// (i.e. a status frame has been applied and should not be overwritten).
pub fn animate_sprites(time: Res<Time>, mut query: Query<(&mut AnimationTimer, &mut Sprite)>) {
    for (mut anim, mut sprite) in &mut query {
        anim.timer.tick(time.delta());
        if anim.timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            // Don't animate over a status frame that lives beyond our strip.
            if atlas.index >= anim.base + anim.frames {
                continue;
            }
            let local = atlas.index.saturating_sub(anim.base);
            atlas.index = anim.base + (local + 1) % anim.frames;
        }
    }
}

/// Show range circles only for the tower the cursor is currently over.
pub fn update_tower_range_visibility(
    tower_q: Query<(&Transform, &Children), With<Tower>>,
    mut visual_q: Query<&mut Visibility, With<TowerRangeVisual>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let cursor = windows
        .single()
        .ok()
        .and_then(|w| w.cursor_position())
        .and_then(|screen_pos| {
            camera_q
                .single()
                .ok()
                .and_then(|(cam, cam_t)| cam.viewport_to_world_2d(cam_t, screen_pos).ok())
        });

    for (tower_t, children) in &tower_q {
        let hovered = cursor
            .is_some_and(|cursor_pos| cursor_pos.distance(tower_t.translation.truncate()) < TOWER_INTERACT_RADIUS);
        for &child in children {
            if let Ok(mut vis) = visual_q.get_mut(child) {
                *vis = if hovered {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

/// Sync the tower sprite to the correct upgrade row when upgrade_level changes.
/// Uses TowerVisualLevel to detect changes without firing every frame.
pub fn sync_tower_upgrade_visuals(
    tower_assets: Res<TowerAssets>,
    mut tower_q: Query<
        (
            &Tower,
            &mut TowerVisualLevel,
            &mut Sprite,
            &mut AnimationTimer,
        ),
        Without<UpgradePreview>,
    >,
) {
    let Some(layout) = tower_assets.upgrade_layout.clone() else {
        return;
    };
    for (tower, mut vis_level, mut sprite, mut anim) in &mut tower_q {
        if vis_level.0 == tower.upgrade_level {
            continue;
        }
        let level = tower.upgrade_level;
        vis_level.0 = level;
        let base = level as usize * 6;
        anim.base = base;
        anim.timer.reset();
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = base;
            atlas.layout = layout.clone();
        }
        if let Some(sheet) = tower_assets.upgrade_sheet(&tower.tower_type) {
            sprite.image = sheet;
        }
    }
}

/// Show the next-level upgrade preview sprite on the hovered tower when the player can afford it.
/// The preview uses the same AnimationTimer fps as the tower so it animates at the same speed.
pub fn update_upgrade_preview(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    economy: Res<GameEconomy>,
    tower_q: Query<(&Tower, &Transform, &Children)>,
    mut preview_q: UpgradePreviewQ,
) {
    let cursor = windows
        .single()
        .ok()
        .and_then(|w| w.cursor_position())
        .and_then(|screen_pos| {
            camera_q
                .single()
                .ok()
                .and_then(|(cam, cam_t)| cam.viewport_to_world_2d(cam_t, screen_pos).ok())
        });

    for (tower, tower_t, children) in &tower_q {
        let hovered = cursor
            .is_some_and(|cursor_pos| cursor_pos.distance(tower_t.translation.truncate()) < TOWER_INTERACT_RADIUS);
        let can_afford = tower.can_upgrade()
            && economy.balance >= tower.tower_type.upgrade_cost(tower.upgrade_level);

        for &child in children {
            let Ok((mut vis, mut anim)) = preview_q.get_mut(child) else {
                continue;
            };
            if hovered && can_afford {
                let next_base = (tower.upgrade_level as usize + 1) * 6;
                if anim.base != next_base {
                    anim.base = next_base;
                    // Reset to start of the new level row
                    anim.timer.reset();
                }
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

/// Left-click a hovered tower in Idle mode to purchase the next upgrade level.
pub fn handle_tower_upgrade_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    placement_mode: Res<PlacementMode>,
    mut economy: ResMut<GameEconomy>,
    mut tower_q: Query<(&mut Tower, &Transform)>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if *placement_mode != PlacementMode::Idle {
        return;
    }

    let Ok(win) = windows.single() else { return };
    let Ok((cam, cam_t)) = camera_q.single() else {
        return;
    };
    let Some(cursor) = win
        .cursor_position()
        .and_then(|screen_pos| cam.viewport_to_world_2d(cam_t, screen_pos).ok())
    else {
        return;
    };

    // Ignore clicks in the bottom bar UI area
    let bot_edge = -win.height() * 0.5 + crate::ui::BOT_BAR_H;
    if cursor.y < bot_edge {
        return;
    }

    for (mut tower, tower_t) in &mut tower_q {
        if tower_t.translation.truncate().distance(cursor) < TOWER_INTERACT_RADIUS {
            if !tower.can_upgrade() {
                continue;
            }
            let cost = tower.tower_type.upgrade_cost(tower.upgrade_level);
            if economy.balance < cost {
                return;
            }
            economy.balance -= cost;
            tower.apply_upgrade();
            return;
        }
    }
}
