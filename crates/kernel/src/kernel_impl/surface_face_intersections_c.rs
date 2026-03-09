fn plane_signed_distance(point: RgmPoint3, origin: RgmPoint3, normal: RgmVec3) -> f64 {
    v3::dot(v3::sub(point, origin), normal)
}

fn point_segment_distance(point: RgmPoint3, a: RgmPoint3, b: RgmPoint3) -> f64 {
    let ab = v3::sub(b, a);
    let ap = v3::sub(point, a);
    let ab_len2 = v3::dot(ab, ab);
    if ab_len2 <= f64::EPSILON {
        return v3::distance(point, a);
    }
    let t = (v3::dot(ap, ab) / ab_len2).clamp(0.0, 1.0);
    let proj = v3::add_vec(a, v3::scale(ab, t));
    v3::distance(point, proj)
}

fn project_surface_plane_curve_point(
    surface: &SurfaceData,
    plane_origin: RgmPoint3,
    normal: RgmVec3,
    uv_seed: RgmUv2,
    uv_anchor: RgmUv2,
    uv_tangent: RgmVec2,
    tol: f64,
) -> Option<(RgmPoint3, RgmUv2)> {
    let tangent_norm2 = uv_tangent.x * uv_tangent.x + uv_tangent.y * uv_tangent.y;
    if tangent_norm2 <= 1e-24 {
        let uv = clamp_surface_uv(surface, uv_seed);
        let frame = eval_surface_data_uv(surface, uv).ok()?;
        return Some((frame.point, uv));
    }

    let mut uv = clamp_surface_uv(surface, uv_seed);
    let mut lambda = 1e-12;
    let mut best: Option<(f64, f64, RgmPoint3, RgmUv2)> = None;
    let mut stagnation: u8 = 0;
    let mut prev_err = f64::INFINITY;
    for _ in 0..20 {
        let frame = eval_surface_data_uv(surface, uv).ok()?;
        let f1 = plane_signed_distance(frame.point, plane_origin, normal);
        let f2 = (uv.u - uv_anchor.u) * uv_tangent.x + (uv.v - uv_anchor.v) * uv_tangent.y;
        let err2 = f1.abs() + f2.abs();
        match best {
            Some((best_err, _, _, _)) if err2 >= best_err => {}
            _ => best = Some((err2, f1.abs(), frame.point, uv)),
        }
        if f1.abs() <= tol && f2.abs() <= 1e-12 {
            return Some((frame.point, uv));
        }
        let improvement = (prev_err - err2) / prev_err.max(1e-30);
        if improvement < 0.01 { stagnation += 1; } else { stagnation = 0; }
        prev_err = prev_err.min(err2);
        if stagnation >= 3 { break; }

        let mut jtj = [[0.0; 2]; 2];
        let mut jtr = [0.0; 2];
        let j1 = [v3::dot(frame.du, normal), v3::dot(frame.dv, normal)];
        let j2 = [uv_tangent.x, uv_tangent.y];
        for i in 0..2 {
            jtr[i] = j1[i] * f1 + j2[i] * f2;
            for j in 0..2 {
                jtj[i][j] = j1[i] * j1[j] + j2[i] * j2[j];
            }
            jtj[i][i] += lambda;
        }
        let rhs = [-jtr[0], -jtr[1]];
        let Some(delta) = solve_linear_system::<2>(jtj, rhs) else {
            lambda *= 10.0;
            continue;
        };

        let mut accepted = false;
        let mut alpha = 1.0;
        while alpha >= (1.0 / 64.0) {
            let trial_uv = clamp_surface_uv(
                surface,
                RgmUv2 {
                    u: uv.u + alpha * delta[0],
                    v: uv.v + alpha * delta[1],
                },
            );
            let Some(trial_frame) = eval_surface_data_uv(surface, trial_uv).ok() else {
                alpha *= 0.5;
                continue;
            };
            let trial_f1 = plane_signed_distance(trial_frame.point, plane_origin, normal);
            let trial_f2 = (trial_uv.u - uv_anchor.u) * uv_tangent.x
                + (trial_uv.v - uv_anchor.v) * uv_tangent.y;
            if trial_f1.abs() + trial_f2.abs() < err2 {
                uv = trial_uv;
                lambda = (lambda * 0.5).max(1e-14);
                accepted = true;
                break;
            }
            alpha *= 0.5;
        }
        if !accepted {
            lambda = (lambda * 10.0).min(1e6);
        }
    }

    best.and_then(|(_, best_f1, point, uv)| {
        if best_f1 <= tol * 8.0 {
            Some((point, uv))
        } else {
            None
        }
    })
}

