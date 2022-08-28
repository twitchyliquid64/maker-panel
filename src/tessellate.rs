use crate::InnerAtom;
use lyon::path::Path;
use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::math::point;
use lyon::tessellation::{FillOptions, FillTessellator};

use std::collections::HashMap;

pub use lyon::tessellation::{geometry_builder::VertexBuffers, math::Point, TessellationError};

pub fn tessellate_2d(
    poly: geo::Polygon<f64>,
    interior: Vec<InnerAtom>,
) -> Result<VertexBuffers<Point, u16>, TessellationError> {
    let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();
    let mut vertex_builder = simple_builder(&mut buffers);
    let mut tessellator = FillTessellator::new();
    let options = FillOptions::default();

    let mut path_builder = Path::builder();

    let mut last: Option<geo::Point<f64>> = None;
    for p in poly.exterior().points_iter() {
        let (x, y) = (p.x() as f32, p.y() as f32);
        if last.is_none() {
            path_builder.begin(point(x, y));
        } else {
            path_builder.line_to(point(x, y));
        }
        last = Some(p);
    }
    path_builder.end(true);

    for hole in poly.interiors() {
        let mut last: Option<geo::Point<f64>> = None;
        for p in hole.points_iter() {
            let (x, y) = (p.x() as f32, p.y() as f32);
            if last.is_none() {
                path_builder.begin(point(x, y));
            } else {
                path_builder.line_to(point(x, y));
            }
            last = Some(p);
        }
        path_builder.end(true);
    }
    for f in interior {
        if let InnerAtom::Drill {
            center,
            radius,
            plated: _,
        } = f
        {
            use geo::{algorithm::rotate::RotatePoint, Point};
            let right_edge: Point<_> = (center.x + radius, center.y).into();

            let start = right_edge.rotate_around_point(0.0, center.into());
            path_builder.begin(point(start.x() as f32, start.y() as f32));
            for i in (0..=360).step_by(8) {
                let p = right_edge.rotate_around_point(i as f64, center.into());
                path_builder.line_to(point(p.x() as f32, p.y() as f32));
            }
            path_builder.end(true);
        }
    }

    let path = path_builder.build();
    tessellator.tessellate_path(&path, &options, &mut vertex_builder)?;
    Ok(buffers)
}

pub fn normals_from_tessellation(verts: &Vec<[f64; 3]>, inds: &Vec<u16>) -> Vec<[f32; 3]> {
    let v_conv = |idx: u16| {
        let v = verts[idx as usize];
        [v[0] as f32, v[1] as f32, v[2] as f32]
    };

    inds.chunks_exact(3)
        .map(|inds| {
            let verts = [v_conv(inds[0]), v_conv(inds[1]), v_conv(inds[2])];

            // Compute the normal of the face via dot product of the verts.
            // We don't use a real math library because im still learning
            // this stuff, and i wanted to do it by hand (feel free to PR).
            let u = [
                verts[1][0] - verts[0][0],
                verts[1][1] - verts[0][1],
                verts[1][2] - verts[0][2],
            ];
            let v = [
                verts[2][0] - verts[0][0],
                verts[2][1] - verts[0][1],
                verts[2][2] - verts[0][2],
            ];
            let normal = [
                (u[1] * v[2]) - (u[2] * v[1]),
                (u[2] * v[0]) - (u[0] * v[2]),
                (u[0] * v[1]) - (u[1] * v[0]),
            ];

            normal
        })
        .collect()
}

pub fn tessellate_3d(buffer: VertexBuffers<Point, u16>) -> (Vec<[f64; 3]>, Vec<u16>) {
    // eprintln!("buffer: {:?} ({})", buffer, buffer.vertices.chunks_exact(3).count());

    // Iterate through the edges represented by the indices, building a map
    // of the indice indexes which use it.
    let mut lines: HashMap<(u16, u16), Vec<(usize, bool)>> =
        HashMap::with_capacity(buffer.indices.len());
    // For the three corners of each triangle ...
    for (i, in3) in buffer.indices.chunks_exact(3).enumerate() {
        // Loop each edge (line) of the triangle ...
        for (i, verts) in &[
            (i * 3 + 0, &[in3[0], in3[1]]),
            (i * 3 + 1, &[in3[1], in3[2]]),
            (i * 3 + 2, &[in3[2], in3[0]]),
        ] {
            // We make sure a forward or reverse edge
            // maps to the same key (2->1 is the same as 1->2).
            let key = (verts[0].min(verts[1]), verts[1].max(verts[0]));

            // ... And track how many times we see an edge between those
            // two points, by inserting it into the hash map.
            match lines.get_mut(&key) {
                Some(v) => v.push((*i, verts[0] < verts[1])),
                None => {
                    lines.insert(key, vec![(*i, verts[0] < verts[1])]);
                }
            }
        }
    }

    // Edges which are on the boundary of the polygon are those which are only
    // part of a single triangle.
    let mut boundary_lines: Vec<_> = lines
        .into_iter()
        .filter(|(_k, v)| v.len() == 1)
        .map(|(k, v)| (k, v[0])) // (v1, v2), (idx, ordered)
        .collect();
    // Sort them into the order in which they appeared in the original index buffer.
    boundary_lines.sort_by(|a, b| a.1 .0.cmp(&b.1 .0));

    // First buffer.vertices.len() items are the vertices of the bottom surface.
    // The last buffer.vertices.len() items are the vertices of the top surface.
    let mut vertices: Vec<[f64; 3]> =
        Vec::with_capacity(2 * buffer.vertices.len() + 6 * boundary_lines.len());
    for v in &buffer.vertices {
        vertices.push([v.x.into(), v.y.into(), -0.8]);
    }
    for v in &buffer.vertices {
        vertices.push([v.x.into(), v.y.into(), 0.8]);
    }

    // Compute the vertices: the front and back faces are easy - we just duplicate
    // the original tessellation, with the back face in reverse order for correct
    // winding.
    let c = buffer.vertices.len() as u16;
    let mut indices: Vec<u16> =
        Vec::with_capacity((buffer.indices.len() * 2) + (buffer.vertices.len() * 6));
    // Front
    for i in &buffer.indices {
        indices.push(*i);
    }
    // Back
    for i3 in buffer.indices.chunks_exact(3) {
        indices.push(i3[2] + c); // reverse order - presumably to represent winding order?
        indices.push(i3[1] + c);
        indices.push(i3[0] + c);
    }
    // For the sides, we loop through the boundary edges to construct 2 triangles
    // for each edge.
    for ((v_low, v_high), (_, original_order)) in boundary_lines {
        if !original_order {
            indices.push(v_low);
            indices.push(v_high);
            indices.push(v_low + c);
            indices.push(v_high);
            indices.push(v_high + c);
            indices.push(v_low + c);
        } else {
            indices.push(v_high);
            indices.push(v_low);
            indices.push(v_high + c);
            indices.push(v_low);
            indices.push(v_low + c);
            indices.push(v_high + c);
        }
    }

    (vertices, indices)
}
