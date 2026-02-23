fn rgm_surface_create_nurbs_impl(
    session: RgmKernelHandle,
    desc: RgmNurbsSurfaceDesc,
    control_points: *const RgmPoint3,
    control_point_count: usize,
    weights: *const f64,
    weight_count: usize,
    knots_u: *const f64,
    knot_u_count: usize,
    knots_v: *const f64,
    knot_v_count: usize,
    tol: RgmToleranceContext,
    out_surface: *mut RgmObjectHandle,
) -> RgmStatus {
    if control_points.is_null() || weights.is_null() || knots_u.is_null() || knots_v.is_null() {
        return map_err_with_session(
            session,
            RgmStatus::InvalidInput,
            "Null surface array pointer",
        );
    }
    let control_points = unsafe { std::slice::from_raw_parts(control_points, control_point_count) };
    let weights = unsafe { std::slice::from_raw_parts(weights, weight_count) };
    let knots_u = unsafe { std::slice::from_raw_parts(knots_u, knot_u_count) };
    let knots_v = unsafe { std::slice::from_raw_parts(knots_v, knot_v_count) };

    create_surface_object(
        session,
        out_surface,
        |_| build_surface_from_desc(desc, control_points, weights, knots_u, knots_v, tol),
        "Surface constructor failed",
    )
}

fn rgm_surface_transform_impl(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    transform: [[f64; 4]; 4],
    out_surface: *mut RgmObjectHandle,
    message: &str,
) -> RgmStatus {
    create_surface_object(
        session,
        out_surface,
        |state| {
            let source = find_surface(state, surface)?;
            let mut next = source.clone();
            next.transform = matrix_mul(transform, source.transform);
            Ok(next)
        },
        message,
    )
}

fn rgm_surface_bake_transform_impl(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    out_surface: *mut RgmObjectHandle,
) -> RgmStatus {
    create_surface_object(
        session,
        out_surface,
        |state| {
            let source = find_surface(state, surface)?;
            let mut next = source.clone();
            for point in &mut next.core.control_points {
                *point = matrix_apply_point(source.transform, *point);
            }
            next.transform = matrix_identity();
            Ok(next)
        },
        "Surface bake transform failed",
    )
}

fn rgm_surface_tessellate_to_mesh_impl(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    options: Option<RgmSurfaceTessellationOptions>,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    create_mesh_object(
        session,
        out_mesh,
        |state| {
            let surface = find_surface(state, surface)?;
            let samples = tessellate_surface_samples(surface, None, options)?;
            Ok(build_mesh_from_tessellation(&samples))
        },
        "Surface tessellation failed",
    )
}

fn rgm_face_create_from_surface_impl(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    out_face: *mut RgmObjectHandle,
) -> RgmStatus {
    create_face_object(
        session,
        out_face,
        |state| {
            let _ = find_surface(state, surface)?;
            Ok(FaceData {
                surface,
                loops: Vec::new(),
            })
        },
        "Face creation failed",
    )
}

fn rgm_face_add_loop_impl(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    points: *const RgmUv2,
    point_count: usize,
    is_outer: bool,
) -> RgmStatus {
    if points.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null trim loop points");
    }
    let points = unsafe { std::slice::from_raw_parts(points, point_count) };
    let result = with_session_mut(session, |state| {
        let face_data = find_face_mut(state, face)?;
        let loop_data = build_trim_loop_from_points(points, is_outer, 1e-8)?;
        face_data.loops.push(loop_data);
        Ok(())
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Face add loop failed"),
    }
}

fn rgm_face_add_loop_edges_impl(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    loop_input: RgmTrimLoopInput,
    edges: *const RgmTrimEdgeInput,
    edge_count: usize,
) -> RgmStatus {
    if edges.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null trim edge array");
    }
    let edges = unsafe { std::slice::from_raw_parts(edges, edge_count) };
    let result = with_session_mut(session, |state| {
        let loop_data = build_trim_loop_from_edges(state, loop_input, edges, 1e-8)?;
        let face_data = find_face_mut(state, face)?;
        face_data.loops.push(loop_data);
        Ok(())
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Face add edge loop failed"),
    }
}

