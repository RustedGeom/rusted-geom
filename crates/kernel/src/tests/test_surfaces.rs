// ── Surface creation & evaluation ────────────────────────────────────────────

#[test]
fn surface_create_nurbs_and_evaluate_corners() {
    let session = create_session();

    let surface = create_bilinear_surface(session, 0.0, 1.0, 2.0, 3.0);

    let corners: [(f64, f64, RgmPoint3); 4] = [
        (0.0, 0.0, RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 }),
        (0.0, 1.0, RgmPoint3 { x: 0.0, y: 1.0, z: 1.0 }),
        (1.0, 0.0, RgmPoint3 { x: 1.0, y: 0.0, z: 2.0 }),
        (1.0, 1.0, RgmPoint3 { x: 1.0, y: 1.0, z: 3.0 }),
    ];

    for &(u, v, expected) in &corners {
        let uv = RgmUv2 { u, v };
        let mut pt = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
        assert_eq!(
            rgm_surface_point_at(session, surface, &uv as *const _, &mut pt as *mut _),
            RgmStatus::Ok
        );
        let err = ((pt.x - expected.x).powi(2) + (pt.y - expected.y).powi(2) + (pt.z - expected.z).powi(2)).sqrt();
        assert!(err < 1e-10, "Corner ({u},{v}): expected ({},{},{}), got ({},{},{}), err={err}",
            expected.x, expected.y, expected.z, pt.x, pt.y, pt.z);
    }

    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

#[test]
fn surface_point_at_center_is_bilinear_average() {
    let session = create_session();
    let surface = create_bilinear_surface(session, 0.0, 1.0, 2.0, 3.0);

    let uv = RgmUv2 { u: 0.5, v: 0.5 };
    let mut pt = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_point_at(session, surface, &uv as *const _, &mut pt as *mut _),
        RgmStatus::Ok
    );
    // bilinear interpolation: z = (1-u)(1-v)*0 + (1-u)v*1 + u(1-v)*2 + uv*3 = 1.5 at center
    assert!((pt.z - 1.5).abs() < 1e-10, "Center z should be 1.5, got {}", pt.z);
    assert!((pt.x - 0.5).abs() < 1e-10);
    assert!((pt.y - 0.5).abs() < 1e-10);

    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

// ── Derivatives ──────────────────────────────────────────────────────────────

#[test]
fn surface_d1_tangents_are_finite_and_nonzero() {
    let session = create_session();
    let surface = create_warped_surface(session, 8, 7, 6.0, 5.0, 1.0);

    let test_uvs = [
        RgmUv2 { u: 0.0, v: 0.0 },
        RgmUv2 { u: 0.5, v: 0.5 },
        RgmUv2 { u: 1.0, v: 1.0 },
        RgmUv2 { u: 0.25, v: 0.75 },
    ];

    for uv in &test_uvs {
        let mut du = RgmVec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut dv = RgmVec3 { x: 0.0, y: 0.0, z: 0.0 };
        assert_eq!(
            rgm_surface_d1_at(session, surface, uv as *const _, &mut du as *mut _, &mut dv as *mut _),
            RgmStatus::Ok
        );
        let du_len = (du.x * du.x + du.y * du.y + du.z * du.z).sqrt();
        let dv_len = (dv.x * dv.x + dv.y * dv.y + dv.z * dv.z).sqrt();
        assert!(du_len.is_finite() && du_len > 1e-12, "du should be finite and nonzero at ({},{})", uv.u, uv.v);
        assert!(dv_len.is_finite() && dv_len > 1e-12, "dv should be finite and nonzero at ({},{})", uv.u, uv.v);
    }

    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

#[test]
fn surface_d2_second_derivatives_are_finite() {
    let session = create_session();
    let surface = create_warped_surface(session, 8, 7, 6.0, 5.0, 1.0);

    let uv = RgmUv2 { u: 0.5, v: 0.5 };
    let mut duu = RgmVec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut duv = RgmVec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut dvv = RgmVec3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_d2_at(
            session, surface, &uv as *const _,
            &mut duu as *mut _, &mut duv as *mut _, &mut dvv as *mut _
        ),
        RgmStatus::Ok
    );
    assert!(duu.x.is_finite() && duu.y.is_finite() && duu.z.is_finite(), "duu must be finite");
    assert!(duv.x.is_finite() && duv.y.is_finite() && duv.z.is_finite(), "duv must be finite");
    assert!(dvv.x.is_finite() && dvv.y.is_finite() && dvv.z.is_finite(), "dvv must be finite");

    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

// ── Normal ───────────────────────────────────────────────────────────────────

#[test]
fn surface_normal_is_unit_length() {
    let session = create_session();
    let surface = create_warped_surface(session, 8, 7, 6.0, 5.0, 1.0);

    for iu in 0..=4 {
        for iv in 0..=4 {
            let uv = RgmUv2 { u: iu as f64 / 4.0, v: iv as f64 / 4.0 };
            let mut normal = RgmVec3 { x: 0.0, y: 0.0, z: 0.0 };
            assert_eq!(
                rgm_surface_normal_at(session, surface, &uv as *const _, &mut normal as *mut _),
                RgmStatus::Ok
            );
            let len = (normal.x * normal.x + normal.y * normal.y + normal.z * normal.z).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-6,
                "Normal at ({},{}) should be unit length, got {len}", uv.u, uv.v
            );
        }
    }

    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

#[test]
fn surface_normal_orthogonal_to_tangents() {
    let session = create_session();
    let surface = create_warped_surface(session, 8, 7, 6.0, 5.0, 1.0);

    let uv = RgmUv2 { u: 0.3, v: 0.7 };

    let mut du = RgmVec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut dv = RgmVec3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_d1_at(session, surface, &uv as *const _, &mut du as *mut _, &mut dv as *mut _),
        RgmStatus::Ok
    );

    let mut normal = RgmVec3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_normal_at(session, surface, &uv as *const _, &mut normal as *mut _),
        RgmStatus::Ok
    );

    let dot_du = normal.x * du.x + normal.y * du.y + normal.z * du.z;
    let dot_dv = normal.x * dv.x + normal.y * dv.y + normal.z * dv.z;
    let du_len = (du.x * du.x + du.y * du.y + du.z * du.z).sqrt();
    let dv_len = (dv.x * dv.x + dv.y * dv.y + dv.z * dv.z).sqrt();
    assert!(
        (dot_du / du_len).abs() < 1e-6,
        "Normal should be orthogonal to du, cosine = {}", dot_du / du_len
    );
    assert!(
        (dot_dv / dv_len).abs() < 1e-6,
        "Normal should be orthogonal to dv, cosine = {}", dot_dv / dv_len
    );

    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

