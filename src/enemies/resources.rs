use bevy::prelude::*;

use super::components::EnemyType;

/// Pre-loaded handles for all enemy sprites and the shared atlas layout.
#[derive(Resource, Default)]
pub struct EnemyAssets {
    pub layout: Option<Handle<TextureAtlasLayout>>,
    pub frontrunner: Option<Handle<Image>>,
    pub backrunner:  Option<Handle<Image>>,
    pub sandwich:    Option<Handle<Image>>,
    pub jitlp:       Option<Handle<Image>>,
}

impl EnemyAssets {
    pub fn texture(&self, enemy_type: &EnemyType) -> Option<Handle<Image>> {
        match enemy_type {
            EnemyType::Frontrunner => self.frontrunner.clone(),
            EnemyType::Backrunner  => self.backrunner.clone(),
            EnemyType::SandwichBot => self.sandwich.clone(),
            EnemyType::JitLp       => self.jitlp.clone(),
        }
    }
}

#[derive(Resource)]
pub struct WaveManager {
    pub wave: u32,
    /// Max active enemies allowed this wave: grows 3 + wave * 2, capped at 20.
    pub wave_target: u32,
    /// One-shot 5 s countdown before the first block.
    pub first_block_timer: Timer,
    pub first_block_done: bool,
    /// Fires every 15 s (Ethereum block time) after the first wave.
    pub block_timer: Timer,
    /// Staggers individual spawns so they don't all appear at once.
    pub spawn_timer: Timer,
    seed: u64,
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            wave: 0,
            wave_target: 0,
            first_block_timer: Timer::from_seconds(5.0, TimerMode::Once),
            first_block_done: false,
            block_timer: Timer::from_seconds(15.0, TimerMode::Repeating),
            spawn_timer: Timer::from_seconds(2.5, TimerMode::Repeating),
            seed: 0xfeed_face_dead_beef,
        }
    }
}

impl WaveManager {
    fn rng(&mut self) -> u64 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        self.seed
    }

    pub fn rand_spawn_pos(&mut self) -> Vec2 {
        const ZONES: &[Vec2] = &[
            Vec2::new(-620.0,  130.0),
            Vec2::new(-620.0, -180.0),
            Vec2::new( 620.0,  130.0),
            Vec2::new( 620.0, -180.0),
            Vec2::new(  20.0,  400.0),
            Vec2::new( -20.0, -400.0),
            Vec2::new(-340.0,  400.0),
            Vec2::new( 340.0, -400.0),
        ];
        let i = (self.rng() as usize) % ZONES.len();
        ZONES[i]
    }

    /// Advance to the next wave and update the active-enemy target.
    /// Ramp: 2, 2, 3, 4, 5, 6 … capped at 20.
    pub fn next_wave(&mut self) {
        self.wave += 1;
        self.wave_target = if self.wave <= 2 { 2 } else { self.wave.min(20) };
    }

    /// Pick one enemy type based on current wave difficulty.
    pub fn pick_enemy(&mut self) -> EnemyType {
        let roll = (self.rng() % 100) as u32;
        let difficulty = self.wave.min(10);
        if roll < 40_u32.saturating_sub(difficulty * 3) {
            EnemyType::Frontrunner
        } else if roll < 65_u32.saturating_sub(difficulty) {
            EnemyType::Backrunner
        } else if roll < 82 {
            EnemyType::SandwichBot
        } else {
            EnemyType::JitLp
        }
    }
}