fn rgm_face_remove_loop_impl(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    loop_index: u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let face_data = find_face_mut(state, face)?;
        let idx = loop_index as usize;
        if idx >= face_data.loops.len() {
            return Err(RgmStatus::OutOfRange);
        }
        face_data.loops.remove(idx);
        Ok(())
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Face remove loop failed"),
    }
}

fn rgm_face_split_trim_edge_impl(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    loop_index: u32,
    edge_index: u32,
    split_t: f64,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let face_data = find_face_mut(state, face)?;
        let lidx = loop_index as usize;
        if lidx >= face_data.loops.len() {
            return Err(RgmStatus::OutOfRange);
        }
        if !(0.0..=1.0).contains(&split_t) {
            return Err(RgmStatus::OutOfRange);
        }
        let loop_data = &mut face_data.loops[lidx];
        let eidx = edge_index as usize;
        if eidx >= loop_data.edges.len() {
            return Err(RgmStatus::OutOfRange);
        }
        let edge = loop_data.edges[eidx].clone();
        let (split_uv, left_samples, right_samples) = split_trim_edge_samples(&edge, split_t, 1e-8);
        loop_data.edges[eidx].end_uv = split_uv;
        loop_data.edges[eidx].uv_samples = left_samples;
        loop_data.edges[eidx].curve_3d = None;
        loop_data.edges.insert(
            eidx + 1,
            TrimEdgeData {
                start_uv: split_uv,
                end_uv: edge.end_uv,
                curve_3d: None,
                uv_samples: right_samples,
            },
        );
        Ok(())
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Face split edge failed"),
    }
}

fn rgm_face_reverse_loop_impl(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    loop_index: u32,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let face_data = find_face_mut(state, face)?;
        let idx = loop_index as usize;
        if idx >= face_data.loops.len() {
            return Err(RgmStatus::OutOfRange);
        }
        let loop_data = &mut face_data.loops[idx];
        loop_data.edges.reverse();
        for edge in &mut loop_data.edges {
            std::mem::swap(&mut edge.start_uv, &mut edge.end_uv);
            edge.uv_samples.reverse();
            ensure_edge_samples(edge, 1e-8);
        }
        Ok(())
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Face reverse loop failed"),
    }
}

fn rgm_face_validate_impl(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    out_valid: *mut bool,
) -> RgmStatus {
    if out_valid.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null out_valid pointer");
    }
    let result = with_session_mut(session, |state| {
        let face_data = find_face(state, face)?;
        let valid = validate_face_data(face_data, 1e-8);
        unsafe {
            *out_valid = valid;
        }
        Ok(())
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Face validation failed"),
    }
}

fn rgm_face_heal_impl(session: RgmKernelHandle, face: RgmObjectHandle) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        let face_data = find_face_mut(state, face)?;
        heal_face(face_data, 1e-8);
        Ok(())
    });
    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Face heal failed"),
    }
}

fn rgm_face_tessellate_to_mesh_impl(
    session: RgmKernelHandle,
    face: RgmObjectHandle,
    options: Option<RgmSurfaceTessellationOptions>,
    out_mesh: *mut RgmObjectHandle,
) -> RgmStatus {
    create_mesh_object(
        session,
        out_mesh,
        |state| {
            let face_data = find_face(state, face)?;
            let surface = find_surface(state, face_data.surface)?;
            let samples = tessellate_surface_samples(surface, Some(face_data), options)?;
            Ok(build_mesh_from_tessellation(&samples))
        },
        "Face tessellation failed",
    )
}

