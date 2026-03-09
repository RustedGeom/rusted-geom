// ─── Sweep & Loft Surface Operations ─────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LoftType {
    Normal,
    Loose,
    Tight,
    Straight,
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
}

struct LoftResult {
    surface: SurfaceData,
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

    Ok(SweepResult { surface })
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

    Ok(LoftResult { surface })
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

    Ok(LoftResult { surface })
}

fn sweep_impl(
    state: &mut SessionState,
    path: RgmObjectHandle,
    profile: RgmObjectHandle,
    n_stations: usize,
    _cap_faces: bool,
) -> Result<RgmObjectHandle, RgmStatus> {
    let path_curve = find_curve(state, path)?.clone();
    let profile_curve = find_curve(state, profile)?.clone();
    let sweep = build_sweep_surface(state, &path_curve, &profile_curve, n_stations)?;
    Ok(insert_surface(state, sweep.surface))
}

fn loft_impl(
    state: &mut SessionState,
    section_handles: &[RgmObjectHandle],
    n_samples: usize,
    _cap_faces: bool,
    loft_type: LoftType,
) -> Result<RgmObjectHandle, RgmStatus> {
    if section_handles.len() < 2 {
        return Err(RgmStatus::InvalidInput);
    }

    let section_curves: Vec<CurveData> = section_handles
        .iter()
        .map(|&h| find_curve(state, h).map(|c| c.clone()))
        .collect::<Result<Vec<_>, _>>()?;

    let section_refs: Vec<&CurveData> = section_curves.iter().collect();
    let loft = build_loft_surface(state, &section_refs, n_samples, loft_type)?;
    Ok(insert_surface(state, loft.surface))
}
