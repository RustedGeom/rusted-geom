mod common;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusted_geom::{
    rgm_object_release, rgm_surface_frame_at, rgm_surface_tessellate_to_mesh, RgmObjectHandle,
    RgmPoint3, RgmStatus, RgmSurfaceEvalFrame, RgmUv2, RgmVec3,
};

fn bench_surface_frame_eval(c: &mut Criterion) {
    let session = common::create_session();
    let surface = common::create_warped_surface(session, 16, 16, 8.0, 7.0, 0.8);
    let uv = RgmUv2 { u: 0.42, v: 0.63 };
    let mut frame = RgmSurfaceEvalFrame {
        point: RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        du: RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        dv: RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        normal: RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    };

    c.bench_function("surface_frame_eval_16x16_nurbs", |b| {
        b.iter(|| {
            let status = rgm_surface_frame_at(session, surface, &uv as *const _, &mut frame as *mut _);
            black_box((status, frame));
        })
    });

    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    common::destroy_session(session);
}

fn bench_surface_tessellate_medium(c: &mut Criterion) {
    let session = common::create_session();
    let surface = common::create_warped_surface(session, 16, 16, 8.0, 7.0, 0.8);

    c.bench_function("surface_tessellate_medium_nurbs", |b| {
        b.iter(|| {
            let mut mesh = RgmObjectHandle(0);
            let status = rgm_surface_tessellate_to_mesh(
                session,
                surface,
                std::ptr::null(),
                &mut mesh as *mut _,
            );
            black_box(status);
            if status == RgmStatus::Ok {
                common::release_object(session, mesh);
            }
        })
    });

    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    common::destroy_session(session);
}

criterion_group!(
    surface_benches,
    bench_surface_frame_eval,
    bench_surface_tessellate_medium,
);
criterion_main!(surface_benches);
