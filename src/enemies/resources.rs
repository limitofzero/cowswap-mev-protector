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
    /// Fires every 15 s (Ethereum block time) — triggers the next wave unconditionally.
    pub block_timer: Timer,
    /// Staggers individual enemy spawns within a wave.
    pub spawn_timer: Timer,
    pub pending: std::collections::VecDeque<EnemyType>,
    seed: u64,
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            wave: 0,
            block_timer: Timer::from_seconds(15.0, TimerMode::Repeating),
            spawn_timer: Timer::from_seconds(1.2, TimerMode::Repeating),
            pending: Default::default(),
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

    pub fn build_wave(&mut self) {
        self.wave += 1;
        self.pending.clear();

        let count = (2 + self.wave as usize).min(10);

        for i in 0..count {
            let roll = (self.rng() % 100) as u32;
            let difficulty = self.wave.min(10);
            let enemy = if roll < 40_u32.saturating_sub(difficulty * 3) {
                EnemyType::Frontrunner
            } else if roll < 65_u32.saturating_sub(difficulty) {
                EnemyType::Backrunner
            } else if roll < 82 {
                EnemyType::SandwichBot
            } else {
                EnemyType::JitLp
            };
            let enemy = if self.wave >= 3 && i == count - 1 {
                EnemyType::JitLp
            } else {
                enemy
            };
            self.pending.push_back(enemy);
        }
    }
}
