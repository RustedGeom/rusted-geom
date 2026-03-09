// ─── Sweep & Loft Surface Operations ─────────────────────────────────────────
//
// Uses add_face_to_brep, add_surface_face_to_brep, add_uv_loop_to_face, FaceId,
// BrepData, BrepShell, BrepSolid from brep_ops (included before this file).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LoftType {
    Normal,
    Loose,
    Tight,
    Straight,
}

fn eval_surface_iso_u(
    core: &NurbsSurfaceCore,
    u_norm: f64,
    n_samples: usize,
) -> Result<Vec<RgmPoint3>, RgmStatus> {
    let n = n_samples.max(3);
    let mut pts = Vec::with_capacity(n);
    for i in 0..n {
        let v_norm = i as f64 / (n - 1) as f64;
        let eval = math::nurbs_surface_eval::eval_nurbs_surface_normalized(
            core,
            RgmUv2 { u: u_norm, v: v_norm },
        )?;
        pts.push(eval.point);
    }
    Ok(pts)
}

fn get_nurbs_for_curve<'a>(
    state: &'a SessionState,
    curve: &'a CurveData,
) -> Result<std::borrow::Cow<'a, NurbsCurveData>, RgmStatus> {
    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        return Ok(std::borrow::Cow::Borrowed(nurbs));
    }
    match curve {
        CurveData::Polycurve(poly) => {
            let converted = polycurve_to_nurbs(state, poly)?;
            Ok(std::borrow::Cow::Owned(converted))
        }
        _ => Err(RgmStatus::InvalidInput),
    }
}

fn is_curve_closed(state: &SessionState, curve: &CurveData) -> bool {
    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        return nurbs.closed || nurbs.core.periodic;
    }
    if let CurveData::Polycurve(poly) = curve {
        if poly.segments.is_empty() {
            return false;
        }
        let first_seg = &poly.segments[0];
        let last_seg = &poly.segments[poly.segments.len() - 1];
        let first_curve = match find_curve(state, first_seg.curve) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let last_curve = match find_curve(state, last_seg.curve) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let first_pt = curve_point_at_normalized_data(state, first_curve, if first_seg.reversed { 1.0 } else { 0.0 });
        let last_pt = curve_point_at_normalized_data(state, last_curve, if last_seg.reversed { 0.0 } else { 1.0 });
        if let (Ok(fp), Ok(lp)) = (first_pt, last_pt) {
            return v3::distance(fp, lp) < 1e-6;
        }
    }
    false
}

fn rotate_vec_rodrigues(v: RgmVec3, axis: RgmVec3, cos_t: f64, sin_t: f64) -> RgmVec3 {
    let kxv = v3::cross(axis, v);
    let kdv = v3::dot(axis, v);
    RgmVec3 {
        x: v.x * cos_t + kxv.x * sin_t + axis.x * kdv * (1.0 - cos_t),
        y: v.y * cos_t + kxv.y * sin_t + axis.y * kdv * (1.0 - cos_t),
        z: v.z * cos_t + kxv.z * sin_t + axis.z * kdv * (1.0 - cos_t),
    }
}

struct SweepResult {
    surface: SurfaceData,
    start_frame: RgmPlane,
    end_frame: RgmPlane,
    start_boundary_pts: Vec<RgmPoint3>,
    end_boundary_pts: Vec<RgmPoint3>,
}

struct LoftResult {
    surface: SurfaceData,
    start_boundary_pts: Vec<RgmPoint3>,
    end_boundary_pts: Vec<RgmPoint3>,
}

struct CompatibleSectionInfo {
    degree: usize,
    n_total_cps: usize,
    periodic: bool,
    knots: Vec<f64>,
    weights: Vec<f64>,
}

