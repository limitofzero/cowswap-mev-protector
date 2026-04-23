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
    /// World position at normalised progress `t ∈ [0, 1]` using Catmull-Rom spline.
    pub fn position_at(&self, t: f32) -> Vec2 {
        let n = self.waypoints.len();
        if n == 0 { return Vec2::ZERO; }
        if n == 1 { return self.waypoints[0]; }
        let segments = (n - 1) as f32;
        let scaled = t.clamp(0.0, 1.0) * segments;
        let idx = (scaled.floor() as usize).min(n - 2);
        let local_t = scaled - idx as f32;
        let p0 = self.waypoints[idx.saturating_sub(1)];
        let p1 = self.waypoints[idx];
        let p2 = self.waypoints[(idx + 1).min(n - 1)];
        let p3 = self.waypoints[(idx + 2).min(n - 1)];
        catmull_rom(p0, p1, p2, p3, local_t)
    }

    /// Approximate total path length sampled along the spline.
    pub fn total_length(&self) -> f32 {
        let samples = 64;
        (0..samples)
            .map(|i| {
                let a = self.position_at(i as f32 / samples as f32);
                let b = self.position_at((i + 1) as f32 / samples as f32);
                a.distance(b)
            })
            .sum()
    }
}

fn catmull_rom(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}
