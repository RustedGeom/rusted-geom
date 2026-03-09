// ── Sweep tests ──────────────────────────────────────────────────────────────

#[test]
fn test_sweep_surface_basic() {
    // Sweep an open polyline profile along a straight path
    // cap_faces = false → should return a surface
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let tol = RgmToleranceContext {
        abs_tol: 1e-6,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    // Path: straight line along X axis, 10 units
    let path_pts = [
        RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 5.0,
            y: 0.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 10.0,
            y: 0.0,
            z: 0.0,
        },
    ];
    let mut path = RgmObjectHandle(0);
    assert_eq!(
        rgm_nurbs_interpolate_fit_points(
            session,
            path_pts.as_ptr(),
            3,
            2,
            false,
            tol,
            &mut path,
        ),
        RgmStatus::Ok
    );

    // Profile: open L-shape in YZ plane
    let profile_pts = [
        RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 0.0,
            y: 1.0,
            z: 1.0,
        },
    ];
    let mut profile = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(
            session,
            profile_pts.as_ptr(),
            3,
            false,
            tol,
            &mut profile,
        ),
        RgmStatus::Ok
    );

    // Sweep with cap_faces = false
    let mut out = RgmObjectHandle(0);
    assert_eq!(
        rgm_sweep(session, path, profile, 10, false, &mut out),
        RgmStatus::Ok
    );
    assert_ne!(out.0, 0);

    // Should be tessellatable as a surface
    let mut mesh = RgmObjectHandle(0);
    assert_eq!(
        rgm_surface_tessellate_to_mesh(session, out, ptr::null(), &mut mesh),
        RgmStatus::Ok
    );
    let mut vertex_count = 0_u32;
    assert_eq!(
        rgm_mesh_vertex_count(session, mesh, &mut vertex_count as *mut _),
        RgmStatus::Ok
    );
    assert!(vertex_count > 0);
    let mut triangle_count = 0_u32;
    assert_eq!(
        rgm_mesh_triangle_count(session, mesh, &mut triangle_count as *mut _),
        RgmStatus::Ok
    );
    assert!(triangle_count > 0);

    rgm_kernel_destroy(session);
}

#[test]
fn test_sweep_solid_closed_profile() {
    // Sweep a closed rectangular profile along a curved path
    // cap_faces = true → should return a BRep solid
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let tol = RgmToleranceContext {
        abs_tol: 1e-6,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    // Path: gentle arc in XZ plane (pre-camber-like)
    let path_pts = [
        RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 5.0,
            y: 0.0,
            z: 0.5,
        },
        RgmPoint3 {
            x: 10.0,
            y: 0.0,
            z: 0.0,
        },
    ];
    let mut path = RgmObjectHandle(0);
    assert_eq!(
        rgm_nurbs_interpolate_fit_points(
            session,
            path_pts.as_ptr(),
            3,
            2,
            false,
            tol,
            &mut path,
        ),
        RgmStatus::Ok
    );

    // Profile: closed rectangle in YZ plane
    let profile_pts = [
        RgmPoint3 {
            x: 0.0,
            y: -0.5,
            z: -0.5,
        },
        RgmPoint3 {
            x: 0.0,
            y: 0.5,
            z: -0.5,
        },
        RgmPoint3 {
            x: 0.0,
            y: 0.5,
            z: 0.5,
        },
        RgmPoint3 {
            x: 0.0,
            y: -0.5,
            z: 0.5,
        },
    ];
    let mut profile = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(
            session,
            profile_pts.as_ptr(),
            4,
            true,
            tol,
            &mut profile,
        ),
        RgmStatus::Ok
    );

    // Sweep with cap_faces = true
    let mut out = RgmObjectHandle(0);
    assert_eq!(
        rgm_sweep(session, path, profile, 20, true, &mut out),
        RgmStatus::Ok
    );
    assert_ne!(out.0, 0);

    // Should be a brep - tessellate
    let mut mesh = RgmObjectHandle(0);
    assert_eq!(
        rgm_brep_tessellate_to_mesh(session, out, ptr::null(), &mut mesh),
        RgmStatus::Ok
    );
    let mut vertex_count = 0_u32;
    assert_eq!(
        rgm_mesh_vertex_count(session, mesh, &mut vertex_count as *mut _),
        RgmStatus::Ok
    );
    assert!(vertex_count > 0);

    // Should have 3 faces (body + 2 caps)
    let mut face_count = 0u32;
    assert_eq!(rgm_brep_face_count(session, out, &mut face_count), RgmStatus::Ok);
    assert_eq!(face_count, 3);

    // Should be a solid
    let mut is_solid = false;
    assert_eq!(rgm_brep_is_solid(session, out, &mut is_solid), RgmStatus::Ok);
    assert!(is_solid);

    rgm_kernel_destroy(session);
}

