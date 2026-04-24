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

    pub fn extract_rate(&self) -> f32 {
        match self {
            EnemyType::Frontrunner => 180.0,
            EnemyType::Backrunner  => 80.0,
            EnemyType::SandwichBot => 140.0,
            EnemyType::JitLp       => 120.0,
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
            EnemyType::Frontrunner => "enemy_frontrunner.png",
            EnemyType::Backrunner  => "enemy_backrunner.png",
            EnemyType::SandwichBot => "enemy_sandwich.png",
            EnemyType::JitLp       => "enemy_jitlp.png",
        }
    }
}

#[derive(Component)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub speed: f32,
    pub extract_rate: f32,
    pub attack_range: f32,
    pub target: Option<Entity>,
    pub hp: f32,
    pub max_hp: f32,
    pub slow_timer: Option<Timer>,
}

impl Enemy {
    pub fn new(enemy_type: EnemyType) -> Self {
        let hp = enemy_type.max_hp();
        Self {
            speed: enemy_type.move_speed(),
            extract_rate: enemy_type.extract_rate(),
            attack_range: enemy_type.attack_range(),
            target: None,
            hp,
            max_hp: hp,
            slow_timer: None,
            enemy_type,
        }
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

/// Shared sprite atlas handle for enemy sprites.
#[derive(Resource, Default)]
pub struct EnemyAssets {
    pub layout: Option<Handle<TextureAtlasLayout>>,
}
