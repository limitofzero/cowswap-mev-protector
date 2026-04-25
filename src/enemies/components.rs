use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnemyType {
    Frontrunner,
    Backrunner,
    SandwichBot,
    JitLp,
}

impl EnemyType {
    pub fn move_speed(&self) -> f32 {
        match self {
            EnemyType::Frontrunner => 130.0,
            EnemyType::Backrunner  => 55.0,
            EnemyType::SandwichBot => 90.0,
            EnemyType::JitLp       => 160.0,
        }
    }

    pub fn max_hp(&self) -> f32 {
        match self {
            EnemyType::Frontrunner => 60.0,
            EnemyType::Backrunner  => 100.0,
            EnemyType::SandwichBot => 80.0,
            EnemyType::JitLp       => 50.0,
        }
    }

    /// Fraction of the tx's initial value drained per second (linear).
    pub fn drain_rate(&self) -> f32 {
        match self {
            EnemyType::Frontrunner => 0.12,
            EnemyType::Backrunner  => 0.08,
            EnemyType::SandwichBot => 0.18,
            EnemyType::JitLp       => 0.22,
        }
    }

    pub fn attack_range(&self) -> f32 {
        match self {
            EnemyType::JitLp => 40.0,
            _                => 65.0,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            EnemyType::Frontrunner => Color::srgb(0.90, 0.15, 0.15),
            EnemyType::Backrunner  => Color::srgb(0.65, 0.10, 0.50),
            EnemyType::SandwichBot => Color::srgb(0.95, 0.50, 0.05),
            EnemyType::JitLp       => Color::srgb(0.85, 0.85, 0.05),
        }
    }

    pub fn size(&self) -> f32 { 48.0 }

    pub fn sprite_path(&self) -> &'static str {
        match self {
            EnemyType::Frontrunner => "enemies/enemy_frontrunner.png",
            EnemyType::Backrunner  => "enemies/enemy_backrunner.png",
            EnemyType::SandwichBot => "enemies/enemy_sandwich.png",
            EnemyType::JitLp       => "enemies/enemy_jitlp.png",
        }
    }

    /// Tint color multiplied onto the sprite for level 1 bots (same sprite, visually distinct).
    pub fn lv1_tint(&self) -> Color {
        match self {
            EnemyType::Frontrunner => Color::srgb(1.0, 0.45, 0.45), // hot red
            EnemyType::Backrunner  => Color::srgb(0.75, 0.40, 1.0), // vivid purple
            EnemyType::SandwichBot => Color::srgb(1.0, 0.65, 0.20), // deep orange
            EnemyType::JitLp       => Color::srgb(0.35, 1.0, 1.0),  // bright cyan
        }
    }
}

/// Level multipliers applied on top of base stats.
const LV1_SPEED_MULT:  f32 = 1.35;
const LV1_HP_MULT:     f32 = 1.80;
const LV1_DRAIN_MULT:  f32 = 1.50;
const LV1_SIZE_MULT:   f32 = 1.20;

#[derive(Component)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub level: u8,
    pub speed: f32,
    pub drain_rate: f32,
    pub attack_range: f32,
    pub target: Option<Entity>,
    pub hp: f32,
    pub max_hp: f32,
    pub slow_timer: Option<Timer>,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType) -> Self {
        Self::new_leveled(enemy_type, 0)
    }

    pub fn new_leveled(enemy_type: EnemyType, level: u8) -> Self {
        let (sm, hm, dm) = if level >= 1 {
            (LV1_SPEED_MULT, LV1_HP_MULT, LV1_DRAIN_MULT)
        } else {
            (1.0, 1.0, 1.0)
        };
        let hp = enemy_type.max_hp() * hm;
        Self {
            speed: enemy_type.move_speed() * sm,
            drain_rate: enemy_type.drain_rate() * dm,
            attack_range: enemy_type.attack_range(),
            target: None,
            hp,
            max_hp: hp,
            slow_timer: None,
            level,
            enemy_type,
        }
    }

    pub fn sprite_size(&self) -> f32 {
        let base = self.enemy_type.size();
        if self.level >= 1 { base * LV1_SIZE_MULT } else { base }
    }

    pub fn sprite_tint(&self) -> Color {
        if self.level >= 1 { self.enemy_type.lv1_tint() } else { Color::WHITE }
    }

    pub fn effective_speed(&self) -> f32 {
        if self.slow_timer.is_some() { self.speed * 0.35 } else { self.speed }
    }

    pub fn apply_slow(&mut self, secs: f32) {
        self.slow_timer = Some(Timer::from_seconds(secs, TimerMode::Once));
    }

    pub fn tick_slow(&mut self, delta: std::time::Duration) {
        if let Some(timer) = &mut self.slow_timer {
            timer.tick(delta);
            if timer.just_finished() { self.slow_timer = None; }
        }
    }
}

/// Marker for the filled portion of an enemy's HP bar (child entity).
#[derive(Component)]
pub struct EnemyHpBarFg;