#[test]
fn test_sweep_cap_faces_open_profile_error() {
    // cap_faces = true with an OPEN profile should fail
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let tol = RgmToleranceContext {
        abs_tol: 1e-6,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    let path_pts = [
        RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 10.0,
            y: 0.0,
            z: 0.0,
        },
    ];
    let mut path = RgmObjectHandle(0);
    assert_eq!(
        rgm_nurbs_interpolate_fit_points(
            session,
            path_pts.as_ptr(),
            2,
            1,
            false,
            tol,
            &mut path,
        ),
        RgmStatus::Ok
    );

    // OPEN profile
    let profile_pts = [
        RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 0.0,
            y: 1.0,
            z: 1.0,
        },
    ];
    let mut profile = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(
            session,
            profile_pts.as_ptr(),
            3,
            false,
            tol,
            &mut profile,
        ),
        RgmStatus::Ok
    );

    // Sweep with cap_faces = true should fail since profile is open
    let mut out = RgmObjectHandle(0);
    let status = rgm_sweep(session, path, profile, 10, true, &mut out);
    assert_eq!(status, RgmStatus::InvalidInput);

    rgm_kernel_destroy(session);
}

// ── Loft tests ───────────────────────────────────────────────────────────────

#[test]
fn test_loft_surface_basic() {
    // Loft between 3 polyline sections, cap_faces = false
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let tol = RgmToleranceContext {
        abs_tol: 1e-6,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    // Section 0 at x=0: small rectangle
    let s0 = [
        RgmPoint3 {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 0.0,
            y: 1.0,
            z: 1.0,
        },
        RgmPoint3 {
            x: 0.0,
            y: -1.0,
            z: 1.0,
        },
    ];
    let mut h0 = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, s0.as_ptr(), 4, false, tol, &mut h0),
        RgmStatus::Ok
    );

    // Section 1 at x=5: wider
    let s1 = [
        RgmPoint3 {
            x: 5.0,
            y: -2.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 5.0,
            y: 2.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 5.0,
            y: 2.0,
            z: 1.5,
        },
        RgmPoint3 {
            x: 5.0,
            y: -2.0,
            z: 1.5,
        },
    ];
    let mut h1 = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, s1.as_ptr(), 4, false, tol, &mut h1),
        RgmStatus::Ok
    );

    // Section 2 at x=10: medium
    let s2 = [
        RgmPoint3 {
            x: 10.0,
            y: -1.5,
            z: 0.0,
        },
        RgmPoint3 {
            x: 10.0,
            y: 1.5,
            z: 0.0,
        },
        RgmPoint3 {
            x: 10.0,
            y: 1.5,
            z: 1.2,
        },
        RgmPoint3 {
            x: 10.0,
            y: -1.5,
            z: 1.2,
        },
    ];
    let mut h2 = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, s2.as_ptr(), 4, false, tol, &mut h2),
        RgmStatus::Ok
    );

    let sections = [h0, h1, h2];
    let mut out = RgmObjectHandle(0);
    assert_eq!(
        rgm_loft(session, sections.as_ptr(), 3, 20, false, &mut out),
        RgmStatus::Ok
    );
    assert_ne!(out.0, 0);

    // Should tessellate as a surface
    let mut mesh = RgmObjectHandle(0);
    assert_eq!(
        rgm_surface_tessellate_to_mesh(session, out, ptr::null(), &mut mesh),
        RgmStatus::Ok
    );
    let mut vertex_count = 0_u32;
    assert_eq!(
        rgm_mesh_vertex_count(session, mesh, &mut vertex_count as *mut _),
        RgmStatus::Ok
    );
    assert!(vertex_count > 0);

    rgm_kernel_destroy(session);
}

