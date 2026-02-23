fn build_mesh_from_indexed(
    vertices: &[RgmPoint3],
    flat_indices: &[u32],
) -> Result<MeshData, RgmStatus> {
    if vertices.len() < 3 || flat_indices.len() < 3 || flat_indices.len() % 3 != 0 {
        return Err(RgmStatus::InvalidInput);
    }

    let mut triangles = Vec::with_capacity(flat_indices.len() / 3);
    for tri in flat_indices.chunks_exact(3) {
        if tri[0] as usize >= vertices.len()
            || tri[1] as usize >= vertices.len()
            || tri[2] as usize >= vertices.len()
        {
            return Err(RgmStatus::OutOfRange);
        }
        triangles.push([tri[0], tri[1], tri[2]]);
    }

    if triangles.is_empty() {
        return Err(RgmStatus::InvalidInput);
    }

    Ok(MeshData {
        vertices: vertices.to_vec(),
        triangles,
        transform: matrix_identity(),
    })
}

fn build_box_mesh(center: RgmPoint3, size: RgmVec3) -> Result<MeshData, RgmStatus> {
    if size.x <= 0.0 || size.y <= 0.0 || size.z <= 0.0 {
        return Err(RgmStatus::InvalidInput);
    }

    let hx = size.x * 0.5;
    let hy = size.y * 0.5;
    let hz = size.z * 0.5;
    let vertices = vec![
        RgmPoint3 {
            x: center.x - hx,
            y: center.y - hy,
            z: center.z - hz,
        },
        RgmPoint3 {
            x: center.x + hx,
            y: center.y - hy,
            z: center.z - hz,
        },
        RgmPoint3 {
            x: center.x + hx,
            y: center.y + hy,
            z: center.z - hz,
        },
        RgmPoint3 {
            x: center.x - hx,
            y: center.y + hy,
            z: center.z - hz,
        },
        RgmPoint3 {
            x: center.x - hx,
            y: center.y - hy,
            z: center.z + hz,
        },
        RgmPoint3 {
            x: center.x + hx,
            y: center.y - hy,
            z: center.z + hz,
        },
        RgmPoint3 {
            x: center.x + hx,
            y: center.y + hy,
            z: center.z + hz,
        },
        RgmPoint3 {
            x: center.x - hx,
            y: center.y + hy,
            z: center.z + hz,
        },
    ];
    let flat_indices: [u32; 36] = [
        0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 1, 2, 6, 1, 6, 5, 2, 3, 7, 2, 7, 6,
        3, 0, 4, 3, 4, 7,
    ];
    build_mesh_from_indexed(&vertices, &flat_indices)
}

fn build_uv_sphere_mesh(
    center: RgmPoint3,
    radius: f64,
    u_steps: u32,
    v_steps: u32,
) -> Result<MeshData, RgmStatus> {
    if radius <= 0.0 || u_steps < 8 || v_steps < 4 {
        return Err(RgmStatus::InvalidInput);
    }
    let u_steps = u_steps as usize;
    let v_steps = v_steps as usize;
    let mut vertices = Vec::with_capacity((u_steps + 1) * (v_steps + 1));
    for v in 0..=v_steps {
        let vv = v as f64 / v_steps as f64;
        let phi = std::f64::consts::PI * vv;
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        for u in 0..=u_steps {
            let uu = u as f64 / u_steps as f64;
            let theta = 2.0 * std::f64::consts::PI * uu;
            let x = radius * theta.cos() * sin_phi;
            let y = radius * theta.sin() * sin_phi;
            let z = radius * cos_phi;
            vertices.push(RgmPoint3 {
                x: center.x + x,
                y: center.y + y,
                z: center.z + z,
            });
        }
    }

    let ring = u_steps + 1;
    let mut indices = Vec::with_capacity(u_steps * v_steps * 6);
    for v in 0..v_steps {
        for u in 0..u_steps {
            let a = (v * ring + u) as u32;
            let b = a + 1;
            let c = ((v + 1) * ring + u) as u32;
            let d = c + 1;
            if v != 0 {
                indices.extend_from_slice(&[a, c, b]);
            }
            if v != v_steps - 1 {
                indices.extend_from_slice(&[b, c, d]);
            }
        }
    }

    build_mesh_from_indexed(&vertices, &indices)
}

