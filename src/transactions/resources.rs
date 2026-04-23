use super::components::TokenType;
use bevy::prelude::*;

#[derive(Resource)]
pub struct TxSpawner {
    pub timer: Timer,
    seed: u64,
    pub layout: Option<Handle<TextureAtlasLayout>>,
    pub textures: Vec<Handle<Image>>,
}

impl TxSpawner {
    pub fn new(interval_secs: f32) -> Self {
        Self {
            timer: Timer::from_seconds(interval_secs, TimerMode::Repeating),
            seed: 0xdeadbeef_cafebabe,
            layout: None,
            textures: Vec::new(),
        }
    }

    fn rand(&mut self) -> u64 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        self.seed
    }

    pub fn rand_f32(&mut self) -> f32 {
        (self.rand() & 0xFFFFFF) as f32 / 0xFFFFFF as f32
    }

    pub fn rand_usize(&mut self, max: usize) -> usize {
        (self.rand() as usize) % max
    }

    pub fn rand_token(&mut self) -> (TokenType, Handle<Image>) {
        let idx = self.rand_usize(TokenType::ALL.len());
        (TokenType::ALL[idx], self.textures[idx].clone())
    }
}