fn rgm_intersect_surface_surface_impl(
    session: RgmKernelHandle,
    surface_a: RgmObjectHandle,
    surface_b: RgmObjectHandle,
    out_intersection: *mut RgmObjectHandle,
) -> RgmStatus {
    create_intersection_object(
        session,
        out_intersection,
        |state| {
            let (surface_a_data, face_a) = resolve_surface_operand(state, surface_a)?;
            let (surface_b_data, face_b) = resolve_surface_operand(state, surface_b)?;
            let face_a_classifier = face_a.as_ref().map(FaceUvClassifier::from_face);
            let face_b_classifier = face_b.as_ref().map(FaceUvClassifier::from_face);
            let options_a = default_surface_intersection_tess_options(surface_a_data.core.tol);
            let options_b = default_surface_intersection_tess_options(surface_b_data.core.tol);
            let tol = surface_a_data
                .core
                .tol
                .abs_tol
                .max(surface_b_data.core.tol.abs_tol)
                .max(1e-7);

            let seeds_raw = generate_surface_surface_seeds(
                &surface_a_data,
                &surface_b_data,
                options_a,
                options_b,
                tol,
            );
            let u_step_a = (surface_a_data.core.u_end - surface_a_data.core.u_start)
                .abs()
                .max(1e-12)
                / options_a.max_u_segments.max(1) as f64;
            let v_step_a = (surface_a_data.core.v_end - surface_a_data.core.v_start)
                .abs()
                .max(1e-12)
                / options_a.max_v_segments.max(1) as f64;
            let u_step_b = (surface_b_data.core.u_end - surface_b_data.core.u_start)
                .abs()
                .max(1e-12)
                / options_b.max_u_segments.max(1) as f64;
            let v_step_b = (surface_b_data.core.v_end - surface_b_data.core.v_start)
                .abs()
                .max(1e-12)
                / options_b.max_v_segments.max(1) as f64;
            let step_base = u_step_a.max(v_step_a).max(u_step_b).max(v_step_b).max(1e-5);
            let step_max = step_base * 2.0;
            let step_min = (step_base * 0.08).max(1e-7);
            let max_steps = (options_a.max_u_segments as usize
                + options_a.max_v_segments as usize
                + options_b.max_u_segments as usize
                + options_b.max_v_segments as usize)
                .saturating_mul(4)
                .clamp(60, 240);
            let model_scale =
                surface_world_scale(&surface_a_data).max(surface_world_scale(&surface_b_data));
            let mut seeds = Vec::new();
            let mut seed_deduper = BranchSpatialDeduper::new(
                (step_base * 24.0).max(model_scale * 0.08).max(tol * 200.0),
            );
            for seed in seeds_raw {
                if seed_deduper.has_duplicate(seed.0) {
                    continue;
                }
                seed_deduper.insert(seed.0);
                seeds.push(seed);
                if seeds.len() >= 6 {
                    break;
                }
            }
            let mut branches = Vec::new();
            let chord_tol = intersection_chord_tol_from_scale(model_scale, tol);
            let mut deduper = BranchSpatialDeduper::new((chord_tol * 1.5).max(tol * 8.0));
            for &(_, seed_uv_a, seed_uv_b) in &seeds {
                let Some(raw_branch) = build_surface_surface_branch_from_seed(
                    &surface_a_data,
                    &surface_b_data,
                    seed_uv_a,
                    seed_uv_b,
                    tol,
                    step_min,
                    step_max,
                    max_steps,
                ) else {
                    continue;
                };
                let branch = refine_surface_surface_branch_polyline(
                    &surface_a_data,
                    &surface_b_data,
                    &raw_branch,
                    chord_tol,
                    tol,
                );
                let clipped = clip_branch_against_faces(
                    &branch,
                    face_a_classifier.as_ref(),
                    face_b_classifier.as_ref(),
                    tol,
                );
                let mut candidates = clipped;
                if let Some(classifier) = face_a_classifier.as_ref() {
                    let mut next = Vec::new();
                    for candidate in &candidates {
                        next.extend(split_branch_inside_runs(candidate, classifier, true, 1e-8));
                    }
                    candidates = next;
                }
                if let Some(classifier) = face_b_classifier.as_ref() {
                    let mut next = Vec::new();
                    for candidate in &candidates {
                        next.extend(split_branch_inside_runs(candidate, classifier, false, 1e-8));
                    }
                    candidates = next;
                }
                for candidate in candidates {
                    push_unique_branch_fast(&mut branches, &mut deduper, candidate);
                }
                if branches.len() >= 4 {
                    break;
                }
            }
            let stitch_tol = adaptive_stitch_tolerance(&branches, tol);
            let branches = stitch_intersection_branches(branches, stitch_tol);
            Ok(IntersectionData { branches })
        },
        "Surface-surface intersection failed",
    )
}

