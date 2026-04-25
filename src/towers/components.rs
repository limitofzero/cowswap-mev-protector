use bevy::prelude::*;

/// Cycles through spritesheet frames at a fixed interval.
#[derive(Component)]
pub struct AnimationTimer {
    pub timer: Timer,
    /// How many frames to cycle through.
    pub frames: usize,
    /// Atlas index of the first frame in this animation strip.
    pub base: usize,
}

impl AnimationTimer {
    pub fn new(fps: f32, frames: usize) -> Self {
        Self {
            timer: Timer::from_seconds(1.0 / fps, TimerMode::Repeating),
            frames,
            base: 0,
        }
    }

    pub fn new_with_offset(fps: f32, frames: usize, base: usize) -> Self {
        Self {
            timer: Timer::from_seconds(1.0 / fps, TimerMode::Repeating),
            frames,
            base,
        }
    }
}

/// All CoW-protocol-based defense tower types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TowerType {
    /// Groups txs into batches — frontrunners cannot target individual txs in a batch.
    BatchAuctioneer,
    /// Pairs a buy tx with a sell tx — CoW-matched pairs become MEV-immune.
    CoWMatcher,
    /// Finds the optimal settlement route — reduces the extractable value per tx.
    Solver,
    /// Tightens slippage tolerance — reduces the profit window for sandwich bots.
    SlippageGuard,
    /// Routes txs through a private mempool — temporarily invisible to bots.
    /// Has a long cooldown to prevent spamming.
    DarkPoolNode,
}

impl TowerType {
    pub fn range(&self) -> f32 {
        match self {
            TowerType::BatchAuctioneer => 130.0,
            TowerType::CoWMatcher => 110.0,
            TowerType::Solver => 85.0,
            TowerType::SlippageGuard => 95.0,
            TowerType::DarkPoolNode => 75.0,
        }
    }

    /// Seconds between activations.
    pub fn cooldown_secs(&self) -> f32 {
        match self {
            TowerType::BatchAuctioneer => 2.5,
            TowerType::CoWMatcher => 3.5,
            TowerType::Solver => 1.5,
            TowerType::SlippageGuard => 0.8,
            TowerType::DarkPoolNode => 10.0,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            TowerType::BatchAuctioneer => Color::srgb(0.15, 0.80, 0.40),
            TowerType::CoWMatcher => Color::srgb(0.20, 0.55, 0.95),
            TowerType::Solver => Color::srgb(0.40, 0.90, 0.45),
            TowerType::SlippageGuard => Color::srgb(0.85, 0.85, 0.20),
            TowerType::DarkPoolNode => Color::srgb(0.25, 0.25, 0.75),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TowerType::BatchAuctioneer => "Batch",
            TowerType::CoWMatcher => "CoW",
            TowerType::Solver => "Solver",
            TowerType::SlippageGuard => "Slippage",
            TowerType::DarkPoolNode => "DarkPool",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            TowerType::BatchAuctioneer => "BA",
            TowerType::CoWMatcher => "CoW",
            TowerType::Solver => "SLV",
            TowerType::SlippageGuard => "SG",
            TowerType::DarkPoolNode => "DP",
        }
    }

    pub fn cost(&self) -> f32 {
        match self {
            TowerType::BatchAuctioneer => 150.0,
            TowerType::CoWMatcher => 200.0,
            TowerType::Solver => 180.0,
            TowerType::SlippageGuard => 130.0,
            TowerType::DarkPoolNode => 220.0,
        }
    }

    /// Index into the shared tower sprite sheets (cow=0, ba=1, slv=2, sg=3, dp=4).
    pub fn atlas_index(&self) -> usize {
        match self {
            TowerType::CoWMatcher => 0,
            TowerType::BatchAuctioneer => 1,
            TowerType::Solver => 2,
            TowerType::SlippageGuard => 3,
            TowerType::DarkPoolNode => 4,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TowerType::CoWMatcher => "Finds matching orders and\ngrants MEV immunity for 6s.",
            TowerType::BatchAuctioneer => {
                "Batches nearby txs together.\nEach extra tx dilutes enemy drain."
            }
            TowerType::Solver => "Fires projectiles at bots\nto reduce their HP.",
            TowerType::SlippageGuard => {
                "Slows enemies inside its range\ndown to 35% movement speed."
            }
            TowerType::DarkPoolNode => "Hides txs from bots with a\ndark pool shield for 4s.",
        }
    }

    pub fn stats_line(&self) -> String {
        format!(
            "Range {:.0}  CD {:.1}s  Cost {:.0}c",
            self.range(),
            self.cooldown_secs(),
            self.cost()
        )
    }

    /// Cost to upgrade from `current_level` to `current_level + 1`.
    /// Formula: base_cost * 0.5 * 1.5^current_level
    pub fn upgrade_cost(&self, current_level: u8) -> f32 {
        self.cost() * 0.5 * 1.5_f32.powi(current_level as i32)
    }

