fn curve_total_length(_state: &SessionState, curve: &CurveData) -> Result<f64, RgmStatus> {
    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        return Ok(nurbs.arc_length.total_length);
    }

    match curve {
        CurveData::Polycurve(poly) => Ok(poly.total_length),
        _ => Err(RgmStatus::InternalError),
    }
}

fn curve_length_at_normalized_data(
    _state: &SessionState,
    curve: &CurveData,
    t_norm: f64,
) -> Result<f64, RgmStatus> {
    if !(0.0..=1.0).contains(&t_norm) {
        return Err(RgmStatus::OutOfRange);
    }

    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        if nurbs.core.periodic && (t_norm - 1.0).abs() <= f64::EPSILON {
            return Ok(nurbs.arc_length.total_length);
        }

        let u = map_normalized_to_u(&nurbs.core, t_norm)?;
        return length_from_u(&nurbs.core, &nurbs.arc_length, u);
    }

    match curve {
        CurveData::Polycurve(poly) => {
            Ok((t_norm * poly.total_length).clamp(0.0, poly.total_length))
        }
        _ => Err(RgmStatus::InternalError),
    }
}

fn evaluate_curve_at_length_data(
    state: &SessionState,
    curve: &CurveData,
    length: f64,
) -> Result<CurveEvalResult, RgmStatus> {
    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        let u = u_from_length(&nurbs.core, &nurbs.arc_length, length)?;
        return eval_nurbs_u(&nurbs.core, u);
    }

    match curve {
        CurveData::Polycurve(poly) => evaluate_polycurve_at_length(state, poly, length),
        _ => Err(RgmStatus::InternalError),
    }
}

fn evaluate_curve_at_normalized_data(
    state: &SessionState,
    curve: &CurveData,
    t_norm: f64,
) -> Result<CurveEvalResult, RgmStatus> {
    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        return eval_nurbs_normalized(&nurbs.core, t_norm);
    }

    match curve {
        CurveData::Polycurve(poly) => evaluate_polycurve_at_normalized(state, poly, t_norm),
        _ => Err(RgmStatus::InternalError),
    }
}

fn evaluate_curve_by_handle_at_length(
    state: &SessionState,
    curve: RgmObjectHandle,
    length: f64,
) -> Result<CurveEvalResult, RgmStatus> {
    let curve_data = find_curve(state, curve)?;
    evaluate_curve_at_length_data(state, curve_data, length)
}

fn curve_point_at_normalized_data(
    state: &SessionState,
    curve: &CurveData,
    t_norm: f64,
) -> Result<RgmPoint3, RgmStatus> {
    let eval = evaluate_curve_at_normalized_data(state, curve, t_norm)?;
    Ok(eval.point)
}

fn curve_abs_tol(state: &SessionState, curve: &CurveData) -> Result<f64, RgmStatus> {
    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        return Ok(nurbs.core.tol.abs_tol.max(1e-9));
    }

    match curve {
        CurveData::Polycurve(poly) => {
            let mut tol = 1e-9_f64;
            for segment in &poly.segments {
                let segment_curve = find_curve(state, segment.curve)?;
                if let Some(nurbs) = curve_canonical_nurbs(segment_curve) {
                    tol = tol.max(nurbs.core.tol.abs_tol.max(1e-9));
                }
            }
            Ok(tol)
        }
        _ => Ok(1e-9),
    }
}

fn intersect_curve_plane_points_data(
    state: &SessionState,
    curve: &CurveData,
    plane: RgmPlane,
) -> Result<Vec<RgmPoint3>, RgmStatus> {
    let abs_tol = curve_abs_tol(state, curve)?;
    intersect_curve_plane_points(
        |t_norm| curve_point_at_normalized_data(state, curve, t_norm),
        plane,
        abs_tol,
    )
}

