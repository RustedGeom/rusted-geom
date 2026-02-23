fn adaptive_sample_pcurve_segment(
    state: &SessionState,
    curve: &CurveData,
    reverse: bool,
    t0: f64,
    uv0: RgmUv2,
    t1: f64,
    uv1: RgmUv2,
    depth: u32,
    max_depth: u32,
    chord_tol: f64,
    out: &mut Vec<RgmUv2>,
) -> Result<(), RgmStatus> {
    let tm = 0.5 * (t0 + t1);
    let sample_t = if reverse { 1.0 - tm } else { tm };
    let uvm = uv_from_point3(curve_point_at_normalized_data(state, curve, sample_t)?);
    let deviation = uv_point_segment_distance(uvm, uv0, uv1);
    if depth >= max_depth || deviation <= chord_tol {
        out.push(uv1);
        return Ok(());
    }

    adaptive_sample_pcurve_segment(
        state,
        curve,
        reverse,
        t0,
        uv0,
        tm,
        uvm,
        depth + 1,
        max_depth,
        chord_tol,
        out,
    )?;
    adaptive_sample_pcurve_segment(
        state,
        curve,
        reverse,
        tm,
        uvm,
        t1,
        uv1,
        depth + 1,
        max_depth,
        chord_tol,
        out,
    )
}

fn sample_trim_edge_curve_uv(
    state: &SessionState,
    curve_handle: RgmObjectHandle,
    start_uv: RgmUv2,
    end_uv: RgmUv2,
    tol: f64,
) -> Result<Vec<RgmUv2>, RgmStatus> {
    let curve = find_curve(state, curve_handle)?;
    let p0 = uv_from_point3(curve_point_at_normalized_data(state, curve, 0.0)?);
    let p1 = uv_from_point3(curve_point_at_normalized_data(state, curve, 1.0)?);

    let forward = uv_distance(p0, start_uv) + uv_distance(p1, end_uv);
    let reverse = uv_distance(p0, end_uv) + uv_distance(p1, start_uv);
    let reverse_dir = reverse + tol < forward;

    let s0 = if reverse_dir { p1 } else { p0 };
    let s1 = if reverse_dir { p0 } else { p1 };
    // Keep trim pcurve flattening accurate but bounded for interactive use.
    let chord_tol = (tol * 2000.0).max(2e-4);
    let max_depth = 9;
    let mut samples = vec![s0];
    adaptive_sample_pcurve_segment(
        state,
        curve,
        reverse_dir,
        0.0,
        s0,
        1.0,
        s1,
        0,
        max_depth,
        chord_tol,
        &mut samples,
    )?;
    Ok(normalize_trim_edge_samples(start_uv, end_uv, samples, tol))
}

fn trim_loop_polyline(loop_data: &TrimLoopData) -> Vec<RgmUv2> {
    if loop_data.edges.is_empty() {
        return Vec::new();
    }

    let mut points = Vec::new();
    for (edge_idx, edge) in loop_data.edges.iter().enumerate() {
        let samples = if edge.uv_samples.len() >= 2 {
            &edge.uv_samples
        } else {
            // fallback for legacy edges without sampled pcurve data
            if edge_idx == 0 {
                points.push(edge.start_uv);
            }
            points.push(edge.end_uv);
            continue;
        };
        if edge_idx == 0 {
            points.push(samples[0]);
        }
        points.extend(samples.iter().skip(1).copied());
    }
    points
}

#[derive(Clone, Debug)]
struct FaceLoopClassifierData {
    is_outer: bool,
    polyline: Vec<RgmUv2>,
    min_u: f64,
    max_u: f64,
    min_v: f64,
    max_v: f64,
}

#[derive(Clone, Debug)]
struct FaceUvClassifier {
    loops: Vec<FaceLoopClassifierData>,
}

