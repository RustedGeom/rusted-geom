    fn rgm_nurbs_interpolate_fit_points(
        session: RgmKernelHandle,
        points: *const RgmPoint3,
        point_count: usize,
        degree: u32,
        closed: bool,
        tol: RgmToleranceContext,
        out_object: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_nurbs_interpolate_fit_points(
            session,
            points,
            point_count,
            degree,
            closed,
            &tol as *const _,
            out_object,
        )
    }

    fn rgm_curve_create_line(
        session: RgmKernelHandle,
        line: RgmLine3,
        tol: RgmToleranceContext,
        out_object: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_curve_create_line(session, &line as *const _, &tol as *const _, out_object)
    }

    fn rgm_curve_create_circle(
        session: RgmKernelHandle,
        circle: RgmCircle3,
        tol: RgmToleranceContext,
        out_object: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_curve_create_circle(session, &circle as *const _, &tol as *const _, out_object)
    }

    fn rgm_curve_create_arc(
        session: RgmKernelHandle,
        arc: RgmArc3,
        tol: RgmToleranceContext,
        out_object: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_curve_create_arc(session, &arc as *const _, &tol as *const _, out_object)
    }

    fn rgm_curve_create_arc_by_angles(
        session: RgmKernelHandle,
        plane: RgmPlane,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        tol: RgmToleranceContext,
        out_object: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_curve_create_arc_by_angles(
            session,
            &plane as *const _,
            radius,
            start_angle,
            end_angle,
            &tol as *const _,
            out_object,
        )
    }

    fn rgm_curve_create_arc_by_3_points(
        session: RgmKernelHandle,
        start: RgmPoint3,
        mid: RgmPoint3,
        end: RgmPoint3,
        tol: RgmToleranceContext,
        out_object: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_curve_create_arc_by_3_points(
            session,
            &start as *const _,
            &mid as *const _,
            &end as *const _,
            &tol as *const _,
            out_object,
        )
    }

    fn rgm_curve_create_polyline(
        session: RgmKernelHandle,
        points: *const RgmPoint3,
        point_count: usize,
        closed: bool,
        tol: RgmToleranceContext,
        out_object: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_curve_create_polyline(
            session,
            points,
            point_count,
            closed,
            &tol as *const _,
            out_object,
        )
    }

    fn rgm_curve_create_polycurve(
        session: RgmKernelHandle,
        segments: *const RgmPolycurveSegment,
        segment_count: usize,
        tol: RgmToleranceContext,
        out_object: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_curve_create_polycurve(
            session,
            segments,
            segment_count,
            &tol as *const _,
            out_object,
        )
    }

    fn rgm_mesh_create_box(
        session: RgmKernelHandle,
        center: RgmPoint3,
        size: RgmVec3,
        out_object: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_mesh_create_box(session, &center as *const _, &size as *const _, out_object)
    }

    fn rgm_mesh_create_uv_sphere(
        session: RgmKernelHandle,
        center: RgmPoint3,
        radius: f64,
        u_steps: u32,
        v_steps: u32,
        out_object: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_mesh_create_uv_sphere(
            session,
            &center as *const _,
            radius,
            u_steps,
            v_steps,
            out_object,
        )
    }

    fn rgm_mesh_create_torus(
        session: RgmKernelHandle,
        center: RgmPoint3,
        major_radius: f64,
        minor_radius: f64,
        major_steps: u32,
        minor_steps: u32,
        out_object: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_mesh_create_torus(
            session,
            &center as *const _,
            major_radius,
            minor_radius,
            major_steps,
            minor_steps,
            out_object,
        )
    }

    fn rgm_mesh_translate(
        session: RgmKernelHandle,
        mesh: RgmObjectHandle,
        delta: RgmVec3,
        out_mesh: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_mesh_translate(session, mesh, &delta as *const _, out_mesh)
    }

    fn rgm_mesh_rotate(
        session: RgmKernelHandle,
        mesh: RgmObjectHandle,
        axis: RgmVec3,
        angle_rad: f64,
        pivot: RgmPoint3,
        out_mesh: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_mesh_rotate(
            session,
            mesh,
            &axis as *const _,
            angle_rad,
            &pivot as *const _,
            out_mesh,
        )
    }

    fn rgm_mesh_scale(
        session: RgmKernelHandle,
        mesh: RgmObjectHandle,
        scale: RgmVec3,
        pivot: RgmPoint3,
        out_mesh: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_mesh_scale(
            session,
            mesh,
            &scale as *const _,
            &pivot as *const _,
            out_mesh,
        )
    }

    fn rgm_intersect_curve_plane(
        session: RgmKernelHandle,
        curve: RgmObjectHandle,
        plane: RgmPlane,
        out_points: *mut RgmPoint3,
        point_capacity: u32,
        out_count: *mut u32,
    ) -> RgmStatus {
        super::rgm_intersect_curve_plane(
            session,
            curve,
            &plane as *const _,
            out_points,
            point_capacity,
            out_count,
        )
    }

    fn rgm_object_compute_bounds(
        session: RgmKernelHandle,
        object: RgmObjectHandle,
        options: Option<RgmBoundsOptions>,
        out_bounds: *mut RgmBounds3,
    ) -> RgmStatus {
        if let Some(opts) = options {
            super::rgm_object_compute_bounds(session, object, &opts as *const _, out_bounds)
        } else {
            super::rgm_object_compute_bounds(session, object, ptr::null(), out_bounds)
        }
    }

    fn rgm_mesh_bake_transform(
        session: RgmKernelHandle,
        mesh: RgmObjectHandle,
        out_mesh: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_mesh_bake_transform(session, mesh, out_mesh)
    }

    fn rgm_mesh_vertex_count(
        session: RgmKernelHandle,
        mesh: RgmObjectHandle,
        out_count: *mut u32,
    ) -> RgmStatus {
        super::rgm_mesh_vertex_count(session, mesh, out_count)
    }

    fn rgm_mesh_triangle_count(
        session: RgmKernelHandle,
        mesh: RgmObjectHandle,
        out_count: *mut u32,
    ) -> RgmStatus {
        super::rgm_mesh_triangle_count(session, mesh, out_count)
    }

    fn rgm_mesh_copy_vertices(
        session: RgmKernelHandle,
        mesh: RgmObjectHandle,
        out_vertices: *mut RgmPoint3,
        vertex_capacity: u32,
        out_count: *mut u32,
    ) -> RgmStatus {
        super::rgm_mesh_copy_vertices(session, mesh, out_vertices, vertex_capacity, out_count)
    }

    fn rgm_mesh_copy_indices(
        session: RgmKernelHandle,
        mesh: RgmObjectHandle,
        out_indices: *mut u32,
        index_capacity: u32,
        out_count: *mut u32,
    ) -> RgmStatus {
        super::rgm_mesh_copy_indices(session, mesh, out_indices, index_capacity, out_count)
    }

    fn rgm_intersect_mesh_plane(
        session: RgmKernelHandle,
        mesh: RgmObjectHandle,
        plane: RgmPlane,
        out_points: *mut RgmPoint3,
        point_capacity: u32,
        out_count: *mut u32,
    ) -> RgmStatus {
        super::rgm_intersect_mesh_plane(
            session,
            mesh,
            &plane as *const _,
            out_points,
            point_capacity,
            out_count,
        )
    }

    fn rgm_intersect_mesh_mesh(
        session: RgmKernelHandle,
        mesh_a: RgmObjectHandle,
        mesh_b: RgmObjectHandle,
        out_points: *mut RgmPoint3,
        point_capacity: u32,
        out_count: *mut u32,
    ) -> RgmStatus {
        super::rgm_intersect_mesh_mesh(
            session,
            mesh_a,
            mesh_b,
            out_points,
            point_capacity,
            out_count,
        )
    }

    fn rgm_mesh_boolean(
        session: RgmKernelHandle,
        mesh_a: RgmObjectHandle,
        mesh_b: RgmObjectHandle,
        op: i32,
        out_mesh: *mut RgmObjectHandle,
    ) -> RgmStatus {
        super::rgm_mesh_boolean(session, mesh_a, mesh_b, op, out_mesh)
    }

    fn create_session() -> RgmKernelHandle {
        let mut session = RgmKernelHandle(0);
        let status = rgm_kernel_create(&mut session as *mut _);
        assert_eq!(status, RgmStatus::Ok);
        session
    }

    fn tol() -> RgmToleranceContext {
        RgmToleranceContext {
            abs_tol: 1e-9,
            rel_tol: 1e-9,
            angle_tol: 1e-9,
        }
    }

    fn sample_points() -> Vec<RgmPoint3> {
        vec![
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 2.0,
                y: 1.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 3.0,
                y: 1.0,
                z: 0.0,
            },
        ]
    }

    fn runtime_curve_points() -> Vec<RgmPoint3> {
        vec![
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 1.0,
                y: 0.25,
                z: 0.0,
            },
            RgmPoint3 {
                x: 2.0,
                y: 1.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 3.0,
                y: 1.25,
                z: 0.0,
            },
        ]
    }

    fn runtime_surface_points() -> Vec<RgmPoint3> {
        vec![
            RgmPoint3 {
                x: -2.0,
                y: -2.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: -2.0,
                y: 0.0,
                z: 0.8,
            },
            RgmPoint3 {
                x: -2.0,
                y: 2.0,
                z: 0.1,
            },
            RgmPoint3 {
                x: 0.0,
                y: -2.0,
                z: 0.7,
            },
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: -0.2,
            },
            RgmPoint3 {
                x: 0.0,
                y: 2.0,
                z: 0.9,
            },
            RgmPoint3 {
                x: 2.0,
                y: -2.0,
                z: -0.3,
            },
            RgmPoint3 {
                x: 2.0,
                y: 0.0,
                z: 0.6,
            },
            RgmPoint3 {
                x: 2.0,
                y: 2.0,
                z: 0.2,
            },
        ]
    }

    fn create_bilinear_surface(
        session: RgmKernelHandle,
        z00: f64,
        z01: f64,
        z10: f64,
        z11: f64,
    ) -> RgmObjectHandle {
        let desc = RgmNurbsSurfaceDesc {
            degree_u: 1,
            degree_v: 1,
            periodic_u: false,
            periodic_v: false,
            control_u_count: 2,
            control_v_count: 2,
        };
        let controls = [
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: z00,
            },
            RgmPoint3 {
                x: 0.0,
                y: 1.0,
                z: z01,
            },
            RgmPoint3 {
                x: 1.0,
                y: 0.0,
                z: z10,
            },
            RgmPoint3 {
                x: 1.0,
                y: 1.0,
                z: z11,
            },
        ];
        let weights = [1.0_f64; 4];
        let knots = [0.0_f64, 0.0, 1.0, 1.0];
        let surface_tol = tol();
        let mut surface = RgmObjectHandle(0);
        assert_eq!(
            rgm_surface_create_nurbs(
                session,
                &desc as *const _,
                controls.as_ptr(),
                controls.len(),
                weights.as_ptr(),
                weights.len(),
                knots.as_ptr(),
                knots.len(),
                knots.as_ptr(),
                knots.len(),
                &surface_tol as *const _,
                &mut surface as *mut _,
            ),
            RgmStatus::Ok
        );
        surface
    }

    fn clamped_uniform_knots(control_count: usize, degree: usize) -> Vec<f64> {
        let knot_count = control_count + degree + 1;
        let mut knots = vec![0.0; knot_count];
        let interior = control_count.saturating_sub(degree + 1);
        for i in 0..=degree {
            knots[i] = 0.0;
            knots[knot_count - 1 - i] = 1.0;
        }
        for i in 1..=interior {
            knots[degree + i] = i as f64 / (interior + 1) as f64;
        }
        knots
    }

    fn create_warped_surface(
        session: RgmKernelHandle,
        u_count: usize,
        v_count: usize,
        span_u: f64,
        span_v: f64,
        warp_scale: f64,
    ) -> RgmObjectHandle {
        let mut points = Vec::with_capacity(u_count * v_count);
        let mut weights = Vec::with_capacity(u_count * v_count);
        let half_u = span_u * 0.5;
        let half_v = span_v * 0.5;
        for iu in 0..u_count {
            let u = iu as f64 / (u_count.saturating_sub(1).max(1)) as f64;
            let x = -half_u + u * span_u;
            for iv in 0..v_count {
                let v = iv as f64 / (v_count.saturating_sub(1).max(1)) as f64;
                let y = -half_v + v * span_v;
                let z = ((u * 2.0 + v * 1.2) * PI).sin() * warp_scale
                    + ((u * 0.8 - v * 1.6) * PI).cos() * (warp_scale * 0.6);
                points.push(RgmPoint3 { x, y, z });
                weights.push(1.0 + 0.08 * ((u + v) * PI).sin());
            }
        }

        let desc = RgmNurbsSurfaceDesc {
            degree_u: 3,
            degree_v: 3,
            periodic_u: false,
            periodic_v: false,
            control_u_count: u_count as u32,
            control_v_count: v_count as u32,
        };
        let knots_u = clamped_uniform_knots(u_count, 3);
        let knots_v = clamped_uniform_knots(v_count, 3);
        let mut surface = RgmObjectHandle(0);
        assert_eq!(
            rgm_surface_create_nurbs(
                session,
                &desc as *const _,
                points.as_ptr(),
                points.len(),
                weights.as_ptr(),
                weights.len(),
                knots_u.as_ptr(),
                knots_u.len(),
                knots_v.as_ptr(),
                knots_v.len(),
                &tol() as *const _,
                &mut surface as *mut _,
            ),
            RgmStatus::Ok
        );
        surface
    }

    fn point_delta(a: RgmPoint3, b: RgmPoint3) -> RgmVec3 {
        RgmVec3 {
            x: a.x - b.x,
            y: a.y - b.y,
            z: a.z - b.z,
        }
    }

    fn point_inside_aabb(aabb: RgmAabb3, point: RgmPoint3, eps: f64) -> bool {
        point.x >= aabb.min.x - eps
            && point.x <= aabb.max.x + eps
            && point.y >= aabb.min.y - eps
            && point.y <= aabb.max.y + eps
            && point.z >= aabb.min.z - eps
            && point.z <= aabb.max.z + eps
    }

    fn project_point_to_obb_local(bounds: RgmBounds3, point: RgmPoint3) -> RgmPoint3 {
        let rel = point_delta(point, bounds.world_obb.center);
        RgmPoint3 {
            x: v3::dot(rel, bounds.world_obb.x_axis),
            y: v3::dot(rel, bounds.world_obb.y_axis),
            z: v3::dot(rel, bounds.world_obb.z_axis),
        }
    }

    fn assert_points_inside_local_aabb(bounds: RgmBounds3, points: &[RgmPoint3], eps: f64) {
        for point in points {
            let local = project_point_to_obb_local(bounds, *point);
            assert!(
                point_inside_aabb(bounds.local_aabb, local, eps),
                "point projected outside local AABB: local=({:.6}, {:.6}, {:.6}) min=({:.6}, {:.6}, {:.6}) max=({:.6}, {:.6}, {:.6})",
                local.x,
                local.y,
                local.z,
                bounds.local_aabb.min.x,
                bounds.local_aabb.min.y,
                bounds.local_aabb.min.z,
                bounds.local_aabb.max.x,
                bounds.local_aabb.max.y,
                bounds.local_aabb.max.z,
            );
        }
    }

    fn obb_volume(bounds: RgmBounds3) -> f64 {
        let e = bounds.world_obb.half_extents;
        (e.x * 2.0) * (e.y * 2.0) * (e.z * 2.0)
    }

    // ── Debug inspectors ──────────────────────────────────────────────────────

    fn debug_get_curve(session: RgmKernelHandle, object: RgmObjectHandle) -> Option<CurveData> {
        let session_entry = super::SESSIONS.get(&session.0)?;
        let state = session_entry.value().read();
        match state.objects.get(&object.0)? {
            GeometryObject::Curve(curve) => Some(curve.clone()),
            GeometryObject::Mesh(_)
            | GeometryObject::Surface(_)
            | GeometryObject::Intersection(_)
            | GeometryObject::LandXmlDoc(_) => None,
        }
    }

    fn debug_get_mesh(session: RgmKernelHandle, object: RgmObjectHandle) -> Option<MeshData> {
        let session_entry = super::SESSIONS.get(&session.0)?;
        let state = session_entry.value().read();
        match state.objects.get(&object.0)? {
            GeometryObject::Mesh(mesh) => Some(mesh.clone()),
            GeometryObject::Curve(_)
            | GeometryObject::Surface(_)
            | GeometryObject::Intersection(_)
            | GeometryObject::LandXmlDoc(_) => None,
        }
    }

    fn rgm_mesh_volume(
        session: RgmKernelHandle,
        mesh: RgmObjectHandle,
        out_volume: *mut f64,
    ) -> RgmStatus {
        super::rgm_mesh_volume(session, mesh, out_volume)
    }

    fn debug_get_intersection(
        session: RgmKernelHandle,
        object: RgmObjectHandle,
    ) -> Option<IntersectionData> {
        let session_entry = super::SESSIONS.get(&session.0)?;
        let state = session_entry.value().read();
        match state.objects.get(&object.0)? {
            GeometryObject::Intersection(data) => Some(data.clone()),
            GeometryObject::Curve(_)
            | GeometryObject::Mesh(_)
            | GeometryObject::Surface(_)
            | GeometryObject::LandXmlDoc(_) => None,
        }
    }