fn rgm_intersect_surface_plane_impl(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    plane: RgmPlane,
    out_intersection: *mut RgmObjectHandle,
) -> RgmStatus {
    let Some(normal) = plane_unit_normal(plane) else {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Invalid plane normal");
    };
    create_intersection_object(
        session,
        out_intersection,
        |state| {
            let (surface_data, face) = resolve_surface_operand(state, surface)?;
            let face_classifier = face.as_ref().map(FaceUvClassifier::from_face);
            let options = default_surface_intersection_tess_options(surface_data.core.tol);
            let segments =
                intersect_surface_plane_uv_segments(&surface_data, plane.origin, normal, options)?;
            let tol = surface_data.core.tol.abs_tol.max(1e-7);
            let uv_tol = {
                let u_span = (surface_data.core.u_end - surface_data.core.u_start)
                    .abs()
                    .max(1e-12);
                let v_span = (surface_data.core.v_end - surface_data.core.v_start)
                    .abs()
                    .max(1e-12);
                let du = u_span / options.max_u_segments.max(1) as f64;
                let dv = v_span / options.max_v_segments.max(1) as f64;
                (du + dv).max(1e-9) * 0.08
            };
            let raw_branches = build_surface_plane_branches(&segments, uv_tol, tol * 8.0);
            let mut branches = Vec::new();
            let mut deduper = BranchSpatialDeduper::new(tol * 5.0);
            let chord_tol =
                intersection_chord_tol_from_scale(surface_world_scale(&surface_data), tol);
            for raw_branch in raw_branches {
                let refined_raw = refine_surface_plane_branch_polyline(
                    &surface_data,
                    plane.origin,
                    normal,
                    &raw_branch,
                    chord_tol,
                    tol,
                );
                let clipped =
                    clip_branch_against_faces(&refined_raw, face_classifier.as_ref(), None, tol);
                for branch in clipped {
                    if let Some(classifier) = face_classifier.as_ref() {
                        if !branch_within_face(&branch, classifier, true, 1e-8) {
                            continue;
                        }
                    }
                    push_unique_branch_fast(&mut branches, &mut deduper, branch);
                }
            }
            Ok(IntersectionData { branches })
        },
        "Surface-plane intersection failed",
    )
}