fn refine_surface_plane_segment_recursive(
    surface: &SurfaceData,
    plane_origin: RgmPoint3,
    normal: RgmVec3,
    p0: RgmPoint3,
    uv0: RgmUv2,
    p1: RgmPoint3,
    uv1: RgmUv2,
    chord_tol: f64,
    tol: f64,
    depth: u32,
    out_points: &mut Vec<RgmPoint3>,
    out_uv: &mut Vec<RgmUv2>,
) {
    if depth >= 10 {
        out_points.push(p1);
        out_uv.push(uv1);
        return;
    }

    let uv_mid_seed = uv_lerp(uv0, uv1, 0.5);
    let uv_tangent = RgmVec2 {
        x: uv1.u - uv0.u,
        y: uv1.v - uv0.v,
    };
    let Some((p_mid, uv_mid)) = project_surface_plane_curve_point(
        surface,
        plane_origin,
        normal,
        uv_mid_seed,
        uv_mid_seed,
        uv_tangent,
        tol,
    ) else {
        out_points.push(p1);
        out_uv.push(uv1);
        return;
    };

    let chord_dev = point_segment_distance(p_mid, p0, p1);
    let plane_res = plane_signed_distance(p_mid, plane_origin, normal).abs();
    if chord_dev <= chord_tol && plane_res <= tol * 2.0 {
        out_points.push(p1);
        out_uv.push(uv1);
        return;
    }

    refine_surface_plane_segment_recursive(
        surface,
        plane_origin,
        normal,
        p0,
        uv0,
        p_mid,
        uv_mid,
        chord_tol,
        tol,
        depth + 1,
        out_points,
        out_uv,
    );
    refine_surface_plane_segment_recursive(
        surface,
        plane_origin,
        normal,
        p_mid,
        uv_mid,
        p1,
        uv1,
        chord_tol,
        tol,
        depth + 1,
        out_points,
        out_uv,
    );
}

fn refine_surface_plane_branch_polyline(
    surface: &SurfaceData,
    plane_origin: RgmPoint3,
    normal: RgmVec3,
    branch: &IntersectionBranchData,
    chord_tol: f64,
    tol: f64,
) -> IntersectionBranchData {
    if branch.points.len() < 2 || branch.uv_a.len() != branch.points.len() {
        return branch.clone();
    }

    let mut out_points = Vec::with_capacity(branch.points.len() * 2);
    let mut out_uv = Vec::with_capacity(branch.uv_a.len() * 2);
    out_points.push(branch.points[0]);
    out_uv.push(branch.uv_a[0]);
    for idx in 1..branch.points.len() {
        refine_surface_plane_segment_recursive(
            surface,
            plane_origin,
            normal,
            branch.points[idx - 1],
            branch.uv_a[idx - 1],
            branch.points[idx],
            branch.uv_a[idx],
            chord_tol,
            tol,
            0,
            &mut out_points,
            &mut out_uv,
        );
    }
    IntersectionBranchData {
        points: out_points,
        uv_a: out_uv,
        uv_b: Vec::new(),
        curve_t: Vec::new(),
        closed: branch.closed,
        flags: branch.flags,
    }
}