// ── Frame ────────────────────────────────────────────────────────────────────

#[test]
fn surface_frame_consistent_with_individual_evaluations() {
    let session = create_session();
    let surface = create_warped_surface(session, 8, 7, 6.0, 5.0, 1.0);

    let uv = RgmUv2 { u: 0.4, v: 0.6 };

    let mut pt = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_point_at(session, surface, &uv as *const _, &mut pt as *mut _),
        RgmStatus::Ok
    );
    let mut du = RgmVec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut dv = RgmVec3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_d1_at(session, surface, &uv as *const _, &mut du as *mut _, &mut dv as *mut _),
        RgmStatus::Ok
    );
    let mut normal = RgmVec3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_normal_at(session, surface, &uv as *const _, &mut normal as *mut _),
        RgmStatus::Ok
    );

    let mut frame = RgmSurfaceEvalFrame {
        point: RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
        du: RgmVec3 { x: 0.0, y: 0.0, z: 0.0 },
        dv: RgmVec3 { x: 0.0, y: 0.0, z: 0.0 },
        normal: RgmVec3 { x: 0.0, y: 0.0, z: 0.0 },
    };
    assert_eq!(
        rgm_surface_frame_at(session, surface, &uv as *const _, &mut frame as *mut _),
        RgmStatus::Ok
    );

    let eps = 1e-12;
    assert!((frame.point.x - pt.x).abs() < eps, "Frame point.x mismatch");
    assert!((frame.point.y - pt.y).abs() < eps, "Frame point.y mismatch");
    assert!((frame.point.z - pt.z).abs() < eps, "Frame point.z mismatch");
    assert!((frame.du.x - du.x).abs() < eps, "Frame du.x mismatch");
    assert!((frame.du.y - du.y).abs() < eps, "Frame du.y mismatch");
    assert!((frame.du.z - du.z).abs() < eps, "Frame du.z mismatch");
    assert!((frame.dv.x - dv.x).abs() < eps, "Frame dv.x mismatch");
    assert!((frame.dv.y - dv.y).abs() < eps, "Frame dv.y mismatch");
    assert!((frame.dv.z - dv.z).abs() < eps, "Frame dv.z mismatch");
    assert!((frame.normal.x - normal.x).abs() < eps, "Frame normal.x mismatch");
    assert!((frame.normal.y - normal.y).abs() < eps, "Frame normal.y mismatch");
    assert!((frame.normal.z - normal.z).abs() < eps, "Frame normal.z mismatch");

    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

