fn bisect_face_boundary_on_edge(
    classifier: &FaceUvClassifier,
    a: RgmUv2,
    b: RgmUv2,
    a_inside: bool,
    tol: f64,
    class_cache: &mut HashMap<(u64, u64), i32>,
) -> RgmUv2 {
    let mut inside_pt = if a_inside { a } else { b };
    let mut outside_pt = if a_inside { b } else { a };

    for _ in 0..32 {
        let mid = uv_lerp(inside_pt, outside_pt, 0.5);
        let class = classify_uv_cached(classifier, mid, tol, class_cache);
        if class >= 0 {
            inside_pt = mid;
        } else {
            outside_pt = mid;
        }
        if uv_distance(inside_pt, outside_pt) <= tol {
            break;
        }
    }

    // Stay conservative: return a point guaranteed inside/on-boundary.
    inside_pt
}

fn clip_triangle_to_face_polygon(
    classifier: &FaceUvClassifier,
    tri: [RgmUv2; 3],
    cls: [i32; 3],
    tol: f64,
    class_cache: &mut HashMap<(u64, u64), i32>,
) -> Vec<RgmUv2> {
    let mut out = Vec::with_capacity(6);
    for edge in 0..3 {
        let curr = tri[edge];
        let next = tri[(edge + 1) % 3];
        let curr_in = cls[edge] >= 0;
        let next_in = cls[(edge + 1) % 3] >= 0;
        match (curr_in, next_in) {
            (true, true) => {
                out.push(next);
            }
            (true, false) => {
                out.push(bisect_face_boundary_on_edge(
                    classifier,
                    curr,
                    next,
                    true,
                    tol,
                    class_cache,
                ));
            }
            (false, true) => {
                out.push(bisect_face_boundary_on_edge(
                    classifier,
                    curr,
                    next,
                    false,
                    tol,
                    class_cache,
                ));
                out.push(next);
            }
            (false, false) => {}
        }
    }
    dedupe_polygon_uv(out, tol)
}

struct TrimmedTessBuilder<'a> {
    surface: &'a SurfaceData,
    vertices: Vec<RgmPoint3>,
    triangles: Vec<[u32; 3]>,
    uv_index: HashMap<(u64, u64), u32>,
}

impl<'a> TrimmedTessBuilder<'a> {
    fn new(surface: &'a SurfaceData, capacity_hint: usize) -> Self {
        Self {
            surface,
            vertices: Vec::with_capacity(capacity_hint),
            triangles: Vec::with_capacity(capacity_hint.saturating_mul(2)),
            uv_index: HashMap::with_capacity(capacity_hint),
        }
    }

    fn vertex_index(&mut self, uv: RgmUv2) -> Result<u32, RgmStatus> {
        let key = canonical_uv_key(uv);
        if let Some(index) = self.uv_index.get(&key) {
            return Ok(*index);
        }
        let frame = eval_surface_data_uv(self.surface, uv)?;
        let index = u32::try_from(self.vertices.len()).map_err(|_| RgmStatus::OutOfRange)?;
        self.vertices.push(frame.point);
        self.uv_index.insert(key, index);
        Ok(index)
    }

    fn add_triangle(&mut self, tri: [RgmUv2; 3], tol: f64) -> Result<(), RgmStatus> {
        if triangle_degenerate_uv(tri, tol) {
            return Ok(());
        }
        let a = self.vertex_index(tri[0])?;
        let b = self.vertex_index(tri[1])?;
        let c = self.vertex_index(tri[2])?;
        if a == b || b == c || c == a {
            return Ok(());
        }
        self.triangles.push([a, b, c]);
        Ok(())
    }

    fn into_mesh(self) -> TessSampleMesh {
        TessSampleMesh {
            vertices: self.vertices,
            triangles: self.triangles,
        }
    }
}

#[derive(Clone, Copy)]
struct TrimRefineParams {
    tol: f64,
    max_depth: u32,
    min_edge: f64,
}

fn triangle_uv_centroid(tri: [RgmUv2; 3]) -> RgmUv2 {
    RgmUv2 {
        u: (tri[0].u + tri[1].u + tri[2].u) / 3.0,
        v: (tri[0].v + tri[1].v + tri[2].v) / 3.0,
    }
}

