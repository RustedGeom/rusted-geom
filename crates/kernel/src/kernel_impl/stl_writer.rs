// ─── STL Writer (ASCII) ─────────────────────────────────────────────────────
//
// Produces an ASCII STL file from mesh objects in the session.
// Mesh triangles are emitted with per-face normals computed from the
// cross product of the first two edges.
//
// This file is `include!`-ed from ffi_impl.rs.

pub(crate) fn export_stl_text(
    session: RgmKernelHandle,
    object_ids: &[u64],
) -> Result<String, String> {
    let entry = SESSIONS
        .get(&session.0)
        .ok_or_else(|| "Session not found".to_string())?;
    let state = entry.value().read();

    let mut out = String::with_capacity(4096);
    out.push_str("solid rusted-geom\n");

    for &obj_id in object_ids {
        let obj = state
            .objects
            .get(&obj_id)
            .ok_or_else(|| format!("Object {obj_id} not found"))?;

        let meshes: Vec<&MeshData> = match obj {
            GeometryObject::Mesh(m) => vec![m],
            GeometryObject::Brep(brep) => {
                let mut ms = Vec::new();
                for face in brep.faces.iter() {
                    if let Some(GeometryObject::Mesh(m)) = state.objects.get(&face.surface.0) {
                        ms.push(m);
                    }
                }
                ms
            }
            _ => continue,
        };

        for mesh in meshes {
            write_mesh_stl(&mut out, mesh);
        }
    }

    out.push_str("endsolid rusted-geom\n");
    Ok(out)
}

fn write_mesh_stl(out: &mut String, mesh: &MeshData) {
    use std::fmt::Write;

    for tri in &mesh.triangles {
        let v0 = matrix_apply_point(mesh.transform, mesh.vertices[tri[0] as usize]);
        let v1 = matrix_apply_point(mesh.transform, mesh.vertices[tri[1] as usize]);
        let v2 = matrix_apply_point(mesh.transform, mesh.vertices[tri[2] as usize]);

        let e1x = v1.x - v0.x;
        let e1y = v1.y - v0.y;
        let e1z = v1.z - v0.z;
        let e2x = v2.x - v0.x;
        let e2y = v2.y - v0.y;
        let e2z = v2.z - v0.z;
        let nx = e1y * e2z - e1z * e2y;
        let ny = e1z * e2x - e1x * e2z;
        let nz = e1x * e2y - e1y * e2x;
        let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-30);

        let _ = writeln!(out, "  facet normal {:.6e} {:.6e} {:.6e}", nx / len, ny / len, nz / len);
        out.push_str("    outer loop\n");
        let _ = writeln!(out, "      vertex {:.6e} {:.6e} {:.6e}", v0.x, v0.y, v0.z);
        let _ = writeln!(out, "      vertex {:.6e} {:.6e} {:.6e}", v1.x, v1.y, v1.z);
        let _ = writeln!(out, "      vertex {:.6e} {:.6e} {:.6e}", v2.x, v2.y, v2.z);
        out.push_str("    endloop\n");
        out.push_str("  endfacet\n");
    }
}
