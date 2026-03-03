mod common;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusted_geom::{
    rgm_face_add_loop, rgm_face_create_from_surface, rgm_face_tessellate_to_mesh,
    rgm_face_validate, rgm_object_release, RgmObjectHandle, RgmStatus, RgmUv2,
};

fn create_trimmed_face(session: rusted_geom::RgmKernelHandle) -> (RgmObjectHandle, RgmObjectHandle) {
    let surface = common::create_warped_surface(session, 6, 6, 3.0, 3.0, 0.2);
    let mut face = RgmObjectHandle(0);
    assert_eq!(
        rgm_face_create_from_surface(session, surface, &mut face as *mut _),
        RgmStatus::Ok
    );

    let outer = [
        RgmUv2 { u: 0.0, v: 0.0 },
        RgmUv2 { u: 1.0, v: 0.0 },
        RgmUv2 { u: 1.0, v: 1.0 },
        RgmUv2 { u: 0.0, v: 1.0 },
    ];
    assert_eq!(
        rgm_face_add_loop(session, face, outer.as_ptr(), outer.len(), true),
        RgmStatus::Ok
    );

    let hole = [
        RgmUv2 { u: 0.3, v: 0.3 },
        RgmUv2 { u: 0.7, v: 0.3 },
        RgmUv2 { u: 0.7, v: 0.7 },
        RgmUv2 { u: 0.3, v: 0.7 },
    ];
    assert_eq!(
        rgm_face_add_loop(session, face, hole.as_ptr(), hole.len(), false),
        RgmStatus::Ok
    );
    (face, surface)
}

fn bench_face_validate_trimmed(c: &mut Criterion) {
    let session = common::create_session();
    let (face, surface) = create_trimmed_face(session);

    c.bench_function("face_validate_trimmed_with_hole", |b| {
        b.iter(|| {
            let mut valid = false;
            let status = rgm_face_validate(session, face, &mut valid as *mut _);
            black_box((status, valid));
        })
    });

    assert_eq!(rgm_object_release(session, face), RgmStatus::Ok);
    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    common::destroy_session(session);
}

fn bench_face_tessellate_trimmed(c: &mut Criterion) {
    let session = common::create_session();
    let (face, surface) = create_trimmed_face(session);

    c.bench_function("face_tessellate_trimmed_with_hole", |b| {
        b.iter(|| {
            let mut mesh = RgmObjectHandle(0);
            let status = rgm_face_tessellate_to_mesh(
                session,
                face,
                std::ptr::null(),
                &mut mesh as *mut _,
            );
            black_box(status);
            if status == RgmStatus::Ok {
                common::release_object(session, mesh);
            }
        })
    });

    assert_eq!(rgm_object_release(session, face), RgmStatus::Ok);
    assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    common::destroy_session(session);
}

criterion_group!(
    face_benches,
    bench_face_validate_trimmed,
    bench_face_tessellate_trimmed,
);
criterion_main!(face_benches);
