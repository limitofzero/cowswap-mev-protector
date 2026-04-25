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
        // Two-lane harness: enters left on the top rail, rounds the right corner,
        // exits left on the bottom rail.  Covers most of the 1280×720 viewport.
        let waypoints = vec![
            Vec2::new(-640.0, 170.0),  // entry — left edge, top rail
            Vec2::new(490.0, 170.0),   // top rail straight
            Vec2::new(545.0, 115.0),   // corner top-right bevel
            Vec2::new(545.0, -115.0),  // right connector
            Vec2::new(490.0, -170.0),  // corner bottom-right bevel
            Vec2::new(-640.0, -170.0), // exit — left edge, bottom rail
        ];
        let mut seg_lens = vec![0.0f32];
        for seg_idx in 1..waypoints.len() {
            let segment_length = waypoints[seg_idx - 1].distance(waypoints[seg_idx]);
            seg_lens.push(seg_lens[seg_idx - 1] + segment_length);
        }
        let total_length = *seg_lens.last().unwrap_or(&0.0);
        Self {
            waypoints,
            seg_lens,
            total_length,
        }
    }
}

impl MempoolPath {
    /// Returns true if `pos` is within `min_dist` world-units of any path segment.
    pub fn is_near_path(&self, pos: Vec2, min_dist: f32) -> bool {
        for seg_idx in 0..self.waypoints.len() - 1 {
            let seg_start = self.waypoints[seg_idx];
            let seg_end = self.waypoints[seg_idx + 1];
            let ab = seg_end - seg_start;
            let len_sq = ab.length_squared();
            let proj = if len_sq < 1e-6 {
                0.0
            } else {
                ((pos - seg_start).dot(ab) / len_sq).clamp(0.0, 1.0)
            };
            if (seg_start + ab * proj).distance(pos) < min_dist {
                return true;
            }
        }
        false
    }

    /// Closest point on the path polyline to `pos`.
    pub fn nearest_point(&self, pos: Vec2) -> Vec2 {
        let mut best = self.waypoints[0];
        let mut best_dist = f32::MAX;
        for seg_idx in 0..self.waypoints.len() - 1 {
            let seg_start = self.waypoints[seg_idx];
            let seg_end = self.waypoints[seg_idx + 1];
            let ab = seg_end - seg_start;
            let len_sq = ab.length_squared();
            let proj = if len_sq < 1e-6 {
                0.0
            } else {
                ((pos - seg_start).dot(ab) / len_sq).clamp(0.0, 1.0)
            };
            let pt = seg_start + ab * proj;
            let dist = pt.distance(pos);
            if dist < best_dist {
                best_dist = dist;
                best = pt;
            }
        }
        best
    }

    /// World position at arc-length-uniform progress `t ∈ [0, 1]`.
    pub fn position_at(&self, progress: f32) -> Vec2 {
        let target = progress.clamp(0.0, 1.0) * self.total_length;
        let seg_idx = self
            .seg_lens
            .partition_point(|&length| length < target)
            .min(self.seg_lens.len() - 1);
        let seg_idx = seg_idx.max(1);
        let seg_start = self.seg_lens[seg_idx - 1];
        let seg_len = self.seg_lens[seg_idx] - seg_start;
        if seg_len < 1e-6 {
            return self.waypoints[seg_idx];
        }
        let frac = (target - seg_start) / seg_len;
        self.waypoints[seg_idx - 1].lerp(self.waypoints[seg_idx], frac)
    }
}
