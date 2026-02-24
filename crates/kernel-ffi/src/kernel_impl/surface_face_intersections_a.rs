#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

fn rgm_mesh_transform_impl(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    transform: [[f64; 4]; 4],
    out_object: *mut RgmObjectHandle,
    message: &str,
) -> RgmStatus {
    create_mesh_object(
        session,
        out_object,
        |state| {
            let source = find_mesh(state, mesh)?;
            let mut next = source.clone();
            next.transform = matrix_mul(transform, source.transform);
            Ok(next)
        },
        message,
    )
}

fn rgm_mesh_bake_transform_impl(
    session: RgmKernelHandle,
    mesh: RgmObjectHandle,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    create_mesh_object(
        session,
        out_object,
        |state| {
            let source = find_mesh(state, mesh)?;
            let vertices = mesh_world_vertices(source);
            Ok(MeshData {
                vertices,
                triangles: source.triangles.clone(),
                transform: matrix_identity(),
            })
        },
        "Mesh bake transform failed",
    )
}

fn rgm_mesh_boolean_impl(
    session: RgmKernelHandle,
    mesh_a: RgmObjectHandle,
    mesh_b: RgmObjectHandle,
    op: i32,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    create_mesh_object(
        session,
        out_object,
        |state| {
            let a = find_mesh(state, mesh_a)?;
            let b = find_mesh(state, mesh_b)?;
            let a_vertices = mesh_world_vertices(a);
            let b_vertices = mesh_world_vertices(b);
            let mut a_pos = Vec::with_capacity(a_vertices.len() * 3);
            for vertex in &a_vertices {
                a_pos.push(vertex.x);
                a_pos.push(vertex.y);
                a_pos.push(vertex.z);
            }
            let mut b_pos = Vec::with_capacity(b_vertices.len() * 3);
            for vertex in &b_vertices {
                b_pos.push(vertex.x);
                b_pos.push(vertex.y);
                b_pos.push(vertex.z);
            }
            let mut a_indices = Vec::with_capacity(a.triangles.len() * 3);
            for tri in &a.triangles {
                a_indices.push(tri[0] as usize);
                a_indices.push(tri[1] as usize);
                a_indices.push(tri[2] as usize);
            }
            let mut b_indices = Vec::with_capacity(b.triangles.len() * 3);
            for tri in &b.triangles {
                b_indices.push(tri[0] as usize);
                b_indices.push(tri[1] as usize);
                b_indices.push(tri[2] as usize);
            }

            let manifold_a =
                Manifold::new(&a_pos, &a_indices).map_err(|_| RgmStatus::DegenerateGeometry)?;
            let manifold_b =
                Manifold::new(&b_pos, &b_indices).map_err(|_| RgmStatus::DegenerateGeometry)?;
            let op = match op {
                0 => BoolOpType::Add,
                1 => BoolOpType::Intersect,
                2 => BoolOpType::Subtract,
                _ => return Err(RgmStatus::InvalidInput),
            };
            let result = compute_boolean(&manifold_a, &manifold_b, op)
                .map_err(|_| RgmStatus::NumericalFailure)?;

            let out_vertices = result
                .ps
                .iter()
                .map(|vertex| RgmPoint3 {
                    x: vertex.x as f64,
                    y: vertex.y as f64,
                    z: vertex.z as f64,
                })
                .collect::<Vec<_>>();
            let out_triangles = result
                .get_indices()
                .iter()
                .map(|tri| {
                    Ok([
                        u32::try_from(tri.x).map_err(|_| RgmStatus::OutOfRange)?,
                        u32::try_from(tri.y).map_err(|_| RgmStatus::OutOfRange)?,
                        u32::try_from(tri.z).map_err(|_| RgmStatus::OutOfRange)?,
                    ])
                })
                .collect::<Result<Vec<[u32; 3]>, RgmStatus>>()?;

            Ok(MeshData {
                vertices: out_vertices,
                triangles: out_triangles,
                transform: matrix_identity(),
            })
        },
        "Mesh boolean failed",
    )
}

