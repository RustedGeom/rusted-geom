// ─── Volume Operations ───────────────────────────────────────────────────────
//
// Mesh volume computation via the divergence theorem (signed tetrahedron sum).
// This file is `include!`-ed from kernel_impl.rs.

fn mesh_volume_compute(vertices: &[RgmPoint3], triangles: &[[u32; 3]]) -> f64 {
    let mut vol = 0.0_f64;
    for tri in triangles {
        let a = vertices[tri[0] as usize];
        let b = vertices[tri[1] as usize];
        let c = vertices[tri[2] as usize];
        vol += a.x * (b.y * c.z - b.z * c.y)
             + a.y * (b.z * c.x - b.x * c.z)
             + a.z * (b.x * c.y - b.y * c.x);
    }
    (vol / 6.0).abs()
}
