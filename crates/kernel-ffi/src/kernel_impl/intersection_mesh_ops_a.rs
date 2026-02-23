fn interpolate_branch_sample(
    branch: &IntersectionBranchData,
    idx0: usize,
    idx1: usize,
    alpha: f64,
    has_uv_a: bool,
    has_uv_b: bool,
    has_curve_t: bool,
) -> (RgmPoint3, Option<RgmUv2>, Option<RgmUv2>, Option<f64>) {
    let p0 = branch.points[idx0];
    let p1 = branch.points[idx1];
    let point = v3::add_vec(p0, v3::scale(v3::sub(p1, p0), alpha));
    let uv_a = if has_uv_a {
        Some(uv_lerp(branch.uv_a[idx0], branch.uv_a[idx1], alpha))
    } else {
        None
    };
    let uv_b = if has_uv_b {
        Some(uv_lerp(branch.uv_b[idx0], branch.uv_b[idx1], alpha))
    } else {
        None
    };
    let curve_t = if has_curve_t {
        Some(branch.curve_t[idx0] + (branch.curve_t[idx1] - branch.curve_t[idx0]) * alpha)
    } else {
        None
    };
    (point, uv_a, uv_b, curve_t)
}

fn append_branch_sample(
    branch: &mut IntersectionBranchData,
    point: RgmPoint3,
    uv_a: Option<RgmUv2>,
    uv_b: Option<RgmUv2>,
    curve_t: Option<f64>,
) {
    branch.points.push(point);
    if let Some(uv) = uv_a {
        branch.uv_a.push(uv);
    }
    if let Some(uv) = uv_b {
        branch.uv_b.push(uv);
    }
    if let Some(t) = curve_t {
        branch.curve_t.push(t);
    }
}

fn reverse_branch(branch: &IntersectionBranchData) -> IntersectionBranchData {
    let mut out = branch.clone();
    out.points.reverse();
    out.uv_a.reverse();
    out.uv_b.reverse();
    out.curve_t.reverse();
    out
}

fn branch_extend_forward(target: &mut IntersectionBranchData, source: &IntersectionBranchData) {
    let has_uv_a =
        target.uv_a.len() == target.points.len() && source.uv_a.len() == source.points.len();
    let has_uv_b =
        target.uv_b.len() == target.points.len() && source.uv_b.len() == source.points.len();
    let has_curve_t =
        target.curve_t.len() == target.points.len() && source.curve_t.len() == source.points.len();
    target.points.extend(source.points.iter().skip(1).copied());
    if has_uv_a {
        target.uv_a.extend(source.uv_a.iter().skip(1).copied());
    } else {
        target.uv_a.clear();
    }
    if has_uv_b {
        target.uv_b.extend(source.uv_b.iter().skip(1).copied());
    } else {
        target.uv_b.clear();
    }
    if has_curve_t {
        target
            .curve_t
            .extend(source.curve_t.iter().skip(1).copied());
    } else {
        target.curve_t.clear();
    }
}

fn stitch_intersection_branches(
    mut branches: Vec<IntersectionBranchData>,
    tol: f64,
) -> Vec<IntersectionBranchData> {
    if branches.len() <= 1 {
        return branches;
    }

    let mut changed = true;
    while changed {
        changed = false;
        'outer: for i in 0..branches.len() {
            if branches[i].points.len() < 2 {
                continue;
            }
            let a_start = branches[i].points[0];
            let a_end = branches[i].points[branches[i].points.len() - 1];
            for j in (i + 1)..branches.len() {
                if branches[j].points.len() < 2 {
                    continue;
                }
                let b_start = branches[j].points[0];
                let b_end = branches[j].points[branches[j].points.len() - 1];

                let mut merged: Option<IntersectionBranchData> = None;
                if v3::distance(a_end, b_start) <= tol {
                    let mut out = branches[i].clone();
                    branch_extend_forward(&mut out, &branches[j]);
                    merged = Some(out);
                } else if v3::distance(a_end, b_end) <= tol {
                    let mut out = branches[i].clone();
                    let rev = reverse_branch(&branches[j]);
                    branch_extend_forward(&mut out, &rev);
                    merged = Some(out);
                } else if v3::distance(a_start, b_end) <= tol {
                    let mut out = branches[j].clone();
                    branch_extend_forward(&mut out, &branches[i]);
                    merged = Some(out);
                } else if v3::distance(a_start, b_start) <= tol {
                    let rev_b = reverse_branch(&branches[j]);
                    let mut out = rev_b;
                    branch_extend_forward(&mut out, &branches[i]);
                    merged = Some(out);
                }

                if let Some(mut out) = merged {
                    if out.points.len() >= 3
                        && v3::distance(out.points[0], out.points[out.points.len() - 1]) <= tol
                    {
                        out.closed = true;
                    }
                    branches[i] = out;
                    branches.swap_remove(j);
                    changed = true;
                    break 'outer;
                }
            }
        }
    }
    branches
}