fn resolve_surface_operand(
    state: &SessionState,
    handle: RgmObjectHandle,
) -> Result<(SurfaceData, Option<FaceData>), RgmStatus> {
    match state.objects.get(&handle.0) {
        Some(GeometryObject::Surface(surface)) => Ok((surface.clone(), None)),
        Some(GeometryObject::Face(face)) => {
            let surface = find_surface(state, face.surface)?.clone();
            Ok((surface, Some(face.clone())))
        }
        Some(_) => Err(RgmStatus::InvalidInput),
        None => Err(RgmStatus::NotFound),
    }
}

fn normalize_param_periodic(value: f64, start: f64, end: f64) -> f64 {
    let period = end - start;
    if period.abs() <= f64::EPSILON {
        return start;
    }
    let mut out = value;
    while out < start {
        out += period;
    }
    while out >= end {
        out -= period;
    }
    out
}

fn clamp_surface_uv(surface: &SurfaceData, uv: RgmUv2) -> RgmUv2 {
    let u = if surface.core.periodic_u {
        normalize_param_periodic(uv.u, surface.core.u_start, surface.core.u_end)
    } else {
        uv.u.clamp(surface.core.u_start, surface.core.u_end)
    };
    let v = if surface.core.periodic_v {
        normalize_param_periodic(uv.v, surface.core.v_start, surface.core.v_end)
    } else {
        uv.v.clamp(surface.core.v_start, surface.core.v_end)
    };
    RgmUv2 { u, v }
}