#[test]
fn test_loft_solid_closed_sections() {
    // Loft between 3 closed sections, cap_faces = true → BRep solid
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let tol = RgmToleranceContext {
        abs_tol: 1e-6,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    // 3 closed rectangular sections at x=0, 5, 10
    let s0 = [
        RgmPoint3 {
            x: 0.0,
            y: -1.0,
            z: -0.5,
        },
        RgmPoint3 {
            x: 0.0,
            y: 1.0,
            z: -0.5,
        },
        RgmPoint3 {
            x: 0.0,
            y: 1.0,
            z: 0.5,
        },
        RgmPoint3 {
            x: 0.0,
            y: -1.0,
            z: 0.5,
        },
    ];
    let mut h0 = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, s0.as_ptr(), 4, true, tol, &mut h0),
        RgmStatus::Ok
    );

    let s1 = [
        RgmPoint3 {
            x: 5.0,
            y: -2.0,
            z: -1.0,
        },
        RgmPoint3 {
            x: 5.0,
            y: 2.0,
            z: -1.0,
        },
        RgmPoint3 {
            x: 5.0,
            y: 2.0,
            z: 1.0,
        },
        RgmPoint3 {
            x: 5.0,
            y: -2.0,
            z: 1.0,
        },
    ];
    let mut h1 = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, s1.as_ptr(), 4, true, tol, &mut h1),
        RgmStatus::Ok
    );

    let s2 = [
        RgmPoint3 {
            x: 10.0,
            y: -1.0,
            z: -0.5,
        },
        RgmPoint3 {
            x: 10.0,
            y: 1.0,
            z: -0.5,
        },
        RgmPoint3 {
            x: 10.0,
            y: 1.0,
            z: 0.5,
        },
        RgmPoint3 {
            x: 10.0,
            y: -1.0,
            z: 0.5,
        },
    ];
    let mut h2 = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, s2.as_ptr(), 4, true, tol, &mut h2),
        RgmStatus::Ok
    );

    let sections = [h0, h1, h2];
    let mut out = RgmObjectHandle(0);
    assert_eq!(
        rgm_loft(session, sections.as_ptr(), 3, 20, true, &mut out),
        RgmStatus::Ok
    );
    assert_ne!(out.0, 0);

    // Should be tessellatable as brep
    let mut mesh = RgmObjectHandle(0);
    assert_eq!(
        rgm_brep_tessellate_to_mesh(session, out, ptr::null(), &mut mesh),
        RgmStatus::Ok
    );
    let mut vertex_count = 0_u32;
    assert_eq!(
        rgm_mesh_vertex_count(session, mesh, &mut vertex_count as *mut _),
        RgmStatus::Ok
    );
    assert!(vertex_count > 0);

    // 3 faces: body + 2 caps
    let mut face_count = 0u32;
    assert_eq!(rgm_brep_face_count(session, out, &mut face_count), RgmStatus::Ok);
    assert_eq!(face_count, 3);

    rgm_kernel_destroy(session);
}

#[test]
fn test_loft_cap_faces_open_section_error() {
    // cap_faces = true with open sections should fail
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let tol = RgmToleranceContext {
        abs_tol: 1e-6,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    let s0 = [
        RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
    ];
    let mut h0 = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, s0.as_ptr(), 2, false, tol, &mut h0),
        RgmStatus::Ok
    );

    let s1 = [
        RgmPoint3 {
            x: 5.0,
            y: 0.0,
            z: 0.0,
        },
        RgmPoint3 {
            x: 5.0,
            y: 2.0,
            z: 0.0,
        },
    ];
    let mut h1 = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, s1.as_ptr(), 2, false, tol, &mut h1),
        RgmStatus::Ok
    );

    let sections = [h0, h1];
    let mut out = RgmObjectHandle(0);
    let status = rgm_loft(session, sections.as_ptr(), 2, 10, true, &mut out);
    assert_eq!(status, RgmStatus::InvalidInput);

    rgm_kernel_destroy(session);
}

// ── Interpolation accuracy tests ─────────────────────────────────────────────