fn intersect_curve_curve_points_data(
    state: &SessionState,
    curve_a: &CurveData,
    curve_b: &CurveData,
) -> Result<Vec<RgmPoint3>, RgmStatus> {
    let abs_tol = curve_abs_tol(state, curve_a)?.max(curve_abs_tol(state, curve_b)?);
    intersect_curve_curve_points(
        |t_norm| curve_point_at_normalized_data(state, curve_a, t_norm),
        |t_norm| curve_point_at_normalized_data(state, curve_b, t_norm),
        abs_tol,
    )
}

fn evaluate_polycurve_at_normalized(
    state: &SessionState,
    poly: &PolycurveData,
    t_norm: f64,
) -> Result<CurveEvalResult, RgmStatus> {
    if !(0.0..=1.0).contains(&t_norm) {
        return Err(RgmStatus::OutOfRange);
    }

    if poly.segments.is_empty() {
        return Err(RgmStatus::DegenerateGeometry);
    }

    if poly.total_length <= f64::EPSILON {
        return evaluate_curve_by_handle_at_length(state, poly.segments[0].curve, 0.0);
    }

    let length = t_norm * poly.total_length;
    evaluate_polycurve_at_length(state, poly, length)
}

fn evaluate_polycurve_at_length(
    state: &SessionState,
    poly: &PolycurveData,
    length: f64,
) -> Result<CurveEvalResult, RgmStatus> {
    if length < 0.0 || length > poly.total_length + 1e-10 {
        return Err(RgmStatus::OutOfRange);
    }

    if poly.segments.is_empty() {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let target = length.clamp(0.0, poly.total_length);

    let idx = poly
        .cumulative_lengths
        .iter()
        .position(|v| target <= *v + 1e-10)
        .unwrap_or(poly.cumulative_lengths.len().saturating_sub(1));

    let seg = &poly.segments[idx];
    let seg_start = if idx == 0 {
        0.0
    } else {
        poly.cumulative_lengths[idx - 1]
    };
    let mut local = target - seg_start;

    if seg.reversed {
        local = seg.length - local;
    }

    let mut eval = evaluate_curve_by_handle_at_length(state, seg.curve, local)?;
    if seg.reversed {
        eval.d1 = v3::neg(eval.d1);
    }

    Ok(eval)
}


fn mesh_world_vertices(mesh: &MeshData) -> Vec<RgmPoint3> {
    mesh.vertices
        .iter()
        .copied()
        .map(|point| matrix_apply_point(mesh.transform, point))
        .collect()
}

fn ensure_mesh_accel(state: &mut SessionState, handle: RgmObjectHandle) -> Result<(), RgmStatus> {
    if state.mesh_accels.contains_key(&handle.0) {
        return Ok(());
    }

    let mesh = find_mesh(state, handle)?.clone();
    let world_vertices = mesh_world_vertices(&mesh);
    let triangles = mesh
        .triangles
        .iter()
        .map(|tri| TriangleRecord::from_mesh(&world_vertices, *tri))
        .collect::<Vec<_>>();
    let bvh = MeshBvh::build(&triangles);
    state
        .mesh_accels
        .insert(handle.0, MeshAccelCache { triangles, bvh });
    Ok(())
}

fn matrix_apply_vec(matrix: [[f64; 4]; 4], vector: RgmVec3) -> RgmVec3 {
    RgmVec3 {
        x: matrix[0][0] * vector.x + matrix[0][1] * vector.y + matrix[0][2] * vector.z,
        y: matrix[1][0] * vector.x + matrix[1][1] * vector.y + matrix[1][2] * vector.z,
        z: matrix[2][0] * vector.x + matrix[2][1] * vector.y + matrix[2][2] * vector.z,
    }
}

fn adaptive_surface_tess_options(
    surface: &NurbsSurfaceCore,
    is_trimmed: bool,
) -> RgmSurfaceTessellationOptions {
    let spans_u = if surface.periodic_u {
        surface.control_u_count.saturating_sub(surface.degree_u).max(1)
    } else {
        (surface.control_u_count - 1).max(1)
    };
    let spans_v = if surface.periodic_v {
        surface.control_v_count.saturating_sub(surface.degree_v).max(1)
    } else {
        (surface.control_v_count - 1).max(1)
    };

    let samples_per_span = |deg: usize| -> usize {
        match deg {
            0 | 1 => 1,
            _ => 2,
        }
    };

    let min_u = (spans_u * samples_per_span(surface.degree_u)).clamp(4, 128) as u32;
    let min_v = (spans_v * samples_per_span(surface.degree_v)).clamp(4, 128) as u32;

    let max_factor: u32 = if is_trimmed { 2 } else { 4 };
    let max_u = (min_u * max_factor).min(256);
    let max_v = (min_v * max_factor).min(256);

    RgmSurfaceTessellationOptions {
        min_u_segments: min_u,
        min_v_segments: min_v,
        max_u_segments: max_u,
        max_v_segments: max_v,
        chord_tol: (surface.tol.abs_tol * 2000.0).max(1e-5),
        normal_tol_rad: 0.08,
    }
}

fn default_surface_intersection_tess_options(
    tol: RgmToleranceContext,
) -> RgmSurfaceTessellationOptions {
    RgmSurfaceTessellationOptions {
        min_u_segments: 14,
        min_v_segments: 14,
        max_u_segments: 56,
        max_v_segments: 56,
        chord_tol: (tol.abs_tol * 5000.0).max(5e-5),
        normal_tol_rad: 0.12,
    }
}

fn sanitize_surface_tess_options(
    options: Option<RgmSurfaceTessellationOptions>,
    surface: &NurbsSurfaceCore,
    is_trimmed: bool,
) -> RgmSurfaceTessellationOptions {
    let mut value = options.unwrap_or_else(|| adaptive_surface_tess_options(surface, is_trimmed));
    if value.min_u_segments < 2 {
        value.min_u_segments = 2;
    }
    if value.min_v_segments < 2 {
        value.min_v_segments = 2;
    }
    if value.max_u_segments < value.min_u_segments {
        value.max_u_segments = value.min_u_segments;
    }
    if value.max_v_segments < value.min_v_segments {
        value.max_v_segments = value.min_v_segments;
    }
    if value.max_u_segments > 1024 {
        value.max_u_segments = 1024;
    }
    if value.max_v_segments > 1024 {
        value.max_v_segments = 1024;
    }
    if value.chord_tol <= 0.0 {
        value.chord_tol = (surface.tol.abs_tol * 2000.0).max(1e-5);
    }
    if value.normal_tol_rad <= 0.0 {
        value.normal_tol_rad = 0.08;
    }
    value
}

fn surface_world_scale(surface: &SurfaceData) -> f64 {
    if surface.core.control_points.is_empty() {
        return 1.0;
    }
    let mut min = RgmPoint3 {
        x: f64::INFINITY,
        y: f64::INFINITY,
        z: f64::INFINITY,
    };
    let mut max = RgmPoint3 {
        x: f64::NEG_INFINITY,
        y: f64::NEG_INFINITY,
        z: f64::NEG_INFINITY,
    };
    for control in &surface.core.control_points {
        let world = matrix_apply_point(surface.transform, *control);
        min.x = min.x.min(world.x);
        min.y = min.y.min(world.y);
        min.z = min.z.min(world.z);
        max.x = max.x.max(world.x);
        max.y = max.y.max(world.y);
        max.z = max.z.max(world.z);
    }
    let dx = max.x - min.x;
    let dy = max.y - min.y;
    let dz = max.z - min.z;
    let scale = (dx * dx + dy * dy + dz * dz).sqrt();
    if scale.is_finite() && scale > 1e-12 {
        scale
    } else {
        1.0
    }
}

fn curve_world_scale(state: &SessionState, curve: &CurveData) -> f64 {
    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        if nurbs.core.control_points.is_empty() {
            return 1.0;
        }
        let mut min = RgmPoint3 {
            x: f64::INFINITY,
            y: f64::INFINITY,
            z: f64::INFINITY,
        };
        let mut max = RgmPoint3 {
            x: f64::NEG_INFINITY,
            y: f64::NEG_INFINITY,
            z: f64::NEG_INFINITY,
        };
        for point in &nurbs.core.control_points {
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            min.z = min.z.min(point.z);
            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
            max.z = max.z.max(point.z);
        }
        let dx = max.x - min.x;
        let dy = max.y - min.y;
        let dz = max.z - min.z;
        let scale = (dx * dx + dy * dy + dz * dz).sqrt();
        if scale.is_finite() && scale > 1e-12 {
            return scale;
        }
    }

    let mut min = RgmPoint3 {
        x: f64::INFINITY,
        y: f64::INFINITY,
        z: f64::INFINITY,
    };
    let mut max = RgmPoint3 {
        x: f64::NEG_INFINITY,
        y: f64::NEG_INFINITY,
        z: f64::NEG_INFINITY,
    };
    let mut hit = 0usize;
    for idx in 0..=32 {
        let t = idx as f64 / 32.0;
        let Ok(point) = curve_point_at_normalized_data(state, curve, t) else {
            continue;
        };
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        min.z = min.z.min(point.z);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
        max.z = max.z.max(point.z);
        hit += 1;
    }
    if hit < 2 {
        return 1.0;
    }
    let dx = max.x - min.x;
    let dy = max.y - min.y;
    let dz = max.z - min.z;
    let scale = (dx * dx + dy * dy + dz * dz).sqrt();
    if scale.is_finite() && scale > 1e-12 {
        scale
    } else {
        1.0
    }
}

