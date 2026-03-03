mod common;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusted_geom::{
    rgm_brep_add_face_from_surface, rgm_brep_add_loop_uv, rgm_brep_create_empty,
    rgm_brep_load_native, rgm_brep_save_native, rgm_brep_tessellate_to_mesh, rgm_brep_validate,
    rgm_object_release, RgmBrepValidationReport, RgmObjectHandle, RgmStatus, RgmUv2,
};
use std::mem::MaybeUninit;

fn make_brep(session: rusted_geom::RgmKernelHandle, face_count: usize) -> RgmObjectHandle {
    let mut brep = RgmObjectHandle(0);
    assert_eq!(
        rgm_brep_create_empty(session, &mut brep as *mut _),
        RgmStatus::Ok
    );

    for i in 0..face_count {
        let surface = common::create_warped_surface(
            session,
            4,
            4,
            2.0,
            2.0,
            0.1 + (i as f64 * 0.01),
        );
        let mut face_id = 0_u32;
        assert_eq!(
            rgm_brep_add_face_from_surface(session, brep, surface, &mut face_id as *mut _),
            RgmStatus::Ok
        );

        let outer = [
            RgmUv2 { u: 0.0, v: 0.0 },
            RgmUv2 { u: 1.0, v: 0.0 },
            RgmUv2 { u: 1.0, v: 1.0 },
            RgmUv2 { u: 0.0, v: 1.0 },
        ];
        let mut loop_id = 0_u32;
        assert_eq!(
            rgm_brep_add_loop_uv(
                session,
                brep,
                face_id,
                outer.as_ptr(),
                outer.len(),
                true,
                &mut loop_id as *mut _,
            ),
            RgmStatus::Ok
        );

        assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
    }
    brep
}

fn bench_brep_build_from_40_trimmed_faces(c: &mut Criterion) {
    let session = common::create_session();
    c.bench_function("brep_build_from_40_trimmed_faces", |b| {
        b.iter(|| {
            let brep = make_brep(session, 40);
            black_box(brep);
            common::release_object(session, brep);
        })
    });
    common::destroy_session(session);
}

fn bench_brep_validate_40_face_shell(c: &mut Criterion) {
    let session = common::create_session();
    let brep = make_brep(session, 40);
    c.bench_function("brep_validate_40_face_shell", |b| {
        b.iter(|| {
            let mut report = MaybeUninit::<RgmBrepValidationReport>::uninit();
            let status = rgm_brep_validate(session, brep, report.as_mut_ptr());
            black_box(status);
        })
    });
    common::release_object(session, brep);
    common::destroy_session(session);
}

fn bench_brep_tessellate_40_face_shell(c: &mut Criterion) {
    let session = common::create_session();
    let brep = make_brep(session, 40);
    c.bench_function("brep_tessellate_40_face_shell", |b| {
        b.iter(|| {
            let mut mesh = RgmObjectHandle(0);
            let status = rgm_brep_tessellate_to_mesh(
                session,
                brep,
                std::ptr::null(),
                &mut mesh as *mut _,
            );
            black_box(status);
            if status == RgmStatus::Ok {
                common::release_object(session, mesh);
            }
        })
    });
    common::release_object(session, brep);
    common::destroy_session(session);
}

fn bench_brep_native_roundtrip_40_face_shell(c: &mut Criterion) {
    let session = common::create_session();
    let brep = make_brep(session, 40);

    c.bench_function("brep_native_roundtrip_40_face_shell", |b| {
        b.iter(|| {
            let mut bytes_needed = 0_u32;
            let status_count =
                rgm_brep_save_native(session, brep, std::ptr::null_mut(), 0, &mut bytes_needed);
            black_box(status_count);
            if status_count != RgmStatus::Ok || bytes_needed == 0 {
                return;
            }

            let mut bytes = vec![0_u8; bytes_needed as usize];
            let mut written = 0_u32;
            let status_save = rgm_brep_save_native(
                session,
                brep,
                bytes.as_mut_ptr(),
                bytes.len() as u32,
                &mut written,
            );
            black_box(status_save);
            if status_save != RgmStatus::Ok {
                return;
            }

            let mut loaded = RgmObjectHandle(0);
            let status_load =
                rgm_brep_load_native(session, bytes.as_ptr(), written as usize, &mut loaded);
            black_box(status_load);
            if status_load == RgmStatus::Ok {
                common::release_object(session, loaded);
            }
        })
    });

    common::release_object(session, brep);
    common::destroy_session(session);
}

criterion_group!(
    brep_benches,
    bench_brep_build_from_40_trimmed_faces,
    bench_brep_validate_40_face_shell,
    bench_brep_tessellate_40_face_shell,
    bench_brep_native_roundtrip_40_face_shell,
);
criterion_main!(brep_benches);