fn build_torus_mesh(
    center: RgmPoint3,
    major_radius: f64,
    minor_radius: f64,
    major_steps: u32,
    minor_steps: u32,
) -> Result<MeshData, RgmStatus> {
    if major_radius <= 0.0 || minor_radius <= 0.0 || major_steps < 8 || minor_steps < 6 {
        return Err(RgmStatus::InvalidInput);
    }

    let major_steps = major_steps as usize;
    let minor_steps = minor_steps as usize;
    let mut vertices = Vec::with_capacity(major_steps * minor_steps);
    for i in 0..major_steps {
        let u = i as f64 / major_steps as f64;
        let theta = 2.0 * std::f64::consts::PI * u;
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        for j in 0..minor_steps {
            let v = j as f64 / minor_steps as f64;
            let phi = 2.0 * std::f64::consts::PI * v;
            let cos_p = phi.cos();
            let sin_p = phi.sin();
            let r = major_radius + minor_radius * cos_p;
            vertices.push(RgmPoint3 {
                x: center.x + r * cos_t,
                y: center.y + r * sin_t,
                z: center.z + minor_radius * sin_p,
            });
        }
    }

    let idx = |i: usize, j: usize| -> u32 {
        ((i % major_steps) * minor_steps + (j % minor_steps)) as u32
    };
    let mut indices = Vec::with_capacity(major_steps * minor_steps * 6);
    for i in 0..major_steps {
        for j in 0..minor_steps {
            let a = idx(i, j);
            let b = idx(i + 1, j);
            let c = idx(i, j + 1);
            let d = idx(i + 1, j + 1);
            indices.extend_from_slice(&[a, b, c, c, b, d]);
        }
    }
    build_mesh_from_indexed(&vertices, &indices)
}

fn mesh_copy_vertices_world(
    mesh: &MeshData,
    out_vertices: *mut RgmPoint3,
    vertex_capacity: u32,
    out_count: *mut u32,
) -> Result<(), RgmStatus> {
    let points = mesh_world_vertices(mesh);
    write_intersection_points(out_vertices, vertex_capacity, &points, out_count)
}

fn mesh_copy_indices(
    mesh: &MeshData,
    out_indices: *mut u32,
    index_capacity: u32,
    out_count: *mut u32,
) -> Result<(), RgmStatus> {
    if out_count.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    let flat_count = mesh.triangles.len().saturating_mul(3);
    // SAFETY: out_count validated above.
    unsafe {
        *out_count = flat_count.try_into().unwrap_or(u32::MAX);
    }

    if index_capacity == 0 {
        return Ok(());
    }
    if out_indices.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    let copy_count = flat_count.min(index_capacity as usize);
    for idx in 0..copy_count {
        let tri = idx / 3;
        let lane = idx % 3;
        // SAFETY: out_indices has capacity guaranteed by caller.
        unsafe {
            *out_indices.add(idx) = mesh.triangles[tri][lane];
        }
    }
    Ok(())
}

fn plane_unit_normal(plane: RgmPlane) -> Option<RgmVec3> {
    vec_normalize(plane.z_axis).or_else(|| vec_normalize(vec_cross(plane.x_axis, plane.y_axis)))
}

fn push_unique_point(points: &mut Vec<RgmPoint3>, candidate: RgmPoint3, tol: f64) {
    if points
        .iter()
        .any(|point| distance(*point, candidate) <= tol)
    {
        return;
    }
    points.push(candidate);
}

