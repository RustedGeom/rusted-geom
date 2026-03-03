mod common;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusted_geom::{
    rgm_intersect_mesh_mesh, rgm_intersect_mesh_plane, rgm_mesh_boolean, rgm_mesh_copy_indices,
    rgm_mesh_copy_vertices, rgm_mesh_translate, RgmObjectHandle, RgmPlane, RgmPoint3, RgmStatus,
    RgmVec3,
};

fn bench_mesh_intersect_plane_torus(c: &mut Criterion) {
    let session = common::create_session();
    let mesh = common::create_torus_mesh(session);
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

    c.bench_function("mesh_intersect_plane_torus", |b| {
        b.iter(|| {
            let mut count = 0_u32;
            let s0 = rgm_intersect_mesh_plane(
                session,
                mesh,
                &plane as *const _,
                std::ptr::null_mut(),
                0,
                &mut count as *mut _,
            );
            black_box((s0, count));
        })
    });

    common::release_object(session, mesh);
    common::destroy_session(session);
}

fn bench_mesh_intersect_mesh_torus_pair(c: &mut Criterion) {
    let session = common::create_session();
    let mesh_a = common::create_torus_mesh(session);
    let mut mesh_b = RgmObjectHandle(0);
    assert_eq!(
        rgm_mesh_translate(
            session,
            mesh_a,
            &RgmVec3 {
                x: 2.3,
                y: 0.2,
                z: 0.1,
            } as *const _,
            &mut mesh_b as *mut _,
        ),
        RgmStatus::Ok
    );

    c.bench_function("mesh_intersect_mesh_torus_pair", |b| {
        b.iter(|| {
            let mut count = 0_u32;
            let s0 = rgm_intersect_mesh_mesh(
                session,
                mesh_a,
                mesh_b,
                std::ptr::null_mut(),
                0,
                &mut count as *mut _,
            );
            black_box((s0, count));
        })
    });

    common::release_object(session, mesh_b);
    common::release_object(session, mesh_a);
    common::destroy_session(session);
}

fn bench_mesh_boolean_union_and_copy(c: &mut Criterion) {
    let session = common::create_session();
    let mesh_a = common::create_torus_mesh(session);
    let mut mesh_b = RgmObjectHandle(0);
    assert_eq!(
        rgm_mesh_translate(
            session,
            mesh_a,
            &RgmVec3 {
                x: 1.5,
                y: 0.0,
                z: 0.0,
            } as *const _,
            &mut mesh_b as *mut _,
        ),
        RgmStatus::Ok
    );

    c.bench_function("mesh_boolean_union_and_copy", |b| {
        b.iter(|| {
            let mut out_mesh = RgmObjectHandle(0);
            let s0 = rgm_mesh_boolean(session, mesh_a, mesh_b, 0, &mut out_mesh as *mut _);
            if s0 != RgmStatus::Ok {
                black_box(s0);
                return;
            }

            let mut vertex_count = 0_u32;
            let mut index_count = 0_u32;
            let mut vertices = vec![
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                };
                65536
            ];
            let mut indices = vec![0_u32; 196608];
            let s1 = rgm_mesh_copy_vertices(
                session,
                out_mesh,
                vertices.as_mut_ptr(),
                vertices.len() as u32,
                &mut vertex_count as *mut _,
            );
            let s2 = rgm_mesh_copy_indices(
                session,
                out_mesh,
                indices.as_mut_ptr(),
                indices.len() as u32,
                &mut index_count as *mut _,
            );
            black_box((s0, s1, s2, vertex_count, index_count));
            common::release_object(session, out_mesh);
        })
    });

    common::release_object(session, mesh_b);
    common::release_object(session, mesh_a);
    common::destroy_session(session);
}

criterion_group!(
    mesh_benches,
    bench_mesh_intersect_plane_torus,
    bench_mesh_intersect_mesh_torus_pair,
    bench_mesh_boolean_union_and_copy,
);
criterion_main!(mesh_benches);