// ── Transforms ───────────────────────────────────────────────────────────────

#[test]
fn surface_translate_shifts_all_points() {
    let session = create_session();
    let surface = create_bilinear_surface(session, 0.0, 0.0, 0.0, 0.0);

    let delta = RgmVec3 { x: 10.0, y: 20.0, z: 30.0 };
    let mut translated = RgmObjectHandle(0);
    assert_eq!(
        rgm_surface_translate(session, surface, &delta as *const _, &mut translated as *mut _),
        RgmStatus::Ok
    );

    let uv = RgmUv2 { u: 0.0, v: 0.0 };
    let mut pt = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_point_at(session, translated, &uv as *const _, &mut pt as *mut _),
        RgmStatus::Ok
    );
    assert!((pt.x - 10.0).abs() < 1e-10);
    assert!((pt.y - 20.0).abs() < 1e-10);
    assert!((pt.z - 30.0).abs() < 1e-10);

    assert_eq!(rgm_object_release(session, translated), RgmStatus::Ok);
    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

#[test]
fn surface_scale_doubles_extent() {
    let session = create_session();
    let surface = create_bilinear_surface(session, 0.0, 0.0, 0.0, 0.0);

    let scale = RgmVec3 { x: 2.0, y: 3.0, z: 1.0 };
    let pivot = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut scaled = RgmObjectHandle(0);
    assert_eq!(
        rgm_surface_scale(session, surface, &scale as *const _, &pivot as *const _, &mut scaled as *mut _),
        RgmStatus::Ok
    );

    let uv = RgmUv2 { u: 1.0, v: 1.0 };
    let mut pt = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_point_at(session, scaled, &uv as *const _, &mut pt as *mut _),
        RgmStatus::Ok
    );
    // original corner (1,1) was at (1.0, 1.0, 0.0), scaled should be (2.0, 3.0, 0.0)
    assert!((pt.x - 2.0).abs() < 1e-10, "Scaled x should be 2.0, got {}", pt.x);
    assert!((pt.y - 3.0).abs() < 1e-10, "Scaled y should be 3.0, got {}", pt.y);

    assert_eq!(rgm_object_release(session, scaled), RgmStatus::Ok);
    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

#[test]
fn surface_rotate_90_degrees_swaps_axes() {
    let session = create_session();
    let surface = create_bilinear_surface(session, 0.0, 0.0, 0.0, 0.0);

    // Rotate 90 degrees around Z axis: (x,y) -> (-y, x)
    let axis = RgmVec3 { x: 0.0, y: 0.0, z: 1.0 };
    let pivot = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut rotated = RgmObjectHandle(0);
    assert_eq!(
        rgm_surface_rotate(
            session, surface,
            &axis as *const _, std::f64::consts::FRAC_PI_2,
            &pivot as *const _, &mut rotated as *mut _
        ),
        RgmStatus::Ok
    );

    // Original (1,0,0) -> rotated should be (0,1,0)
    let uv = RgmUv2 { u: 1.0, v: 0.0 };
    let mut pt = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_point_at(session, rotated, &uv as *const _, &mut pt as *mut _),
        RgmStatus::Ok
    );
    assert!((pt.x - 0.0).abs() < 1e-10, "Rotated x should be ~0, got {}", pt.x);
    assert!((pt.y - 1.0).abs() < 1e-10, "Rotated y should be ~1, got {}", pt.y);

    assert_eq!(rgm_object_release(session, rotated), RgmStatus::Ok);
    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

// ── Tessellation ─────────────────────────────────────────────────────────────

