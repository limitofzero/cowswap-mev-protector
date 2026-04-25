use bevy::prelude::*;

use super::components::EnemyType;

/// Pre-loaded handles for all enemy sprites and the shared atlas layout.
#[derive(Resource, Default)]
pub struct EnemyAssets {
    /// 6 cols × 2 rows, 96×96 — row 0 = Lv1, row 1 = Lv2.
    pub upgrade_layout: Option<Handle<TextureAtlasLayout>>,
    pub frontrunner_upgrades: Option<Handle<Image>>,
    pub backrunner_upgrades: Option<Handle<Image>>,
    pub sandwich_upgrades: Option<Handle<Image>>,
    pub jitlp_upgrades: Option<Handle<Image>>,
}

impl EnemyAssets {
    pub fn upgrade_texture(&self, enemy_type: &EnemyType) -> Option<Handle<Image>> {
        match enemy_type {
            EnemyType::Frontrunner => self.frontrunner_upgrades.clone(),
            EnemyType::Backrunner => self.backrunner_upgrades.clone(),
            EnemyType::SandwichBot => self.sandwich_upgrades.clone(),
            EnemyType::JitLp => self.jitlp_upgrades.clone(),
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
    /// Elevated-level bots still to spawn this wave: [lv1, lv2, lv3].
    pub level_quotas: [u32; 3],
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
            level_quotas: [0; 3],
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
            Vec2::new(-620.0, 130.0),
            Vec2::new(-620.0, -180.0),
            Vec2::new(620.0, 130.0),
            Vec2::new(620.0, -180.0),
            Vec2::new(20.0, 400.0),
            Vec2::new(-20.0, -400.0),
            Vec2::new(-340.0, 400.0),
            Vec2::new(340.0, -400.0),
        ];
        let zone_idx = (self.rng() as usize) % ZONES.len();
        ZONES[zone_idx]
    }

    /// Quotas of elevated bots per wave: [lv1, lv2, lv3].
    /// Schedule: Lv1 bots arrive first (wave 8), Lv2 from wave 20, Lv3 from wave 28.
    /// In the endgame all active enemies are elite (Lv3).
    fn wave_quotas(wave: u32) -> [u32; 3] {
        match wave {
            0..=7 => [0, 0, 0],
            8..=11 => [1, 0, 0],
            12..=15 => [2, 0, 0],
            16..=19 => [3, 0, 0],
            20..=23 => [3, 1, 0],
            24..=27 => [4, 2, 0],
            28..=31 => [4, 3, 1],
            32..=35 => [4, 4, 2],
            36..=39 => [3, 4, 3],
            40..=43 => [2, 4, 4],
            44..=47 => [0, 3, 6],
            48..=51 => [0, 1, 10],
            _ => [0, 0, 20], // all elite
        }
    }

    /// Advance to the next wave and update the active-enemy target.
    /// Ramp: 2, 2, 3, 4, 5, 6 … capped at 20.
    pub fn next_wave(&mut self) {
        self.wave += 1;
        self.wave_target = if self.wave <= 2 { 2 } else { self.wave.min(20) };
        self.level_quotas = Self::wave_quotas(self.wave);
    }

    /// Pick a random enemy type (equal weight across all 4 types).
    fn pick_random_type(&mut self) -> EnemyType {
        match (self.rng() % 4) as u32 {
            0 => EnemyType::Frontrunner,
            1 => EnemyType::Backrunner,
            2 => EnemyType::SandwichBot,
            _ => EnemyType::JitLp,
        }
    }

    /// Pick the level for the next spawn, consuming quota from highest to lowest.
    /// Returns (EnemyType, level).
    pub fn pick_spawn(&mut self) -> (EnemyType, u8) {
        for lv in (1u8..=3).rev() {
            let slot = &mut self.level_quotas[(lv - 1) as usize];
            if *slot > 0 {
                *slot -= 1;
                return (self.pick_random_type(), lv);
            }
        }
        (self.pick_enemy(), 0)
    }

    /// Pick one enemy type based on current wave difficulty (Lv0 only).
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