fn rgm_intersect_surface_curve_impl(
    session: RgmKernelHandle,
    surface: RgmObjectHandle,
    curve: RgmObjectHandle,
    out_intersection: *mut RgmObjectHandle,
) -> RgmStatus {
    create_intersection_object(
        session,
        out_intersection,
        |state| {
            let curve_data = find_curve(state, curve)?;
            let (surface_data, face) = resolve_surface_operand(state, surface)?;
            let face_classifier = face.as_ref().map(FaceUvClassifier::from_face);
            let tol = surface_data.core.tol.abs_tol.max(1e-7);
            let sample_count = surface_curve_seed_sample_count(curve_data);
            let scale =
                surface_world_scale(&surface_data).max(curve_world_scale(state, curve_data));
            let seed_tol = intersection_chord_tol_from_scale(scale, tol) * 8.0;
            let candidates = generate_surface_curve_candidates(
                state,
                &surface_data,
                curve_data,
                sample_count,
                tol,
                seed_tol,
            );

            let mut branches = Vec::new();
            let mut deduper = BranchSpatialDeduper::new(tol * 4.0);
            for (hit, uv, t_hit) in candidates {
                let branch = IntersectionBranchData {
                    points: vec![hit],
                    uv_a: vec![uv],
                    uv_b: Vec::new(),
                    curve_t: vec![t_hit],
                    closed: false,
                    flags: 0,
                };
                let clipped =
                    clip_branch_against_faces(&branch, face_classifier.as_ref(), None, tol);
                for value in clipped {
                    if let Some(classifier) = face_classifier.as_ref() {
                        if !branch_within_face(&value, classifier, true, 1e-8) {
                            continue;
                        }
                    }
                    push_unique_branch_fast(&mut branches, &mut deduper, value);
                }
            }

            Ok(IntersectionData { branches })
        },
        "Surface-curve intersection failed",
    )
}

fn rgm_curve_create_line_impl(
    session: RgmKernelHandle,
    line: RgmLine3,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    create_curve_object(
        session,
        out_object,
        |_| {
            let canonical_nurbs = build_line_nurbs(line, tol)?;
            Ok(CurveData::Line(canonical_nurbs))
        },
        "Line constructor failed",
    )
}

fn rgm_curve_create_circle_impl(
    session: RgmKernelHandle,
    circle: RgmCircle3,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    create_curve_object(
        session,
        out_object,
        |_| {
            let canonical_nurbs = build_circle_nurbs(circle, tol)?;
            Ok(CurveData::Circle(canonical_nurbs))
        },
        "Circle constructor failed",
    )
}

fn rgm_curve_create_arc_impl(
    session: RgmKernelHandle,
    arc: RgmArc3,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    create_curve_object(
        session,
        out_object,
        |_| {
            let canonical_nurbs = build_arc_nurbs(arc, tol)?;
            Ok(CurveData::Arc(canonical_nurbs))
        },
        "Arc constructor failed",
    )
}

fn rgm_curve_create_arc_by_angles_impl(
    session: RgmKernelHandle,
    plane: RgmPlane,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    let arc = match build_arc_from_start_end_angles(plane, radius, start_angle, end_angle, tol) {
        Ok(value) => value,
        Err(status) => {
            return map_err_with_session(session, status, "Arc-by-angles constructor failed");
        }
    };

    rgm_curve_create_arc_impl(session, arc, tol, out_object)
}

fn rgm_curve_create_arc_by_3_points_impl(
    session: RgmKernelHandle,
    start: RgmPoint3,
    mid: RgmPoint3,
    end: RgmPoint3,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    let arc = match build_arc_from_three_points(start, mid, end, tol) {
        Ok(value) => value,
        Err(status) => {
            return map_err_with_session(session, status, "Arc-by-3-points constructor failed");
        }
    };

    rgm_curve_create_arc_impl(session, arc, tol, out_object)
}

fn rgm_curve_create_polyline_impl(
    session: RgmKernelHandle,
    points: *const RgmPoint3,
    point_count: usize,
    closed: bool,
    tol: RgmToleranceContext,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if points.is_null() {
        return map_err_with_session(session, RgmStatus::InvalidInput, "Null points pointer");
    }

    // SAFETY: pointer and count validated by caller contract.
    let points = unsafe { std::slice::from_raw_parts(points, point_count) };

    create_curve_object(
        session,
        out_object,
        |_| {
            let canonical_nurbs = build_polyline_nurbs(points, closed, tol)?;
            Ok(CurveData::Polyline(canonical_nurbs))
        },
        "Polyline constructor failed",
    )
}