fn intersection_chord_tol_from_scale(scale: f64, tol: f64) -> f64 {
    let scale = scale.max(1e-3);
    let min_chord = (tol * 40.0).max(1e-7);
    let target = scale * 5e-5;
    let max_chord = (scale * 2e-3).max(min_chord * 4.0);
    target.max(min_chord).min(max_chord)
}

fn surface_curve_seed_sample_count(curve: &CurveData) -> usize {
    if let Some(nurbs) = curve_canonical_nurbs(curve) {
        let control = nurbs.core.control_points.len();
        let degree = nurbs.core.degree.max(1);
        return (control.saturating_mul(40) + degree.saturating_mul(80)).clamp(240, 1200);
    }
    match curve {
        CurveData::Polycurve(poly) => poly.segments.len().saturating_mul(200).clamp(240, 1200),
        _ => 320,
    }
}

fn build_surface_from_desc(
    desc: RgmNurbsSurfaceDesc,
    control_points: &[RgmPoint3],
    weights: &[f64],
    knots_u: &[f64],
    knots_v: &[f64],
    tol: RgmToleranceContext,
) -> Result<SurfaceData, RgmStatus> {
    let control_u_count = desc.control_u_count as usize;
    let control_v_count = desc.control_v_count as usize;
    if control_u_count == 0 || control_v_count == 0 {
        return Err(RgmStatus::InvalidInput);
    }
    let control_count = control_u_count
        .checked_mul(control_v_count)
        .ok_or(RgmStatus::OutOfRange)?;
    if control_points.len() != control_count || weights.len() != control_count {
        return Err(RgmStatus::InvalidInput);
    }

    let degree_u = desc.degree_u as usize;
    let degree_v = desc.degree_v as usize;
    if control_u_count <= degree_u || control_v_count <= degree_v {
        return Err(RgmStatus::InvalidInput);
    }

    let mut core = NurbsSurfaceCore {
        degree_u,
        degree_v,
        periodic_u: desc.periodic_u,
        periodic_v: desc.periodic_v,
        control_u_count,
        control_v_count,
        control_points: control_points.to_vec(),
        weights: weights.to_vec(),
        knots_u: knots_u.to_vec(),
        knots_v: knots_v.to_vec(),
        u_start: 0.0,
        u_end: 0.0,
        v_start: 0.0,
        v_end: 0.0,
        tol,
    };
    core.u_start = core.knots_u[core.degree_u];
    core.u_end = core.knots_u[core.control_u_count];
    core.v_start = core.knots_v[core.degree_v];
    core.v_end = core.knots_v[core.control_v_count];
    validate_surface(&core)?;

    Ok(SurfaceData {
        core,
        transform: matrix_identity(),
    })
}