fn refine_surface_plane_edge_hit(
    surface: &SurfaceData,
    a: SurfacePlaneGridSample,
    b: SurfacePlaneGridSample,
    plane_origin: RgmPoint3,
    normal: RgmVec3,
    tol: f64,
) -> Result<(RgmPoint3, RgmUv2), RgmStatus> {
    if a.signed.abs() <= tol {
        return Ok((a.point, a.uv));
    }
    if b.signed.abs() <= tol {
        return Ok((b.point, b.uv));
    }

    let mut ta = 0.0;
    let mut tb = 1.0;
    let mut fa = a.signed;
    let mut fb = b.signed;
    let mut best_point = a.point;
    let mut best_uv = a.uv;
    let mut best_abs = fa.abs();
    if fb.abs() < best_abs {
        best_abs = fb.abs();
        best_point = b.point;
        best_uv = b.uv;
    }

    for _ in 0..24 {
        let tm = 0.5 * (ta + tb);
        let uv = uv_lerp(a.uv, b.uv, tm);
        let frame = eval_surface_data_uv(surface, uv)?;
        let fm = plane_signed_distance(frame.point, plane_origin, normal);
        let abs_fm = fm.abs();
        if abs_fm < best_abs {
            best_abs = abs_fm;
            best_point = frame.point;
            best_uv = uv;
        }
        if abs_fm <= tol || (tb - ta).abs() <= 1e-8 {
            return Ok((frame.point, uv));
        }

        if (fa > 0.0 && fm < 0.0) || (fa < 0.0 && fm > 0.0) {
            tb = tm;
            fb = fm;
        } else if (fb > 0.0 && fm < 0.0) || (fb < 0.0 && fm > 0.0) {
            ta = tm;
            fa = fm;
        } else {
            // No strict bracketing (near tangent); narrow towards best side deterministically.
            if fa.abs() <= fb.abs() {
                tb = tm;
                fb = fm;
            } else {
                ta = tm;
                fa = fm;
            }
        }
    }

    Ok((best_point, best_uv))
}

fn edge_crosses_plane(f0: f64, f1: f64, tol: f64) -> bool {
    f0.abs() <= tol || f1.abs() <= tol || (f0 > tol && f1 < -tol) || (f0 < -tol && f1 > tol)
}

fn intersect_surface_plane_uv_segments_for_grid(
    surface: &SurfaceData,
    plane_origin: RgmPoint3,
    normal: RgmVec3,
    u_steps: usize,
    v_steps: usize,
    tol: f64,
) -> Result<Vec<((RgmPoint3, RgmUv2), (RgmPoint3, RgmUv2))>, RgmStatus> {
    let mut grid = Vec::with_capacity((u_steps + 1) * (v_steps + 1));
    for iu in 0..=u_steps {
        let u_norm = iu as f64 / u_steps as f64;
        for iv in 0..=v_steps {
            let v_norm = iv as f64 / v_steps as f64;
            let uv_norm = RgmUv2 {
                u: u_norm,
                v: v_norm,
            };
            let uv =
                math::nurbs_surface_eval::map_normalized_to_surface_uv(&surface.core, uv_norm)?;
            let frame = eval_surface_data_uv(surface, uv)?;
            let signed = plane_signed_distance(frame.point, plane_origin, normal);
            grid.push(SurfacePlaneGridSample {
                uv,
                point: frame.point,
                signed,
            });
        }
    }

    let index_of = |iu: usize, iv: usize| -> usize { iu * (v_steps + 1) + iv };
    let mut out = Vec::new();
    for iu in 0..u_steps {
        for iv in 0..v_steps {
            let a = grid[index_of(iu, iv)];
            let b = grid[index_of(iu + 1, iv)];
            let c = grid[index_of(iu, iv + 1)];
            let d = grid[index_of(iu + 1, iv + 1)];
            let edges = [(a, b), (b, d), (d, c), (c, a)];
            let mut hits = Vec::new();
            for (e0, e1) in edges {
                if !edge_crosses_plane(e0.signed, e1.signed, tol) {
                    continue;
                }
                let hit =
                    refine_surface_plane_edge_hit(surface, e0, e1, plane_origin, normal, tol)?;
                if hits
                    .iter()
                    .all(|(_, uv)| uv_distance(*uv, hit.1) > tol * 8.0)
                {
                    hits.push(hit);
                }
            }

            if hits.len() == 2 {
                out.push((hits[0], hits[1]));
            } else if hits.len() == 4 {
                out.push((hits[0], hits[1]));
                out.push((hits[2], hits[3]));
            }
        }
    }
    Ok(out)
}

