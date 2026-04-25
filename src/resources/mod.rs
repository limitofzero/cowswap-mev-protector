use bevy::prelude::*;

use crate::towers::TowerType;

/// 1 COW = this many USD (used for display only; all game math stays in COW).
pub const COW_USD_RATE: f32 = 0.15;

/// Running totals for the current game session.
#[derive(Resource, Default)]
pub struct GameScore {
    pub txs_settled: u32,
    pub value_protected: f32,
    pub value_extracted: f32,
}

/// Player economy: balance in COW tokens, fee earned per settled tx.
#[derive(Resource)]
pub struct GameEconomy {
    /// Current COW balance (spendable on towers).
    pub balance: f32,
    /// Fraction of a tx's COW value paid as fee on settlement (e.g. 0.01 = 1%).
    pub fee_rate: f32,
}

impl Default for GameEconomy {
    fn default() -> Self {
        Self {
            balance: 300.0,
            fee_rate: 0.01,
        }
    }
}

/// Tracks which wave of enemies we're on and their spawn timing.
#[derive(Resource)]
pub struct WaveState {
    pub wave: u32,
    pub spawn_timer: Timer,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            wave: 1,
            spawn_timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        }
    }
}

/// Whether the player is currently placing or removing a tower.
#[derive(Resource, Default, PartialEq)]
pub enum PlacementMode {
    #[default]
    Idle,
    Placing(TowerType),
    Removing,
}

/// Whether the game is currently paused (Space toggles this during Playing).
#[derive(Resource, Default)]
pub struct PauseState {
    pub paused: bool,
}

/// Run condition: true while the game is NOT paused.
pub fn not_paused(ps: Res<PauseState>) -> bool {
    !ps.paused
}

/// Current Ethereum-like network congestion level (0 = free, 1 = busy, 2 = very busy).
/// Changes by at most ±1 per block starting from wave 4. Slows transaction progress.
#[derive(Resource)]
pub struct NetworkLoad {
    pub level: u8,
    seed: u64,
}

impl Default for NetworkLoad {
    fn default() -> Self {
        Self {
            level: 0,
            seed: 0xc0ff_eede_adbe_ef00,
        }
    }
}

impl NetworkLoad {
    fn rng(&mut self) -> u64 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        self.seed
    }

    /// Called once per block when wave >= 4. Randomly steps level ±1.
    /// Distribution: 50% increase, 25% stay, 25% decrease — biased upward so
    /// level 2 is reachable within a normal game session.
    pub fn tick_block(&mut self, wave: u32) {
        if wave < 4 {
            return;
        }
        match (self.rng() % 4) as u8 {
            0 => self.level = self.level.saturating_sub(1), // 25% down
            1 => {}                                         // 25% stay
            _ => self.level = (self.level + 1).min(2),      // 50% up
        }
    }

    /// Speed multiplier applied to all transaction progress each frame.
    pub fn speed_mult(&self) -> f32 {
        match self.level {
            0 => 1.00,
            1 => 0.90,
            _ => 0.75,
        }
    }

    pub fn label(&self) -> &'static str {
        match self.level {
            0 => "LOW",
            1 => "BUSY",
            _ => "HIGH",
        }
    }

    /// Seconds between transaction spawns — single source of truth for spawn rate.
    pub fn spawn_interval(&self) -> f32 {
        match self.level {
            0 => 3.0,
            1 => 2.0,
            _ => 1.0,
        }
    }

    /// Derived: tx/s as a display string (e.g. "0.33", "0.5", "1").
    pub fn txs_per_sec_str(&self) -> String {
        let rate = 1.0 / self.spawn_interval();
        if rate < 1.0 {
            format!("{:.2}", rate)
        } else {
            format!("{:.0}", rate)
        }
    }

    /// Derived: speed percentage shown to the player (e.g. 100, 90, 75).
    pub fn speed_pct(&self) -> u8 {
        (self.speed_mult() * 100.0).round() as u8
    }

    /// Derived: how much speed is lost (for the "-X%" display in the stat bar).
    pub fn speed_loss_pct(&self) -> u8 {
        100 - self.speed_pct()
    }

    pub fn status_line(&self) -> &'static str {
        match self.level {
            0 => "Network is almost free",
            1 => "Network is busy",
            _ => "Network is very busy",
        }
    }
}

pub struct GameResourcesPlugin;

impl Plugin for GameResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameScore>()
            .init_resource::<GameEconomy>()
            .init_resource::<PlacementMode>()
            .init_resource::<PauseState>()
            .init_resource::<WaveState>()
            .init_resource::<NetworkLoad>();
    }
}