fn build_sweep_surface(
    state: &SessionState,
    path: &CurveData,
    profile: &CurveData,
    n_stations: usize,
) -> Result<SweepResult, RgmStatus> {
    if n_stations < 2 {
        return Err(RgmStatus::InvalidInput);
    }

    let profile_nurbs = get_nurbs_for_curve(state, profile)?;
    let core = &profile_nurbs.core;
    let n_profile_cps = core.control_points.len();

    if n_profile_cps < 2 {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let tol = curve_abs_tol(state, path)?;

    let start_eval = evaluate_curve_at_normalized_data(state, path, 0.0)?;
    let start_frame = frame_plane(start_eval, tol)?;

    let local_coords: Vec<(f64, f64)> = core
        .control_points
        .iter()
        .map(|cp| {
            let offset = v3::sub(*cp, start_frame.origin);
            let ly = v3::dot(offset, start_frame.y_axis);
            let lz = v3::dot(offset, start_frame.z_axis);
            (ly, lz)
        })
        .collect();

    let mut station_origins = Vec::with_capacity(n_stations);
    let mut grid = Vec::with_capacity(n_stations * n_profile_cps);

    let mut prev_tangent = start_frame.x_axis;
    let mut prev_y = start_frame.y_axis;
    let mut prev_z = start_frame.z_axis;

    for i in 0..n_stations {
        let t = i as f64 / (n_stations - 1) as f64;
        let eval = evaluate_curve_at_normalized_data(state, path, t)?;
        let origin = eval.point;

        let cur_tangent = v3::normalize(eval.d1).unwrap_or(prev_tangent);

        let (y_axis, z_axis) = if i == 0 {
            (start_frame.y_axis, start_frame.z_axis)
        } else {
            let cross_vec = v3::cross(prev_tangent, cur_tangent);
            let sin_t = v3::norm(cross_vec);
            let cos_t = v3::dot(prev_tangent, cur_tangent).clamp(-1.0, 1.0);

            if sin_t < 1e-12 {
                (prev_y, prev_z)
            } else {
                let axis = v3::scale(cross_vec, 1.0 / sin_t);
                let new_y = rotate_vec_rodrigues(prev_y, axis, cos_t, sin_t);
                let new_z = rotate_vec_rodrigues(prev_z, axis, cos_t, sin_t);
                (new_y, new_z)
            }
        };

        prev_tangent = cur_tangent;
        prev_y = y_axis;
        prev_z = z_axis;
        station_origins.push(origin);

        for &(ly, lz) in &local_coords {
            grid.push(point_from_frame(origin, y_axis, z_axis, ly, lz));
        }
    }

    let degree_path = 3.min(n_stations - 1);
    let params_path = chord_length_params(&station_origins);
    let knots_path = clamped_open_knots(n_stations, degree_path, &params_path);

    for j in 0..n_profile_cps {
        let column: Vec<RgmPoint3> = (0..n_stations)
            .map(|i| grid[i * n_profile_cps + j])
            .collect();
        let solved = solve_bspline_interpolation(&column, &params_path, &knots_path, degree_path)?;
        for i in 0..n_stations {
            grid[i * n_profile_cps + j] = solved[i];
        }
    }

    let mut weights = Vec::with_capacity(n_stations * n_profile_cps);
    for _ in 0..n_stations {
        weights.extend_from_slice(&core.weights);
    }

    let desc = RgmNurbsSurfaceDesc {
        degree_u: degree_path as u32,
        degree_v: core.degree as u32,
        periodic_u: false,
        periodic_v: core.periodic,
        control_u_count: n_stations as u32,
        control_v_count: n_profile_cps as u32,
    };

    let surface_tol = core.tol;
    let surface = build_surface_from_desc(desc, &grid, &weights, &knots_path, &core.knots, surface_tol)?;

    let n_boundary = n_profile_cps.max(24);
    let start_boundary = eval_surface_iso_u(&surface.core, 0.0, n_boundary)?;
    let end_boundary = eval_surface_iso_u(&surface.core, 1.0, n_boundary)?;

    let end_frame = RgmPlane {
        origin: *station_origins.last().unwrap(),
        x_axis: prev_tangent,
        y_axis: prev_y,
        z_axis: prev_z,
    };

    Ok(SweepResult {
        surface,
        start_frame,
        end_frame,
        start_boundary_pts: start_boundary,
        end_boundary_pts: end_boundary,
    })
}

fn sections_are_compatible(
    state: &SessionState,
    section_curves: &[&CurveData],
) -> Option<CompatibleSectionInfo> {
    if section_curves.is_empty() {
        return None;
    }
    let first = get_nurbs_for_curve(state, section_curves[0]).ok()?;
    let fc = &first.core;
    let degree = fc.degree;
    let n_total = fc.control_points.len();
    let periodic = fc.periodic;

    for &section in &section_curves[1..] {
        let nurbs = get_nurbs_for_curve(state, section).ok()?;
        let c = &nurbs.core;
        if c.degree != degree || c.control_points.len() != n_total || c.periodic != periodic {
            return None;
        }
        if c.weights.len() != fc.weights.len() {
            return None;
        }
        let weights_match = c.weights.iter().zip(fc.weights.iter()).all(|(a, b)| (a - b).abs() < 1e-14);
        if !weights_match {
            return None;
        }
    }

    Some(CompatibleSectionInfo {
        degree,
        n_total_cps: n_total,
        periodic,
        knots: fc.knots.clone(),
        weights: fc.weights.clone(),
    })
}

fn build_loft_compatible(
    state: &SessionState,
    section_curves: &[&CurveData],
    compat: &CompatibleSectionInfo,
    loft_type: LoftType,
) -> Result<LoftResult, RgmStatus> {
    let m = section_curves.len();
    let nv = compat.n_total_cps;

    let mut grid = Vec::with_capacity(m * nv);
    let mut section_midpoints = Vec::with_capacity(m);

    for &section in section_curves {
        let nurbs = get_nurbs_for_curve(state, section)?;
        let mid = curve_point_at_normalized_data(state, section, 0.5)?;
        section_midpoints.push(mid);
        grid.extend_from_slice(&nurbs.core.control_points);
    }

    let degree_u = match loft_type {
        LoftType::Straight => 1.min(m - 1),
        _ => 3.min(m - 1),
    };

    let params_u = match loft_type {
        LoftType::Tight => centripetal_params(&section_midpoints),
        _ => chord_length_params(&section_midpoints),
    };
    let knots_u = clamped_open_knots(m, degree_u, &params_u);

    if loft_type != LoftType::Loose {
        for j in 0..nv {
            let col: Vec<RgmPoint3> = (0..m).map(|i| grid[i * nv + j]).collect();
            let solved = solve_bspline_interpolation(&col, &params_u, &knots_u, degree_u)?;
            for i in 0..m {
                grid[i * nv + j] = solved[i];
            }
        }
    }

    let mut weights = Vec::with_capacity(m * nv);
    for _ in 0..m {
        weights.extend_from_slice(&compat.weights);
    }

    let tol = curve_abs_tol(state, section_curves[0])?;
    let surface_tol = RgmToleranceContext {
        abs_tol: tol,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    let desc = RgmNurbsSurfaceDesc {
        degree_u: degree_u as u32,
        degree_v: compat.degree as u32,
        periodic_u: false,
        periodic_v: compat.periodic,
        control_u_count: m as u32,
        control_v_count: nv as u32,
    };

    let surface = build_surface_from_desc(desc, &grid, &weights, &knots_u, &compat.knots, surface_tol)?;

    let n_boundary = nv.max(24);
    let start_boundary = eval_surface_iso_u(&surface.core, 0.0, n_boundary)?;
    let end_boundary = eval_surface_iso_u(&surface.core, 1.0, n_boundary)?;

    Ok(LoftResult {
        surface,
        start_boundary_pts: start_boundary,
        end_boundary_pts: end_boundary,
    })
}

fn build_loft_surface(
    state: &SessionState,
    section_curves: &[&CurveData],
    n_samples: usize,
    loft_type: LoftType,
) -> Result<LoftResult, RgmStatus> {
    let m = section_curves.len();
    if m < 2 {
        return Err(RgmStatus::InvalidInput);
    }

    if let Some(compat) = sections_are_compatible(state, section_curves) {
        return build_loft_compatible(state, section_curves, &compat, loft_type);
    }

    if n_samples < 2 {
        return Err(RgmStatus::InvalidInput);
    }
    build_loft_sampled(state, section_curves, n_samples, loft_type)
}

fn build_loft_sampled(
    state: &SessionState,
    section_curves: &[&CurveData],
    n_samples: usize,
    loft_type: LoftType,
) -> Result<LoftResult, RgmStatus> {
    let m_sections = section_curves.len();

    let all_closed = section_curves.iter().all(|c| is_curve_closed(state, c));

    let mut point_grid = Vec::with_capacity(m_sections * n_samples);
    let mut section_midpoints = Vec::with_capacity(m_sections);

    for &section in section_curves.iter() {
        let mut section_pts = Vec::with_capacity(n_samples);
        for j in 0..n_samples {
            let t = if all_closed {
                j as f64 / n_samples as f64
            } else {
                j as f64 / (n_samples - 1) as f64
            };
            let pt = curve_point_at_normalized_data(state, section, t)?;
            section_pts.push(pt);
        }

        let mid = RgmPoint3 {
            x: section_pts.iter().map(|p| p.x).sum::<f64>() / n_samples as f64,
            y: section_pts.iter().map(|p| p.y).sum::<f64>() / n_samples as f64,
            z: section_pts.iter().map(|p| p.z).sum::<f64>() / n_samples as f64,
        };
        section_midpoints.push(mid);
        point_grid.extend(section_pts);
    }

    let degree_sample = 3.min(n_samples - 1);
    let degree_section = match loft_type {
        LoftType::Straight => 1.min(m_sections - 1),
        _ => 3.min(m_sections - 1),
    };

    let params_section = match loft_type {
        LoftType::Tight => centripetal_params(&section_midpoints),
        _ => chord_length_params(&section_midpoints),
    };
    let knots_section = clamped_open_knots(m_sections, degree_section, &params_section);

    if loft_type != LoftType::Loose {
        for j in 0..n_samples {
            let column: Vec<RgmPoint3> = (0..m_sections)
                .map(|i| point_grid[i * n_samples + j])
                .collect();
            let solved = solve_bspline_interpolation(&column, &params_section, &knots_section, degree_section)?;
            for i in 0..m_sections {
                point_grid[i * n_samples + j] = solved[i];
            }
        }
    }

    let (n_sample_cps, knots_sample, periodic_sample) = if all_closed {
        let base_count = n_samples;
        let wrapped_count = base_count + degree_sample;
        let knots = uniform_periodic_knots(wrapped_count, degree_sample);

        let mut wrapped_grid = Vec::with_capacity(m_sections * wrapped_count);
        for i in 0..m_sections {
            let row_start = i * n_samples;
            for j in 0..n_samples {
                wrapped_grid.push(point_grid[row_start + j]);
            }
            for j in 0..degree_sample {
                wrapped_grid.push(point_grid[row_start + j]);
            }
        }
        point_grid = wrapped_grid;
        (wrapped_count, knots, true)
    } else {
        let first_section_pts: Vec<RgmPoint3> = point_grid[..n_samples].to_vec();
        let params_sample = chord_length_params(&first_section_pts);
        let knots = clamped_open_knots(n_samples, degree_sample, &params_sample);

        // 2D interpolation: solve U-direction for each section row
        // (Piegl & Tiller Section 9.2.5)
        for i in 0..m_sections {
            let row: Vec<RgmPoint3> = (0..n_samples)
                .map(|j| point_grid[i * n_samples + j])
                .collect();
            let solved = solve_bspline_interpolation(&row, &params_sample, &knots, degree_sample)?;
            for j in 0..n_samples {
                point_grid[i * n_samples + j] = solved[j];
            }
        }

        (n_samples, knots, false)
    };

    let weights = vec![1.0; m_sections * n_sample_cps];

    let tol = curve_abs_tol(state, section_curves[0])?;
    let surface_tol = RgmToleranceContext {
        abs_tol: tol,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    let desc = RgmNurbsSurfaceDesc {
        degree_u: degree_section as u32,
        degree_v: degree_sample as u32,
        periodic_u: false,
        periodic_v: periodic_sample,
        control_u_count: m_sections as u32,
        control_v_count: n_sample_cps as u32,
    };

    let surface = build_surface_from_desc(desc, &point_grid, &weights, &knots_section, &knots_sample, surface_tol)?;

    let n_boundary = n_samples.max(24);
    let start_boundary = eval_surface_iso_u(&surface.core, 0.0, n_boundary)?;
    let end_boundary = eval_surface_iso_u(&surface.core, 1.0, n_boundary)?;

    Ok(LoftResult {
        surface,
        start_boundary_pts: start_boundary,
        end_boundary_pts: end_boundary,
    })
}

fn build_planar_cap_face(
    state: &mut SessionState,
    curve_points: &[RgmPoint3],
    frame: &RgmPlane,
    outward_hint: RgmVec3,
) -> Result<RgmObjectHandle, RgmStatus> {
    if curve_points.len() < 3 {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let axes = [frame.x_axis, frame.y_axis, frame.z_axis];
    let mut ranges = [(f64::MAX, f64::MIN); 3];
    for p in curve_points {
        let offset = v3::sub(*p, frame.origin);
        for (i, ax) in axes.iter().enumerate() {
            let d = v3::dot(offset, *ax);
            if d < ranges[i].0 { ranges[i].0 = d; }
            if d > ranges[i].1 { ranges[i].1 = d; }
        }
    }
    let extents: Vec<f64> = ranges.iter().map(|(lo, hi)| hi - lo).collect();

    let (a0, a1) = if extents[0] <= extents[1] && extents[0] <= extents[2] {
        (1, 2)
    } else if extents[1] <= extents[2] {
        (0, 2)
    } else {
        (0, 1)
    };

    let (mut min_u, mut max_u) = ranges[a0];
    let (mut min_v, mut max_v) = ranges[a1];
    let range_u = max_u - min_u;
    let range_v = max_v - min_v;

    if range_u < 1e-12 || range_v < 1e-12 {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let pad = 0.02;
    min_u -= range_u * pad;
    max_u += range_u * pad;
    min_v -= range_v * pad;
    max_v += range_v * pad;
    let range_u = max_u - min_u;
    let range_v = max_v - min_v;

    let raw_axis_u = axes[a0];
    let raw_axis_v = axes[a1];
    let surf_normal = v3::cross(raw_axis_u, raw_axis_v);
    let needs_flip = v3::dot(surf_normal, outward_hint) < 0.0;
    let (axis_u, axis_v, cap_min_u, cap_max_u, cap_min_v, cap_max_v, cap_range_u, cap_range_v) =
        if needs_flip {
            (raw_axis_v, raw_axis_u, min_v, max_v, min_u, max_u, range_v, range_u)
        } else {
            (raw_axis_u, raw_axis_v, min_u, max_u, min_v, max_v, range_u, range_v)
        };

    let cp00 = point_from_frame(frame.origin, axis_u, axis_v, cap_min_u, cap_min_v);
    let cp10 = point_from_frame(frame.origin, axis_u, axis_v, cap_max_u, cap_min_v);
    let cp01 = point_from_frame(frame.origin, axis_u, axis_v, cap_min_u, cap_max_v);
    let cp11 = point_from_frame(frame.origin, axis_u, axis_v, cap_max_u, cap_max_v);

    let control_points = vec![cp00, cp10, cp01, cp11];
    let weights = vec![1.0; 4];
    let knots_u = vec![0.0, 0.0, 1.0, 1.0];
    let knots_v = vec![0.0, 0.0, 1.0, 1.0];

    let desc = RgmNurbsSurfaceDesc {
        degree_u: 1,
        degree_v: 1,
        periodic_u: false,
        periodic_v: false,
        control_u_count: 2,
        control_v_count: 2,
    };

    let tol = RgmToleranceContext {
        abs_tol: 1e-6,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    let surface = build_surface_from_desc(desc, &control_points, &weights, &knots_u, &knots_v, tol)?;
    let surf_handle = insert_surface(state, surface);

    let uv_points: Vec<RgmUv2> = curve_points
        .iter()
        .map(|p| {
            let offset = v3::sub(*p, frame.origin);
            let pu = v3::dot(offset, axis_u);
            let pv = v3::dot(offset, axis_v);
            RgmUv2 {
                u: (pu - cap_min_u) / cap_range_u,
                v: (pv - cap_min_v) / cap_range_v,
            }
        })
        .collect();

    let face = FaceData {
        surface: surf_handle,
        loops: vec![TrimLoopData {
            edges: trim_loop_from_uv_polyline(&uv_points),
            is_outer: true,
        }],
    };

    Ok(insert_face(state, face))
}

fn trim_loop_from_uv_polyline(uv_points: &[RgmUv2]) -> Vec<TrimEdgeData> {
    let n = uv_points.len();
    if n < 2 {
        return Vec::new();
    }
    let mut edges = Vec::with_capacity(n);
    for i in 0..n {
        let start = uv_points[i];
        let end = uv_points[(i + 1) % n];
        edges.push(TrimEdgeData {
            start_uv: start,
            end_uv: end,
            curve_3d: None,
            uv_samples: vec![start, end],
        });
    }
    edges
}

fn assemble_brep_with_caps(
    state: &mut SessionState,
    body_surface: SurfaceData,
    start_cap_face: RgmObjectHandle,
    end_cap_face: RgmObjectHandle,
) -> Result<RgmObjectHandle, RgmStatus> {
    let body_surf_handle = insert_surface(state, body_surface);

    let mut brep = BrepData::new();

    add_surface_face_to_brep(&mut brep, body_surf_handle);

    let start_face = find_face(state, start_cap_face)?.clone();
    add_face_to_brep(&mut brep, &start_face)?;

    let end_face = find_face(state, end_cap_face)?.clone();
    add_face_to_brep(&mut brep, &end_face)?;

    let faces = brep.faces.indices().collect::<Vec<FaceId>>();
    let shell_id = brep.shells.push(BrepShell {
        faces,
        closed: true,
    });

    brep.solids.push(BrepSolid {
        shells: vec![shell_id],
    });
    brep.finalized = true;

    Ok(insert_brep(state, brep))
}

fn sweep_impl(
    state: &mut SessionState,
    path: RgmObjectHandle,
    profile: RgmObjectHandle,
    n_stations: usize,
    cap_faces: bool,
) -> Result<RgmObjectHandle, RgmStatus> {
    let path_curve = find_curve(state, path)?.clone();
    let profile_curve = find_curve(state, profile)?.clone();

    if cap_faces && !is_curve_closed(state, &profile_curve) {
        return Err(RgmStatus::InvalidInput);
    }

    let sweep = build_sweep_surface(state, &path_curve, &profile_curve, n_stations)?;

    if !cap_faces {
        return Ok(insert_surface(state, sweep.surface));
    }

    let start_outward = v3::scale(sweep.start_frame.x_axis, -1.0);
    let end_outward = sweep.end_frame.x_axis;
    let start_cap = build_planar_cap_face(state, &sweep.start_boundary_pts, &sweep.start_frame, start_outward)?;
    let end_cap = build_planar_cap_face(state, &sweep.end_boundary_pts, &sweep.end_frame, end_outward)?;

    assemble_brep_with_caps(state, sweep.surface, start_cap, end_cap)
}

fn loft_impl(
    state: &mut SessionState,
    section_handles: &[RgmObjectHandle],
    n_samples: usize,
    cap_faces: bool,
    loft_type: LoftType,
) -> Result<RgmObjectHandle, RgmStatus> {
    if section_handles.len() < 2 {
        return Err(RgmStatus::InvalidInput);
    }

    let section_curves: Vec<CurveData> = section_handles
        .iter()
        .map(|&h| find_curve(state, h).map(|c| c.clone()))
        .collect::<Result<Vec<_>, _>>()?;

    if cap_faces {
        for curve in &section_curves {
            if !is_curve_closed(state, curve) {
                return Err(RgmStatus::InvalidInput);
            }
        }
    }

    let section_refs: Vec<&CurveData> = section_curves.iter().collect();
    let loft = build_loft_surface(state, &section_refs, n_samples, loft_type)?;

    if !cap_faces {
        return Ok(insert_surface(state, loft.surface));
    }

    let start_centroid = centroid_of(&loft.start_boundary_pts);
    let end_centroid = centroid_of(&loft.end_boundary_pts);
    let loft_dir = v3::normalize(v3::sub(end_centroid, start_centroid))
        .unwrap_or(RgmVec3 { x: 1.0, y: 0.0, z: 0.0 });
    let start_outward = v3::scale(loft_dir, -1.0);
    let end_outward = loft_dir;

    let first_frame = build_cap_frame_from_points(&loft.start_boundary_pts)?;
    let start_cap = build_planar_cap_face(state, &loft.start_boundary_pts, &first_frame, start_outward)?;

    let last_frame = build_cap_frame_from_points(&loft.end_boundary_pts)?;
    let end_cap = build_planar_cap_face(state, &loft.end_boundary_pts, &last_frame, end_outward)?;

    assemble_brep_with_caps(state, loft.surface, start_cap, end_cap)
}

fn centroid_of(points: &[RgmPoint3]) -> RgmPoint3 {
    let n = points.len().max(1) as f64;
    RgmPoint3 {
        x: points.iter().map(|p| p.x).sum::<f64>() / n,
        y: points.iter().map(|p| p.y).sum::<f64>() / n,
        z: points.iter().map(|p| p.z).sum::<f64>() / n,
    }
}

fn build_cap_frame_from_points(points: &[RgmPoint3]) -> Result<RgmPlane, RgmStatus> {
    if points.len() < 3 {
        return Err(RgmStatus::DegenerateGeometry);
    }
    let centroid = RgmPoint3 {
        x: points.iter().map(|p| p.x).sum::<f64>() / points.len() as f64,
        y: points.iter().map(|p| p.y).sum::<f64>() / points.len() as f64,
        z: points.iter().map(|p| p.z).sum::<f64>() / points.len() as f64,
    };

    let mut best_normal = RgmVec3 { x: 0.0, y: 0.0, z: 0.0 };
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        let a = v3::sub(points[i], centroid);
        let b = v3::sub(points[j], centroid);
        let c = v3::cross(a, b);
        best_normal.x += c.x;
        best_normal.y += c.y;
        best_normal.z += c.z;
    }

    let z_axis = v3::normalize(best_normal).ok_or(RgmStatus::DegenerateGeometry)?;

    let first_offset = v3::sub(points[0], centroid);
    let y_axis = v3::normalize(first_offset).ok_or(RgmStatus::DegenerateGeometry)?;
    let x_axis = v3::normalize(v3::cross(y_axis, z_axis)).ok_or(RgmStatus::DegenerateGeometry)?;
    let y_axis = v3::cross(z_axis, x_axis);

    Ok(RgmPlane {
        origin: centroid,
        x_axis,
        y_axis,
        z_axis,
    })
}
