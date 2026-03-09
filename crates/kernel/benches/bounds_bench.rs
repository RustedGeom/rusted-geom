use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusted_geom::{
    rgm_curve_create_polyline, rgm_kernel_create, rgm_kernel_destroy, rgm_mesh_create_torus,
    rgm_object_compute_bounds, rgm_object_release, rgm_surface_create_nurbs, RgmBounds3,
    RgmBoundsMode, RgmBoundsOptions, RgmKernelHandle, RgmNurbsSurfaceDesc, RgmObjectHandle,
    RgmPoint3, RgmStatus, RgmToleranceContext,
};

fn tol() -> RgmToleranceContext {
    RgmToleranceContext {
        abs_tol: 1e-9,
        rel_tol: 1e-9,
        angle_tol: 1e-9,
    }
}

fn create_session() -> RgmKernelHandle {
    let mut session = RgmKernelHandle(0);
    assert_eq!(rgm_kernel_create(&mut session as *mut _), RgmStatus::Ok);
    session
}

fn destroy_session(session: RgmKernelHandle) {
    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
}

fn clamped_uniform_knots(control_count: usize, degree: usize) -> Vec<f64> {
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

fn create_warped_surface(
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

fn curve_fixture(session: RgmKernelHandle) -> RgmObjectHandle {
    let points = [
        RgmPoint3 {
            x: -6.0,
            y: -2.0,
            z: 1.5,
        },
        RgmPoint3 {
            x: -3.0,
            y: 2.5,
            z: -1.0,
        },
        RgmPoint3 {
            x: -1.0,
            y: 1.0,
            z: 2.0,
        },
        RgmPoint3 {
            x: 2.5,
            y: -2.0,
            z: -1.2,
        },
        RgmPoint3 {
            x: 5.0,
            y: 3.0,
            z: 1.6,
        },
    ];
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

fn mesh_fixture(session: RgmKernelHandle) -> RgmObjectHandle {
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

fn bounds_options(mode: RgmBoundsMode, sample_budget: u32) -> RgmBoundsOptions {
    RgmBoundsOptions {
        mode,
        sample_budget,
        padding: 0.0,
    }
}

fn zero_bounds() -> RgmBounds3 {
    RgmBounds3 {
        world_aabb: rusted_geom::RgmAabb3 {
            min: RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            max: RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
        world_obb: rusted_geom::RgmObb3 {
            center: RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            x_axis: rusted_geom::RgmVec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            y_axis: rusted_geom::RgmVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            z_axis: rusted_geom::RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            half_extents: rusted_geom::RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
        local_aabb: rusted_geom::RgmAabb3 {
            min: RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            max: RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
    }
}

fn bench_curve_bounds_fast(c: &mut Criterion) {
    let session = create_session();
    let curve = curve_fixture(session);
    let options = bounds_options(RgmBoundsMode::Fast, 0);
    let mut out = zero_bounds();

    c.bench_function("curve_bounds_fast", |b| {
        b.iter(|| {
            let status =
                rgm_object_compute_bounds(session, curve, &options as *const _, &mut out as *mut _);
            black_box(status);
            black_box(out);
        })
    });

    assert_eq!(rgm_object_release(session, curve), RgmStatus::Ok);
    destroy_session(session);
}

fn bench_surface_bounds_fast(c: &mut Criterion) {
    let session = create_session();
    let surface = create_warped_surface(session, 9, 8, 8.0, 7.0, 0.75);
    let options = bounds_options(RgmBoundsMode::Fast, 0);
    let mut out = zero_bounds();

    c.bench_function("surface_bounds_fast", |b| {
        b.iter(|| {
            let status = rgm_object_compute_bounds(
                session,
                surface,
                &options as *const _,
                &mut out as *mut _,
            );
            black_box(status);
            black_box(out);
        })
    });

    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    destroy_session(session);
}

fn bench_mesh_bounds_fast_cached(c: &mut Criterion) {
    let session = create_session();
    let mesh = mesh_fixture(session);
    let options = bounds_options(RgmBoundsMode::Fast, 1024);
    let mut out = zero_bounds();
    assert_eq!(
        rgm_object_compute_bounds(session, mesh, &options as *const _, &mut out as *mut _),
        RgmStatus::Ok
    );

    c.bench_function("mesh_bounds_fast_cached", |b| {
        b.iter(|| {
            let status =
                rgm_object_compute_bounds(session, mesh, &options as *const _, &mut out as *mut _);
            black_box(status);
            black_box(out);
        })
    });

    assert_eq!(rgm_object_release(session, mesh), RgmStatus::Ok);
    destroy_session(session);
}

fn bench_obb_optimal_refine(c: &mut Criterion) {
    let session = create_session();
    let mesh = mesh_fixture(session);
    let options = bounds_options(RgmBoundsMode::Optimal, 8192);
    let mut out = zero_bounds();

    c.bench_function("obb_optimal_refine", |b| {
        b.iter(|| {
            let status =
                rgm_object_compute_bounds(session, mesh, &options as *const _, &mut out as *mut _);
            black_box(status);
            black_box(out);
        })
    });

    assert_eq!(rgm_object_release(session, mesh), RgmStatus::Ok);
    destroy_session(session);
}

criterion_group!(
    bounds_benches,
    bench_curve_bounds_fast,
    bench_surface_bounds_fast,
    bench_mesh_bounds_fast_cached,
    bench_obb_optimal_refine,
);
criterion_main!(bounds_benches);
