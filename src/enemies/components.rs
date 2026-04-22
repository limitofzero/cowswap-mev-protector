use bevy::prelude::*;

/// All MEV bot archetypes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnemyType {
    /// Copies a tx and inserts ahead — fast, races to the front.
    Frontrunner,
    /// Captures arbitrage after a tx settles — slow, follows behind.
    Backrunner,
    /// Two units bracket a tx — spawned in pairs, flanking from both sides.
    SandwichBot,
    /// Clones any profitable tx — adaptive, hard to counter without batching.
    GeneralizedFrontrunner,
    /// Hunts low-value ("low-collateral") transactions specifically.
    Liquidator,
    /// Appears only near the settlement zone; steals fees at the last second.
    JitLp,
}

impl EnemyType {
    pub fn move_speed(&self) -> f32 {
        match self {
            EnemyType::Frontrunner => 130.0,
            EnemyType::Backrunner => 55.0,
            EnemyType::SandwichBot => 90.0,
            EnemyType::GeneralizedFrontrunner => 110.0,
            EnemyType::Liquidator => 75.0,
            EnemyType::JitLp => 160.0,
        }
    }

    /// ETH extracted from target per second when in attack range.
    pub fn extract_rate(&self) -> f32 {
        match self {
            EnemyType::Frontrunner => 0.18,
            EnemyType::Backrunner => 0.08,
            EnemyType::SandwichBot => 0.14,
            EnemyType::GeneralizedFrontrunner => 0.22,
            EnemyType::Liquidator => 0.30,
            EnemyType::JitLp => 0.12,
        }
    }

    pub fn attack_range(&self) -> f32 {
        match self {
            EnemyType::JitLp => 40.0, // only attacks when very close to settlement
            _ => 65.0,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            EnemyType::Frontrunner => Color::srgb(0.90, 0.15, 0.15),
            EnemyType::Backrunner => Color::srgb(0.65, 0.10, 0.50),
            EnemyType::SandwichBot => Color::srgb(0.95, 0.50, 0.05),
            EnemyType::GeneralizedFrontrunner => Color::srgb(1.00, 0.05, 0.30),
            EnemyType::Liquidator => Color::srgb(0.55, 0.00, 0.60),
            EnemyType::JitLp => Color::srgb(0.85, 0.85, 0.05),
        }
    }

    pub fn size(&self) -> f32 {
        match self {
            EnemyType::GeneralizedFrontrunner => 22.0,
            EnemyType::Liquidator => 20.0,
            _ => 18.0,
        }
    }
}

/// The MEV bot entity component.
#[derive(Component)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub speed: f32,
    /// ETH extracted per second when attacking.
    pub extract_rate: f32,
    pub attack_range: f32,
    /// Current attack target (a Transaction entity).
    pub target: Option<Entity>,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType) -> Self {
        Self {
            speed: enemy_type.move_speed(),
            extract_rate: enemy_type.extract_rate(),
            attack_range: enemy_type.attack_range(),
            target: None,
            enemy_type,
        }
    }
}

/// Links the two halves of a sandwich attack.
#[derive(Component)]
pub struct SandwichPair {
    pub partner: Option<Entity>,
    /// `true` = the "front" unit (inserts before target), `false` = "back" unit.
    pub is_front: bool,
}