fn adaptive_stitch_tolerance(branches: &[IntersectionBranchData], base_tol: f64) -> f64 {
    let mut length_sum = 0.0;
    let mut count = 0usize;
    for branch in branches {
        for idx in 1..branch.points.len() {
            let len = v3::distance(branch.points[idx - 1], branch.points[idx]);
            if len.is_finite() && len > 0.0 {
                length_sum += len;
                count += 1;
            }
        }
    }
    if count == 0 {
        return base_tol.max(1e-9);
    }
    let avg = length_sum / count as f64;
    (avg * 0.04).max(base_tol * 8.0).min(avg * 0.5)
}

#[derive(Clone, Copy)]
struct SurfacePlaneSegmentEdge {
    a: usize,
    b: usize,
    p0: RgmPoint3,
    p1: RgmPoint3,
    uv0: RgmUv2,
    uv1: RgmUv2,
}

fn cluster_uv_node(
    uv: RgmUv2,
    point: RgmPoint3,
    uv_tol: f64,
    world_tol: f64,
    nodes_uv: &mut Vec<RgmUv2>,
    nodes_point: &mut Vec<RgmPoint3>,
    buckets: &mut HashMap<(i64, i64), Vec<usize>>,
) -> usize {
    let inv = 1.0 / uv_tol.max(1e-14);
    let key = ((uv.u * inv).round() as i64, (uv.v * inv).round() as i64);
    for du in -1..=1 {
        for dv in -1..=1 {
            let nkey = (key.0 + du, key.1 + dv);
            if let Some(indices) = buckets.get(&nkey) {
                for idx in indices {
                    if uv_distance(nodes_uv[*idx], uv) <= uv_tol
                        || v3::distance(nodes_point[*idx], point) <= world_tol
                    {
                        return *idx;
                    }
                }
            }
        }
    }
    let idx = nodes_uv.len();
    nodes_uv.push(uv);
    nodes_point.push(point);
    buckets.entry(key).or_default().push(idx);
    idx
}

fn build_surface_plane_branches(
    segments: &[((RgmPoint3, RgmUv2), (RgmPoint3, RgmUv2))],
    uv_tol: f64,
    world_tol: f64,
) -> Vec<IntersectionBranchData> {
    if segments.is_empty() {
        return Vec::new();
    }

    let mut nodes_uv = Vec::<RgmUv2>::new();
    let mut nodes_point = Vec::<RgmPoint3>::new();
    let mut buckets: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    let mut edges = Vec::<SurfacePlaneSegmentEdge>::with_capacity(segments.len());

    for ((p0, uv0), (p1, uv1)) in segments.iter().copied() {
        let a = cluster_uv_node(
            uv0,
            p0,
            uv_tol,
            world_tol,
            &mut nodes_uv,
            &mut nodes_point,
            &mut buckets,
        );
        let b = cluster_uv_node(
            uv1,
            p1,
            uv_tol,
            world_tol,
            &mut nodes_uv,
            &mut nodes_point,
            &mut buckets,
        );
        if a == b {
            continue;
        }
        edges.push(SurfacePlaneSegmentEdge {
            a,
            b,
            p0,
            p1,
            uv0,
            uv1,
        });
    }

    if edges.is_empty() {
        return Vec::new();
    }

    let mut adjacency = vec![Vec::<usize>::new(); nodes_uv.len()];
    for (ei, edge) in edges.iter().enumerate() {
        adjacency[edge.a].push(ei);
        adjacency[edge.b].push(ei);
    }

    let mut used = vec![false; edges.len()];
    let mut branches = Vec::new();

    for edge_idx in 0..edges.len() {
        if used[edge_idx] {
            continue;
        }

        let edge = edges[edge_idx];
        let start_node = if adjacency[edge.a].len() != 2 {
            edge.a
        } else {
            edge.b
        };
        let mut current_node = start_node;
        let mut branch = IntersectionBranchData {
            points: Vec::new(),
            uv_a: Vec::new(),
            uv_b: Vec::new(),
            curve_t: Vec::new(),
            closed: false,
            flags: 1,
        };

        loop {
            let mut next_edge_idx: Option<usize> = None;
            for candidate in &adjacency[current_node] {
                if !used[*candidate] {
                    next_edge_idx = Some(*candidate);
                    break;
                }
            }
            let Some(ei) = next_edge_idx else {
                break;
            };
            used[ei] = true;
            let e = edges[ei];
            let (p_start, uv_start, p_end, uv_end, next_node) = if e.a == current_node {
                (e.p0, e.uv0, e.p1, e.uv1, e.b)
            } else {
                (e.p1, e.uv1, e.p0, e.uv0, e.a)
            };

            if branch.points.is_empty() {
                branch.points.push(p_start);
                branch.uv_a.push(uv_start);
            }
            branch.points.push(p_end);
            branch.uv_a.push(uv_end);
            current_node = next_node;
            if current_node == start_node {
                branch.closed = true;
                break;
            }
        }

        if branch.points.len() >= 2 {
            branches.push(branch);
        }
    }

    branches
}