fn intersect_triangle_plane_segment(
    a: RgmPoint3,
    b: RgmPoint3,
    c: RgmPoint3,
    plane_origin: RgmPoint3,
    plane_normal: RgmVec3,
    tol: f64,
) -> Option<(RgmPoint3, RgmPoint3)> {
    let d0 = vec_dot(point_sub(a, plane_origin), plane_normal);
    let d1 = vec_dot(point_sub(b, plane_origin), plane_normal);
    let d2 = vec_dot(point_sub(c, plane_origin), plane_normal);
    if d0.abs() <= tol && d1.abs() <= tol && d2.abs() <= tol {
        return None;
    }

    let mut points = Vec::new();
    let mut edge_hit = |p0: RgmPoint3, p1: RgmPoint3, s0: f64, s1: f64| {
        if s0.abs() <= tol {
            push_unique_point(&mut points, p0, tol);
        }
        if s1.abs() <= tol {
            push_unique_point(&mut points, p1, tol);
        }
        if (s0 > tol && s1 < -tol) || (s0 < -tol && s1 > tol) {
            let t = s0 / (s0 - s1);
            let segment = point_sub(p1, p0);
            let hit = point_add_vec(p0, vec_scale(segment, t));
            push_unique_point(&mut points, hit, tol);
        }
    };

    edge_hit(a, b, d0, d1);
    edge_hit(b, c, d1, d2);
    edge_hit(c, a, d2, d0);

    if points.len() < 2 {
        return None;
    }

    let mut best = (points[0], points[1]);
    let mut best_len = distance(points[0], points[1]);
    for i in 0..points.len() {
        for j in (i + 1)..points.len() {
            let len = distance(points[i], points[j]);
            if len > best_len {
                best_len = len;
                best = (points[i], points[j]);
            }
        }
    }
    if best_len <= tol {
        None
    } else {
        Some(best)
    }
}

fn triangle_aabb(points: [RgmPoint3; 3]) -> (RgmPoint3, RgmPoint3) {
    let mut min = points[0];
    let mut max = points[0];
    for point in points.iter().skip(1) {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        min.z = min.z.min(point.z);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
        max.z = max.z.max(point.z);
    }
    (min, max)
}

#[derive(Clone, Copy, Debug)]
struct TriangleRecord {
    points: [RgmPoint3; 3],
    min: RgmPoint3,
    max: RgmPoint3,
}

impl TriangleRecord {
    fn from_mesh(vertices: &[RgmPoint3], tri: [u32; 3]) -> Self {
        let points = [
            vertices[tri[0] as usize],
            vertices[tri[1] as usize],
            vertices[tri[2] as usize],
        ];
        let (min, max) = triangle_aabb(points);
        Self { points, min, max }
    }
}

struct MeshAccelCache {
    triangles: Vec<TriangleRecord>,
    bvh: Option<MeshBvh>,
}

#[derive(Clone, Copy)]
struct BvhNode {
    min: RgmPoint3,
    max: RgmPoint3,
    left: Option<usize>,
    right: Option<usize>,
    start: usize,
    count: usize,
}

impl BvhNode {
    fn leaf(min: RgmPoint3, max: RgmPoint3, start: usize, count: usize) -> Self {
        Self {
            min,
            max,
            left: None,
            right: None,
            start,
            count,
        }
    }

    fn branch(min: RgmPoint3, max: RgmPoint3, left: usize, right: usize) -> Self {
        Self {
            min,
            max,
            left: Some(left),
            right: Some(right),
            start: 0,
            count: 0,
        }
    }

    fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

struct MeshBvh {
    root: usize,
    nodes: Vec<BvhNode>,
    tri_indices: Vec<usize>,
}

impl MeshBvh {
    fn build(records: &[TriangleRecord]) -> Option<Self> {
        if records.is_empty() {
            return None;
        }

        let mut tri_indices = (0..records.len()).collect::<Vec<_>>();
        let tri_count = tri_indices.len();
        let mut nodes = Vec::new();
        let root = Self::build_node(records, &mut tri_indices, &mut nodes, 0, tri_count);
        Some(Self {
            root,
            nodes,
            tri_indices,
        })
    }