fn add_trimmed_triangle_if_inside(
    builder: &mut TrimmedTessBuilder<'_>,
    classifier: &FaceUvClassifier,
    tri: [RgmUv2; 3],
    tol: f64,
    class_cache: &mut HashMap<(u64, u64), i32>,
) -> Result<(), RgmStatus> {
    let centroid = triangle_uv_centroid(tri);
    if classify_uv_cached(classifier, centroid, tol, class_cache) < 0 {
        return Ok(());
    }
    builder.add_triangle(tri, tol)
}

fn refine_trimmed_triangle(
    builder: &mut TrimmedTessBuilder<'_>,
    classifier: &FaceUvClassifier,
    trim_segments: &[(RgmUv2, RgmUv2)],
    trim_bvh: Option<&TrimSegmentBvh>,
    class_cache: &mut HashMap<(u64, u64), i32>,
    tri: [RgmUv2; 3],
    cls: [i32; 3],
    depth: u32,
    params: TrimRefineParams,
) -> Result<(), RgmStatus> {
    if triangle_degenerate_uv(tri, params.tol) {
        return Ok(());
    }

    let all_inside = cls.iter().all(|v| *v >= 0);
    let all_outside = cls.iter().all(|v| *v < 0);
    let crossing = triangle_has_trim_crossing(tri, trim_segments, trim_bvh, params.tol);
    let centroid = triangle_uv_centroid(tri);
    let m01 = uv_lerp(tri[0], tri[1], 0.5);
    let m12 = uv_lerp(tri[1], tri[2], 0.5);
    let m20 = uv_lerp(tri[2], tri[0], 0.5);
    let mut has_interior_outside_probe = false;
    if all_inside {
        has_interior_outside_probe =
            classify_uv_cached(classifier, centroid, params.tol, class_cache) < 0
                || classify_uv_cached(classifier, m01, params.tol, class_cache) < 0
                || classify_uv_cached(classifier, m12, params.tol, class_cache) < 0
                || classify_uv_cached(classifier, m20, params.tol, class_cache) < 0;
    }
    let effective_crossing = crossing || has_interior_outside_probe;

    if !effective_crossing {
        if all_inside {
            return add_trimmed_triangle_if_inside(
                builder,
                classifier,
                tri,
                params.tol,
                class_cache,
            );
        }
        if all_outside {
            return Ok(());
        }
    }

    let max_edge = triangle_max_uv_edge(tri);
    if depth >= params.max_depth || max_edge <= params.min_edge {
        if all_inside {
            if has_interior_outside_probe {
                return Ok(());
            }
            return add_trimmed_triangle_if_inside(
                builder,
                classifier,
                tri,
                params.tol,
                class_cache,
            );
        }
        if all_outside {
            return Ok(());
        }

        let clipped = clip_triangle_to_face_polygon(classifier, tri, cls, params.tol, class_cache);
        if clipped.len() < 3 {
            return Ok(());
        }
        let anchor = clipped[0];
        for idx in 1..(clipped.len() - 1) {
            add_trimmed_triangle_if_inside(
                builder,
                classifier,
                [anchor, clipped[idx], clipped[idx + 1]],
                params.tol,
                class_cache,
            )?;
        }
        return Ok(());
    }

    let ab = uv_lerp(tri[0], tri[1], 0.5);
    let bc = uv_lerp(tri[1], tri[2], 0.5);
    let ca = uv_lerp(tri[2], tri[0], 0.5);

    let cab = classify_uv_cached(classifier, ab, params.tol, class_cache);
    let cbc = classify_uv_cached(classifier, bc, params.tol, class_cache);
    let cca = classify_uv_cached(classifier, ca, params.tol, class_cache);

    refine_trimmed_triangle(
        builder,
        classifier,
        trim_segments,
        trim_bvh,
        class_cache,
        [tri[0], ab, ca],
        [cls[0], cab, cca],
        depth + 1,
        params,
    )?;
    refine_trimmed_triangle(
        builder,
        classifier,
        trim_segments,
        trim_bvh,
        class_cache,
        [ab, tri[1], bc],
        [cab, cls[1], cbc],
        depth + 1,
        params,
    )?;
    refine_trimmed_triangle(
        builder,
        classifier,
        trim_segments,
        trim_bvh,
        class_cache,
        [ca, bc, tri[2]],
        [cca, cbc, cls[2]],
        depth + 1,
        params,
    )?;
    refine_trimmed_triangle(
        builder,
        classifier,
        trim_segments,
        trim_bvh,
        class_cache,
        [ab, bc, ca],
        [cab, cbc, cca],
        depth + 1,
        params,
    )?;

    Ok(())
}

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
    face: Option<&FaceData>,
    options: Option<RgmSurfaceTessellationOptions>,
) -> Result<TessSampleMesh, RgmStatus> {
    let is_trimmed = face.map_or(false, |f| !f.loops.is_empty());
    let options = sanitize_surface_tess_options(options, &surface.core, is_trimmed);
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

    if let Some(face) = face {
        if !face.loops.is_empty() {
            let tol = surface.core.tol.abs_tol.max(1e-8);
            let classifier = FaceUvClassifier::from_face(face);
            let trim_segments = collect_trim_segments(face);
            let trim_bvh = build_trim_segment_bvh(&trim_segments);
            let domain_u = (surface.core.u_end - surface.core.u_start).abs().max(1e-12);
            let domain_v = (surface.core.v_end - surface.core.v_start).abs().max(1e-12);
            let base_du = domain_u / (options.max_u_segments.max(1) as f64);
            let base_dv = domain_v / (options.max_v_segments.max(1) as f64);
            let min_edge = ((base_du * base_du + base_dv * base_dv).sqrt() * 0.125).max(1e-10);
            let ratio_u =
                (options.max_u_segments as f64 / options.min_u_segments.max(1) as f64).max(1.0);
            let ratio_v =
                (options.max_v_segments as f64 / options.min_v_segments.max(1) as f64).max(1.0);
            let depth_u = ratio_u.log2().ceil() as u32;
            let depth_v = ratio_v.log2().ceil() as u32;
            let max_depth = (depth_u.max(depth_v) + 3).clamp(3, 8);
            let params = TrimRefineParams {
                tol,
                max_depth,
                min_edge,
            };

            let mut class_cache = HashMap::new();
            let mut builder =
                TrimmedTessBuilder::new(surface, (u_segments + 1).saturating_mul(v_segments + 1));

            for iu in 0..u_segments {
                for iv in 0..v_segments {
                    let a = grid_uvs[index_of(iu, iv)];
                    let b = grid_uvs[index_of(iu + 1, iv)];
                    let c = grid_uvs[index_of(iu, iv + 1)];
                    let d = grid_uvs[index_of(iu + 1, iv + 1)];

                    let tri0 = [a, b, c];
                    let tri1 = [b, d, c];
                    let cls0 = [
                        classify_uv_cached(&classifier, tri0[0], tol, &mut class_cache),
                        classify_uv_cached(&classifier, tri0[1], tol, &mut class_cache),
                        classify_uv_cached(&classifier, tri0[2], tol, &mut class_cache),
                    ];
                    let cls1 = [
                        classify_uv_cached(&classifier, tri1[0], tol, &mut class_cache),
                        classify_uv_cached(&classifier, tri1[1], tol, &mut class_cache),
                        classify_uv_cached(&classifier, tri1[2], tol, &mut class_cache),
                    ];

                    refine_trimmed_triangle(
                        &mut builder,
                        &classifier,
                        &trim_segments,
                        trim_bvh.as_ref(),
                        &mut class_cache,
                        tri0,
                        cls0,
                        0,
                        params,
                    )?;
                    refine_trimmed_triangle(
                        &mut builder,
                        &classifier,
                        &trim_segments,
                        trim_bvh.as_ref(),
                        &mut class_cache,
                        tri1,
                        cls1,
                        0,
                        params,
                    )?;
                }
            }

            return Ok(builder.into_mesh());
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

