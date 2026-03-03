use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusted_geom::wasm::KernelSession;

const C3D_XML: &str = include_str!("../../../docs/landxml-test-files/C3DDesignExample.xml");
const OPENROAD_XML: &str = include_str!("../../../docs/landxml-test-files/OpenRoadTin.xml");

fn bench_landxml_parse_strict(c: &mut Criterion) {
    c.bench_function("landxml_parse_strict_c3d_design", |b| {
        b.iter(|| {
            let session = KernelSession::new().expect("session");
            let doc = session
                .landxml_parse(C3D_XML, 0, 0, 0)
                .expect("landxml parse strict");
            let surface_count = session.landxml_surface_count(&doc).expect("surface count");
            black_box(surface_count);
        })
    });
}

fn bench_landxml_parse_lenient(c: &mut Criterion) {
    c.bench_function("landxml_parse_lenient_openroad_tin", |b| {
        b.iter(|| {
            let session = KernelSession::new().expect("session");
            let doc = session
                .landxml_parse(OPENROAD_XML, 1, 0, 0)
                .expect("landxml parse lenient");
            let alignment_count = session.landxml_alignment_count(&doc).expect("alignment count");
            black_box(alignment_count);
        })
    });
}

fn bench_landxml_sample_alignment(c: &mut Criterion) {
    let session = KernelSession::new().expect("session");
    let doc = session
        .landxml_parse(C3D_XML, 1, 0, 0)
        .expect("landxml parse");

    c.bench_function("landxml_sample_horiz_2d_segments", |b| {
        b.iter(|| {
            let packed = session
                .landxml_sample_horiz_2d_segments(&doc, 0)
                .expect("sample horiz");
            black_box(packed.len());
        })
    });
}

fn bench_landxml_extract_surface_mesh(c: &mut Criterion) {
    let session = KernelSession::new().expect("session");
    let doc = session
        .landxml_parse(OPENROAD_XML, 1, 0, 0)
        .expect("landxml parse");

    c.bench_function("landxml_extract_surface_mesh_and_count_triangles", |b| {
        b.iter(|| {
            let mesh = session
                .landxml_extract_surface_mesh(&doc, 0)
                .expect("extract mesh");
            let tri_count = session.mesh_triangle_count(&mesh).expect("triangle count");
            black_box(tri_count);
        })
    });
}

criterion_group!(
    landxml_benches,
    bench_landxml_parse_strict,
    bench_landxml_parse_lenient,
    bench_landxml_sample_alignment,
    bench_landxml_extract_surface_mesh,
);
criterion_main!(landxml_benches);
