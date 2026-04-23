use bevy::prelude::*;

/// Polyline path — U-shape matching the mockup.
/// Linear interpolation between waypoints gives arc-length-uniform movement for free.
#[derive(Resource)]
pub struct MempoolPath {
    pub waypoints: Vec<Vec2>,
    /// Cumulative arc lengths: seg_lens[i] = distance from waypoint 0 to waypoint i.
    seg_lens: Vec<f32>,
    pub total_length: f32,
}

impl Default for MempoolPath {
    fn default() -> Self {
        let waypoints = vec![
            Vec2::new(-640.0,  114.0),
            Vec2::new(-441.0,  114.0),
            Vec2::new(-353.0,   38.0),
            Vec2::new(-353.0,  -76.0),
            Vec2::new(-265.0, -152.0),
            Vec2::new(  66.0, -152.0),
            Vec2::new( 154.0,  -76.0),
            Vec2::new( 154.0,   38.0),
            Vec2::new( 243.0,  114.0),
            Vec2::new( 640.0,  114.0),
        ];
        let mut seg_lens = vec![0.0f32];
        for i in 1..waypoints.len() {
            let d = waypoints[i - 1].distance(waypoints[i]);
            seg_lens.push(seg_lens[i - 1] + d);
        }
        let total_length = *seg_lens.last().unwrap_or(&0.0);
        Self { waypoints, seg_lens, total_length }
    }
}

impl MempoolPath {
    /// Returns true if `pos` is within `min_dist` world-units of any path segment.
    pub fn is_near_path(&self, pos: Vec2, min_dist: f32) -> bool {
        for i in 0..self.waypoints.len() - 1 {
            let a = self.waypoints[i];
            let b = self.waypoints[i + 1];
            let ab = b - a;
            let len_sq = ab.length_squared();
            let t = if len_sq < 1e-6 { 0.0 } else { ((pos - a).dot(ab) / len_sq).clamp(0.0, 1.0) };
            if (a + ab * t).distance(pos) < min_dist {
                return true;
            }
        }
        false
    }

    /// World position at arc-length-uniform progress `t ∈ [0, 1]`.
    pub fn position_at(&self, t: f32) -> Vec2 {
        let target = t.clamp(0.0, 1.0) * self.total_length;
        let i = self.seg_lens.partition_point(|&l| l < target).min(self.seg_lens.len() - 1);
        let i = i.max(1);
        let seg_start = self.seg_lens[i - 1];
        let seg_len = self.seg_lens[i] - seg_start;
        if seg_len < 1e-6 {
            return self.waypoints[i];
        }
        let frac = (target - seg_start) / seg_len;
        self.waypoints[i - 1].lerp(self.waypoints[i], frac)
    }
}