#[test]
fn test_sweep_interpolates_profile_at_stations() {
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let tol = RgmToleranceContext {
        abs_tol: 1e-6,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    let path_pts = [
        RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
        RgmPoint3 { x: 5.0, y: 0.0, z: 1.0 },
        RgmPoint3 { x: 10.0, y: 0.0, z: 0.0 },
    ];
    let mut path = RgmObjectHandle(0);
    assert_eq!(
        rgm_nurbs_interpolate_fit_points(session, path_pts.as_ptr(), 3, 2, false, tol, &mut path),
        RgmStatus::Ok
    );

    let profile_pts = [
        RgmPoint3 { x: 0.0, y: -0.5, z: -0.3 },
        RgmPoint3 { x: 0.0, y: 0.5,  z: -0.3 },
        RgmPoint3 { x: 0.0, y: 0.5,  z: 0.3 },
        RgmPoint3 { x: 0.0, y: -0.5, z: 0.3 },
    ];
    let mut profile = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, profile_pts.as_ptr(), 4, false, tol, &mut profile),
        RgmStatus::Ok
    );

    let n_stations = 8;
    let mut out = RgmObjectHandle(0);
    assert_eq!(
        rgm_sweep(session, path, profile, n_stations, false, &mut out),
        RgmStatus::Ok
    );

    // U = stations (along path), V = profile
    let check_v = 0.5;
    let mut expected_mid = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_curve_point_at(session, profile, check_v, &mut expected_mid),
        RgmStatus::Ok
    );

    let mut path_mid = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_curve_point_at(session, path, 0.5, &mut path_mid),
        RgmStatus::Ok
    );

    let uv = RgmUv2 { u: 0.5, v: check_v };
    let mut surface_pt = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_point_at(session, out, &uv, &mut surface_pt),
        RgmStatus::Ok
    );

    let offset_y = surface_pt.y - path_mid.y;
    let offset_z = surface_pt.z - path_mid.z;
    let expected_offset_y = expected_mid.y;
    let expected_offset_z = expected_mid.z;

    let err = ((offset_y - expected_offset_y).powi(2)
        + (offset_z - expected_offset_z).powi(2))
    .sqrt();
    assert!(
        err < 0.05,
        "Sweep surface should interpolate profile at mid-station: offset err = {err}"
    );

    rgm_kernel_destroy(session);
}