#[test]
fn surface_tessellate_produces_nonzero_mesh() {
    let session = create_session();
    let surface = create_warped_surface(session, 8, 7, 6.0, 5.0, 1.0);

    let mut mesh = RgmObjectHandle(0);
    assert_eq!(
        rgm_surface_tessellate_to_mesh(session, surface, ptr::null(), &mut mesh as *mut _),
        RgmStatus::Ok
    );

    let mut vertex_count = 0_u32;
    let mut triangle_count = 0_u32;
    assert_eq!(rgm_mesh_vertex_count(session, mesh, &mut vertex_count as *mut _), RgmStatus::Ok);
    assert_eq!(rgm_mesh_triangle_count(session, mesh, &mut triangle_count as *mut _), RgmStatus::Ok);
    assert!(vertex_count > 0, "Tessellation should produce vertices");
    assert!(triangle_count > 0, "Tessellation should produce triangles");
    assert!(triangle_count >= 2, "Even minimal tessellation needs at least 2 triangles");

    assert_eq!(rgm_object_release(session, mesh), RgmStatus::Ok);
    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

#[test]
fn surface_tessellation_vertices_lie_on_surface() {
    let session = create_session();
    let surface = create_bilinear_surface(session, 0.0, 1.0, 2.0, 3.0);

    let mut mesh = RgmObjectHandle(0);
    assert_eq!(
        rgm_surface_tessellate_to_mesh(session, surface, ptr::null(), &mut mesh as *mut _),
        RgmStatus::Ok
    );

    let mut vert_count = 0_u32;
    assert_eq!(rgm_mesh_vertex_count(session, mesh, &mut vert_count as *mut _), RgmStatus::Ok);
    assert!(vert_count > 0);

    let mesh_data = debug_get_mesh(session, mesh).expect("mesh must exist");
    for vertex in &mesh_data.vertices {
        assert!(vertex.x >= -0.01 && vertex.x <= 1.01, "Vertex x={} out of surface domain", vertex.x);
        assert!(vertex.y >= -0.01 && vertex.y <= 1.01, "Vertex y={} out of surface domain", vertex.y);
    }

    assert_eq!(rgm_object_release(session, mesh), RgmStatus::Ok);
    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

// ── Runtime contract ─────────────────────────────────────────────────────────

#[test]
fn runtime_contract_surface_session_flow() {
    let session = create_session();

    let points = runtime_surface_points();
    let weights: Vec<f64> = vec![1.0; 9];
    let knots = clamped_uniform_knots(3, 2);

    let desc = RgmNurbsSurfaceDesc {
        degree_u: 2,
        degree_v: 2,
        periodic_u: false,
        periodic_v: false,
        control_u_count: 3,
        control_v_count: 3,
    };

    let mut surface = RgmObjectHandle(0);
    assert_eq!(
        rgm_surface_create_nurbs(
            session,
            &desc as *const _,
            points.as_ptr(),
            points.len(),
            weights.as_ptr(),
            weights.len(),
            knots.as_ptr(),
            knots.len(),
            knots.as_ptr(),
            knots.len(),
            &tol() as *const _,
            &mut surface as *mut _,
        ),
        RgmStatus::Ok
    );
    assert_ne!(surface.0, 0, "Surface handle should be non-zero");

    let uv = RgmUv2 { u: 0.5, v: 0.5 };
    let mut pt = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_point_at(session, surface, &uv as *const _, &mut pt as *mut _),
        RgmStatus::Ok
    );
    assert!(pt.x.is_finite() && pt.y.is_finite() && pt.z.is_finite());

    let mut frame = RgmSurfaceEvalFrame {
        point: RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
        du: RgmVec3 { x: 0.0, y: 0.0, z: 0.0 },
        dv: RgmVec3 { x: 0.0, y: 0.0, z: 0.0 },
        normal: RgmVec3 { x: 0.0, y: 0.0, z: 0.0 },
    };
    assert_eq!(
        rgm_surface_frame_at(session, surface, &uv as *const _, &mut frame as *mut _),
        RgmStatus::Ok
    );
    assert!((frame.point.x - pt.x).abs() < 1e-12);
    assert!((frame.point.y - pt.y).abs() < 1e-12);
    assert!((frame.point.z - pt.z).abs() < 1e-12);

    let mut mesh = RgmObjectHandle(0);
    assert_eq!(
        rgm_surface_tessellate_to_mesh(session, surface, ptr::null(), &mut mesh as *mut _),
        RgmStatus::Ok
    );
    let mut tri_count = 0_u32;
    assert_eq!(rgm_mesh_triangle_count(session, mesh, &mut tri_count as *mut _), RgmStatus::Ok);
    assert!(tri_count > 0, "Tessellation must produce triangles");

    let delta = RgmVec3 { x: 5.0, y: 0.0, z: 0.0 };
    let mut translated = RgmObjectHandle(0);
    assert_eq!(
        rgm_surface_translate(session, surface, &delta as *const _, &mut translated as *mut _),
        RgmStatus::Ok
    );
    let mut pt2 = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_point_at(session, translated, &uv as *const _, &mut pt2 as *mut _),
        RgmStatus::Ok
    );
    assert!((pt2.x - pt.x - 5.0).abs() < 1e-10, "Translated x should shift by 5");

    assert_eq!(rgm_object_release(session, translated), RgmStatus::Ok);
    assert_eq!(rgm_object_release(session, mesh), RgmStatus::Ok);
    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}
