#![allow(dead_code)]

use rusted_geom::{
    rgm_curve_create_polyline, rgm_kernel_create, rgm_kernel_destroy, rgm_mesh_create_torus,
    rgm_object_release, rgm_surface_create_nurbs, RgmKernelHandle, RgmNurbsSurfaceDesc,
    RgmObjectHandle, RgmPoint3, RgmStatus, RgmToleranceContext,
};

pub fn tol() -> RgmToleranceContext {
    RgmToleranceContext {
        abs_tol: 1e-9,
        rel_tol: 1e-9,
        angle_tol: 1e-9,
    }
}

pub fn create_session() -> RgmKernelHandle {
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session as *mut _), RgmStatus::Ok);
    session
}

pub fn destroy_session(session: RgmKernelHandle) {
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

pub fn release_object(session: RgmKernelHandle, object: RgmObjectHandle) {
    assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
}

pub fn clamped_uniform_knots(control_count: usize, degree: usize) -> Vec<f64> {
    let knot_count = control_count + degree + 1;
    let mut knots = vec![0.0; knot_count];
    let interior = control_count.saturating_sub(degree + 1);
    for i in 0..=degree {
        knots[i] = 0.0;
        knots[knot_count - 1 - i] = 1.0;
    }
    for i in 1..=interior {
        knots[degree + i] = i as f64 / (interior + 1) as f64;
    }
    knots
}

pub fn create_polyline_curve(session: RgmKernelHandle, count: usize) -> RgmObjectHandle {
    let mut points = Vec::with_capacity(count);
    let denom = count.saturating_sub(1).max(1) as f64;
    for i in 0..count {
        let t = i as f64 / denom;
        points.push(RgmPoint3 {
            x: t * 12.0 - 6.0,
            y: (t * std::f64::consts::PI * 8.0).sin() * 2.1,
            z: (t * std::f64::consts::PI * 6.0).cos() * 0.7,
        });
    }
    let mut curve = RgmObjectHandle(0);
    assert_eq!(
        rgm_curve_create_polyline(
            session,
            points.as_ptr(),
            points.len(),
            false,
            &tol() as *const _,
            &mut curve as *mut _,
        ),
        RgmStatus::Ok
    );
    curve
}

pub fn create_warped_surface(
    session: RgmKernelHandle,
    u_count: usize,
    v_count: usize,
    span_u: f64,
    span_v: f64,
    warp_scale: f64,
) -> RgmObjectHandle {
    let mut points = Vec::with_capacity(u_count * v_count);
    let mut weights = Vec::with_capacity(u_count * v_count);
    let half_u = span_u * 0.5;
    let half_v = span_v * 0.5;
    for iu in 0..u_count {
        let u = iu as f64 / (u_count.saturating_sub(1).max(1)) as f64;
        let x = -half_u + u * span_u;
        for iv in 0..v_count {
            let v = iv as f64 / (v_count.saturating_sub(1).max(1)) as f64;
            let y = -half_v + v * span_v;
            let z = ((u * 2.0 + v * 1.2) * std::f64::consts::PI).sin() * warp_scale
                + ((u * 0.8 - v * 1.6) * std::f64::consts::PI).cos() * (warp_scale * 0.6);
            points.push(RgmPoint3 { x, y, z });
            weights.push(1.0 + 0.08 * ((u + v) * std::f64::consts::PI).sin());
        }
    }

    let desc = RgmNurbsSurfaceDesc {
        degree_u: 3,
        degree_v: 3,
        periodic_u: false,
        periodic_v: false,
        control_u_count: u_count as u32,
        control_v_count: v_count as u32,
    };
    let knots_u = clamped_uniform_knots(u_count, 3);
    let knots_v = clamped_uniform_knots(v_count, 3);
    let mut surface = RgmObjectHandle(0);
    assert_eq!(
        rgm_surface_create_nurbs(
            session,
            &desc as *const _,
            points.as_ptr(),
            points.len(),
            weights.as_ptr(),
            weights.len(),
            knots_u.as_ptr(),
            knots_u.len(),
            knots_v.as_ptr(),
            knots_v.len(),
            &tol() as *const _,
            &mut surface as *mut _,
        ),
        RgmStatus::Ok
    );
    surface
}

pub fn create_torus_mesh(session: RgmKernelHandle) -> RgmObjectHandle {
    let mut mesh = RgmObjectHandle(0);
    assert_eq!(
        rgm_mesh_create_torus(
            session,
            &RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            } as *const _,
            4.2,
            1.2,
            80,
            56,
            &mut mesh as *mut _,
        ),
        RgmStatus::Ok
    );
    mesh
}