fn clip_alpha_to_face_boundary(
    classifier: &FaceUvClassifier,
    uv0: RgmUv2,
    uv1: RgmUv2,
    inside_at_uv0: bool,
    tol: f64,
    cache: &mut HashMap<(u64, u64), i32>,
) -> f64 {
    let mut inside_t = if inside_at_uv0 { 0.0 } else { 1.0 };
    let mut outside_t = if inside_at_uv0 { 1.0 } else { 0.0 };
    for _ in 0..40 {
        let mid = 0.5 * (inside_t + outside_t);
        let uv_mid = uv_lerp(uv0, uv1, mid);
        let inside = classify_uv_cached(classifier, uv_mid, tol, cache) >= 0;
        if inside {
            inside_t = mid;
        } else {
            outside_t = mid;
        }
        if (inside_t - outside_t).abs() <= 1e-12 {
            break;
        }
    }
    inside_t.clamp(0.0, 1.0)
}

fn clip_branch_by_face(
    branch: &IntersectionBranchData,
    classifier: &FaceUvClassifier,
    use_uv_a: bool,
    tol: f64,
) -> Vec<IntersectionBranchData> {
    let count = branch.points.len();
    if count == 0 {
        return Vec::new();
    }

    let has_uv_a = branch.uv_a.len() == count;
    let has_uv_b = branch.uv_b.len() == count;
    let has_curve_t = branch.curve_t.len() == count;
    if (use_uv_a && !has_uv_a) || (!use_uv_a && !has_uv_b) {
        return vec![branch.clone()];
    }

    let uv_at = |idx: usize| -> RgmUv2 {
        if use_uv_a {
            branch.uv_a[idx]
        } else {
            branch.uv_b[idx]
        }
    };

    let mut cache = HashMap::new();
    let inside = (0..count)
        .map(|idx| classify_uv_cached(classifier, uv_at(idx), tol, &mut cache) >= 0)
        .collect::<Vec<_>>();

    if count == 1 {
        if inside[0] {
            return vec![branch.clone()];
        }
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut current: Option<IntersectionBranchData> = None;

    for idx in 0..(count - 1) {
        if current.is_none() && inside[idx] {
            let mut seed = IntersectionBranchData {
                points: Vec::new(),
                uv_a: Vec::new(),
                uv_b: Vec::new(),
                curve_t: Vec::new(),
                closed: false,
                flags: branch.flags,
            };
            append_branch_sample(
                &mut seed,
                branch.points[idx],
                if has_uv_a {
                    Some(branch.uv_a[idx])
                } else {
                    None
                },
                if has_uv_b {
                    Some(branch.uv_b[idx])
                } else {
                    None
                },
                if has_curve_t {
                    Some(branch.curve_t[idx])
                } else {
                    None
                },
            );
            current = Some(seed);
        }

        match (inside[idx], inside[idx + 1]) {
            (true, true) => {
                if let Some(active) = current.as_mut() {
                    append_branch_sample(
                        active,
                        branch.points[idx + 1],
                        if has_uv_a {
                            Some(branch.uv_a[idx + 1])
                        } else {
                            None
                        },
                        if has_uv_b {
                            Some(branch.uv_b[idx + 1])
                        } else {
                            None
                        },
                        if has_curve_t {
                            Some(branch.curve_t[idx + 1])
                        } else {
                            None
                        },
                    );
                }
            }
            (true, false) => {
                let alpha = clip_alpha_to_face_boundary(
                    classifier,
                    uv_at(idx),
                    uv_at(idx + 1),
                    true,
                    tol,
                    &mut cache,
                );
                let (point, uv_a, uv_b, curve_t) = interpolate_branch_sample(
                    branch,
                    idx,
                    idx + 1,
                    alpha,
                    has_uv_a,
                    has_uv_b,
                    has_curve_t,
                );
                if let Some(active) = current.as_mut() {
                    append_branch_sample(active, point, uv_a, uv_b, curve_t);
                }
                if let Some(active) = current.take() {
                    if active.points.len() >= 2 || (active.points.len() == 1 && count == 1) {
                        out.push(active);
                    }
                }
            }
            (false, true) => {
                let alpha = clip_alpha_to_face_boundary(
                    classifier,
                    uv_at(idx),
                    uv_at(idx + 1),
                    false,
                    tol,
                    &mut cache,
                );
                let (point, uv_a, uv_b, curve_t) = interpolate_branch_sample(
                    branch,
                    idx,
                    idx + 1,
                    alpha,
                    has_uv_a,
                    has_uv_b,
                    has_curve_t,
                );
                let mut active = IntersectionBranchData {
                    points: Vec::new(),
                    uv_a: Vec::new(),
                    uv_b: Vec::new(),
                    curve_t: Vec::new(),
                    closed: false,
                    flags: branch.flags,
                };
                append_branch_sample(&mut active, point, uv_a, uv_b, curve_t);
                append_branch_sample(
                    &mut active,
                    branch.points[idx + 1],
                    if has_uv_a {
                        Some(branch.uv_a[idx + 1])
                    } else {
                        None
                    },
                    if has_uv_b {
                        Some(branch.uv_b[idx + 1])
                    } else {
                        None
                    },
                    if has_curve_t {
                        Some(branch.curve_t[idx + 1])
                    } else {
                        None
                    },
                );
                current = Some(active);
            }
            (false, false) => {
                let uv_mid = uv_lerp(uv_at(idx), uv_at(idx + 1), 0.5);
                if classify_uv_cached(classifier, uv_mid, tol, &mut cache) >= 0 {
                    let alpha_enter_local = clip_alpha_to_face_boundary(
                        classifier,
                        uv_at(idx),
                        uv_mid,
                        false,
                        tol,
                        &mut cache,
                    );
                    let alpha_enter = (0.5 * alpha_enter_local).clamp(0.0, 1.0);
                    let alpha_exit_local = clip_alpha_to_face_boundary(
                        classifier,
                        uv_at(idx + 1),
                        uv_mid,
                        false,
                        tol,
                        &mut cache,
                    );
                    let alpha_exit = (1.0 - 0.5 * alpha_exit_local).clamp(0.0, 1.0);
                    if alpha_exit > alpha_enter + 1e-9 {
                        let (p0, uv_a0, uv_b0, t0) = interpolate_branch_sample(
                            branch,
                            idx,
                            idx + 1,
                            alpha_enter,
                            has_uv_a,
                            has_uv_b,
                            has_curve_t,
                        );
                        let (p1, uv_a1, uv_b1, t1) = interpolate_branch_sample(
                            branch,
                            idx,
                            idx + 1,
                            alpha_exit,
                            has_uv_a,
                            has_uv_b,
                            has_curve_t,
                        );
                        let mut active = IntersectionBranchData {
                            points: Vec::new(),
                            uv_a: Vec::new(),
                            uv_b: Vec::new(),
                            curve_t: Vec::new(),
                            closed: false,
                            flags: branch.flags,
                        };
                        append_branch_sample(&mut active, p0, uv_a0, uv_b0, t0);
                        append_branch_sample(&mut active, p1, uv_a1, uv_b1, t1);
                        out.push(active);
                    }
                }
            }
        }
    }

    if let Some(active) = current.take() {
        if active.points.len() >= 2 {
            out.push(active);
        }
    }

    out
}

fn clip_branch_against_faces(
    branch: &IntersectionBranchData,
    face_a_classifier: Option<&FaceUvClassifier>,
    face_b_classifier: Option<&FaceUvClassifier>,
    tol: f64,
) -> Vec<IntersectionBranchData> {
    let mut branches = vec![branch.clone()];
    if let Some(classifier) = face_a_classifier {
        let mut next = Vec::new();
        for candidate in &branches {
            next.extend(clip_branch_by_face(candidate, classifier, true, tol));
        }
        branches = next;
    }
    if let Some(classifier) = face_b_classifier {
        let mut next = Vec::new();
        for candidate in &branches {
            next.extend(clip_branch_by_face(candidate, classifier, false, tol));
        }
        branches = next;
    }
    branches
}

fn branch_within_face(
    branch: &IntersectionBranchData,
    classifier: &FaceUvClassifier,
    use_uv_a: bool,
    tol: f64,
) -> bool {
    let uvs = if use_uv_a { &branch.uv_a } else { &branch.uv_b };
    if uvs.is_empty() {
        return true;
    }
    uvs.iter().all(|uv| classifier.classify(*uv, tol) >= 0)
}

fn split_branch_inside_runs(
    branch: &IntersectionBranchData,
    classifier: &FaceUvClassifier,
    use_uv_a: bool,
    tol: f64,
) -> Vec<IntersectionBranchData> {
    let count = branch.points.len();
    if count == 0 {
        return Vec::new();
    }
    let uvs = if use_uv_a { &branch.uv_a } else { &branch.uv_b };
    if uvs.len() != count {
        return vec![branch.clone()];
    }
    let has_uv_a = branch.uv_a.len() == count;
    let has_uv_b = branch.uv_b.len() == count;
    let has_curve_t = branch.curve_t.len() == count;
    let inside = uvs
        .iter()
        .map(|uv| classifier.classify(*uv, tol) >= 0)
        .collect::<Vec<_>>();

    let mut runs = Vec::new();
    let mut start = 0usize;
    while start < count {
        while start < count && !inside[start] {
            start += 1;
        }
        if start >= count {
            break;
        }
        let mut end = start;
        while end + 1 < count && inside[end + 1] {
            end += 1;
        }
        if end > start || count == 1 {
            let mut out = IntersectionBranchData {
                points: Vec::new(),
                uv_a: Vec::new(),
                uv_b: Vec::new(),
                curve_t: Vec::new(),
                closed: false,
                flags: branch.flags,
            };
            for idx in start..=end {
                out.points.push(branch.points[idx]);
                if has_uv_a {
                    out.uv_a.push(branch.uv_a[idx]);
                }
                if has_uv_b {
                    out.uv_b.push(branch.uv_b[idx]);
                }
                if has_curve_t {
                    out.curve_t.push(branch.curve_t[idx]);
                }
            }
            if out.points.len() >= 2 || (count == 1 && out.points.len() == 1) {
                runs.push(out);
            }
        }
        start = end + 1;
    }
    runs
}

fn ensure_edge_samples(edge: &mut TrimEdgeData, tol: f64) {
    edge.uv_samples =
        normalize_trim_edge_samples(edge.start_uv, edge.end_uv, edge.uv_samples.clone(), tol);
}

fn ensure_loop_closed(loop_data: &mut TrimLoopData, tol: f64) {
    if loop_data.edges.is_empty() {
        return;
    }
    for edge in &mut loop_data.edges {
        ensure_edge_samples(edge, tol);
    }
    for idx in 1..loop_data.edges.len() {
        let prev_end = loop_data.edges[idx - 1].end_uv;
        if uv_distance(loop_data.edges[idx].start_uv, prev_end) <= tol {
            loop_data.edges[idx].start_uv = prev_end;
            if let Some(first) = loop_data.edges[idx].uv_samples.first_mut() {
                *first = prev_end;
            }
        }
    }
    let first_start = loop_data.edges[0].start_uv;
    let last_idx = loop_data.edges.len() - 1;
    if uv_distance(loop_data.edges[last_idx].end_uv, first_start) <= tol {
        loop_data.edges[last_idx].end_uv = first_start;
        if let Some(last) = loop_data.edges[last_idx].uv_samples.last_mut() {
            *last = first_start;
        }
    }
}

fn loop_signed_area(loop_data: &TrimLoopData) -> f64 {
    let poly = trim_loop_polyline(loop_data);
    if poly.len() < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    for idx in 0..(poly.len() - 1) {
        let a = poly[idx];
        let b = poly[idx + 1];
        area += a.u * b.v - b.u * a.v;
    }
    area * 0.5
}

fn heal_face(face: &mut FaceData, tol: f64) {
    for loop_data in &mut face.loops {
        loop_data
            .edges
            .retain(|edge| uv_distance(edge.start_uv, edge.end_uv) > tol);
        ensure_loop_closed(loop_data, tol);
        let area = loop_signed_area(loop_data);
        if loop_data.is_outer && area < 0.0 {
            loop_data.edges.reverse();
            for edge in &mut loop_data.edges {
                std::mem::swap(&mut edge.start_uv, &mut edge.end_uv);
                edge.uv_samples.reverse();
                ensure_edge_samples(edge, tol);
            }
        }
        if !loop_data.is_outer && area > 0.0 {
            loop_data.edges.reverse();
            for edge in &mut loop_data.edges {
                std::mem::swap(&mut edge.start_uv, &mut edge.end_uv);
                edge.uv_samples.reverse();
                ensure_edge_samples(edge, tol);
            }
        }
    }
}