    /// Solver: projectile damage multiplier at upgrade level.
    pub fn solver_damage_mult(&self, level: u8) -> f32 {
        1.0 + 0.10 * level as f32
    }

    /// Solver: absolute HP damage per projectile at the given upgrade level.
    pub fn solver_damage(&self, level: u8) -> f32 {
        SOLVER_BASE_DAMAGE * self.solver_damage_mult(level)
    }

    /// CoW: fraction of incoming drain blocked (0.0 = none, 0.3 = 30% at max).
    pub fn cow_drain_resist(&self, level: u8) -> f32 {
        0.10 * level as f32
    }

    /// Effective cooldown in seconds after upgrades.
    pub fn cooldown_secs_upgraded(&self, level: u8) -> f32 {
        let reduction = match self {
            TowerType::BatchAuctioneer => 0.30 * level as f32,
            // DP: Lv1 -0.5s, Lv2 -0.5-0.7=1.2s, Lv3 -0.5-0.7-0.9=2.1s
            TowerType::DarkPoolNode => [0.0_f32, 0.5, 1.2, 2.1][level.min(3) as usize],
            _ => 0.0,
        };
        (self.cooldown_secs() - reduction).max(0.2)
    }

    /// SlippageGuard: effective slow-to speed percentage (e.g. 35 = 35% speed).
    pub fn slow_pct(&self, level: u8) -> u32 {
        35u32.saturating_sub(5 * level as u32).max(10)
    }

    /// One-line description of what each upgrade level adds.
    pub fn upgrade_effect_desc(&self, level: u8) -> String {
        match self {
            TowerType::Solver => format!("+{}% projectile damage", level * 10),
            TowerType::CoWMatcher => format!("+{}% drain resistance", level * 10),
            TowerType::BatchAuctioneer => format!("-{:.1}s cooldown", 0.30 * level as f32),
            TowerType::DarkPoolNode => match level {
                1 => "-0.5s cooldown".into(),
                2 => "-0.7s cooldown".into(),
                3 => "-0.9s cooldown".into(),
                _ => String::new(),
            },
            TowerType::SlippageGuard => format!("+{}% slow intensity", level * 10),
        }
    }
}

pub const MAX_UPGRADE_LEVEL: u8 = 3;
pub const SOLVER_BASE_DAMAGE: f32 = 50.0;
/// Seconds a tower is locked from upgrading after placement or upgrade (prevents misclicks).
pub const UPGRADE_LOCK_SECS: f32 = 2.5;

/// Tracks the last rendered upgrade level so the sprite sync system only re-runs on change.
#[derive(Component)]
pub struct TowerVisualLevel(pub u8);

/// Semi-transparent preview sprite showing the next upgrade level; child of a Tower entity.
#[derive(Component)]
pub struct UpgradePreview;

/// Marks the range fill/border children — hidden unless the tower is hovered.
#[derive(Component)]
pub struct TowerRangeVisual;

/// Marks the semi-transparent ghost tower that follows the cursor during placement.
#[derive(Component)]
pub struct GhostTower(pub TowerType);

/// Marks the delete-icon sprite that follows the cursor during remove mode.
#[derive(Component)]
pub struct DeleteCursor;

/// Attached to each tower shop button in the HUD.
#[derive(Component)]
pub struct TowerShopButton(pub TowerType);

/// A homing projectile fired by the Solver tower.
#[derive(Component)]
pub struct Projectile {
    pub target: Entity,
    pub speed: f32,
    pub damage: f32,
}

/// One-shot hit animation — advances each tick and despawns after all frames.
#[derive(Component)]
pub struct HitEffect {
    pub timer: Timer,
    pub frames: usize,
    pub frame: usize,
}

/// A placed defense tower.
#[derive(Component)]
pub struct Tower {
    pub tower_type: TowerType,
    pub upgrade_level: u8,
    pub range: f32,
    pub cooldown: Timer,
    /// Seconds remaining before the next upgrade is allowed (prevents misclick on placement/upgrade).
    pub upgrade_cooldown: f32,
}

impl Tower {
    pub fn new(tower_type: TowerType) -> Self {
        let range = tower_type.range();
        let secs = tower_type.cooldown_secs();
        Self {
            range,
            upgrade_level: 0,
            cooldown: Timer::from_seconds(secs, TimerMode::Repeating),
            upgrade_cooldown: UPGRADE_LOCK_SECS,
            tower_type,
        }
    }

    /// Apply the next upgrade level: bump the counter and update affected stats.
    pub fn apply_upgrade(&mut self) {
        if self.upgrade_level >= MAX_UPGRADE_LEVEL {
            return;
        }
        self.upgrade_level += 1;
        let new_cd = self.tower_type.cooldown_secs_upgraded(self.upgrade_level);
        self.cooldown = Timer::from_seconds(new_cd, TimerMode::Repeating);
        self.upgrade_cooldown = UPGRADE_LOCK_SECS;
    }

    pub fn can_upgrade(&self) -> bool {
        self.upgrade_level < MAX_UPGRADE_LEVEL && self.upgrade_cooldown <= 0.0
    }
}
