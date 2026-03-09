// ─── glTF 2.0 Writer ────────────────────────────────────────────────────────
//
// Produces a glTF 2.0 JSON file with an embedded base64-encoded binary buffer.
// Meshes are emitted as primitives with POSITION attributes and triangle indices.
//
// This file is `include!`-ed from ffi_impl.rs.

pub(crate) fn export_gltf_text(
    session: RgmKernelHandle,
    object_ids: &[u64],
) -> Result<String, String> {
    let entry = SESSIONS
        .get(&session.0)
        .ok_or_else(|| "Session not found".to_string())?;
    let state = entry.value().read();

    let mut owned_meshes: Vec<MeshData> = Vec::new();
    let stage_paths = collect_stage_subtree_paths(&state.stage, &collect_export_root_paths(&state, object_ids));

    for path in stage_paths {
        if let Some(mesh_prim) = state.stage.get::<rusted_usd::schema::generated::UsdGeomMesh>(&path) {
            let mut mesh = mesh_data_from_prim(mesh_prim);
            mesh.transform = world_transform_for_path(&state.stage, &path);
            owned_meshes.push(mesh);
        }
    }

    if owned_meshes.is_empty() {
        for &obj_id in object_ids {
            if let Some(GeometryObject::Mesh(m)) = state.objects.get(&obj_id) {
                owned_meshes.push(m.clone());
            }
        }
    }

    if owned_meshes.is_empty() {
        return Err("No mesh objects found for glTF export".to_string());
    }

    let all_meshes: Vec<&MeshData> = owned_meshes.iter().collect();

    let mut buffer_bytes: Vec<u8> = Vec::new();
    let mut buffer_views_json = Vec::new();
    let mut accessors_json = Vec::new();
    let mut primitives_json = Vec::new();

    let mut accessor_idx = 0usize;

    for mesh in &all_meshes {
        let n_verts = mesh.vertices.len();
        let n_tris = mesh.triangles.len();

        // Positions: f32 x 3 per vertex
        let pos_view_offset = buffer_bytes.len();
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut min_z = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        let mut max_z = f64::NEG_INFINITY;

        for v in &mesh.vertices {
            let p = matrix_apply_point(mesh.transform, *v);
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            min_z = min_z.min(p.z);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
            max_z = max_z.max(p.z);
            buffer_bytes.extend_from_slice(&(p.x as f32).to_le_bytes());
            buffer_bytes.extend_from_slice(&(p.y as f32).to_le_bytes());
            buffer_bytes.extend_from_slice(&(p.z as f32).to_le_bytes());
        }
        let pos_view_len = buffer_bytes.len() - pos_view_offset;

        // Pad to 4-byte boundary
        while buffer_bytes.len() % 4 != 0 {
            buffer_bytes.push(0);
        }

        // Indices: u32 x 3 per triangle
        let idx_view_offset = buffer_bytes.len();
        let n_indices = n_tris * 3;
        for tri in &mesh.triangles {
            buffer_bytes.extend_from_slice(&tri[0].to_le_bytes());
            buffer_bytes.extend_from_slice(&tri[1].to_le_bytes());
            buffer_bytes.extend_from_slice(&tri[2].to_le_bytes());
        }
        let idx_view_len = buffer_bytes.len() - idx_view_offset;

        while buffer_bytes.len() % 4 != 0 {
            buffer_bytes.push(0);
        }

        let pos_view_idx = buffer_views_json.len();
        buffer_views_json.push(format!(
            r#"{{"buffer":0,"byteOffset":{},"byteLength":{},"target":34962}}"#,
            pos_view_offset, pos_view_len,
        ));

        let idx_view_idx = buffer_views_json.len();
        buffer_views_json.push(format!(
            r#"{{"buffer":0,"byteOffset":{},"byteLength":{},"target":34963}}"#,
            idx_view_offset, idx_view_len,
        ));

        let pos_accessor = accessor_idx;
        accessors_json.push(format!(
            r#"{{"bufferView":{},"componentType":5126,"count":{},"type":"VEC3","min":[{:.8},{:.8},{:.8}],"max":[{:.8},{:.8},{:.8}]}}"#,
            pos_view_idx, n_verts, min_x, min_y, min_z, max_x, max_y, max_z,
        ));
        accessor_idx += 1;

        let idx_accessor = accessor_idx;
        accessors_json.push(format!(
            r#"{{"bufferView":{},"componentType":5125,"count":{},"type":"SCALAR"}}"#,
            idx_view_idx, n_indices,
        ));
        accessor_idx += 1;

        primitives_json.push(format!(
            r#"{{"attributes":{{"POSITION":{}}},"indices":{},"mode":4}}"#,
            pos_accessor, idx_accessor,
        ));
    }

    let b64 = base64_encode(&buffer_bytes);
    let data_uri = format!("data:application/octet-stream;base64,{b64}");

    let mut json = String::with_capacity(4096);
    json.push_str(r#"{"asset":{"version":"2.0","generator":"rusted-geom"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":[{"mesh":0}],"meshes":[{"primitives":["#);
    json.push_str(&primitives_json.join(","));
    json.push_str(r#"]}],"accessors":["#);
    json.push_str(&accessors_json.join(","));
    json.push_str(r#"],"bufferViews":["#);
    json.push_str(&buffer_views_json.join(","));
    json.push_str(r#"],"buffers":[{"uri":""#);
    json.push_str(&data_uri);
    json.push_str(r#"","byteLength":"#);
    json.push_str(&buffer_bytes.len().to_string());
    json.push_str(r#"}]}"#);

    Ok(json)
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    let chunks = data.chunks(3);
    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