fn intersect_surface_plane_uv_segments(
    surface: &SurfaceData,
    plane_origin: RgmPoint3,
    normal: RgmVec3,
    options: RgmSurfaceTessellationOptions,
) -> Result<Vec<((RgmPoint3, RgmUv2), (RgmPoint3, RgmUv2))>, RgmStatus> {
    let min_u_steps = options.min_u_segments.max(4) as usize;
    let min_v_steps = options.min_v_segments.max(4) as usize;
    let max_u_steps = options.max_u_segments.max(options.min_u_segments).max(4) as usize;
    let max_v_steps = options.max_v_segments.max(options.min_v_segments).max(4) as usize;
    let tol = surface.core.tol.abs_tol.max(1e-7);

    // Control-hull plane pre-check (NURBS convex-hull property):
    // if all control points in world space are strictly on one side, guaranteed miss.
    {
        let tol_check = surface.core.tol.abs_tol.max(1e-7);
        let mut all_positive = true;
        let mut all_negative = true;
        for cp in &surface.core.control_points {
            let world_cp = matrix_apply_point(surface.transform, *cp);
            let sd = plane_signed_distance(world_cp, plane_origin, normal);
            if sd <= tol_check {
                all_positive = false;
            }
            if sd >= -tol_check {
                all_negative = false;
            }
            if !all_positive && !all_negative {
                break;
            }
        }
        if all_positive || all_negative {
            return Ok(Vec::new());
        }
    }

    let mut u_steps = min_u_steps;
    let mut v_steps = min_v_steps;
    let mut best_segments = Vec::new();
    loop {
        let segments = intersect_surface_plane_uv_segments_for_grid(
            surface,
            plane_origin,
            normal,
            u_steps,
            v_steps,
            tol,
        )?;
        if !segments.is_empty() {
            best_segments = segments;
        }
        if u_steps == max_u_steps && v_steps == max_v_steps {
            return Ok(best_segments);
        }

        let next_u = (u_steps.saturating_mul(2)).min(max_u_steps).max(u_steps);
        let next_v = (v_steps.saturating_mul(2)).min(max_v_steps).max(v_steps);
        if next_u == u_steps && next_v == v_steps {
            return Ok(best_segments);
        }
        u_steps = next_u;
        v_steps = next_v;
    }
}

struct BranchSpatialDeduper {
    tol: f64,
    inv_cell: f64,
    buckets: HashMap<(i64, i64, i64), Vec<usize>>,
    mids: Vec<RgmPoint3>,
}

impl BranchSpatialDeduper {
    fn new(tol: f64) -> Self {
        let cell = tol.max(1e-12);
        Self {
            tol: cell,
            inv_cell: 1.0 / cell,
            buckets: HashMap::new(),
            mids: Vec::new(),
        }
    }

    fn key(&self, point: RgmPoint3) -> (i64, i64, i64) {
        (
            (point.x * self.inv_cell).floor() as i64,
            (point.y * self.inv_cell).floor() as i64,
            (point.z * self.inv_cell).floor() as i64,
        )
    }

    fn has_duplicate(&self, point: RgmPoint3) -> bool {
        let (kx, ky, kz) = self.key(point);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let key = (kx + dx, ky + dy, kz + dz);
                    if let Some(indices) = self.buckets.get(&key) {
                        for idx in indices {
                            if v3::distance(point, self.mids[*idx]) <= self.tol {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn insert(&mut self, point: RgmPoint3) {
        let idx = self.mids.len();
        self.mids.push(point);
        let key = self.key(point);
        self.buckets.entry(key).or_default().push(idx);
    }
}

fn push_unique_branch_fast(
    branches: &mut Vec<IntersectionBranchData>,
    deduper: &mut BranchSpatialDeduper,
    branch: IntersectionBranchData,
) {
    if branch.points.is_empty() {
        return;
    }
    let mid = if branch.points.len() == 1 {
        branch.points[0]
    } else {
        midpoint(branch.points[0], branch.points[branch.points.len() - 1])
    };
    if deduper.has_duplicate(mid) {
        return;
    }
    deduper.insert(mid);
    branches.push(branch);
}