fn surface_eval_result_to_frame(
    eval: SurfaceEvalResult,
    transform: [[f64; 4]; 4],
) -> Result<RgmSurfaceEvalFrame, RgmStatus> {
    let point = matrix_apply_point(transform, eval.point);
    let du = matrix_apply_vec(transform, eval.du);
    let dv = matrix_apply_vec(transform, eval.dv);
    let normal = v3::cross(du, dv);
    let normal = v3::normalize(normal).ok_or(RgmStatus::DegenerateGeometry)?;

    Ok(RgmSurfaceEvalFrame {
        point,
        du,
        dv,
        normal,
    })
}

fn eval_surface_data_uv(
    surface: &SurfaceData,
    uv: RgmUv2,
) -> Result<RgmSurfaceEvalFrame, RgmStatus> {
    // Surface is validated at construction time in `build_surface_from_desc`;
    // skip the redundant validate_surface call on this hot intersection path.
    let eval = eval_nurbs_surface_uv_unchecked(&surface.core, uv)?;
    surface_eval_result_to_frame(eval, surface.transform)
}

fn eval_surface_data_normalized(
    surface: &SurfaceData,
    uv_norm: RgmUv2,
) -> Result<RgmSurfaceEvalFrame, RgmStatus> {
    let eval = eval_nurbs_surface_normalized(&surface.core, uv_norm)?;
    surface_eval_result_to_frame(eval, surface.transform)
}