fn refine_surface_surface_uv_pair(
    surface_a: &SurfaceData,
    surface_b: &SurfaceData,
    uv_a_seed: RgmUv2,
    uv_b_seed: RgmUv2,
    tol: f64,
) -> Option<(RgmPoint3, RgmUv2, RgmUv2)> {
    let mut uv_a = clamp_surface_uv(surface_a, uv_a_seed);
    let mut uv_b = clamp_surface_uv(surface_b, uv_b_seed);
    let mut lambda = 1e-12;
    let mut best: Option<(f64, RgmPoint3, RgmUv2, RgmUv2)> = None;
    let mut stagnation: u8 = 0;
    let mut prev_best = f64::INFINITY;

    for _ in 0..24 {
        let frame_a = eval_surface_data_uv(surface_a, uv_a).ok()?;
        let frame_b = eval_surface_data_uv(surface_b, uv_b).ok()?;
        let residual = v3::sub(frame_a.point, frame_b.point);
        let residual_norm = v3::norm(residual);

        let mid = midpoint(frame_a.point, frame_b.point);
        match best {
            Some((best_norm, _, _, _)) if residual_norm >= best_norm => {}
            _ => best = Some((residual_norm, mid, uv_a, uv_b)),
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
            return Some((mid, uv_a, uv_b));
        }

        let cols = [
            frame_a.du,
            frame_a.dv,
            v3::neg(frame_b.du),
            v3::neg(frame_b.dv),
        ];
        let mut jtj = [[0.0; 4]; 4];
        let mut jtr = [0.0; 4];
        for i in 0..4 {
            jtr[i] = v3::dot(cols[i], residual);
            for j in i..4 {
                let value = v3::dot(cols[i], cols[j]);
                jtj[i][j] = value;
                jtj[j][i] = value;
            }
            jtj[i][i] += lambda;
        }

        let rhs = [-jtr[0], -jtr[1], -jtr[2], -jtr[3]];
        let Some(delta) = solve_linear_system::<4>(jtj, rhs) else {
            lambda *= 10.0;
            continue;
        };
        let step_norm =
            (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2] + delta[3] * delta[3])
                .sqrt();
        if step_norm <= 1e-14 && residual_norm <= tol * 8.0 {
            return Some((mid, uv_a, uv_b));
        }

        let mut accepted = false;
        let mut alpha = 1.0;
        while alpha >= (1.0 / 64.0) {
            let trial_uv_a = clamp_surface_uv(
                surface_a,
                RgmUv2 {
                    u: uv_a.u + alpha * delta[0],
                    v: uv_a.v + alpha * delta[1],
                },
            );
            let trial_uv_b = clamp_surface_uv(
                surface_b,
                RgmUv2 {
                    u: uv_b.u + alpha * delta[2],
                    v: uv_b.v + alpha * delta[3],
                },
            );
            let Some(trial_a) = eval_surface_data_uv(surface_a, trial_uv_a).ok() else {
                alpha *= 0.5;
                continue;
            };
            let Some(trial_b) = eval_surface_data_uv(surface_b, trial_uv_b).ok() else {
                alpha *= 0.5;
                continue;
            };
            let trial_residual_norm = v3::norm(v3::sub(trial_a.point, trial_b.point));
            if trial_residual_norm < residual_norm {
                uv_a = trial_uv_a;
                uv_b = trial_uv_b;
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

    best.and_then(|(residual_norm, point, uv_a, uv_b)| {
        if residual_norm <= tol * 16.0 {
            Some((point, uv_a, uv_b))
        } else {
            None
        }
    })
}

fn project_surface_surface_curve_point(
    surface_a: &SurfaceData,
    surface_b: &SurfaceData,
    uv_a_seed: RgmUv2,
    uv_b_seed: RgmUv2,
    uv_a_anchor: RgmUv2,
    uv_b_anchor: RgmUv2,
    dir4: [f64; 4],
    tol: f64,
) -> Option<(RgmPoint3, RgmUv2, RgmUv2)> {
    let dir_norm2 = dir4[0] * dir4[0] + dir4[1] * dir4[1] + dir4[2] * dir4[2] + dir4[3] * dir4[3];
    if dir_norm2 <= 1e-24 {
        return refine_surface_surface_uv_pair(surface_a, surface_b, uv_a_seed, uv_b_seed, tol);
    }

    let mut uv_a = clamp_surface_uv(surface_a, uv_a_seed);
    let mut uv_b = clamp_surface_uv(surface_b, uv_b_seed);
    let mut best: Option<(f64, f64, RgmPoint3, RgmUv2, RgmUv2)> = None;
    let mut stagnation: u8 = 0;
    let mut prev_best = f64::INFINITY;
    for _ in 0..24 {
        let fa = eval_surface_data_uv(surface_a, uv_a).ok()?;
        let fb = eval_surface_data_uv(surface_b, uv_b).ok()?;
        let diff = v3::sub(fa.point, fb.point);
        let r3 = (uv_a.u - uv_a_anchor.u) * dir4[0]
            + (uv_a.v - uv_a_anchor.v) * dir4[1]
            + (uv_b.u - uv_b_anchor.u) * dir4[2]
            + (uv_b.v - uv_b_anchor.v) * dir4[3];
        let spatial = v3::norm(diff);
        let err = spatial + r3.abs();
        let mid = midpoint(fa.point, fb.point);
        match best {
            Some((best_err, _, _, _, _)) if err >= best_err => {}
            _ => best = Some((err, spatial, mid, uv_a, uv_b)),
        }
        let improvement = (prev_best - spatial) / prev_best.max(1e-30);
        if improvement < 0.01 {
            stagnation += 1;
        } else {
            stagnation = 0;
        }
        prev_best = prev_best.min(spatial);
        if stagnation >= 3 {
            break;
        }
        if spatial <= tol && r3.abs() <= 1e-11 {
            return Some((mid, uv_a, uv_b));
        }

        let j = [
            [fa.du.x, fa.dv.x, -fb.du.x, -fb.dv.x],
            [fa.du.y, fa.dv.y, -fb.du.y, -fb.dv.y],
            [fa.du.z, fa.dv.z, -fb.du.z, -fb.dv.z],
            [dir4[0], dir4[1], dir4[2], dir4[3]],
        ];
        let rhs = [-diff.x, -diff.y, -diff.z, -r3];
        let Some(delta) = solve_linear_system::<4>(j, rhs) else {
            return best.and_then(|(_, best_spatial, point, uv_a, uv_b)| {
                if best_spatial <= tol * 12.0 {
                    Some((point, uv_a, uv_b))
                } else {
                    None
                }
            });
        };

        let mut accepted = false;
        let mut alpha = 1.0;
        while alpha >= (1.0 / 64.0) {
            let trial_uv_a = clamp_surface_uv(
                surface_a,
                RgmUv2 {
                    u: uv_a.u + alpha * delta[0],
                    v: uv_a.v + alpha * delta[1],
                },
            );
            let trial_uv_b = clamp_surface_uv(
                surface_b,
                RgmUv2 {
                    u: uv_b.u + alpha * delta[2],
                    v: uv_b.v + alpha * delta[3],
                },
            );
            let Some(tfa) = eval_surface_data_uv(surface_a, trial_uv_a).ok() else {
                alpha *= 0.5;
                continue;
            };
            let Some(tfb) = eval_surface_data_uv(surface_b, trial_uv_b).ok() else {
                alpha *= 0.5;
                continue;
            };
            let tdiff = v3::sub(tfa.point, tfb.point);
            let tr3 = (trial_uv_a.u - uv_a_anchor.u) * dir4[0]
                + (trial_uv_a.v - uv_a_anchor.v) * dir4[1]
                + (trial_uv_b.u - uv_b_anchor.u) * dir4[2]
                + (trial_uv_b.v - uv_b_anchor.v) * dir4[3];
            let terr = v3::norm(tdiff) + tr3.abs();
            if terr < err {
                uv_a = trial_uv_a;
                uv_b = trial_uv_b;
                accepted = true;
                break;
            }
            alpha *= 0.5;
        }
        if !accepted {
            break;
        }
    }

    best.and_then(|(_, best_spatial, point, uv_a, uv_b)| {
        if best_spatial <= tol * 12.0 {
            Some((point, uv_a, uv_b))
        } else {
            None
        }
    })
}

fn refine_surface_surface_segment_recursive(
    surface_a: &SurfaceData,
    surface_b: &SurfaceData,
    p0: RgmPoint3,
    uv_a0: RgmUv2,
    uv_b0: RgmUv2,
    p1: RgmPoint3,
    uv_a1: RgmUv2,
    uv_b1: RgmUv2,
    chord_tol: f64,
    tol: f64,
    depth: u32,
    out_points: &mut Vec<RgmPoint3>,
    out_uv_a: &mut Vec<RgmUv2>,
    out_uv_b: &mut Vec<RgmUv2>,
) {
    if depth >= 10 {
        out_points.push(p1);
        out_uv_a.push(uv_a1);
        out_uv_b.push(uv_b1);
        return;
    }

    let uv_a_mid_seed = uv_lerp(uv_a0, uv_a1, 0.5);
    let uv_b_mid_seed = uv_lerp(uv_b0, uv_b1, 0.5);
    let dir4 = [
        uv_a1.u - uv_a0.u,
        uv_a1.v - uv_a0.v,
        uv_b1.u - uv_b0.u,
        uv_b1.v - uv_b0.v,
    ];
    let Some((p_mid, uv_a_mid, uv_b_mid)) = project_surface_surface_curve_point(
        surface_a,
        surface_b,
        uv_a_mid_seed,
        uv_b_mid_seed,
        uv_a_mid_seed,
        uv_b_mid_seed,
        dir4,
        tol,
    ) else {
        out_points.push(p1);
        out_uv_a.push(uv_a1);
        out_uv_b.push(uv_b1);
        return;
    };

    let chord_dev = point_segment_distance(p_mid, p0, p1);
    if chord_dev <= chord_tol {
        let spatial_res = refine_surface_surface_uv_pair(surface_a, surface_b, uv_a_mid, uv_b_mid, tol)
            .map(|(rp, _, _)| v3::distance(rp, p_mid))
            .unwrap_or(tol * 2.0);
        if spatial_res <= tol * 2.0 {
            out_points.push(p1);
            out_uv_a.push(uv_a1);
            out_uv_b.push(uv_b1);
            return;
        }
    }

    refine_surface_surface_segment_recursive(
        surface_a,
        surface_b,
        p0,
        uv_a0,
        uv_b0,
        p_mid,
        uv_a_mid,
        uv_b_mid,
        chord_tol,
        tol,
        depth + 1,
        out_points,
        out_uv_a,
        out_uv_b,
    );
    refine_surface_surface_segment_recursive(
        surface_a,
        surface_b,
        p_mid,
        uv_a_mid,
        uv_b_mid,
        p1,
        uv_a1,
        uv_b1,
        chord_tol,
        tol,
        depth + 1,
        out_points,
        out_uv_a,
        out_uv_b,
    );
}

fn refine_surface_surface_branch_polyline(
    surface_a: &SurfaceData,
    surface_b: &SurfaceData,
    branch: &IntersectionBranchData,
    chord_tol: f64,
    tol: f64,
) -> IntersectionBranchData {
    if branch.points.len() < 2
        || branch.uv_a.len() != branch.points.len()
        || branch.uv_b.len() != branch.points.len()
    {
        return branch.clone();
    }

    let n = branch.points.len();

    // On native targets rayon gives real parallelism; on WASM it would run
    // sequentially while still paying scheduler overhead — use sequential there.
    #[cfg(not(target_arch = "wasm32"))]
    let segment_results: Vec<(Vec<RgmPoint3>, Vec<RgmUv2>, Vec<RgmUv2>)> = (1..n)
        .into_par_iter()
        .map(|idx| {
            let mut seg_points = Vec::new();
            let mut seg_uv_a = Vec::new();
            let mut seg_uv_b = Vec::new();
            refine_surface_surface_segment_recursive(
                surface_a,
                surface_b,
                branch.points[idx - 1],
                branch.uv_a[idx - 1],
                branch.uv_b[idx - 1],
                branch.points[idx],
                branch.uv_a[idx],
                branch.uv_b[idx],
                chord_tol,
                tol,
                0,
                &mut seg_points,
                &mut seg_uv_a,
                &mut seg_uv_b,
            );
            (seg_points, seg_uv_a, seg_uv_b)
        })
        .collect();

    #[cfg(target_arch = "wasm32")]
    let segment_results: Vec<(Vec<RgmPoint3>, Vec<RgmUv2>, Vec<RgmUv2>)> = (1..n)
        .map(|idx| {
            let mut seg_points = Vec::new();
            let mut seg_uv_a = Vec::new();
            let mut seg_uv_b = Vec::new();
            refine_surface_surface_segment_recursive(
                surface_a,
                surface_b,
                branch.points[idx - 1],
                branch.uv_a[idx - 1],
                branch.uv_b[idx - 1],
                branch.points[idx],
                branch.uv_a[idx],
                branch.uv_b[idx],
                chord_tol,
                tol,
                0,
                &mut seg_points,
                &mut seg_uv_a,
                &mut seg_uv_b,
            );
            (seg_points, seg_uv_a, seg_uv_b)
        })
        .collect();

    let cap = segment_results.iter().map(|(p, _, _)| p.len()).sum::<usize>() + 1;
    let mut points = Vec::with_capacity(cap);
    let mut uv_a = Vec::with_capacity(cap);
    let mut uv_b = Vec::with_capacity(cap);
    points.push(branch.points[0]);
    uv_a.push(branch.uv_a[0]);
    uv_b.push(branch.uv_b[0]);
    for (seg_pts, seg_uva, seg_uvb) in segment_results {
        points.extend(seg_pts);
        uv_a.extend(seg_uva);
        uv_b.extend(seg_uvb);
    }
    IntersectionBranchData {
        points,
        uv_a,
        uv_b,
        curve_t: Vec::new(),
        closed: branch.closed,
        flags: branch.flags,
    }
}

fn refine_surface_curve_hit(
    state: &SessionState,
    surface: &SurfaceData,
    curve: &CurveData,
    uv_seed: RgmUv2,
    t_seed: f64,
    tol: f64,
) -> Option<(RgmPoint3, RgmUv2, f64)> {
    let mut uv = clamp_surface_uv(surface, uv_seed);
    let mut t = t_seed.clamp(0.0, 1.0);
    let mut lambda = 1e-12;
    let mut best: Option<(f64, RgmPoint3, RgmUv2, f64)> = None;
    let mut stagnation: u8 = 0;
    let mut prev_best = f64::INFINITY;

    for _ in 0..24 {
        let frame = eval_surface_data_uv(surface, uv).ok()?;
        let curve_eval = evaluate_curve_at_normalized_data(state, curve, t).ok()?;
        let residual = v3::sub(frame.point, curve_eval.point);
        let residual_norm = v3::norm(residual);

        match best {
            Some((best_norm, _, _, _)) if residual_norm >= best_norm => {}
            _ => {
                best = Some((
                    residual_norm,
                    midpoint(frame.point, curve_eval.point),
                    uv,
                    t,
                ))
            }
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
            return Some((midpoint(frame.point, curve_eval.point), uv, t));
        }

        let cols = [frame.du, frame.dv, v3::neg(curve_eval.d1)];
        let mut jtj = [[0.0; 3]; 3];
        let mut jtr = [0.0; 3];
        for i in 0..3 {
            jtr[i] = v3::dot(cols[i], residual);
            for j in i..3 {
                let value = v3::dot(cols[i], cols[j]);
                jtj[i][j] = value;
                jtj[j][i] = value;
            }
            jtj[i][i] += lambda;
        }
        let rhs = [-jtr[0], -jtr[1], -jtr[2]];
        let Some(delta) = solve_linear_system::<3>(jtj, rhs) else {
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
            let trial_t = (t + alpha * delta[2]).clamp(0.0, 1.0);
            let Some(trial_surface) = eval_surface_data_uv(surface, trial_uv).ok() else {
                alpha *= 0.5;
                continue;
            };
            let Some(trial_curve) = evaluate_curve_at_normalized_data(state, curve, trial_t).ok()
            else {
                alpha *= 0.5;
                continue;
            };
            let trial_norm = v3::norm(v3::sub(trial_surface.point, trial_curve.point));
            if trial_norm < residual_norm {
                uv = trial_uv;
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

    best.and_then(|(residual_norm, point, uv, t)| {
        if residual_norm <= tol * 128.0 {
            Some((point, uv, t))
        } else {
            None
        }
    })
}

fn dot4(a: [f64; 4], b: [f64; 4]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

fn norm4(v: [f64; 4]) -> f64 {
    dot4(v, v).sqrt()
}

fn det3(a: RgmVec3, b: RgmVec3, c: RgmVec3) -> f64 {
    v3::dot(a, v3::cross(b, c))
}

fn surface_surface_tangent_dir(
    frame_a: RgmSurfaceEvalFrame,
    frame_b: RgmSurfaceEvalFrame,
) -> Option<[f64; 4]> {
    let c0 = frame_a.du;
    let c1 = frame_a.dv;
    let c2 = v3::neg(frame_b.du);
    let c3 = v3::neg(frame_b.dv);

    let mut dir = [
        det3(c1, c2, c3),
        -det3(c0, c2, c3),
        det3(c0, c1, c3),
        -det3(c0, c1, c2),
    ];
    let n = norm4(dir);
    if n <= 1e-14 {
        return None;
    }
    for value in &mut dir {
        *value /= n;
    }
    Some(dir)
}

fn project_point_to_surface(
    surface: &SurfaceData,
    point: RgmPoint3,
    uv_seed: RgmUv2,
    tol: f64,
) -> Option<(RgmPoint3, RgmUv2, f64)> {
    let mut uv = clamp_surface_uv(surface, uv_seed);
    let mut lambda = 1e-12;
    let mut best: Option<(f64, RgmPoint3, RgmUv2)> = None;
    let mut stagnation: u8 = 0;
    let mut prev_best = f64::INFINITY;
    for _ in 0..24 {
        let frame = eval_surface_data_uv(surface, uv).ok()?;
        let residual = v3::sub(frame.point, point);
        let residual_norm = v3::norm(residual);
        match best {
            Some((best_norm, _, _)) if residual_norm >= best_norm => {}
            _ => best = Some((residual_norm, frame.point, uv)),
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
            return Some((frame.point, uv, residual_norm));
        }

        let a00 = v3::dot(frame.du, frame.du) + lambda;
        let a01 = v3::dot(frame.du, frame.dv);
        let a11 = v3::dot(frame.dv, frame.dv) + lambda;
        let b0 = -v3::dot(frame.du, residual);
        let b1 = -v3::dot(frame.dv, residual);
        let Some(delta) = solve_linear_system::<2>([[a00, a01], [a01, a11]], [b0, b1]) else {
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
            let trial_norm = v3::distance(trial_frame.point, point);
            if trial_norm < residual_norm {
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

    best.map(|(residual_norm, point, uv)| (point, uv, residual_norm))
}

fn project_point_to_surface_multi_seed(
    surface: &SurfaceData,
    point: RgmPoint3,
    seeds: &[RgmUv2],
    tol: f64,
) -> Option<(RgmPoint3, RgmUv2, f64)> {
    let mut best: Option<(RgmPoint3, RgmUv2, f64)> = None;
    for seed in seeds {
        let Some(candidate) = project_point_to_surface(surface, point, *seed, tol) else {
            continue;
        };
        match best {
            Some((_, _, best_res)) if candidate.2 >= best_res => {}
            _ => best = Some(candidate),
        }
    }
    best
}