impl FaceUvClassifier {
    fn from_face(face: &FaceData) -> Self {
        let mut loops = Vec::with_capacity(face.loops.len());
        for loop_data in &face.loops {
            let polyline = trim_loop_polyline(loop_data);
            if polyline.len() < 3 {
                continue;
            }
            let mut min_u = f64::INFINITY;
            let mut max_u = f64::NEG_INFINITY;
            let mut min_v = f64::INFINITY;
            let mut max_v = f64::NEG_INFINITY;
            for point in &polyline {
                min_u = min_u.min(point.u);
                max_u = max_u.max(point.u);
                min_v = min_v.min(point.v);
                max_v = max_v.max(point.v);
            }
            loops.push(FaceLoopClassifierData {
                is_outer: loop_data.is_outer,
                polyline,
                min_u,
                max_u,
                min_v,
                max_v,
            });
        }
        Self { loops }
    }

    fn classify(&self, uv: RgmUv2, tol: f64) -> i32 {
        if self.loops.is_empty() {
            return 1;
        }

        for loop_data in &self.loops {
            if uv.u < loop_data.min_u - tol
                || uv.u > loop_data.max_u + tol
                || uv.v < loop_data.min_v - tol
                || uv.v > loop_data.max_v + tol
            {
                continue;
            }
            for seg in loop_data.polyline.windows(2) {
                if is_uv_on_segment(uv, seg[0], seg[1], tol) {
                    return 0;
                }
            }
        }

        let mut in_outer = false;
        for loop_data in &self.loops {
            if uv.u < loop_data.min_u - tol
                || uv.u > loop_data.max_u + tol
                || uv.v < loop_data.min_v - tol
                || uv.v > loop_data.max_v + tol
            {
                continue;
            }
            let inside = point_in_polygon_uv(uv, &loop_data.polyline);
            if loop_data.is_outer {
                if inside {
                    in_outer = true;
                }
            } else if inside {
                return -1;
            }
        }

        if in_outer {
            1
        } else {
            -1
        }
    }
}

fn point_in_polygon_uv(point: RgmUv2, polygon: &[RgmUv2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let pi = polygon[i];
        let pj = polygon[j];
        let intersects = ((pi.v > point.v) != (pj.v > point.v))
            && (point.u < (pj.u - pi.u) * (point.v - pi.v) / (pj.v - pi.v + f64::EPSILON) + pi.u);
        if intersects {
            inside = !inside;
        }
        j = i;
    }

    inside
}

fn is_uv_on_segment(point: RgmUv2, a: RgmUv2, b: RgmUv2, tol: f64) -> bool {
    let ab = RgmVec2 {
        x: b.u - a.u,
        y: b.v - a.v,
    };
    let ap = RgmVec2 {
        x: point.u - a.u,
        y: point.v - a.v,
    };
    let cross = ab.x * ap.y - ab.y * ap.x;
    if cross.abs() > tol {
        return false;
    }
    let dot = ap.x * ab.x + ap.y * ab.y;
    if dot < -tol {
        return false;
    }
    let len2 = ab.x * ab.x + ab.y * ab.y;
    dot <= len2 + tol
}

#[cfg(test)]
fn classify_uv_in_face(face: &FaceData, uv: RgmUv2, tol: f64) -> i32 {
    let classifier = FaceUvClassifier::from_face(face);
    classifier.classify(uv, tol)
}

fn canonical_uv_key(uv: RgmUv2) -> (u64, u64) {
    let u = if uv.u == -0.0 { 0.0 } else { uv.u };
    let v = if uv.v == -0.0 { 0.0 } else { uv.v };
    (u.to_bits(), v.to_bits())
}

fn classify_uv_cached(
    classifier: &FaceUvClassifier,
    uv: RgmUv2,
    tol: f64,
    cache: &mut HashMap<(u64, u64), i32>,
) -> i32 {
    let key = canonical_uv_key(uv);
    if let Some(value) = cache.get(&key) {
        return *value;
    }
    let value = classifier.classify(uv, tol);
    cache.insert(key, value);
    value
}

