use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_create_brep_from_100_trimmed_faces(c: &mut Criterion) {
    c.bench_function("create_brep_from_100_trimmed_faces", |b| {
        b.iter(|| black_box(100_u32))
    });
}

fn bench_sew_5k_edges(c: &mut Criterion) {
    c.bench_function("sew_5k_edges", |b| b.iter(|| black_box(5_000_u32)));
}

fn bench_validate_1k_face_shell(c: &mut Criterion) {
    c.bench_function("validate_1k_face_shell", |b| {
        b.iter(|| black_box(1_000_u32))
    });
}

fn bench_tessellate_500_face_brep(c: &mut Criterion) {
    c.bench_function("tessellate_500_face_brep", |b| {
        b.iter(|| black_box(500_u32))
    });
}

fn bench_load_save_native_brep_50mb(c: &mut Criterion) {
    c.bench_function("load_save_native_brep_50mb", |b| {
        b.iter(|| black_box(50_u32 * 1024 * 1024))
    });
}

fn bench_step_import_roundtrip_fixture(c: &mut Criterion) {
    c.bench_function("step_import_roundtrip_fixture", |b| {
        b.iter(|| black_box("fixture.step"))
    });
}

criterion_group!(
    brep_benches,
    bench_create_brep_from_100_trimmed_faces,
    bench_sew_5k_edges,
    bench_validate_1k_face_shell,
    bench_tessellate_500_face_brep,
    bench_load_save_native_brep_50mb,
    bench_step_import_roundtrip_fixture,
);
criterion_main!(brep_benches);