fn uv_distance(a: RgmUv2, b: RgmUv2) -> f64 {
    let du = a.u - b.u;
    let dv = a.v - b.v;
    (du * du + dv * dv).sqrt()
}

fn uv_lerp(a: RgmUv2, b: RgmUv2, t: f64) -> RgmUv2 {
    RgmUv2 {
        u: a.u + (b.u - a.u) * t,
        v: a.v + (b.v - a.v) * t,
    }
}

fn uv_from_point3(point: RgmPoint3) -> RgmUv2 {
    RgmUv2 {
        u: point.x,
        v: point.y,
    }
}

fn uv_point_segment_distance(point: RgmUv2, a: RgmUv2, b: RgmUv2) -> f64 {
    let ab_u = b.u - a.u;
    let ab_v = b.v - a.v;
    let len2 = ab_u * ab_u + ab_v * ab_v;
    if len2 <= f64::EPSILON {
        return uv_distance(point, a);
    }
    let ap_u = point.u - a.u;
    let ap_v = point.v - a.v;
    let t = ((ap_u * ab_u + ap_v * ab_v) / len2).clamp(0.0, 1.0);
    let proj = RgmUv2 {
        u: a.u + t * ab_u,
        v: a.v + t * ab_v,
    };
    uv_distance(point, proj)
}

fn normalize_trim_edge_samples(
    start_uv: RgmUv2,
    end_uv: RgmUv2,
    samples: Vec<RgmUv2>,
    tol: f64,
) -> Vec<RgmUv2> {
    let mut out = Vec::with_capacity(samples.len().max(2));
    if samples.is_empty() {
        out.push(start_uv);
        out.push(end_uv);
        return out;
    }

    out.push(start_uv);
    for sample in samples {
        if out
            .last()
            .map(|last| uv_distance(*last, sample) > tol)
            .unwrap_or(true)
        {
            out.push(sample);
        }
    }

    if uv_distance(*out.last().unwrap_or(&start_uv), end_uv) > tol {
        out.push(end_uv);
    } else if let Some(last) = out.last_mut() {
        *last = end_uv;
    }

    if out.len() < 2 {
        out.push(end_uv);
    }

    out
}
