mod common;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusted_geom::{
    rgm_intersect_curve_plane, rgm_intersect_surface_surface, rgm_intersection_branch_count,
    rgm_object_release, RgmObjectHandle, RgmPlane, RgmPoint3, RgmStatus, RgmVec3,
};

fn bench_intersect_curve_plane_polyline(c: &mut Criterion) {
    let session = common::create_session();
    let curve = common::create_polyline_curve(session, 1000);
    let plane = RgmPlane {
        origin: RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        x_axis: RgmVec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        y_axis: RgmVec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        z_axis: RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
    };

    c.bench_function("intersect_curve_plane_polyline_1k", |b| {
        b.iter(|| {
            let mut count = 0_u32;
            let status = rgm_intersect_curve_plane(
                session,
                curve,
                &plane as *const _,
                std::ptr::null_mut(),
                0,
                &mut count as *mut _,
            );
            black_box((status, count));
        })
    });

    assert_eq!(rgm_object_release(session, curve), RgmStatus::Ok);
    common::destroy_session(session);
}

fn bench_intersect_surface_surface_and_count_branches(c: &mut Criterion) {
    let session = common::create_session();
    let surface_a = common::create_warped_surface(session, 10, 10, 6.0, 6.0, 0.4);
    let surface_b = common::create_warped_surface(session, 10, 10, 6.0, 6.0, 0.45);

    c.bench_function("intersect_surface_surface_and_count_branches", |b| {
        b.iter(|| {
            let mut intersection = RgmObjectHandle(0);
            let status_intersection =
                rgm_intersect_surface_surface(session, surface_a, surface_b, &mut intersection);
            if status_intersection != RgmStatus::Ok {
                black_box(status_intersection);
                return;
            }
            let mut branch_count = 0_u32;
            let status_count =
                rgm_intersection_branch_count(session, intersection, &mut branch_count as *mut _);
            black_box((status_intersection, status_count, branch_count));
            common::release_object(session, intersection);
        })
    });

    assert_eq!(rgm_object_release(session, surface_b), RgmStatus::Ok);
    assert_eq!(rgm_object_release(session, surface_a), RgmStatus::Ok);
    common::destroy_session(session);
}

criterion_group!(
    intersection_benches,
    bench_intersect_curve_plane_polyline,
    bench_intersect_surface_surface_and_count_branches,
);
criterion_main!(intersection_benches);
