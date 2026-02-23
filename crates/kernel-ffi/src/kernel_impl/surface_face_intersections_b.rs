fn project_point_to_curve(
    state: &SessionState,
    curve: &CurveData,
    point: RgmPoint3,
    t_seed: f64,
    tol: f64,
) -> Option<(RgmPoint3, f64, f64)> {
    let mut t = t_seed.clamp(0.0, 1.0);
    let mut lambda = 1e-12;
    let mut best: Option<(f64, RgmPoint3, f64)> = None;
    let mut stagnation: u8 = 0;
    let mut prev_best = f64::INFINITY;
    for _ in 0..24 {
        let eval = evaluate_curve_at_normalized_data(state, curve, t).ok()?;
        let residual = v3::sub(eval.point, point);
        let residual_norm = v3::norm(residual);
        match best {
            Some((best_norm, _, _)) if residual_norm >= best_norm => {}
            _ => best = Some((residual_norm, eval.point, t)),
        }
        let improvement = (prev_best - residual_norm) / prev_best.max(1e-30);
        if improvement < 0.01 {
            stagnation += 1;
        } else {
            stagnation = 0;
        }
        prev_best = prev_best.min(residual_norm);
        if stagnation >= 3 {
            break;
        }
        if residual_norm <= tol {
            return Some((eval.point, t, residual_norm));
        }

        let grad = v3::dot(eval.d1, residual);
        let hess = v3::dot(eval.d1, eval.d1) + v3::dot(eval.d2, residual) + lambda;
        if hess.abs() <= 1e-16 {
            lambda *= 10.0;
            continue;
        }
        let delta = -grad / hess;

        let mut accepted = false;
        let mut alpha = 1.0;
        while alpha >= (1.0 / 64.0) {
            let trial_t = (t + alpha * delta).clamp(0.0, 1.0);
            let Some(trial_eval) = evaluate_curve_at_normalized_data(state, curve, trial_t).ok()
            else {
                alpha *= 0.5;
                continue;
            };
            let trial_norm = v3::distance(trial_eval.point, point);
            if trial_norm < residual_norm {
                t = trial_t;
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

    best.map(|(residual_norm, point, t)| (point, t, residual_norm))
}

fn project_point_to_curve_multi_seed(
    state: &SessionState,
    curve: &CurveData,
    point: RgmPoint3,
    seeds: &[f64],
    tol: f64,
) -> Option<(RgmPoint3, f64, f64)> {
    let mut best: Option<(RgmPoint3, f64, f64)> = None;
    for seed in seeds {
        let Some(candidate) = project_point_to_curve(state, curve, point, *seed, tol) else {
            continue;
        };
        match best {
            Some((_, _, best_res)) if candidate.2 >= best_res => {}
            _ => best = Some(candidate),
        }
    }
    best
}

#[derive(Clone, Copy)]
struct SurfaceProjectionSeed {
    uv: RgmUv2,
    point: RgmPoint3,
}

fn build_surface_projection_seed_grid(
    surface: &SurfaceData,
    u_steps: usize,
    v_steps: usize,
) -> Vec<SurfaceProjectionSeed> {
    let mut out = Vec::with_capacity((u_steps + 1) * (v_steps + 1));
    for iu in 0..=u_steps {
        let u_norm = iu as f64 / u_steps.max(1) as f64;
        for iv in 0..=v_steps {
            let v_norm = iv as f64 / v_steps.max(1) as f64;
            let uv_norm = RgmUv2 {
                u: u_norm,
                v: v_norm,
            };
            let Ok(uv) =
                math::nurbs_surface_eval::map_normalized_to_surface_uv(&surface.core, uv_norm)
            else {
                continue;
            };
            let Ok(frame) = eval_surface_data_uv(surface, uv) else {
                continue;
            };
            out.push(SurfaceProjectionSeed {
                uv,
                point: frame.point,
            });
        }
    }
    out
}

/// Spatial acceleration structure for O(1) nearest-seed UV lookup.
/// Seeds are binned into 3-D world-space cells for fast neighbor queries.
struct SurfaceSeedGrid {
    seeds: Vec<SurfaceProjectionSeed>,
    cells: std::collections::HashMap<(i32, i32, i32), Vec<usize>>,
    inv_cell: f64,
    #[allow(dead_code)]
    tol: f64,
}

impl SurfaceSeedGrid {
    fn build(seeds: Vec<SurfaceProjectionSeed>, cell_size: f64) -> Self {
        let cell = cell_size.max(1e-12);
        let inv_cell = 1.0 / cell;
        let mut cells: std::collections::HashMap<(i32, i32, i32), Vec<usize>> =
            std::collections::HashMap::new();
        for (idx, seed) in seeds.iter().enumerate() {
            let key = (
                (seed.point.x * inv_cell).floor() as i32,
                (seed.point.y * inv_cell).floor() as i32,
                (seed.point.z * inv_cell).floor() as i32,
            );
            cells.entry(key).or_default().push(idx);
        }
        Self { seeds, cells, inv_cell, tol: cell }
    }

    fn nearest_k(&self, point: RgmPoint3, k: usize) -> Vec<RgmUv2> {
        let kx = (point.x * self.inv_cell).floor() as i32;
        let ky = (point.y * self.inv_cell).floor() as i32;
        let kz = (point.z * self.inv_cell).floor() as i32;

        let mut candidates: Vec<(f64, RgmUv2)> = Vec::with_capacity(k * 4);
        for dx in -1_i32..=1 {
            for dy in -1_i32..=1 {
                for dz in -1_i32..=1 {
                    if let Some(indices) = self.cells.get(&(kx + dx, ky + dy, kz + dz)) {
                        for &idx in indices {
                            let d = v3::distance(self.seeds[idx].point, point);
                            candidates.push((d, self.seeds[idx].uv));
                        }
                    }
                }
            }
        }

        // If no neighbors in 3x3x3 neighborhood, fall back to full scan
        if candidates.is_empty() {
            candidates = self
                .seeds
                .iter()
                .map(|s| (v3::distance(s.point, point), s.uv))
                .collect();
        }

        candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
        candidates.into_iter().take(k.max(1)).map(|(_, uv)| uv).collect()
    }
}

fn nearest_surface_seed_uvs(
    seeds: &[SurfaceProjectionSeed],
    point: RgmPoint3,
    max_count: usize,
) -> Vec<RgmUv2> {
    let mut ranked = seeds
        .iter()
        .map(|seed| {
            let d = v3::distance(seed.point, point);
            (d, seed.uv)
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| a.0.total_cmp(&b.0));
    ranked
        .into_iter()
        .take(max_count.max(1))
        .map(|(_, uv)| uv)
        .collect()
}

#[derive(Clone, Copy)]
struct CurveProjectionSeed {
    t: f64,
    point: RgmPoint3,
}

fn build_curve_projection_seed_grid(
    state: &SessionState,
    curve: &CurveData,
    sample_count: usize,
) -> Vec<CurveProjectionSeed> {
    let mut out = Vec::with_capacity(sample_count + 1);
    for i in 0..=sample_count {
        let t = i as f64 / sample_count.max(1) as f64;
        let Ok(point) = curve_point_at_normalized_data(state, curve, t) else {
            continue;
        };
        out.push(CurveProjectionSeed { t, point });
    }
    out
}

fn nearest_curve_seed_t(
    seeds: &[CurveProjectionSeed],
    point: RgmPoint3,
    max_count: usize,
) -> Vec<f64> {
    let mut ranked = seeds
        .iter()
        .map(|seed| {
            let d = v3::distance(seed.point, point);
            (d, seed.t)
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| a.0.total_cmp(&b.0));
    ranked
        .into_iter()
        .take(max_count.max(1))
        .map(|(_, t)| t)
        .collect()
}

fn push_unique_uv_seed(seeds: &mut Vec<RgmUv2>, uv: RgmUv2, tol: f64) {
    if seeds.iter().any(|value| uv_distance(*value, uv) <= tol) {
        return;
    }
    seeds.push(uv);
}

fn push_unique_t_seed(seeds: &mut Vec<f64>, t: f64, tol: f64) {
    if seeds.iter().any(|value| (value - t).abs() <= tol) {
        return;
    }
    seeds.push(t);
}

fn collect_surface_surface_projection_seeds(
    source: &SurfaceData,
    target: &SurfaceData,
    _target_seed_grid: &[SurfaceProjectionSeed],
    target_seed_grid_index: &SurfaceSeedGrid,
    source_is_a: bool,
    u_steps: usize,
    v_steps: usize,
    seed_tol: f64,
    tol: f64,
    out: &mut Vec<(RgmPoint3, RgmUv2, RgmUv2)>,
    deduper: &mut BranchSpatialDeduper,
) {
    // Build flat list of (iu, iv) positions
    let positions: Vec<(usize, usize)> = (0..=u_steps)
        .flat_map(|iu| (0..=v_steps).map(move |iv| (iu, iv)))
        .collect();

    // On native targets run in parallel; on WASM single-threaded avoids rayon overhead.
    macro_rules! map_seed_cell {
        ($iter:expr) => {
            $iter
                .map(|&(iu, iv)| {
                    let u_norm = iu as f64 / u_steps.max(1) as f64;
                    let v_norm = iv as f64 / v_steps.max(1) as f64;
                    let uv_norm = RgmUv2 { u: u_norm, v: v_norm };
                    let uv_src = math::nurbs_surface_eval::map_normalized_to_surface_uv(
                        &source.core, uv_norm,
                    ).ok()?;
                    let frame_src = eval_surface_data_uv(source, uv_src).ok()?;

                    let mut seeds = Vec::new();
                    for nearest in target_seed_grid_index.nearest_k(frame_src.point, 6) {
                        push_unique_uv_seed(&mut seeds, nearest, tol * 2.0);
                    }
                    if let Ok(mapped) = math::nurbs_surface_eval::map_normalized_to_surface_uv(
                        &target.core, uv_norm,
                    ) {
                        push_unique_uv_seed(&mut seeds, mapped, tol * 2.0);
                    }
                    if let Ok(mapped_swapped) = math::nurbs_surface_eval::map_normalized_to_surface_uv(
                        &target.core,
                        RgmUv2 { u: v_norm, v: u_norm },
                    ) {
                        push_unique_uv_seed(&mut seeds, mapped_swapped, tol * 2.0);
                    }
                    if seeds.is_empty() {
                        return None;
                    }

                    let (_, uv_target, residual) =
                        project_point_to_surface_multi_seed(target, frame_src.point, &seeds, tol)?;
                    if residual > seed_tol {
                        return None;
                    }

                    let (seed_uv_a, seed_uv_b) = if source_is_a {
                        (uv_src, uv_target)
                    } else {
                        (uv_target, uv_src)
                    };
                    let (point, uv_a, uv_b) = if source_is_a {
                        refine_surface_surface_uv_pair(source, target, seed_uv_a, seed_uv_b, tol)?
                    } else {
                        refine_surface_surface_uv_pair(target, source, seed_uv_a, seed_uv_b, tol)?
                    };
                    Some((point, uv_a, uv_b))
                })
                .collect::<Vec<_>>()
        };
    }
    #[cfg(not(target_arch = "wasm32"))]
    let candidates: Vec<Option<(RgmPoint3, RgmUv2, RgmUv2)>> = map_seed_cell!(positions.par_iter());
    #[cfg(target_arch = "wasm32")]
    let candidates: Vec<Option<(RgmPoint3, RgmUv2, RgmUv2)>> = map_seed_cell!(positions.iter());

    // Sequential dedup pass
    for candidate in candidates.into_iter().flatten() {
        let (point, uv_a, uv_b) = candidate;
        if deduper.has_duplicate(point) {
            continue;
        }
        deduper.insert(point);
        out.push((point, uv_a, uv_b));
    }
}

fn estimate_seed_grid_cell_size(seeds: &[SurfaceProjectionSeed]) -> f64 {
    if seeds.len() < 2 {
        return 1.0;
    }
    // Compute average spacing as a rough cell size
    let mut total = 0.0;
    let count = (seeds.len() - 1).min(20);
    for i in 0..count {
        total += v3::distance(seeds[i].point, seeds[i + 1].point);
    }
    (total / count as f64 * 2.0).max(1e-6)
}

fn generate_surface_surface_seeds(
    surface_a: &SurfaceData,
    surface_b: &SurfaceData,
    options_a: RgmSurfaceTessellationOptions,
    options_b: RgmSurfaceTessellationOptions,
    tol: f64,
) -> Vec<(RgmPoint3, RgmUv2, RgmUv2)> {
    let scale = surface_world_scale(surface_a).max(surface_world_scale(surface_b));
    let seed_tol = intersection_chord_tol_from_scale(scale, tol)
        .max(options_a.chord_tol.max(options_b.chord_tol))
        * 24.0;

    let mut out = Vec::new();
    let mut deduper = BranchSpatialDeduper::new((seed_tol * 1.5).max(tol * 100.0));

    let mut u_steps = options_a.min_u_segments.max(6) as usize;
    let mut v_steps = options_a.min_v_segments.max(6) as usize;
    let max_u = options_a
        .max_u_segments
        .max(options_a.min_u_segments)
        .max(options_b.max_u_segments)
        .max(options_b.min_u_segments)
        .min(30)
        .max(6) as usize;
    let max_v = options_a
        .max_v_segments
        .max(options_a.min_v_segments)
        .max(options_b.max_v_segments)
        .max(options_b.min_v_segments)
        .min(30)
        .max(6) as usize;
    let target_seed_count = 3usize;
    loop {
        let prev_count = out.len();
        let seed_grid_a =
            build_surface_projection_seed_grid(surface_a, u_steps.min(12), v_steps.min(12));
        let seed_grid_b =
            build_surface_projection_seed_grid(surface_b, u_steps.min(12), v_steps.min(12));
        // Compute cell size from typical point spacing
        let cell_size_a = estimate_seed_grid_cell_size(&seed_grid_a);
        let cell_size_b = estimate_seed_grid_cell_size(&seed_grid_b);
        let index_a = SurfaceSeedGrid::build(seed_grid_a.clone(), cell_size_a);
        let index_b = SurfaceSeedGrid::build(seed_grid_b.clone(), cell_size_b);
        collect_surface_surface_projection_seeds(
            surface_a,
            surface_b,
            &seed_grid_b,
            &index_b,
            true,
            u_steps,
            v_steps,
            seed_tol,
            tol,
            &mut out,
            &mut deduper,
        );
        collect_surface_surface_projection_seeds(
            surface_b,
            surface_a,
            &seed_grid_a,
            &index_a,
            false,
            u_steps,
            v_steps,
            seed_tol,
            tol,
            &mut out,
            &mut deduper,
        );
        if out.len() >= target_seed_count {
            break;
        }
        if out.len() > 0 && out.len() == prev_count {
            break;
        }
        if u_steps == max_u && v_steps == max_v {
            break;
        }
        let next_u = (u_steps.saturating_mul(2)).min(max_u).max(u_steps);
        let next_v = (v_steps.saturating_mul(2)).min(max_v).max(v_steps);
        if next_u == u_steps && next_v == v_steps {
            break;
        }
        u_steps = next_u;
        v_steps = next_v;
    }
    out
}

fn march_surface_surface_direction(
    surface_a: &SurfaceData,
    surface_b: &SurfaceData,
    seed_point: RgmPoint3,
    seed_uv_a: RgmUv2,
    seed_uv_b: RgmUv2,
    direction_sign: f64,
    step_min: f64,
    step_max: f64,
    max_steps: usize,
    tol: f64,
) -> (Vec<(RgmPoint3, RgmUv2, RgmUv2)>, bool) {
    let mut uv_a = seed_uv_a;
    let mut uv_b = seed_uv_b;
    let mut point = seed_point;
    let mut step = step_max;
    let mut prev_dir: Option<[f64; 4]> = None;
    let mut out = Vec::new();
    let mut closed = false;

    for _ in 0..max_steps {
        let Ok(frame_a) = eval_surface_data_uv(surface_a, uv_a) else {
            break;
        };
        let Ok(frame_b) = eval_surface_data_uv(surface_b, uv_b) else {
            break;
        };
        let Some(mut dir) = surface_surface_tangent_dir(frame_a, frame_b) else {
            break;
        };
        if direction_sign < 0.0 {
            for value in &mut dir {
                *value = -*value;
            }
        }
        if let Some(prev) = prev_dir {
            if dot4(dir, prev) < 0.0 {
                for value in &mut dir {
                    *value = -*value;
                }
            }
        }

        let pred_uv_a = clamp_surface_uv(
            surface_a,
            RgmUv2 {
                u: uv_a.u + step * dir[0],
                v: uv_a.v + step * dir[1],
            },
        );
        let pred_uv_b = clamp_surface_uv(
            surface_b,
            RgmUv2 {
                u: uv_b.u + step * dir[2],
                v: uv_b.v + step * dir[3],
            },
        );
        let Some((next_point, next_uv_a, next_uv_b)) = project_surface_surface_curve_point(
            surface_a, surface_b, pred_uv_a, pred_uv_b, pred_uv_a, pred_uv_b, dir, tol,
        ) else {
            step *= 0.5;
            if step < step_min {
                break;
            }
            continue;
        };

        let seg_len = v3::distance(point, next_point);
        if seg_len <= tol * 0.5 {
            step *= 0.5;
            if step < step_min {
                break;
            }
            continue;
        }

        out.push((next_point, next_uv_a, next_uv_b));
        if out.len() > 16 && v3::distance(next_point, seed_point) <= step * 0.9 {
            closed = true;
            break;
        }
        point = next_point;
        uv_a = next_uv_a;
        uv_b = next_uv_b;
        prev_dir = Some(dir);
        step = (step * 1.15).min(step_max);
    }
    (out, closed)
}

fn build_surface_surface_branch_from_seed(
    surface_a: &SurfaceData,
    surface_b: &SurfaceData,
    seed_uv_a: RgmUv2,
    seed_uv_b: RgmUv2,
    tol: f64,
    step_min: f64,
    step_max: f64,
    max_steps: usize,
) -> Option<IntersectionBranchData> {
    let (seed_point, seed_uv_a, seed_uv_b) =
        refine_surface_surface_uv_pair(surface_a, surface_b, seed_uv_a, seed_uv_b, tol)?;
    // rayon::join gives real parallelism on native; WASM is single-threaded so
    // pay no scheduling overhead — run sequentially there.
    #[cfg(not(target_arch = "wasm32"))]
    let ((forward, closed_fwd), (backward, closed_bwd)) = rayon::join(
        || march_surface_surface_direction(
            surface_a, surface_b, seed_point, seed_uv_a, seed_uv_b,
            1.0, step_min, step_max, max_steps, tol,
        ),
        || march_surface_surface_direction(
            surface_a, surface_b, seed_point, seed_uv_a, seed_uv_b,
            -1.0, step_min, step_max, max_steps, tol,
        ),
    );
    #[cfg(target_arch = "wasm32")]
    let ((forward, closed_fwd), (backward, closed_bwd)) = (
        march_surface_surface_direction(
            surface_a, surface_b, seed_point, seed_uv_a, seed_uv_b,
            1.0, step_min, step_max, max_steps, tol,
        ),
        march_surface_surface_direction(
            surface_a, surface_b, seed_point, seed_uv_a, seed_uv_b,
            -1.0, step_min, step_max, max_steps, tol,
        ),
    );

    let mut points = Vec::new();
    let mut uv_a = Vec::new();
    let mut uv_b = Vec::new();
    for sample in backward.iter().rev() {
        points.push(sample.0);
        uv_a.push(sample.1);
        uv_b.push(sample.2);
    }
    points.push(seed_point);
    uv_a.push(seed_uv_a);
    uv_b.push(seed_uv_b);
    for sample in &forward {
        points.push(sample.0);
        uv_a.push(sample.1);
        uv_b.push(sample.2);
    }
    if points.len() < 2 {
        return None;
    }

    let mut compact_points = Vec::with_capacity(points.len());
    let mut compact_uv_a = Vec::with_capacity(uv_a.len());
    let mut compact_uv_b = Vec::with_capacity(uv_b.len());
    for idx in 0..points.len() {
        if compact_points
            .last()
            .map(|prev| v3::distance(*prev, points[idx]) <= tol * 0.5)
            .unwrap_or(false)
        {
            continue;
        }
        compact_points.push(points[idx]);
        compact_uv_a.push(uv_a[idx]);
        compact_uv_b.push(uv_b[idx]);
    }

    Some(IntersectionBranchData {
        points: compact_points,
        uv_a: compact_uv_a,
        uv_b: compact_uv_b,
        curve_t: Vec::new(),
        closed: closed_fwd || closed_bwd,
        flags: 0,
    })
}

fn generate_surface_curve_candidates(
    state: &SessionState,
    surface: &SurfaceData,
    curve: &CurveData,
    sample_count: usize,
    tol: f64,
    seed_tol: f64,
) -> Vec<(RgmPoint3, RgmUv2, f64)> {
    let mut out = Vec::new();
    let mut deduper = BranchSpatialDeduper::new(seed_tol * 1.5);
    let surface_seed_grid = build_surface_projection_seed_grid(surface, 14, 14);
    let curve_seed_grid = build_curve_projection_seed_grid(state, curve, sample_count.max(96));
    let mut last_uv = clamp_surface_uv(
        surface,
        RgmUv2 {
            u: 0.5 * (surface.core.u_start + surface.core.u_end),
            v: 0.5 * (surface.core.v_start + surface.core.v_end),
        },
    );
    let mut last_t = 0.5;
    for i in 0..=sample_count {
        let t = i as f64 / sample_count.max(1) as f64;
        let Ok(curve_point) = curve_point_at_normalized_data(state, curve, t) else {
            continue;
        };
        let uv_norm_a = RgmUv2 { u: t, v: 0.5 };
        let uv_norm_b = RgmUv2 {
            u: (1.0 - t).clamp(0.0, 1.0),
            v: 0.5,
        };
        let mut seeds = Vec::new();
        push_unique_uv_seed(&mut seeds, last_uv, tol * 2.0);
        for nearest in nearest_surface_seed_uvs(&surface_seed_grid, curve_point, 6) {
            push_unique_uv_seed(&mut seeds, nearest, tol * 2.0);
        }
        if let Ok(seed_a) =
            math::nurbs_surface_eval::map_normalized_to_surface_uv(&surface.core, uv_norm_a)
        {
            push_unique_uv_seed(&mut seeds, seed_a, tol * 2.0);
        }
        if let Ok(seed_b) =
            math::nurbs_surface_eval::map_normalized_to_surface_uv(&surface.core, uv_norm_b)
        {
            push_unique_uv_seed(&mut seeds, seed_b, tol * 2.0);
        }
        if seeds.is_empty() {
            continue;
        }
        let Some((_, uv, residual)) =
            project_point_to_surface_multi_seed(surface, curve_point, &seeds, tol)
        else {
            continue;
        };
        last_uv = uv;
        last_t = t;
        if residual > seed_tol * 1.8 {
            continue;
        }
        let Some((point, uv, t_hit)) = refine_surface_curve_hit(state, surface, curve, uv, t, tol)
        else {
            continue;
        };
        if deduper.has_duplicate(point) {
            continue;
        }
        deduper.insert(point);
        out.push((point, uv, t_hit));
    }

    if out.is_empty() {
        for seed in &surface_seed_grid {
            let mut t_seeds = Vec::new();
            push_unique_t_seed(&mut t_seeds, last_t, 1e-6);
            for nearest in nearest_curve_seed_t(&curve_seed_grid, seed.point, 8) {
                push_unique_t_seed(&mut t_seeds, nearest, 1e-6);
            }
            if t_seeds.is_empty() {
                continue;
            }
            let Some((_, t_seed, residual)) =
                project_point_to_curve_multi_seed(state, curve, seed.point, &t_seeds, tol)
            else {
                continue;
            };
            if residual > seed_tol * 2.2 {
                continue;
            }
            let Some((point, uv, t_hit)) =
                refine_surface_curve_hit(state, surface, curve, seed.uv, t_seed, tol)
            else {
                continue;
            };
            last_t = t_hit;
            if deduper.has_duplicate(point) {
                continue;
            }
            deduper.insert(point);
            out.push((point, uv, t_hit));
            if out.len() >= 24 {
                return out;
            }
        }
    }

    if out.is_empty() {
        let dense_surface_seed_grid = build_surface_projection_seed_grid(surface, 24, 24);
        for seed in &dense_surface_seed_grid {
            let t_seeds = nearest_curve_seed_t(&curve_seed_grid, seed.point, 12);
            let Some((_, t_seed, residual)) =
                project_point_to_curve_multi_seed(state, curve, seed.point, &t_seeds, tol)
            else {
                continue;
            };
            if residual > seed_tol * 2.8 {
                continue;
            }
            let Some((point, uv, t_hit)) =
                refine_surface_curve_hit(state, surface, curve, seed.uv, t_seed, tol)
            else {
                continue;
            };
            if deduper.has_duplicate(point) {
                continue;
            }
            deduper.insert(point);
            out.push((point, uv, t_hit));
            if out.len() >= 32 {
                break;
            }
        }
    }
    out
}

#[derive(Clone, Copy)]
struct SurfacePlaneGridSample {
    uv: RgmUv2,
    point: RgmPoint3,
    signed: f64,
}

