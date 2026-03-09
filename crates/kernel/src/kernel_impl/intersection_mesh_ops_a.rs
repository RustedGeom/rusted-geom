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
