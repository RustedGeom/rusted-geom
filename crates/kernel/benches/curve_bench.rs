mod common;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusted_geom::{
    rgm_curve_d1_at, rgm_curve_length, rgm_curve_point_at, rgm_nurbs_interpolate_fit_points,
    rgm_object_release, RgmObjectHandle, RgmPoint3, RgmStatus, RgmVec3,
};

fn bench_curve_point_and_d1_at(c: &mut Criterion) {
    let session = common::create_session();
    let curve = common::create_polyline_curve(session, 1000);
    let mut point = RgmPoint3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let mut d1 = RgmVec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    c.bench_function("curve_point_and_d1_at_polyline_1k", |b| {
        b.iter(|| {
            let s0 = rgm_curve_point_at(session, curve, 0.37, &mut point as *mut _);
            let s1 = rgm_curve_d1_at(session, curve, 0.37, &mut d1 as *mut _);
            black_box((s0, s1, point, d1));
        })
    });

    assert_eq!(rgm_object_release(session, curve), RgmStatus::Ok);
    common::destroy_session(session);
}

fn bench_curve_length_polyline_1k(c: &mut Criterion) {
    let session = common::create_session();
    let curve = common::create_polyline_curve(session, 1000);
    let mut length = 0.0_f64;

    c.bench_function("curve_length_polyline_1k", |b| {
        b.iter(|| {
            let status = rgm_curve_length(session, curve, &mut length as *mut _);
            black_box((status, length));
        })
    });

    assert_eq!(rgm_object_release(session, curve), RgmStatus::Ok);
    common::destroy_session(session);
}

fn bench_curve_fit_nurbs_from_points(c: &mut Criterion) {
    let session = common::create_session();
    let mut points = Vec::with_capacity(200);
    for i in 0..200 {
        let t = i as f64 / 199.0;
        points.push(RgmPoint3 {
            x: t * 10.0,
            y: (t * std::f64::consts::PI * 6.0).sin() * 1.5,
            z: (t * std::f64::consts::PI * 4.0).cos() * 0.5,
        });
    }

    c.bench_function("curve_fit_nurbs_from_200_points", |b| {
        b.iter(|| {
            let mut curve = RgmObjectHandle(0);
            let status = rgm_nurbs_interpolate_fit_points(
                session,
                points.as_ptr(),
                points.len(),
                3,
                false,
                &common::tol() as *const _,
                &mut curve as *mut _,
            );
            black_box(status);
            if status == RgmStatus::Ok {
                common::release_object(session, curve);
            }
        })
    });

    common::destroy_session(session);
}

criterion_group!(
    curve_benches,
    bench_curve_point_and_d1_at,
    bench_curve_length_polyline_1k,
    bench_curve_fit_nurbs_from_points,
);
criterion_main!(curve_benches);
