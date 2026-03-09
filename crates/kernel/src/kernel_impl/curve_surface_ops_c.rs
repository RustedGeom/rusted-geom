struct TessSampleMesh {
    vertices: Vec<RgmPoint3>,
    triangles: Vec<[u32; 3]>,
}

fn add_triangle_indices(
    triangles: &mut Vec<[u32; 3]>,
    a: usize,
    b: usize,
    c: usize,
) -> Result<(), RgmStatus> {
    triangles.push([
        u32::try_from(a).map_err(|_| RgmStatus::OutOfRange)?,
        u32::try_from(b).map_err(|_| RgmStatus::OutOfRange)?,
        u32::try_from(c).map_err(|_| RgmStatus::OutOfRange)?,
    ]);
    Ok(())
}

fn tessellate_surface_samples(
    surface: &SurfaceData,
    options: Option<RgmSurfaceTessellationOptions>,
) -> Result<TessSampleMesh, RgmStatus> {
    let options = sanitize_surface_tess_options(options, &surface.core, false);
    let u_segments = options.min_u_segments.min(options.max_u_segments) as usize;
    let v_segments = options.min_v_segments.min(options.max_v_segments) as usize;
    let u_segments = u_segments.max(2);
    let v_segments = v_segments.max(2);

    let index_of = |iu: usize, iv: usize| -> usize { iu * (v_segments + 1) + iv };
    let mut grid_uvs = Vec::with_capacity((u_segments + 1) * (v_segments + 1));
    for iu in 0..=u_segments {
        let u_norm = iu as f64 / u_segments as f64;
        for iv in 0..=v_segments {
            let v_norm = iv as f64 / v_segments as f64;
            let uv_norm = RgmUv2 {
                u: u_norm,
                v: v_norm,
            };
            let uv =
                math::nurbs_surface_eval::map_normalized_to_surface_uv(&surface.core, uv_norm)?;
            grid_uvs.push(uv);
        }
    }

    let per_u = surface.core.periodic_u;
    let per_v = surface.core.periodic_v;

    let mut vertices = Vec::with_capacity(grid_uvs.len());
    for uv in &grid_uvs {
        let eval = eval_nurbs_surface_uv_unchecked(&surface.core, *uv)?;
        vertices.push(matrix_apply_point(surface.transform, eval.point));
    }

    let remap = |iu: usize, iv: usize| -> usize {
        let iu = if per_u && iu == u_segments { 0 } else { iu };
        let iv = if per_v && iv == v_segments { 0 } else { iv };
        index_of(iu, iv)
    };

    let mut triangles = Vec::new();
    for iu in 0..u_segments {
        for iv in 0..v_segments {
            let a = remap(iu, iv);
            let b = remap(iu + 1, iv);
            let c = remap(iu, iv + 1);
            let d = remap(iu + 1, iv + 1);
            add_triangle_indices(&mut triangles, a, b, c)?;
            add_triangle_indices(&mut triangles, b, d, c)?;
        }
    }

    Ok(TessSampleMesh {
        vertices,
        triangles,
    })
}

fn build_mesh_from_tessellation(samples: &TessSampleMesh) -> MeshData {
    MeshData {
        vertices: samples.vertices.clone(),
        triangles: samples.triangles.clone(),
        transform: matrix_identity(),
    }
}

