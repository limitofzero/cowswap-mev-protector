use bevy::prelude::*;

/// The ordered waypoints that define the mempool river's path across the screen.
/// Transactions follow these points from index 0 (spawn) to the last (settlement zone).
#[derive(Resource)]
pub struct MempoolPath {
    pub waypoints: Vec<Vec2>,
}

impl Default for MempoolPath {
    fn default() -> Self {
        // S-curve from left edge to right edge at 1280×720
        Self {
            waypoints: vec![
                Vec2::new(-580.0, 60.0),
                Vec2::new(-340.0, 200.0),
                Vec2::new(-80.0, 80.0),
                Vec2::new(80.0, -80.0),
                Vec2::new(340.0, -200.0),
                Vec2::new(580.0, -60.0),
            ],
        }
    }
}

impl MempoolPath {
    /// World position at normalised progress `t ∈ [0, 1]`.
    /// Linearly interpolates between the nearest pair of waypoints.
    pub fn position_at(&self, t: f32) -> Vec2 {
        let n = self.waypoints.len();
        if n == 0 {
            return Vec2::ZERO;
        }
        if n == 1 {
            return self.waypoints[0];
        }
        let segments = (n - 1) as f32;
        let scaled = t.clamp(0.0, 1.0) * segments;
        let idx = (scaled.floor() as usize).min(n - 2);
        self.waypoints[idx].lerp(self.waypoints[idx + 1], scaled - idx as f32)
    }

    /// Approximate total path length (sum of segment lengths).
    pub fn total_length(&self) -> f32 {
        self.waypoints.windows(2).map(|w| w[0].distance(w[1])).sum()
    }
}