#[test]
fn test_loft_interpolates_sections() {
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let tol = RgmToleranceContext {
        abs_tol: 1e-6,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    let s0 = [
        RgmPoint3 { x: 0.0, y: -1.0, z: 0.0 },
        RgmPoint3 { x: 0.0, y: 1.0,  z: 0.0 },
        RgmPoint3 { x: 0.0, y: 1.0,  z: 1.0 },
        RgmPoint3 { x: 0.0, y: -1.0, z: 1.0 },
    ];
    let mut h0 = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, s0.as_ptr(), 4, false, tol, &mut h0),
        RgmStatus::Ok
    );

    let s1 = [
        RgmPoint3 { x: 5.0, y: -2.0, z: 0.0 },
        RgmPoint3 { x: 5.0, y: 2.0,  z: 0.0 },
        RgmPoint3 { x: 5.0, y: 2.0,  z: 1.5 },
        RgmPoint3 { x: 5.0, y: -2.0, z: 1.5 },
    ];
    let mut h1 = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, s1.as_ptr(), 4, false, tol, &mut h1),
        RgmStatus::Ok
    );

    let s2 = [
        RgmPoint3 { x: 10.0, y: -1.5, z: 0.0 },
        RgmPoint3 { x: 10.0, y: 1.5,  z: 0.0 },
        RgmPoint3 { x: 10.0, y: 1.5,  z: 1.2 },
        RgmPoint3 { x: 10.0, y: -1.5, z: 1.2 },
    ];
    let mut h2 = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, s2.as_ptr(), 4, false, tol, &mut h2),
        RgmStatus::Ok
    );

    let sections = [h0, h1, h2];
    let n_samples = 12;
    let mut out = RgmObjectHandle(0);
    assert_eq!(
        rgm_loft(session, sections.as_ptr(), 3, n_samples, false, &mut out),
        RgmStatus::Ok
    );

    let mut section1_start = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(rgm_curve_point_at(session, h1, 0.0, &mut section1_start), RgmStatus::Ok);
    let mut section1_end = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(rgm_curve_point_at(session, h1, 1.0, &mut section1_end), RgmStatus::Ok);

    // U = sections direction, V = samples direction
    let uv_start = RgmUv2 { u: 0.5, v: 0.0 };
    let mut surf_start = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_point_at(session, out, &uv_start, &mut surf_start),
        RgmStatus::Ok
    );

    // u=0.5 is close but not exactly the middle section's chord-length parameter,
    // so we allow a small geometric tolerance.
    let err_start = ((surf_start.x - section1_start.x).powi(2)
        + (surf_start.y - section1_start.y).powi(2)
        + (surf_start.z - section1_start.z).powi(2))
    .sqrt();
    assert!(
        err_start < 0.05,
        "Loft surface should interpolate near middle section at v=0: err = {err_start}, \
         expected ({},{},{}), got ({},{},{})",
        section1_start.x, section1_start.y, section1_start.z,
        surf_start.x, surf_start.y, surf_start.z
    );

    let uv_end = RgmUv2 { u: 0.5, v: 1.0 };
    let mut surf_end = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(
        rgm_surface_point_at(session, out, &uv_end, &mut surf_end),
        RgmStatus::Ok
    );

    let err_end = ((surf_end.x - section1_end.x).powi(2)
        + (surf_end.y - section1_end.y).powi(2)
        + (surf_end.z - section1_end.z).powi(2))
    .sqrt();
    assert!(
        err_end < 0.05,
        "Loft surface should interpolate near middle section at v=1: err = {err_end}, \
         expected ({},{},{}), got ({},{},{})",
        section1_end.x, section1_end.y, section1_end.z,
        surf_end.x, surf_end.y, surf_end.z
    );

    // Verify endpoint sections are EXACT (u=0 = section 0, u=1 = section 2)
    let mut s0_start = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(rgm_curve_point_at(session, h0, 0.0, &mut s0_start), RgmStatus::Ok);
    let uv_s0 = RgmUv2 { u: 0.0, v: 0.0 };
    let mut surf_s0 = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(rgm_surface_point_at(session, out, &uv_s0, &mut surf_s0), RgmStatus::Ok);

    let err_s0 = ((surf_s0.x - s0_start.x).powi(2)
        + (surf_s0.y - s0_start.y).powi(2)
        + (surf_s0.z - s0_start.z).powi(2)).sqrt();
    assert!(
        err_s0 < 1e-6,
        "Loft should exactly interpolate first section at u=0: err={err_s0}"
    );

    rgm_kernel_destroy(session);
}

#[test]
fn test_loft_closed_sections_periodic_u() {
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let tol = RgmToleranceContext {
        abs_tol: 1e-6,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    let s0 = [
        RgmPoint3 { x: 0.0, y: -1.0, z: -0.5 },
        RgmPoint3 { x: 0.0, y: 1.0,  z: -0.5 },
        RgmPoint3 { x: 0.0, y: 1.0,  z: 0.5 },
        RgmPoint3 { x: 0.0, y: -1.0, z: 0.5 },
    ];
    let mut h0 = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, s0.as_ptr(), 4, true, tol, &mut h0),
        RgmStatus::Ok
    );

    let s1 = [
        RgmPoint3 { x: 10.0, y: -2.0, z: -1.0 },
        RgmPoint3 { x: 10.0, y: 2.0,  z: -1.0 },
        RgmPoint3 { x: 10.0, y: 2.0,  z: 1.0 },
        RgmPoint3 { x: 10.0, y: -2.0, z: 1.0 },
    ];
    let mut h1 = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(session, s1.as_ptr(), 4, true, tol, &mut h1),
        RgmStatus::Ok
    );

    let sections = [h0, h1];
    let mut out = RgmObjectHandle(0);
    assert_eq!(
        rgm_loft(session, sections.as_ptr(), 2, 12, false, &mut out),
        RgmStatus::Ok
    );

    let mut mesh = RgmObjectHandle(0);
    assert_eq!(
        rgm_surface_tessellate_to_mesh(session, out, ptr::null(), &mut mesh),
        RgmStatus::Ok
    );
    let mut vertex_count = 0u32;
    assert_eq!(
        rgm_mesh_vertex_count(session, mesh, &mut vertex_count as *mut _),
        RgmStatus::Ok
    );
    assert!(vertex_count > 0, "Periodic-U loft should tessellate");

    // With compatible-CP skinning, the surface should exactly interpolate both section curves.
    // Check that a point on section 0 (u=0) at v=0.25 matches the curve evaluation at t=0.25.
    let mut curve_pt = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(rgm_curve_point_at(session, h0, 0.25, &mut curve_pt), RgmStatus::Ok);

    let uv = RgmUv2 { u: 0.0, v: 0.25 };
    let mut surf_pt = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
    assert_eq!(rgm_surface_point_at(session, out, &uv, &mut surf_pt), RgmStatus::Ok);

    let err = ((surf_pt.x - curve_pt.x).powi(2)
        + (surf_pt.y - curve_pt.y).powi(2)
        + (surf_pt.z - curve_pt.z).powi(2)).sqrt();
    assert!(
        err < 1e-4,
        "Periodic loft should interpolate section 0 at v=0.25: err={err}, \
         expected ({},{},{}), got ({},{},{})",
        curve_pt.x, curve_pt.y, curve_pt.z,
        surf_pt.x, surf_pt.y, surf_pt.z
    );

    rgm_kernel_destroy(session);
}

