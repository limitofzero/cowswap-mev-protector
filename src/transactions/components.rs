use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Eth, Usdt, Usdc, Cow, Dai, Wbtc,
}

impl TokenType {
    pub const ALL: [TokenType; 6] = [
        TokenType::Eth, TokenType::Usdt, TokenType::Usdc,
        TokenType::Cow, TokenType::Dai,  TokenType::Wbtc,
    ];

    /// How many COW tokens 1 unit of this token is worth.
    pub fn cow_rate(self) -> f32 {
        match self {
            TokenType::Eth  => 5_000.0,
            TokenType::Wbtc => 100_000.0,
            TokenType::Cow  => 1.0,
            TokenType::Usdt | TokenType::Usdc | TokenType::Dai => 2.0,
        }
    }

    pub fn color(self) -> Color {
        match self {
            TokenType::Eth  => Color::srgb(0.38, 0.47, 0.86),
            TokenType::Usdt => Color::srgb(0.06, 0.69, 0.44),
            TokenType::Usdc => Color::srgb(0.16, 0.47, 0.88),
            TokenType::Cow  => Color::srgb(0.51, 0.35, 0.82),
            TokenType::Dai  => Color::srgb(0.96, 0.65, 0.13),
            TokenType::Wbtc => Color::srgb(0.95, 0.58, 0.18),
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            TokenType::Eth  => "ETH",
            TokenType::Usdt => "USDT",
            TokenType::Usdc => "USDC",
            TokenType::Cow  => "COW",
            TokenType::Dai  => "DAI",
            TokenType::Wbtc => "WBTC",
        }
    }

    /// (min, max) native token amount spawned per transaction.
    pub fn amount_range(self) -> (f32, f32) {
        match self {
            TokenType::Eth  => (0.05, 5.0),
            TokenType::Wbtc => (0.001, 0.5),
            TokenType::Cow  => (100.0, 10_000.0),
            TokenType::Usdt | TokenType::Usdc | TokenType::Dai => (100.0, 5_000.0),
        }
    }

    pub fn sprite_path(self) -> &'static str {
        "tx_usdc.png"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImmunitySource { CoWMatch, DarkPool }

/// Marker for the amount label child entity on a transaction.
#[derive(Component)]
pub struct TxAmountLabel;

/// Marker for the colored border highlight child sprite on a transaction.
#[derive(Component)]
pub struct TxHighlight;

/// All mutable state lives here — no component inserts/removes, no archetype changes.
#[derive(Component)]
pub struct Transaction {
    pub progress: f32,
    pub speed: f32,
    pub value: f32,
    pub remaining_value: f32,
    /// Active MEV immunity, if any.
    pub immunity: Option<(Timer, ImmunitySource)>,
    /// Batch membership: (batch_id, batch_size).
    pub batch: Option<(u32, u32)>,
}

impl Transaction {
    pub fn new(value: f32, speed: f32) -> Self {
        Self {
            progress: 0.0,
            speed,
            value,
            remaining_value: value,
            immunity: None,
            batch: None,
        }
    }

    pub fn grant_immunity(&mut self, secs: f32, source: ImmunitySource) {
        self.immunity = Some((Timer::from_seconds(secs, TimerMode::Once), source));
    }

    pub fn set_batch(&mut self, id: u32, size: u32) {
        self.batch = Some((id, size));
    }

    pub fn is_immune(&self) -> bool { self.immunity.is_some() }
    pub fn is_batched(&self) -> bool { self.batch.is_some() }

    pub fn tick_immunity(&mut self, delta: std::time::Duration) {
        if let Some((timer, _)) = &mut self.immunity {
            timer.tick(delta);
            if timer.just_finished() {
                self.immunity = None;
            }
        }
    }

    pub fn value_extracted(&self) -> f32 { self.value - self.remaining_value }
    pub fn is_worthless(&self) -> bool { self.remaining_value <= 0.0 }
}