fn orient2d_uv(a: RgmUv2, b: RgmUv2, c: RgmUv2) -> f64 {
    (b.u - a.u) * (c.v - a.v) - (b.v - a.v) * (c.u - a.u)
}

fn point_in_triangle_uv_strict(point: RgmUv2, tri: [RgmUv2; 3], tol: f64) -> bool {
    let o0 = orient2d_uv(tri[0], tri[1], point);
    let o1 = orient2d_uv(tri[1], tri[2], point);
    let o2 = orient2d_uv(tri[2], tri[0], point);
    (o0 > tol && o1 > tol && o2 > tol) || (o0 < -tol && o1 < -tol && o2 < -tol)
}

fn segments_intersect_uv(a0: RgmUv2, a1: RgmUv2, b0: RgmUv2, b1: RgmUv2, tol: f64) -> bool {
    let o1 = orient2d_uv(a0, a1, b0);
    let o2 = orient2d_uv(a0, a1, b1);
    let o3 = orient2d_uv(b0, b1, a0);
    let o4 = orient2d_uv(b0, b1, a1);

    let strict_cross = ((o1 > tol && o2 < -tol) || (o1 < -tol && o2 > tol))
        && ((o3 > tol && o4 < -tol) || (o3 < -tol && o4 > tol));
    if strict_cross {
        return true;
    }

    (o1.abs() <= tol && is_uv_on_segment(b0, a0, a1, tol))
        || (o2.abs() <= tol && is_uv_on_segment(b1, a0, a1, tol))
        || (o3.abs() <= tol && is_uv_on_segment(a0, b0, b1, tol))
        || (o4.abs() <= tol && is_uv_on_segment(a1, b0, b1, tol))
}

fn triangle_max_uv_edge(tri: [RgmUv2; 3]) -> f64 {
    uv_distance(tri[0], tri[1])
        .max(uv_distance(tri[1], tri[2]))
        .max(uv_distance(tri[2], tri[0]))
}

fn triangle_degenerate_uv(tri: [RgmUv2; 3], tol: f64) -> bool {
    orient2d_uv(tri[0], tri[1], tri[2]).abs() <= tol * tol
}

fn dedupe_polygon_uv(mut points: Vec<RgmUv2>, tol: f64) -> Vec<RgmUv2> {
    if points.is_empty() {
        return points;
    }
    let mut deduped = Vec::with_capacity(points.len());
    for point in points.drain(..) {
        if deduped
            .last()
            .map(|last| uv_distance(*last, point) <= tol)
            .unwrap_or(false)
        {
            continue;
        }
        deduped.push(point);
    }
    if deduped.len() >= 2 && uv_distance(deduped[0], deduped[deduped.len() - 1]) <= tol {
        deduped.pop();
    }
    deduped
}

fn edge_aabb_overlap(a0: RgmUv2, a1: RgmUv2, tri: [RgmUv2; 3], tol: f64) -> bool {
    let seg_umin = a0.u.min(a1.u) - tol;
    let seg_umax = a0.u.max(a1.u) + tol;
    let seg_vmin = a0.v.min(a1.v) - tol;
    let seg_vmax = a0.v.max(a1.v) + tol;

    let tri_umin = tri[0].u.min(tri[1].u).min(tri[2].u) - tol;
    let tri_umax = tri[0].u.max(tri[1].u).max(tri[2].u) + tol;
    let tri_vmin = tri[0].v.min(tri[1].v).min(tri[2].v) - tol;
    let tri_vmax = tri[0].v.max(tri[1].v).max(tri[2].v) + tol;

    !(seg_umax < tri_umin || seg_umin > tri_umax || seg_vmax < tri_vmin || seg_vmin > tri_vmax)
}

#[derive(Clone, Copy)]
struct TrimSegmentRecord {
    a0: RgmUv2,
    a1: RgmUv2,
    min_u: f64,
    max_u: f64,
    min_v: f64,
    max_v: f64,
    centroid_u: f64,
    centroid_v: f64,
}