#[test]
fn test_loft_compatible_cp_exact_interpolation() {
    // Verify that compatible-CP loft (all polylines with same vertex count)
    // produces a surface that exactly passes through ALL section curves.
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let tol = RgmToleranceContext {
        abs_tol: 1e-6,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    // 5 closed rectangular sections at x=0, 7.5, 15, 22.5, 30 (same as showcase)
    let defs: [(f64, f64, f64); 5] = [
        (0.0, 1.5, 0.4),
        (7.5, 2.0, 0.5),
        (15.0, 2.5, 0.6),
        (22.5, 2.0, 0.5),
        (30.0, 1.5, 0.4),
    ];

    let mut handles = Vec::new();
    for &(x, hw, hh) in &defs {
        let pts = [
            RgmPoint3 { x, y: -hw, z: -hh },
            RgmPoint3 { x, y: hw, z: -hh },
            RgmPoint3 { x, y: hw, z: hh },
            RgmPoint3 { x, y: -hw, z: hh },
        ];
        let mut h = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_polyline(session, pts.as_ptr(), 4, true, tol, &mut h),
            RgmStatus::Ok
        );
        handles.push(h);
    }

    let mut out = RgmObjectHandle(0);
    assert_eq!(
        rgm_loft(session, handles.as_ptr(), 5, 12, false, &mut out),
        RgmStatus::Ok
    );

    // Check several points on each section. U params: 0.0, chord-based, ..., 1.0
    // For 5 sections with chord-length parameterization, section i should be at
    // the u parameter that maps to it. The first section is at u=0, last at u=1.
    let u_values = [0.0_f64, 1.0_f64];
    let section_indices = [0_usize, 4_usize];
    let v_values = [0.0_f64, 0.125, 0.25, 0.5, 0.75];

    for (&u_val, &sec_idx) in u_values.iter().zip(section_indices.iter()) {
        for &v_val in &v_values {
            let mut curve_pt = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
            assert_eq!(
                rgm_curve_point_at(session, handles[sec_idx], v_val, &mut curve_pt),
                RgmStatus::Ok
            );

            let uv = RgmUv2 { u: u_val, v: v_val };
            let mut surf_pt = RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 };
            assert_eq!(
                rgm_surface_point_at(session, out, &uv, &mut surf_pt),
                RgmStatus::Ok
            );

            let err = ((surf_pt.x - curve_pt.x).powi(2)
                + (surf_pt.y - curve_pt.y).powi(2)
                + (surf_pt.z - curve_pt.z).powi(2)).sqrt();
            assert!(
                err < 1e-6,
                "Section {sec_idx} at u={u_val}, v={v_val}: err = {err}, \
                 expected ({},{},{}), got ({},{},{})",
                curve_pt.x, curve_pt.y, curve_pt.z,
                surf_pt.x, surf_pt.y, surf_pt.z
            );
        }
    }

    // Also verify the surface tessellates
    let mut mesh = RgmObjectHandle(0);
    assert_eq!(
        rgm_surface_tessellate_to_mesh(session, out, ptr::null(), &mut mesh),
        RgmStatus::Ok
    );
    let mut tri_count = 0u32;
    assert_eq!(
        rgm_mesh_triangle_count(session, mesh, &mut tri_count as *mut _),
        RgmStatus::Ok
    );
    assert!(tri_count > 0, "Compatible-CP loft should tessellate");

    rgm_kernel_destroy(session);
}

