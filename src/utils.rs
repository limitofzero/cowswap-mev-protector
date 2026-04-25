use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// Builds a filled rounded-rectangle mesh centered at the origin.
/// `steps` is the number of arc subdivisions per corner (8 gives smooth 90° arcs).
pub fn make_rounded_rect(width: f32, height: f32, radius: f32, steps: u32) -> Mesh {
    use std::f32::consts::FRAC_PI_2;
    let hw = width * 0.5;
    let hh = height * 0.5;
    let arc_radius = radius.min(hw).min(hh);
    // (corner_center_x, corner_center_y, arc_start_angle)
    let corners: [(f32, f32, f32); 4] = [
        (hw - arc_radius, hh - arc_radius, 0.0),
        (-hw + arc_radius, hh - arc_radius, FRAC_PI_2),
        (-hw + arc_radius, -hh + arc_radius, FRAC_PI_2 * 2.0),
        (hw - arc_radius, -hh + arc_radius, FRAC_PI_2 * 3.0),
    ];
    let mut positions: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]]; // center fan vertex
    let mut indices: Vec<u32> = Vec::new();
    for (cx, cy, start) in corners {
        let base = positions.len() as u32;
        for step in 0..=steps {
            let angle = start + (step as f32 / steps as f32) * FRAC_PI_2;
            positions.push([
                cx + arc_radius * angle.cos(),
                cy + arc_radius * angle.sin(),
                0.0,
            ]);
        }
        for step in 0..steps {
            indices.extend_from_slice(&[0, base + step, base + step + 1]);
        }
    }
    // Bridge triangles between consecutive corner arcs
    let arc_vert_count = steps + 1;
    for corner_idx in 0..4u32 {
        let next = (corner_idx + 1) % 4;
        indices.extend_from_slice(&[
            0,
            1 + corner_idx * arc_vert_count + steps,
            1 + next * arc_vert_count,
        ]);
    }
    let normals = vec![[0.0_f32, 0.0, 1.0]; positions.len()];
    let uvs = vec![[0.0_f32, 0.0]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}