#[derive(Clone, Copy)]
struct TrimSegmentBvhNode {
    min_u: f64,
    max_u: f64,
    min_v: f64,
    max_v: f64,
    left: Option<usize>,
    right: Option<usize>,
    start: usize,
    count: usize,
}

impl TrimSegmentBvhNode {
    fn is_leaf(self) -> bool {
        self.count > 0
    }
}

struct TrimSegmentBvh {
    records: Vec<TrimSegmentRecord>,
    indices: Vec<usize>,
    nodes: Vec<TrimSegmentBvhNode>,
    root: usize,
}

fn build_trim_segment_bvh(segments: &[(RgmUv2, RgmUv2)]) -> Option<TrimSegmentBvh> {
    if segments.is_empty() {
        return None;
    }
    let records = segments
        .iter()
        .map(|(a0, a1)| TrimSegmentRecord {
            a0: *a0,
            a1: *a1,
            min_u: a0.u.min(a1.u),
            max_u: a0.u.max(a1.u),
            min_v: a0.v.min(a1.v),
            max_v: a0.v.max(a1.v),
            centroid_u: (a0.u + a1.u) * 0.5,
            centroid_v: (a0.v + a1.v) * 0.5,
        })
        .collect::<Vec<_>>();
    let mut indices = (0..records.len()).collect::<Vec<_>>();
    let mut nodes = Vec::<TrimSegmentBvhNode>::new();

    fn build_node(
        records: &[TrimSegmentRecord],
        indices: &mut [usize],
        global_start: usize,
        nodes: &mut Vec<TrimSegmentBvhNode>,
    ) -> usize {
        let mut min_u = f64::INFINITY;
        let mut max_u = -f64::INFINITY;
        let mut min_v = f64::INFINITY;
        let mut max_v = -f64::INFINITY;
        let mut cmin_u = f64::INFINITY;
        let mut cmax_u = -f64::INFINITY;
        let mut cmin_v = f64::INFINITY;
        let mut cmax_v = -f64::INFINITY;
        for &idx in indices.iter() {
            let rec = records[idx];
            min_u = min_u.min(rec.min_u);
            max_u = max_u.max(rec.max_u);
            min_v = min_v.min(rec.min_v);
            max_v = max_v.max(rec.max_v);
            cmin_u = cmin_u.min(rec.centroid_u);
            cmax_u = cmax_u.max(rec.centroid_u);
            cmin_v = cmin_v.min(rec.centroid_v);
            cmax_v = cmax_v.max(rec.centroid_v);
        }

        if indices.len() <= 12 {
            let node_idx = nodes.len();
            nodes.push(TrimSegmentBvhNode {
                min_u,
                max_u,
                min_v,
                max_v,
                left: None,
                right: None,
                start: global_start,
                count: indices.len(),
            });
            return node_idx;
        }

        let axis_u = (cmax_u - cmin_u).abs();
        let axis_v = (cmax_v - cmin_v).abs();
        if axis_u >= axis_v {
            indices.sort_unstable_by(|a, b| {
                records[*a]
                    .centroid_u
                    .partial_cmp(&records[*b].centroid_u)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        } else {
            indices.sort_unstable_by(|a, b| {
                records[*a]
                    .centroid_v
                    .partial_cmp(&records[*b].centroid_v)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        let mid = indices.len() / 2;
        let (left_slice, right_slice) = indices.split_at_mut(mid);
        let node_idx = nodes.len();
        nodes.push(TrimSegmentBvhNode {
            min_u,
            max_u,
            min_v,
            max_v,
            left: None,
            right: None,
            start: 0,
            count: 0,
        });
        let left = build_node(records, left_slice, global_start, nodes);
        let right = build_node(records, right_slice, global_start + mid, nodes);
        nodes[node_idx].left = Some(left);
        nodes[node_idx].right = Some(right);
        node_idx
    }

    let root = build_node(&records, &mut indices, 0, &mut nodes);
    Some(TrimSegmentBvh {
        records,
        indices,
        nodes,
        root,
    })
}

fn trim_segment_aabb_overlap_tri(rec: TrimSegmentRecord, tri: [RgmUv2; 3], tol: f64) -> bool {
    let tri_min_u = tri[0].u.min(tri[1].u).min(tri[2].u) - tol;
    let tri_max_u = tri[0].u.max(tri[1].u).max(tri[2].u) + tol;
    let tri_min_v = tri[0].v.min(tri[1].v).min(tri[2].v) - tol;
    let tri_max_v = tri[0].v.max(tri[1].v).max(tri[2].v) + tol;
    !(rec.max_u + tol < tri_min_u
        || rec.min_u - tol > tri_max_u
        || rec.max_v + tol < tri_min_v
        || rec.min_v - tol > tri_max_v)
}

fn trim_node_aabb_overlap_tri(node: TrimSegmentBvhNode, tri: [RgmUv2; 3], tol: f64) -> bool {
    let tri_min_u = tri[0].u.min(tri[1].u).min(tri[2].u) - tol;
    let tri_max_u = tri[0].u.max(tri[1].u).max(tri[2].u) + tol;
    let tri_min_v = tri[0].v.min(tri[1].v).min(tri[2].v) - tol;
    let tri_max_v = tri[0].v.max(tri[1].v).max(tri[2].v) + tol;
    !(node.max_u + tol < tri_min_u
        || node.min_u - tol > tri_max_u
        || node.max_v + tol < tri_min_v
        || node.min_v - tol > tri_max_v)
}

fn triangle_has_trim_crossing(
    tri: [RgmUv2; 3],
    trim_segments: &[(RgmUv2, RgmUv2)],
    trim_bvh: Option<&TrimSegmentBvh>,
    tol: f64,
) -> bool {
    let tri_edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
    if let Some(bvh) = trim_bvh {
        let mut stack = vec![bvh.root];
        while let Some(node_idx) = stack.pop() {
            let node = bvh.nodes[node_idx];
            if !trim_node_aabb_overlap_tri(node, tri, tol) {
                continue;
            }
            if node.is_leaf() {
                for rec_idx in &bvh.indices[node.start..(node.start + node.count)] {
                    let rec = bvh.records[*rec_idx];
                    if !trim_segment_aabb_overlap_tri(rec, tri, tol) {
                        continue;
                    }
                    if point_in_triangle_uv_strict(rec.a0, tri, tol)
                        || point_in_triangle_uv_strict(rec.a1, tri, tol)
                    {
                        return true;
                    }
                    for (e0, e1) in tri_edges {
                        if segments_intersect_uv(rec.a0, rec.a1, e0, e1, tol) {
                            return true;
                        }
                    }
                }
            } else {
                if let Some(left) = node.left {
                    stack.push(left);
                }
                if let Some(right) = node.right {
                    stack.push(right);
                }
            }
        }
        return false;
    }

    for (s0, s1) in trim_segments {
        if !edge_aabb_overlap(*s0, *s1, tri, tol) {
            continue;
        }

        if point_in_triangle_uv_strict(*s0, tri, tol) || point_in_triangle_uv_strict(*s1, tri, tol)
        {
            return true;
        }

        for (e0, e1) in tri_edges {
            if segments_intersect_uv(*s0, *s1, e0, e1, tol) {
                return true;
            }
        }
    }
    false
}

fn collect_trim_segments(face: &FaceData) -> Vec<(RgmUv2, RgmUv2)> {
    let mut out = Vec::new();
    for loop_data in &face.loops {
        for edge in &loop_data.edges {
            let samples = if edge.uv_samples.len() >= 2 {
                edge.uv_samples.as_slice()
            } else {
                &[] as &[RgmUv2]
            };
            if samples.is_empty() {
                out.push((edge.start_uv, edge.end_uv));
            } else {
                for seg in samples.windows(2) {
                    out.push((seg[0], seg[1]));
                }
            }
        }
    }
    out
}