// ── Loft type tests ──────────────────────────────────────────────────────────

fn create_loft_test_sections(session: RgmKernelHandle, closed: bool) -> Vec<RgmObjectHandle> {
    let tol = RgmToleranceContext {
        abs_tol: 1e-6,
        rel_tol: 1e-4,
        angle_tol: 1e-6,
    };

    let defs: [(f64, f64, f64); 3] = [
        (0.0, 1.0, 0.5),
        (5.0, 2.0, 1.0),
        (10.0, 1.5, 0.6),
    ];

    let mut handles = Vec::new();
    for &(x, hw, hh) in &defs {
        let pts = [
            RgmPoint3 { x, y: -hw, z: -hh },
            RgmPoint3 { x, y: hw, z: -hh },
            RgmPoint3 { x, y: hw, z: hh },
            RgmPoint3 { x, y: -hw, z: hh },
        ];
        let mut h = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_polyline(session, pts.as_ptr(), 4, closed, tol, &mut h),
            RgmStatus::Ok
        );
        handles.push(h);
    }
    handles
}

#[test]
fn test_loft_type_straight() {
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let handles = create_loft_test_sections(session, true);
    let mut out = RgmObjectHandle(0);
    assert_eq!(
        rgm_loft_typed(session, handles.as_ptr(), 3, 12, true, 3, &mut out),
        RgmStatus::Ok
    );
    assert_ne!(out.0, 0);

    let mut mesh = RgmObjectHandle(0);
    assert_eq!(
        rgm_brep_tessellate_to_mesh(session, out, ptr::null(), &mut mesh),
        RgmStatus::Ok
    );
    let mut tri_count = 0u32;
    assert_eq!(
        rgm_mesh_triangle_count(session, mesh, &mut tri_count as *mut _),
        RgmStatus::Ok
    );
    assert!(tri_count > 0, "Straight loft should tessellate");

    rgm_kernel_destroy(session);
}

#[test]
fn test_loft_type_loose() {
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let handles = create_loft_test_sections(session, true);
    let mut out = RgmObjectHandle(0);
    assert_eq!(
        rgm_loft_typed(session, handles.as_ptr(), 3, 12, true, 1, &mut out),
        RgmStatus::Ok
    );
    assert_ne!(out.0, 0);

    let mut mesh = RgmObjectHandle(0);
    assert_eq!(
        rgm_brep_tessellate_to_mesh(session, out, ptr::null(), &mut mesh),
        RgmStatus::Ok
    );
    let mut tri_count = 0u32;
    assert_eq!(
        rgm_mesh_triangle_count(session, mesh, &mut tri_count as *mut _),
        RgmStatus::Ok
    );
    assert!(tri_count > 0, "Loose loft should tessellate");

    rgm_kernel_destroy(session);
}

#[test]
fn test_loft_type_tight() {
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let handles = create_loft_test_sections(session, true);
    let mut out = RgmObjectHandle(0);
    assert_eq!(
        rgm_loft_typed(session, handles.as_ptr(), 3, 12, true, 2, &mut out),
        RgmStatus::Ok
    );
    assert_ne!(out.0, 0);

    let mut mesh = RgmObjectHandle(0);
    assert_eq!(
        rgm_brep_tessellate_to_mesh(session, out, ptr::null(), &mut mesh),
        RgmStatus::Ok
    );
    let mut tri_count = 0u32;
    assert_eq!(
        rgm_mesh_triangle_count(session, mesh, &mut tri_count as *mut _),
        RgmStatus::Ok
    );
    assert!(tri_count > 0, "Tight loft should tessellate");

    rgm_kernel_destroy(session);
}

#[test]
fn test_loft_type_invalid() {
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session), RgmStatus::Ok);

    let handles = create_loft_test_sections(session, true);
    let mut out = RgmObjectHandle(0);
    let status = rgm_loft_typed(session, handles.as_ptr(), 3, 12, true, 99, &mut out);
    assert_eq!(status, RgmStatus::InvalidInput);

    rgm_kernel_destroy(session);
}