fn rgm_curve_create_polycurve_impl(
    session: RgmKernelHandle,
    segments: *const RgmPolycurveSegment,
    segment_count: usize,
    out_object: *mut RgmObjectHandle,
) -> RgmStatus {
    if segments.is_null() || segment_count == 0 {
        return map_err_with_session(
            session,
            RgmStatus::InvalidInput,
            "Invalid polycurve segments",
        );
    }

    // SAFETY: pointer/count validated above.
    let segments = unsafe { std::slice::from_raw_parts(segments, segment_count) };

    create_curve_object(
        session,
        out_object,
        |state| {
            let mut segment_data = Vec::with_capacity(segments.len());
            let mut cumulative = Vec::with_capacity(segments.len());
            let mut total = 0.0;

            for seg in segments {
                let curve = find_curve(state, seg.curve)?;
                if matches!(curve, CurveData::Polycurve(_)) {
                    return Err(RgmStatus::InvalidInput);
                }
                let len = curve_total_length(state, curve)?;
                total += len;
                cumulative.push(total);
                segment_data.push(PolycurveSegmentData {
                    curve: seg.curve,
                    reversed: seg.reversed,
                    length: len,
                });
            }

            Ok(CurveData::Polycurve(PolycurveData {
                segments: segment_data,
                cumulative_lengths: cumulative,
                total_length: total,
            }))
        },
        "Polycurve constructor failed",
    )
}


include!("../ffi/ptr.rs");
include!("../ffi/exports/kernel.rs");
include!("../ffi/exports/memory.rs");
include!("../ffi/exports/curve.rs");
include!("../ffi/exports/mesh.rs");
include!("../ffi/exports/surface.rs");
include!("../ffi/exports/face.rs");
include!("../ffi/exports/intersection.rs");

#[cfg(test)]
fn debug_get_curve(session: RgmKernelHandle, object: RgmObjectHandle) -> Option<CurveData> {
    let sessions = SESSIONS.lock().ok()?;
    let state = sessions.get(&session.0)?;
    match state.objects.get(&object.0)? {
        GeometryObject::Curve(curve) => Some(curve.clone()),
        GeometryObject::Mesh(_)
        | GeometryObject::Surface(_)
        | GeometryObject::Face(_)
        | GeometryObject::Intersection(_) => None,
    }
}

#[cfg(test)]
fn debug_get_face(session: RgmKernelHandle, object: RgmObjectHandle) -> Option<FaceData> {
    let sessions = SESSIONS.lock().ok()?;
    let state = sessions.get(&session.0)?;
    match state.objects.get(&object.0)? {
        GeometryObject::Face(face) => Some(face.clone()),
        GeometryObject::Curve(_)
        | GeometryObject::Mesh(_)
        | GeometryObject::Surface(_)
        | GeometryObject::Intersection(_) => None,
    }
}

#[cfg(test)]
fn debug_get_mesh(session: RgmKernelHandle, object: RgmObjectHandle) -> Option<MeshData> {
    let sessions = SESSIONS.lock().ok()?;
    let state = sessions.get(&session.0)?;
    match state.objects.get(&object.0)? {
        GeometryObject::Mesh(mesh) => Some(mesh.clone()),
        GeometryObject::Curve(_)
        | GeometryObject::Surface(_)
        | GeometryObject::Face(_)
        | GeometryObject::Intersection(_) => None,
    }
}

#[cfg(test)]
fn debug_get_intersection(
    session: RgmKernelHandle,
    object: RgmObjectHandle,
) -> Option<IntersectionData> {
    let sessions = SESSIONS.lock().ok()?;
    let state = sessions.get(&session.0)?;
    match state.objects.get(&object.0)? {
        GeometryObject::Intersection(data) => Some(data.clone()),
        GeometryObject::Curve(_)
        | GeometryObject::Mesh(_)
        | GeometryObject::Surface(_)
        | GeometryObject::Face(_) => None,
    }
}
#[cfg(test)]
include!("../tests/kernel_ffi.rs");
