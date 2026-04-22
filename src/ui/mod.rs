use bevy::prelude::*;

use crate::{game::GameState, resources::GameScore};

pub struct UiPlugin;

#[derive(Component)]
struct SettledText;
#[derive(Component)]
struct ProtectedText;
#[derive(Component)]
struct ExtractedText;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(
                Update,
                update_hud.run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_hud(mut commands: Commands) {
    // Full-screen transparent overlay
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(14.0)),
                ..default()
            },
            Name::new("HUD"),
        ))
        .with_children(|root| {
            // ── Top bar ──────────────────────────────────────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(40.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|bar| {
                bar.spawn((
                    Text::new("Settled: 0"),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(Color::WHITE),
                    SettledText,
                ));
                bar.spawn((
                    Text::new("Protected: 0.00 ETH"),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(Color::srgb(0.30, 1.00, 0.45)),
                    ProtectedText,
                ));
                bar.spawn((
                    Text::new("Extracted: 0.00 ETH"),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(Color::srgb(1.00, 0.35, 0.35)),
                    ExtractedText,
                ));
            });

            // ── Bottom legend ─────────────────────────────────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(24.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|bar| {
                legend_entry(bar, Color::srgb(0.9, 0.65, 0.05), "Tx");
                legend_entry(bar, Color::srgb(0.9, 0.15, 0.15), "Frontrunner");
                legend_entry(bar, Color::srgb(0.95, 0.50, 0.05), "Sandwich");
                legend_entry(bar, Color::srgb(0.15, 0.80, 0.40), "BatchTower");
                legend_entry(bar, Color::srgb(0.20, 0.55, 0.95), "CoW Tower");
                legend_entry(bar, Color::srgb(0.20, 0.90, 0.90), "Shielded Tx");
            });
        });
}

fn legend_entry(parent: &mut ChildBuilder, color: Color, label: &str) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(5.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            // Colour swatch
            row.spawn((
                Node {
                    width: Val::Px(12.0),
                    height: Val::Px(12.0),
                    ..default()
                },
                BackgroundColor(color),
            ));
            row.spawn((
                Text::new(label),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.75, 0.75, 0.75)),
            ));
        });
}

fn update_hud(
    score: Res<GameScore>,
    mut settled_q: Query<
        &mut Text,
        (With<SettledText>, Without<ProtectedText>, Without<ExtractedText>),
    >,
    mut protected_q: Query<
        &mut Text,
        (With<ProtectedText>, Without<SettledText>, Without<ExtractedText>),
    >,
    mut extracted_q: Query<
        &mut Text,
        (With<ExtractedText>, Without<SettledText>, Without<ProtectedText>),
    >,
) {
    if !score.is_changed() {
        return;
    }
    if let Ok(mut t) = settled_q.get_single_mut() {
        t.0 = format!("Settled: {}", score.txs_settled);
    }
    if let Ok(mut t) = protected_q.get_single_mut() {
        t.0 = format!("Protected: {:.2} ETH", score.value_protected);
    }
    if let Ok(mut t) = extracted_q.get_single_mut() {
        t.0 = format!("Extracted: {:.2} ETH", score.value_extracted);
    }
}
