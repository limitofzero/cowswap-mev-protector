use bevy::prelude::*;

/// Cycles through spritesheet frames at a fixed interval.
#[derive(Component)]
pub struct AnimationTimer {
    pub timer: Timer,
    /// How many frames to cycle through.
    pub frames: usize,
    /// Atlas index of the first frame in this animation strip.
    pub base: usize,
}

impl AnimationTimer {
    pub fn new(fps: f32, frames: usize) -> Self {
        Self {
            timer: Timer::from_seconds(1.0 / fps, TimerMode::Repeating),
            frames,
            base: 0,
        }
    }

    pub fn new_with_offset(fps: f32, frames: usize, base: usize) -> Self {
        Self {
            timer: Timer::from_seconds(1.0 / fps, TimerMode::Repeating),
            frames,
            base,
        }
    }
}

/// All CoW-protocol-based defense tower types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TowerType {
    /// Groups txs into batches — frontrunners cannot target individual txs in a batch.
    BatchAuctioneer,
    /// Pairs a buy tx with a sell tx — CoW-matched pairs become MEV-immune.
    CoWMatcher,
    /// Finds the optimal settlement route — reduces the extractable value per tx.
    Solver,
    /// Tightens slippage tolerance — reduces the profit window for sandwich bots.
    SlippageGuard,
    /// Routes txs through a private mempool — temporarily invisible to bots.
    /// Has a long cooldown to prevent spamming.
    DarkPoolNode,
}

impl TowerType {
    pub fn range(&self) -> f32 {
        match self {
            TowerType::BatchAuctioneer => 130.0,
            TowerType::CoWMatcher => 110.0,
            TowerType::Solver => 85.0,
            TowerType::SlippageGuard => 95.0,
            TowerType::DarkPoolNode => 75.0,
        }
    }

    /// Seconds between activations.
    pub fn cooldown_secs(&self) -> f32 {
        match self {
            TowerType::BatchAuctioneer => 2.5,
            TowerType::CoWMatcher => 3.5,
            TowerType::Solver => 1.5,
            TowerType::SlippageGuard => 0.8,
            TowerType::DarkPoolNode => 10.0,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            TowerType::BatchAuctioneer => Color::srgb(0.15, 0.80, 0.40),
            TowerType::CoWMatcher => Color::srgb(0.20, 0.55, 0.95),
            TowerType::Solver => Color::srgb(0.40, 0.90, 0.45),
            TowerType::SlippageGuard => Color::srgb(0.85, 0.85, 0.20),
            TowerType::DarkPoolNode => Color::srgb(0.25, 0.25, 0.75),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TowerType::BatchAuctioneer => "Batch",
            TowerType::CoWMatcher => "CoW",
            TowerType::Solver => "Solver",
            TowerType::SlippageGuard => "Slippage",
            TowerType::DarkPoolNode => "DarkPool",
        }
    }

    pub fn cost(&self) -> f32 {
        match self {
            TowerType::BatchAuctioneer => 150.0,
            TowerType::CoWMatcher => 200.0,
            TowerType::Solver => 180.0,
            TowerType::SlippageGuard => 130.0,
            TowerType::DarkPoolNode => 220.0,
        }
    }

    /// Index into the shared tower sprite sheets (cow=0, ba=1, slv=2, sg=3, dp=4).
    pub fn atlas_index(&self) -> usize {
        match self {
            TowerType::CoWMatcher      => 0,
            TowerType::BatchAuctioneer => 1,
            TowerType::Solver          => 2,
            TowerType::SlippageGuard   => 3,
            TowerType::DarkPoolNode    => 4,
        }
    }

    /// First atlas index for this tower's 6-frame animation row in cowswap_towers_anim.png.
    pub fn anim_base(&self) -> usize { self.atlas_index() * 6 }
}

/// Marks the range fill/border children — hidden unless the tower is hovered.
#[derive(Component)]
pub struct TowerRangeVisual;

/// Marks the semi-transparent ghost tower that follows the cursor during placement.
#[derive(Component)]
pub struct GhostTower(pub TowerType);

/// Marks the delete-icon sprite that follows the cursor during remove mode.
#[derive(Component)]
pub struct DeleteCursor;

/// Attached to each tower shop button in the HUD.
#[derive(Component)]
pub struct TowerShopButton(pub TowerType);

/// A homing projectile fired by the Solver tower.
#[derive(Component)]
pub struct Projectile {
    pub target: Entity,
    pub speed: f32,
    pub damage: f32,
}

/// Shared sprite sheet assets for towers.
#[derive(Resource, Default)]
pub struct TowerAssets {
    /// cowswap_towers_anim.png — 6 cols × 5 rows, 84×110 per frame
    pub anim_layout: Option<Handle<TextureAtlasLayout>>,
    /// cowswap_towers_ghost.png — 5 cols × 1 row, 84×110 per frame
    pub ghost_layout: Option<Handle<TextureAtlasLayout>>,
    /// cowswap_towers_icons.png — 5 cols × 1 row, 46×59 per frame
    pub icon_layout: Option<Handle<TextureAtlasLayout>>,
    pub anim_sheet: Option<Handle<Image>>,
    pub ghost_sheet: Option<Handle<Image>>,
    pub icon_sheet: Option<Handle<Image>>,
    /// towers/tower_delete.png — icon for the remove button
    pub delete_icon: Option<Handle<Image>>,
}

/// A placed defense tower.
#[derive(Component)]
pub struct Tower {
    pub tower_type: TowerType,
    pub range: f32,
    pub cooldown: Timer,
}

impl Tower {
    pub fn new(tower_type: TowerType) -> Self {
        let range = tower_type.range();
        let secs = tower_type.cooldown_secs();
        Self {
            range,
            cooldown: Timer::from_seconds(secs, TimerMode::Repeating),
            tower_type,
        }
    }
}
