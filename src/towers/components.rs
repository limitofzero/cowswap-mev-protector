use bevy::prelude::*;

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
    /// Withholds tx details until the last moment — bots can't act on hidden info.
    CommitRevealBeacon,
}

impl TowerType {
    pub fn range(&self) -> f32 {
        match self {
            TowerType::BatchAuctioneer => 130.0,
            TowerType::CoWMatcher => 110.0,
            TowerType::Solver => 85.0,
            TowerType::SlippageGuard => 95.0,
            TowerType::DarkPoolNode => 75.0,
            TowerType::CommitRevealBeacon => 115.0,
        }
    }

    /// Seconds between activations.
    pub fn cooldown_secs(&self) -> f32 {
        match self {
            TowerType::BatchAuctioneer => 2.5,
            TowerType::CoWMatcher => 3.5,
            TowerType::Solver => 1.5,
            TowerType::SlippageGuard => 0.8,
            TowerType::DarkPoolNode => 10.0, // OP if spammed — high cooldown
            TowerType::CommitRevealBeacon => 4.0,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            TowerType::BatchAuctioneer => Color::srgb(0.15, 0.80, 0.40),
            TowerType::CoWMatcher => Color::srgb(0.20, 0.55, 0.95),
            TowerType::Solver => Color::srgb(0.40, 0.90, 0.45),
            TowerType::SlippageGuard => Color::srgb(0.85, 0.85, 0.20),
            TowerType::DarkPoolNode => Color::srgb(0.25, 0.25, 0.75),
            TowerType::CommitRevealBeacon => Color::srgb(0.80, 0.35, 0.85),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TowerType::BatchAuctioneer => "Batch",
            TowerType::CoWMatcher => "CoW",
            TowerType::Solver => "Solver",
            TowerType::SlippageGuard => "Slippage",
            TowerType::DarkPoolNode => "DarkPool",
            TowerType::CommitRevealBeacon => "C-Reveal",
        }
    }
}

/// A placed defense tower.
#[derive(Component)]
pub struct Tower {
    pub tower_type: TowerType,
    pub range: f32,
    pub cooldown: Timer,
}

impl Tower {
    pub fn new(tower_type: TowerType) -> Self {
        let range = tower_type.range();
        let secs = tower_type.cooldown_secs();
        Self {
            range,
            cooldown: Timer::from_seconds(secs, TimerMode::Repeating),
            tower_type,
        }
    }
}
