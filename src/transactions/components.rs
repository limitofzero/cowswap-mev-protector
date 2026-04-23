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

    pub fn sprite_path(self) -> &'static str {
        match self {
            TokenType::Eth  => "tx_eth.png",
            TokenType::Usdt => "tx_usdt.png",
            TokenType::Usdc => "tx_usdc.png",
            TokenType::Cow  => "tx_cow.png",
            TokenType::Dai  => "tx_dai.png",
            TokenType::Wbtc => "tx_wbtc.png",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImmunitySource { CoWMatch, DarkPool, CommitReveal }

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