    fn build_node(
        records: &[TriangleRecord],
        tri_indices: &mut Vec<usize>,
        nodes: &mut Vec<BvhNode>,
        start: usize,
        end: usize,
    ) -> usize {
        let node_idx = nodes.len();
        nodes.push(BvhNode::leaf(
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            start,
            end.saturating_sub(start),
        ));

        let (min, max) = Self::range_bounds(records, &tri_indices[start..end]);
        let count = end.saturating_sub(start);
        const LEAF_TRIANGLES: usize = 8;
        if count <= LEAF_TRIANGLES {
            nodes[node_idx] = BvhNode::leaf(min, max, start, count);
            return node_idx;
        }

        let axis = Self::split_axis(records, &tri_indices[start..end]);
        tri_indices[start..end].sort_by(|a, b| {
            let ca = Self::triangle_centroid_axis(records[*a], axis);
            let cb = Self::triangle_centroid_axis(records[*b], axis);
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = start + count / 2;
        if mid == start || mid == end {
            nodes[node_idx] = BvhNode::leaf(min, max, start, count);
            return node_idx;
        }

        let left = Self::build_node(records, tri_indices, nodes, start, mid);
        let right = Self::build_node(records, tri_indices, nodes, mid, end);
        nodes[node_idx] = BvhNode::branch(min, max, left, right);
        node_idx
    }

    fn range_bounds(records: &[TriangleRecord], indices: &[usize]) -> (RgmPoint3, RgmPoint3) {
        let mut min = records[indices[0]].min;
        let mut max = records[indices[0]].max;
        for &idx in indices.iter().skip(1) {
            let tri = records[idx];
            min.x = min.x.min(tri.min.x);
            min.y = min.y.min(tri.min.y);
            min.z = min.z.min(tri.min.z);
            max.x = max.x.max(tri.max.x);
            max.y = max.y.max(tri.max.y);
            max.z = max.z.max(tri.max.z);
        }
        (min, max)
    }

    fn split_axis(records: &[TriangleRecord], indices: &[usize]) -> usize {
        let mut cmin = Self::triangle_centroid(records[indices[0]]);
        let mut cmax = cmin;
        for &idx in indices.iter().skip(1) {
            let c = Self::triangle_centroid(records[idx]);
            cmin.x = cmin.x.min(c.x);
            cmin.y = cmin.y.min(c.y);
            cmin.z = cmin.z.min(c.z);
            cmax.x = cmax.x.max(c.x);
            cmax.y = cmax.y.max(c.y);
            cmax.z = cmax.z.max(c.z);
        }
        let ex = cmax.x - cmin.x;
        let ey = cmax.y - cmin.y;
        let ez = cmax.z - cmin.z;
        if ex >= ey && ex >= ez {
            0
        } else if ey >= ez {
            1
        } else {
            2
        }
    }

    fn triangle_centroid(record: TriangleRecord) -> RgmPoint3 {
        RgmPoint3 {
            x: (record.min.x + record.max.x) * 0.5,
            y: (record.min.y + record.max.y) * 0.5,
            z: (record.min.z + record.max.z) * 0.5,
        }
    }

    fn triangle_centroid_axis(record: TriangleRecord, axis: usize) -> f64 {
        match axis {
            0 => (record.min.x + record.max.x) * 0.5,
            1 => (record.min.y + record.max.y) * 0.5,
            _ => (record.min.z + record.max.z) * 0.5,
        }
    }
}

fn aabb_node_plane_overlap(
    min: RgmPoint3,
    max: RgmPoint3,
    plane_origin: RgmPoint3,
    plane_normal: RgmVec3,
    tol: f64,
) -> bool {
    let center = RgmPoint3 {
        x: (min.x + max.x) * 0.5,
        y: (min.y + max.y) * 0.5,
        z: (min.z + max.z) * 0.5,
    };
    let half = RgmVec3 {
        x: (max.x - min.x) * 0.5,
        y: (max.y - min.y) * 0.5,
        z: (max.z - min.z) * 0.5,
    };
    let dist = vec_dot(point_sub(center, plane_origin), plane_normal);
    let radius = half.x * plane_normal.x.abs()
        + half.y * plane_normal.y.abs()
        + half.z * plane_normal.z.abs();
    dist.abs() <= radius + tol
}

fn node_span(node: BvhNode) -> f64 {
    (node.max.x - node.min.x).abs()
        + (node.max.y - node.min.y).abs()
        + (node.max.z - node.min.z).abs()
}

fn aabb_overlap(
    a_min: RgmPoint3,
    a_max: RgmPoint3,
    b_min: RgmPoint3,
    b_max: RgmPoint3,
    tol: f64,
) -> bool {
    !(a_max.x < b_min.x - tol
        || b_max.x < a_min.x - tol
        || a_max.y < b_min.y - tol
        || b_max.y < a_min.y - tol
        || a_max.z < b_min.z - tol
        || b_max.z < a_min.z - tol)
}

fn segment_triangle_intersection(
    p0: RgmPoint3,
    p1: RgmPoint3,
    t0: RgmPoint3,
    t1: RgmPoint3,
    t2: RgmPoint3,
    tol: f64,
) -> Option<RgmPoint3> {
    segment_triangle_intersection_with_params(p0, p1, t0, t1, t2, tol).map(|v| v.0)
}

fn segment_triangle_intersection_with_params(
    p0: RgmPoint3,
    p1: RgmPoint3,
    t0: RgmPoint3,
    t1: RgmPoint3,
    t2: RgmPoint3,
    tol: f64,
) -> Option<(RgmPoint3, f64, f64, f64)> {
    let dir = point_sub(p1, p0);
    let edge1 = point_sub(t1, t0);
    let edge2 = point_sub(t2, t0);
    let pvec = vec_cross(dir, edge2);
    let det = vec_dot(edge1, pvec);
    if det.abs() <= tol {
        return None;
    }
    let inv_det = 1.0 / det;
    let tvec = point_sub(p0, t0);
    let u = vec_dot(tvec, pvec) * inv_det;
    if u < -tol || u > 1.0 + tol {
        return None;
    }
    let qvec = vec_cross(tvec, edge1);
    let v = vec_dot(dir, qvec) * inv_det;
    if v < -tol || u + v > 1.0 + tol {
        return None;
    }
    let t = vec_dot(edge2, qvec) * inv_det;
    if t < -tol || t > 1.0 + tol {
        return None;
    }

    Some((point_add_vec(p0, vec_scale(dir, t)), t, u, v))
}

fn tri_tri_intersection_segment(
    a0: RgmPoint3,
    a1: RgmPoint3,
    a2: RgmPoint3,
    b0: RgmPoint3,
    b1: RgmPoint3,
    b2: RgmPoint3,
    tol: f64,
) -> Option<(RgmPoint3, RgmPoint3)> {
    let mut points = Vec::new();
    let mut collect = |hit: Option<RgmPoint3>| {
        if let Some(point) = hit {
            push_unique_point(&mut points, point, tol * 4.0);
        }
    };

    collect(segment_triangle_intersection(a0, a1, b0, b1, b2, tol));
    collect(segment_triangle_intersection(a1, a2, b0, b1, b2, tol));
    collect(segment_triangle_intersection(a2, a0, b0, b1, b2, tol));
    collect(segment_triangle_intersection(b0, b1, a0, a1, a2, tol));
    collect(segment_triangle_intersection(b1, b2, a0, a1, a2, tol));
    collect(segment_triangle_intersection(b2, b0, a0, a1, a2, tol));

    if points.len() < 2 {
        return None;
    }
    let mut best = (points[0], points[1]);
    let mut best_len = distance(points[0], points[1]);
    for i in 0..points.len() {
        for j in (i + 1)..points.len() {
            let len = distance(points[i], points[j]);
            if len > best_len {
                best_len = len;
                best = (points[i], points[j]);
            }
        }
    }
    if best_len <= tol {
        None
    } else {
        Some(best)
    }
}

