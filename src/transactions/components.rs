use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Eth,
    Usdt,
    Usdc,
    Cow,
    Dai,
    Wbtc,
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

/// A pending transaction moving through the mempool river.
#[derive(Component)]
pub struct Transaction {
    /// Normalised progress along the path `[0.0 … 1.0]`.
    pub progress: f32,
    /// Progress units advanced per second.
    pub speed: f32,
    /// Original ETH value submitted.
    pub value: f32,
    /// Value remaining after MEV extraction.
    pub remaining_value: f32,
}

impl Transaction {
    pub fn new(value: f32, speed: f32) -> Self {
        Self {
            progress: 0.0,
            speed,
            value,
            remaining_value: value,
        }
    }

    pub fn value_extracted(&self) -> f32 {
        self.value - self.remaining_value
    }

    pub fn is_worthless(&self) -> bool {
        self.remaining_value <= 0.0
    }
}

/// Marks a transaction as part of a CoW batch auction.
/// Batched transactions cannot be individually front-run.
#[derive(Component)]
pub struct Batched {
    pub batch_id: u32,
    pub batch_size: u32,
}

/// Temporary MEV immunity shield.
/// Enemies cannot extract value from a shielded transaction.
#[derive(Component)]
pub struct MevImmunity {
    pub duration: Timer,
    pub source: ImmunitySource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImmunitySource {
    /// Granted by a CoW Matcher tower pairing a buy + sell tx.
    CoWMatch,
    /// Granted by a Dark Pool Node tower temporarily hiding the tx.
    DarkPool,
    /// Granted by a Commit-Reveal Beacon tower.
    CommitReveal,
}
