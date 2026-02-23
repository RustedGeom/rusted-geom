
#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    use std::time::Instant;

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

    fn add_outer_rect_loop(session: RgmKernelHandle, face: RgmObjectHandle) {
        let outer = [
            RgmUv2 { u: 0.0, v: 0.0 },
            RgmUv2 { u: 1.0, v: 0.0 },
            RgmUv2 { u: 1.0, v: 1.0 },
            RgmUv2 { u: 0.0, v: 1.0 },
        ];
        assert_eq!(
            rgm_face_add_loop(session, face, outer.as_ptr(), outer.len(), true),
            RgmStatus::Ok
        );
    }

    fn add_curved_hole_loop(
        session: RgmKernelHandle,
        face: RgmObjectHandle,
        center_u: f64,
        center_v: f64,
        radius: f64,
    ) -> Vec<RgmObjectHandle> {
        let plane = RgmPlane {
            origin: RgmPoint3 {
                x: center_u,
                y: center_v,
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

        let mut curve_handles = Vec::new();
        let mut edges = Vec::new();
        for idx in 0..4 {
            let start_angle = idx as f64 * (PI * 0.5);
            let end_angle = (idx as f64 + 1.0) * (PI * 0.5);
            let mut arc = RgmObjectHandle(0);
            assert_eq!(
                rgm_curve_create_arc_by_angles(
                    session,
                    plane,
                    radius,
                    start_angle,
                    end_angle,
                    tol(),
                    &mut arc as *mut _,
                ),
                RgmStatus::Ok
            );
            curve_handles.push(arc);
            edges.push(RgmTrimEdgeInput {
                start_uv: RgmUv2 {
                    u: center_u + radius * start_angle.cos(),
                    v: center_v + radius * start_angle.sin(),
                },
                end_uv: RgmUv2 {
                    u: center_u + radius * end_angle.cos(),
                    v: center_v + radius * end_angle.sin(),
                },
                curve_3d: arc,
                has_curve_3d: true,
            });
        }

        let loop_input = RgmTrimLoopInput {
            edge_count: edges.len() as u32,
            is_outer: false,
        };
        assert_eq!(
            rgm_face_add_loop_edges(
                session,
                face,
                &loop_input as *const _,
                edges.as_ptr(),
                edges.len(),
            ),
            RgmStatus::Ok
        );
        curve_handles
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

    #[test]
    fn trimmed_face_tessellation_keeps_triangles_in_trim_domain() {
        let session = create_session();

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
                z: 0.0,
            },
            RgmPoint3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 1.0,
                y: 1.0,
                z: 0.0,
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

        let mut face = RgmObjectHandle(0);
        assert_eq!(
            rgm_face_create_from_surface(session, surface, &mut face as *mut _),
            RgmStatus::Ok
        );

        let outer = [
            RgmUv2 { u: 0.0, v: 0.0 },
            RgmUv2 { u: 1.0, v: 0.0 },
            RgmUv2 { u: 1.0, v: 1.0 },
            RgmUv2 { u: 0.0, v: 1.0 },
        ];
        assert_eq!(
            rgm_face_add_loop(session, face, outer.as_ptr(), outer.len(), true),
            RgmStatus::Ok
        );

        let mut hole = Vec::with_capacity(24);
        let center_u = 0.52;
        let center_v = 0.48;
        let radius = 0.2;
        for i in 0..24 {
            let a = (i as f64 / 24.0) * PI * 2.0;
            hole.push(RgmUv2 {
                u: center_u + radius * a.cos(),
                v: center_v + radius * a.sin(),
            });
        }
        assert_eq!(
            rgm_face_add_loop(session, face, hole.as_ptr(), hole.len(), false),
            RgmStatus::Ok
        );
        assert_eq!(rgm_face_heal(session, face), RgmStatus::Ok);

        let options = RgmSurfaceTessellationOptions {
            min_u_segments: 10,
            min_v_segments: 10,
            max_u_segments: 80,
            max_v_segments: 80,
            chord_tol: 1e-4,
            normal_tol_rad: 0.05,
        };
        let mut mesh = RgmObjectHandle(0);
        assert_eq!(
            rgm_face_tessellate_to_mesh(session, face, &options as *const _, &mut mesh as *mut _),
            RgmStatus::Ok
        );

        let mesh_data = debug_get_mesh(session, mesh).expect("mesh exists");
        let face_data = debug_get_face(session, face).expect("face exists");
        assert!(!mesh_data.triangles.is_empty());

        for tri in &mesh_data.triangles {
            let a = mesh_data.vertices[tri[0] as usize];
            let b = mesh_data.vertices[tri[1] as usize];
            let c = mesh_data.vertices[tri[2] as usize];
            let ua = RgmUv2 { u: a.x, v: a.y };
            let ub = RgmUv2 { u: b.x, v: b.y };
            let uc = RgmUv2 { u: c.x, v: c.y };
            let uv = RgmUv2 {
                u: (a.x + b.x + c.x) / 3.0,
                v: (a.y + b.y + c.y) / 3.0,
            };
            let class = classify_uv_in_face(&face_data, uv, 1e-8);
            if class < 0 {
                let ca = classify_uv_in_face(&face_data, ua, 1e-8);
                let cb = classify_uv_in_face(&face_data, ub, 1e-8);
                let cc = classify_uv_in_face(&face_data, uc, 1e-8);
                panic!(
                    "triangle centroid outside trim domain: tri=({:?},{:?},{:?}) cls=({}, {}, {}) centroid={:?}",
                    ua, ub, uc, ca, cb, cc, uv
                );
            }
        }

        assert_eq!(rgm_object_release(session, mesh), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, face), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn curved_trim_edges_are_sampled_and_validated() {
        let session = create_session();
        let surface = create_bilinear_surface(session, 0.0, 0.0, 0.0, 0.0);
        let mut face = RgmObjectHandle(0);
        assert_eq!(
            rgm_face_create_from_surface(session, surface, &mut face as *mut _),
            RgmStatus::Ok
        );
        add_outer_rect_loop(session, face);
        let curve_handles = add_curved_hole_loop(session, face, 0.5, 0.5, 0.2);
        assert_eq!(rgm_face_heal(session, face), RgmStatus::Ok);

        let face_data = debug_get_face(session, face).expect("face exists");
        assert!(validate_face_data(&face_data, 1e-8));
        assert_eq!(face_data.loops.len(), 2);
        let hole_poly = trim_loop_polyline(&face_data.loops[1]);
        assert!(
            hole_poly.len() > 16,
            "curved trim edge sampling should generate dense hole polyline"
        );

        for handle in curve_handles {
            assert_eq!(rgm_object_release(session, handle), RgmStatus::Ok);
        }
        assert_eq!(rgm_object_release(session, face), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn surface_curve_intersection_is_trim_clipped() {
        let session = create_session();
        let surface = create_bilinear_surface(session, 0.0, 0.0, 1.0, 1.0);
        let mut face = RgmObjectHandle(0);
        assert_eq!(
            rgm_face_create_from_surface(session, surface, &mut face as *mut _),
            RgmStatus::Ok
        );
        add_outer_rect_loop(session, face);
        let curve_handles = add_curved_hole_loop(session, face, 0.5, 0.5, 0.2);
        assert_eq!(rgm_face_heal(session, face), RgmStatus::Ok);

        let mut line = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 0.5,
                        y: 0.5,
                        z: -1.0,
                    },
                    end: RgmPoint3 {
                        x: 0.5,
                        y: 0.5,
                        z: 2.0,
                    },
                },
                tol(),
                &mut line as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut intersection = RgmObjectHandle(0);
        assert_eq!(
            rgm_intersect_surface_curve(session, face, line, &mut intersection as *mut _),
            RgmStatus::Ok
        );
        let data = debug_get_intersection(session, intersection).expect("intersection exists");
        assert!(
            data.branches.is_empty(),
            "vertical hit at hole center must be trimmed away"
        );

        assert_eq!(rgm_object_release(session, intersection), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
        for handle in curve_handles {
            assert_eq!(rgm_object_release(session, handle), RgmStatus::Ok);
        }
        assert_eq!(rgm_object_release(session, face), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn surface_plane_and_surface_surface_intersections_are_trim_clipped() {
        let session = create_session();
        let surface_a = create_bilinear_surface(session, 0.0, 0.0, 1.0, 1.0);
        let mut face_a = RgmObjectHandle(0);
        assert_eq!(
            rgm_face_create_from_surface(session, surface_a, &mut face_a as *mut _),
            RgmStatus::Ok
        );
        add_outer_rect_loop(session, face_a);
        let curve_handles = add_curved_hole_loop(session, face_a, 0.5, 0.5, 0.2);
        assert_eq!(rgm_face_heal(session, face_a), RgmStatus::Ok);
        let face_data = debug_get_face(session, face_a).expect("face exists");

        let plane = RgmPlane {
            origin: RgmPoint3 {
                x: 0.5,
                y: 0.0,
                z: 0.0,
            },
            x_axis: RgmVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            y_axis: RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            z_axis: RgmVec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        };
        let mut plane_intersection = RgmObjectHandle(0);
        assert_eq!(
            rgm_intersect_surface_plane(
                session,
                face_a,
                &plane as *const _,
                &mut plane_intersection as *mut _
            ),
            RgmStatus::Ok
        );
        let plane_data =
            debug_get_intersection(session, plane_intersection).expect("intersection exists");
        assert!(!plane_data.branches.is_empty());
        for branch in &plane_data.branches {
            for uv in &branch.uv_a {
                assert!(
                    classify_uv_in_face(&face_data, *uv, 1e-8) >= 0,
                    "surface-plane branch must stay in trimmed face"
                );
            }
        }

        let desc = RgmNurbsSurfaceDesc {
            degree_u: 1,
            degree_v: 1,
            periodic_u: false,
            periodic_v: false,
            control_u_count: 2,
            control_v_count: 2,
        };
        let controls_b = [
            RgmPoint3 {
                x: 0.5,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 0.5,
                y: 1.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 0.5,
                y: 0.0,
                z: 1.0,
            },
            RgmPoint3 {
                x: 0.5,
                y: 1.0,
                z: 1.0,
            },
        ];
        let weights = [1.0_f64; 4];
        let knots = [0.0_f64, 0.0, 1.0, 1.0];
        let surface_tol = tol();
        let mut surface_b = RgmObjectHandle(0);
        assert_eq!(
            rgm_surface_create_nurbs(
                session,
                &desc as *const _,
                controls_b.as_ptr(),
                controls_b.len(),
                weights.as_ptr(),
                weights.len(),
                knots.as_ptr(),
                knots.len(),
                knots.as_ptr(),
                knots.len(),
                &surface_tol as *const _,
                &mut surface_b as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut ss_intersection = RgmObjectHandle(0);
        assert_eq!(
            rgm_intersect_surface_surface(
                session,
                face_a,
                surface_b,
                &mut ss_intersection as *mut _
            ),
            RgmStatus::Ok
        );
        let ss_data =
            debug_get_intersection(session, ss_intersection).expect("intersection exists");
        assert!(!ss_data.branches.is_empty());
        for branch in &ss_data.branches {
            for uv in &branch.uv_a {
                assert!(
                    classify_uv_in_face(&face_data, *uv, 1e-8) >= 0,
                    "surface-surface branch must stay in trimmed face"
                );
            }
        }

        assert_eq!(rgm_object_release(session, ss_intersection), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface_b), RgmStatus::Ok);
        assert_eq!(
            rgm_object_release(session, plane_intersection),
            RgmStatus::Ok
        );
        for handle in curve_handles {
            assert_eq!(rgm_object_release(session, handle), RgmStatus::Ok);
        }
        assert_eq!(rgm_object_release(session, face_a), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface_a), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn viewer_surface_surface_example_returns_non_empty_branches() {
        let session = create_session();

        let surface_a = create_warped_surface(session, 16, 15, 12.0, 10.0, 1.0);
        let surface_b0 = create_warped_surface(session, 15, 16, 11.0, 11.0, 1.25);
        let mut surface_bt = RgmObjectHandle(0);
        assert_eq!(
            rgm_surface_translate(
                session,
                surface_b0,
                &RgmVec3 {
                    x: 0.6,
                    y: 0.3,
                    z: -0.1,
                } as *const _,
                &mut surface_bt as *mut _,
            ),
            RgmStatus::Ok
        );
        let mut surface_b = RgmObjectHandle(0);
        assert_eq!(
            rgm_surface_rotate(
                session,
                surface_bt,
                &RgmVec3 {
                    x: 0.3,
                    y: 1.0,
                    z: 0.2,
                } as *const _,
                0.72,
                &RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                } as *const _,
                &mut surface_b as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut intersection = RgmObjectHandle(0);
        let start = Instant::now();
        assert_eq!(
            rgm_intersect_surface_surface(
                session,
                surface_a,
                surface_b,
                &mut intersection as *mut _
            ),
            RgmStatus::Ok
        );
        let elapsed = start.elapsed();
        let data = debug_get_intersection(session, intersection).expect("intersection exists");
        let total_points: usize = data.branches.iter().map(|branch| branch.points.len()).sum();
        assert!(
            !data.branches.is_empty(),
            "viewer surface-surface must produce branches (elapsed={elapsed:?}, points={total_points})"
        );
        assert!(
            total_points >= 2,
            "viewer surface-surface must produce polyline samples (elapsed={elapsed:?})"
        );

        assert_eq!(rgm_object_release(session, intersection), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface_b), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface_bt), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface_b0), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface_a), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn viewer_surface_curve_example_returns_intersection_hits() {
        let session = create_session();

        let surface = create_warped_surface(session, 16, 16, 12.0, 12.0, 1.2);
        let curve_points = [
            RgmPoint3 {
                x: -6.2,
                y: -3.4,
                z: -2.0,
            },
            RgmPoint3 {
                x: -3.1,
                y: -0.2,
                z: 2.5,
            },
            RgmPoint3 {
                x: -0.5,
                y: 2.8,
                z: -1.8,
            },
            RgmPoint3 {
                x: 2.2,
                y: 1.1,
                z: 2.2,
            },
            RgmPoint3 {
                x: 4.8,
                y: -1.6,
                z: -2.3,
            },
            RgmPoint3 {
                x: 6.1,
                y: 2.3,
                z: 1.9,
            },
        ];
        let mut curve = RgmObjectHandle(0);
        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                curve_points.as_ptr(),
                curve_points.len(),
                3,
                false,
                tol(),
                &mut curve as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut intersection = RgmObjectHandle(0);
        let start = Instant::now();
        assert_eq!(
            rgm_intersect_surface_curve(session, surface, curve, &mut intersection as *mut _),
            RgmStatus::Ok
        );
        let elapsed = start.elapsed();
        let data = debug_get_intersection(session, intersection).expect("intersection exists");
        let total_hits: usize = data.branches.iter().map(|branch| branch.points.len()).sum();
        assert!(
            total_hits > 0,
            "viewer surface-curve must produce hits (elapsed={elapsed:?}, branches={})",
            data.branches.len()
        );

        assert_eq!(rgm_object_release(session, intersection), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, curve), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn surface_curve_finds_multiple_hits_on_warped_surface() {
        let session = create_session();

        let surface = create_warped_surface(session, 16, 16, 12.0, 12.0, 1.2);
        let mut line = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: -6.2,
                        y: 6.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 6.2,
                        y: 6.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut intersection = RgmObjectHandle(0);
        assert_eq!(
            rgm_intersect_surface_curve(session, surface, line, &mut intersection as *mut _),
            RgmStatus::Ok
        );
        let data = debug_get_intersection(session, intersection).expect("intersection exists");
        let hit_count: usize = data.branches.iter().map(|branch| branch.points.len()).sum();
        assert!(
            hit_count >= 2,
            "expected multiple line-surface hits on warped surface, got {hit_count}"
        );

        assert_eq!(rgm_object_release(session, intersection), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn session_create_and_destroy() {
        let mut session = RgmKernelHandle(0);
        assert_eq!(rgm_kernel_create(&mut session as *mut _), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::NotFound);
    }

    #[test]
    fn alloc_and_dealloc_roundtrip() {
        let mut ptr: *mut u8 = ptr::null_mut();
        assert_eq!(rgm_alloc(64, 8, &mut ptr as *mut _), RgmStatus::Ok);
        assert!(!ptr.is_null());

        // SAFETY: ptr is allocated for 64 bytes above.
        unsafe {
            for idx in 0..64 {
                *ptr.add(idx) = idx as u8;
            }
        }

        assert_eq!(rgm_dealloc(ptr, 64, 8), RgmStatus::Ok);
    }

    #[test]
    fn interpolate_open_curve_creates_nurbs() {
        let session = create_session();
        let points = sample_points();
        let mut object = RgmObjectHandle(0);

        let status = rgm_nurbs_interpolate_fit_points(
            session,
            points.as_ptr(),
            points.len(),
            2,
            false,
            tol(),
            &mut object as *mut _,
        );
        assert_eq!(status, RgmStatus::Ok);

        let curve = debug_get_curve(session, object).expect("curve exists");
        let CurveData::NurbsCurve(curve) = curve else {
            panic!("expected NURBS curve");
        };
        assert_eq!(curve.core.weights, vec![1.0; points.len()]);
        assert!((curve.core.knots[0] - 0.0).abs() < 1e-12);
        assert!((curve.core.knots[curve.core.knots.len() - 1] - 1.0).abs() < 1e-12);
        assert!(curve.arc_length.total_length > 0.0);

        assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn can_evaluate_point_derivatives_and_plane() {
        let session = create_session();
        let points = sample_points();
        let mut object = RgmObjectHandle(0);

        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                points.as_ptr(),
                points.len(),
                2,
                false,
                tol(),
                &mut object as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut point = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(
            rgm_curve_point_at(session, object, 0.5, &mut point as *mut _),
            RgmStatus::Ok
        );

        let mut d1 = RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(
            rgm_curve_d1_at(session, object, 0.5, &mut d1 as *mut _),
            RgmStatus::Ok
        );

        let mut d2 = RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(
            rgm_curve_d2_at(session, object, 0.5, &mut d2 as *mut _),
            RgmStatus::Ok
        );
        assert!(v3::norm(d2) > 0.0);

        let mut plane = RgmPlane {
            origin: point,
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

        assert_eq!(
            rgm_curve_plane_at(session, object, 0.5, &mut plane as *mut _),
            RgmStatus::Ok
        );

        assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn length_queries_are_available() {
        let session = create_session();
        let points = sample_points();
        let mut object = RgmObjectHandle(0);

        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                points.as_ptr(),
                points.len(),
                2,
                false,
                tol(),
                &mut object as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut total: f64 = 0.0;
        assert_eq!(
            rgm_curve_length(session, object, &mut total as *mut _),
            RgmStatus::Ok
        );
        assert!(total > 0.0);

        let mut s0: f64 = -1.0;
        let mut s1: f64 = -1.0;
        let mut smid: f64 = -1.0;
        assert_eq!(
            rgm_curve_length_at(session, object, 0.0, &mut s0 as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_length_at(session, object, 0.5, &mut smid as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_length_at(session, object, 1.0, &mut s1 as *mut _),
            RgmStatus::Ok
        );

        assert!(s0.abs() < 1e-8);
        assert!(smid > s0);
        assert!(smid < s1);
        assert!((s1 - total).abs() < 1e-7);

        assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn line_constructor_is_exact_and_linear() {
        let session = create_session();
        let mut line = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 10.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut p = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(
            rgm_curve_point_at(session, line, 0.25, &mut p as *mut _),
            RgmStatus::Ok
        );
        assert!((p.x - 2.5).abs() < 1e-9);

        let mut d2 = RgmVec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        assert_eq!(
            rgm_curve_d2_at(session, line, 0.5, &mut d2 as *mut _),
            RgmStatus::Ok
        );
        assert!(v3::norm(d2) < 1e-7);

        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn line_length_matches_euclidean_distance() {
        let session = create_session();
        let mut line = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 3.0,
                        y: 4.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, line, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - 5.0).abs() < 1e-9);

        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn zero_length_line_is_supported() {
        let session = create_session();
        let mut line = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                    end: RgmPoint3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    },
                },
                tol(),
                &mut line as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = -1.0_f64;
        assert_eq!(
            rgm_curve_length(session, line, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!(length.abs() < 1e-12);

        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn arc_by_angles_length_matches_radius_times_sweep() {
        let session = create_session();
        let mut arc = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_arc_by_angles(
                session,
                RgmPlane {
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
                },
                10.0,
                0.0,
                PI,
                tol(),
                &mut arc as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, arc, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - 10.0 * PI).abs() < 1e-8);

        assert_eq!(rgm_object_release(session, arc), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn arc_by_angles_length_is_positive_when_angles_are_reversed() {
        let session = create_session();
        let mut arc = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_arc_by_angles(
                session,
                RgmPlane {
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
                },
                5.0,
                PI,
                0.0,
                tol(),
                &mut arc as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, arc, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - 5.0 * PI).abs() < 1e-8);

        assert_eq!(rgm_object_release(session, arc), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn arc_by_3_points_preserves_radius_and_endpoints() {
        let session = create_session();
        let mut arc = RgmObjectHandle(0);
        let start = RgmPoint3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        };
        let mid = RgmPoint3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };
        let end = RgmPoint3 {
            x: -1.0,
            y: 0.0,
            z: 0.0,
        };

        assert_eq!(
            rgm_curve_create_arc_by_3_points(session, start, mid, end, tol(), &mut arc as *mut _),
            RgmStatus::Ok
        );

        let mut p0 = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut p1 = p0;
        assert_eq!(
            rgm_curve_point_at(session, arc, 0.0, &mut p0),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, arc, 1.0, &mut p1),
            RgmStatus::Ok
        );
        assert!(v3::distance(start, p0) < 1e-7);
        assert!(v3::distance(end, p1) < 1e-7);

        let center = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let mut p = RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            assert_eq!(rgm_curve_point_at(session, arc, t, &mut p), RgmStatus::Ok);
            assert!((v3::distance(center, p) - 1.0).abs() < 1e-6);
        }

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, arc, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - PI).abs() < 1e-7);

        assert_eq!(rgm_object_release(session, arc), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn polyline_length_sums_open_segments() {
        let session = create_session();
        let mut polyline = RgmObjectHandle(0);
        let points = [
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 10.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 10.0,
                y: 10.0,
                z: 0.0,
            },
        ];

        assert_eq!(
            rgm_curve_create_polyline(
                session,
                points.as_ptr(),
                points.len(),
                false,
                tol(),
                &mut polyline as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, polyline, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - 20.0).abs() < 1e-9);

        assert_eq!(rgm_object_release(session, polyline), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn polyline_length_includes_closing_segment_when_closed() {
        let session = create_session();
        let mut polyline = RgmObjectHandle(0);
        let points = [
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 10.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 10.0,
                y: 10.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            },
        ];

        assert_eq!(
            rgm_curve_create_polyline(
                session,
                points.as_ptr(),
                points.len(),
                true,
                tol(),
                &mut polyline as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, polyline, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - 40.0).abs() < 1e-7);

        assert_eq!(rgm_object_release(session, polyline), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn polycurve_length_sums_children() {
        let session = create_session();
        let mut line1 = RgmObjectHandle(0);
        let mut line2 = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 10.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line1 as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 10.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 20.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line2 as *mut _,
            ),
            RgmStatus::Ok
        );

        let segments = [
            RgmPolycurveSegment {
                curve: line1,
                reversed: false,
            },
            RgmPolycurveSegment {
                curve: line2,
                reversed: false,
            },
        ];
        let mut polycurve = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_polycurve(
                session,
                segments.as_ptr(),
                segments.len(),
                tol(),
                &mut polycurve as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut length = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, polycurve, &mut length as *mut _),
            RgmStatus::Ok
        );
        assert!((length - 20.0).abs() < 1e-9);

        assert_eq!(rgm_object_release(session, polycurve), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line2), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line1), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn point_coordinate_system_conversion_swaps_axes() {
        let session = create_session();
        let mut out = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        assert_eq!(
            rgm_point_convert_coordinate_system(
                session,
                10.0,
                20.0,
                30.0,
                RgmAlignmentCoordinateSystem::EastingNorthing as i32,
                RgmAlignmentCoordinateSystem::NorthingEasting as i32,
                &mut out as *mut _,
            ),
            RgmStatus::Ok
        );
        assert!((out.x - 20.0).abs() < 1e-12);
        assert!((out.y - 10.0).abs() < 1e-12);
        assert!((out.z - 30.0).abs() < 1e-12);

        assert_eq!(
            rgm_point_convert_coordinate_system(
                session,
                out.x,
                out.y,
                out.z,
                RgmAlignmentCoordinateSystem::NorthingEasting as i32,
                RgmAlignmentCoordinateSystem::EastingNorthing as i32,
                &mut out as *mut _,
            ),
            RgmStatus::Ok
        );
        assert!((out.x - 10.0).abs() < 1e-12);
        assert!((out.y - 20.0).abs() < 1e-12);
        assert!((out.z - 30.0).abs() < 1e-12);

        assert_eq!(
            rgm_point_convert_coordinate_system(session, 1.0, 2.0, 3.0, 42, 0, &mut out as *mut _),
            RgmStatus::InvalidInput
        );

        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn circle_constructor_is_periodic() {
        let session = create_session();
        let mut circle = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_circle(
                session,
                RgmCircle3 {
                    plane: RgmPlane {
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
                    },
                    radius: 5.0,
                },
                tol(),
                &mut circle as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut p0 = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut p1 = p0;
        assert_eq!(
            rgm_curve_point_at(session, circle, 0.0, &mut p0),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, circle, 1.0, &mut p1),
            RgmStatus::Ok
        );
        assert!(v3::distance(p0, p1) < 1e-6);
        for t in [0.0, 0.13, 0.27, 0.51, 0.79, 1.0] {
            let mut p = RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            assert_eq!(
                rgm_curve_point_at(session, circle, t, &mut p),
                RgmStatus::Ok
            );
            let r = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
            assert!((r - 5.0).abs() < 1e-5);
        }

        assert_eq!(rgm_object_release(session, circle), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn arc_constructor_preserves_radius_and_endpoints() {
        let session = create_session();
        let mut arc = RgmObjectHandle(0);

        let start = -0.4_f64;
        let sweep = 1.2_f64;
        let radius = 3.25_f64;
        let center = RgmPoint3 {
            x: 1.2,
            y: -0.7,
            z: 0.5,
        };

        assert_eq!(
            rgm_curve_create_arc(
                session,
                RgmArc3 {
                    plane: RgmPlane {
                        origin: center,
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
                    },
                    radius,
                    start_angle: start,
                    sweep_angle: sweep,
                },
                tol(),
                &mut arc as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut p0 = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut p1 = p0;
        let mut pm = p0;

        assert_eq!(
            rgm_curve_point_at(session, arc, 0.0, &mut p0),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, arc, 0.5, &mut pm),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, arc, 1.0, &mut p1),
            RgmStatus::Ok
        );

        let expected_start = RgmPoint3 {
            x: center.x + radius * start.cos(),
            y: center.y + radius * start.sin(),
            z: center.z,
        };
        let expected_end = RgmPoint3 {
            x: center.x + radius * (start + sweep).cos(),
            y: center.y + radius * (start + sweep).sin(),
            z: center.z,
        };

        assert!(v3::distance(p0, expected_start) < 1e-6);
        assert!(v3::distance(p1, expected_end) < 1e-6);

        for p in [p0, pm, p1] {
            let r = v3::distance(center, p);
            assert!((r - radius).abs() < 1e-5);
        }

        assert_eq!(rgm_object_release(session, arc), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn polycurve_is_global_curve() {
        let session = create_session();
        let mut line = RgmObjectHandle(0);
        let mut arc = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line as *mut _,
            ),
            RgmStatus::Ok
        );

        assert_eq!(
            rgm_curve_create_arc(
                session,
                RgmArc3 {
                    plane: RgmPlane {
                        origin: RgmPoint3 {
                            x: 1.0,
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
                    },
                    radius: 1.0,
                    start_angle: 0.0,
                    sweep_angle: FRAC_PI_2,
                },
                tol(),
                &mut arc as *mut _,
            ),
            RgmStatus::Ok
        );

        let segments = [
            RgmPolycurveSegment {
                curve: line,
                reversed: false,
            },
            RgmPolycurveSegment {
                curve: arc,
                reversed: false,
            },
        ];

        let mut poly = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_polycurve(
                session,
                segments.as_ptr(),
                segments.len(),
                tol(),
                &mut poly as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut p = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(
            rgm_curve_point_at(session, poly, 0.75, &mut p as *mut _),
            RgmStatus::Ok
        );

        let mut nurbs = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_to_nurbs(session, poly, &mut nurbs as *mut _),
            RgmStatus::Ok
        );
        let converted = debug_get_curve(session, nurbs).expect("converted curve exists");
        let CurveData::NurbsCurve(converted) = converted else {
            panic!("expected converted NURBS curve");
        };
        assert_eq!(converted.core.degree, 2);
        assert_eq!(converted.core.control_points.len(), 6);

        let total = 1.0 + FRAC_PI_2;
        for dist in [0.0, 0.25, 0.75, 1.25, total] {
            let mut from_poly = RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            let mut from_nurbs = from_poly;
            assert_eq!(
                rgm_curve_point_at_length(session, poly, dist, &mut from_poly as *mut _),
                RgmStatus::Ok
            );
            assert_eq!(
                rgm_curve_point_at_length(session, nurbs, dist, &mut from_nurbs as *mut _),
                RgmStatus::Ok
            );
            assert!(v3::distance(from_poly, from_nurbs) < 1e-6);
        }

        assert_eq!(rgm_object_release(session, nurbs), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, poly), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, arc), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn polycurve_to_nurbs_supports_mixed_degrees_exactly() {
        let session = create_session();
        let fit_points = [
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 1.0,
                y: 1.5,
                z: 0.0,
            },
            RgmPoint3 {
                x: 2.0,
                y: -1.0,
                z: 0.0,
            },
            RgmPoint3 {
                x: 3.5,
                y: 0.8,
                z: 0.0,
            },
            RgmPoint3 {
                x: 4.5,
                y: -0.3,
                z: 0.0,
            },
            RgmPoint3 {
                x: 5.5,
                y: 0.4,
                z: 0.0,
            },
        ];

        let mut cubic = RgmObjectHandle(0);
        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                fit_points.as_ptr(),
                fit_points.len(),
                3,
                false,
                tol(),
                &mut cubic as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut arc = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_arc(
                session,
                RgmArc3 {
                    plane: RgmPlane {
                        origin: RgmPoint3 {
                            x: 7.0,
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
                    },
                    radius: 1.2,
                    start_angle: PI,
                    sweep_angle: FRAC_PI_2,
                },
                tol(),
                &mut arc as *mut _,
            ),
            RgmStatus::Ok
        );

        let segments = [
            RgmPolycurveSegment {
                curve: cubic,
                reversed: false,
            },
            RgmPolycurveSegment {
                curve: arc,
                reversed: false,
            },
        ];
        let mut poly = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_polycurve(
                session,
                segments.as_ptr(),
                segments.len(),
                tol(),
                &mut poly as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut nurbs = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_to_nurbs(session, poly, &mut nurbs as *mut _),
            RgmStatus::Ok
        );

        let converted = debug_get_curve(session, nurbs).expect("converted curve exists");
        let CurveData::NurbsCurve(converted) = converted else {
            panic!("expected converted NURBS curve");
        };
        assert_eq!(converted.core.degree, 3);
        assert_eq!(converted.core.control_points.len(), 10);

        let mut total = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, poly, &mut total as *mut _),
            RgmStatus::Ok
        );
        for fraction in [0.0, 0.13, 0.27, 0.51, 0.79, 1.0] {
            let s = total * fraction;
            let mut from_poly = RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            let mut from_nurbs = from_poly;
            assert_eq!(
                rgm_curve_point_at_length(session, poly, s, &mut from_poly as *mut _),
                RgmStatus::Ok
            );
            assert_eq!(
                rgm_curve_point_at_length(session, nurbs, s, &mut from_nurbs as *mut _),
                RgmStatus::Ok
            );
            assert!(v3::distance(from_poly, from_nurbs) < 1e-6);
        }

        assert_eq!(rgm_object_release(session, nurbs), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, poly), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, arc), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, cubic), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn polycurve_to_nurbs_supports_periodic_segments() {
        let session = create_session();
        let fit_points = [
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
                x: 1.4,
                y: 0.8,
                z: 0.0,
            },
            RgmPoint3 {
                x: 0.4,
                y: 1.2,
                z: 0.0,
            },
            RgmPoint3 {
                x: -0.4,
                y: 0.6,
                z: 0.0,
            },
        ];

        let mut periodic = RgmObjectHandle(0);
        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                fit_points.as_ptr(),
                fit_points.len(),
                3,
                true,
                tol(),
                &mut periodic as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut line = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 2.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 3.0,
                        y: 0.4,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line as *mut _,
            ),
            RgmStatus::Ok
        );

        let segments = [
            RgmPolycurveSegment {
                curve: periodic,
                reversed: false,
            },
            RgmPolycurveSegment {
                curve: line,
                reversed: false,
            },
        ];
        let mut poly = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_polycurve(
                session,
                segments.as_ptr(),
                segments.len(),
                tol(),
                &mut poly as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut nurbs = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_to_nurbs(session, poly, &mut nurbs as *mut _),
            RgmStatus::Ok
        );

        let mut poly_total = 0.0_f64;
        let mut nurbs_total = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, poly, &mut poly_total as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_length(session, nurbs, &mut nurbs_total as *mut _),
            RgmStatus::Ok
        );
        assert!(poly_total > 0.0);
        assert!(nurbs_total > 0.0);
        assert!((poly_total - nurbs_total).abs() / poly_total.max(1e-9) < 0.12);

        let mut poly_start = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut nurbs_start = poly_start;
        let mut poly_end = poly_start;
        let mut nurbs_end = poly_start;
        assert_eq!(
            rgm_curve_point_at(session, poly, 0.0, &mut poly_start as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, nurbs, 0.0, &mut nurbs_start as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, poly, 1.0, &mut poly_end as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_point_at(session, nurbs, 1.0, &mut nurbs_end as *mut _),
            RgmStatus::Ok
        );
        assert!(v3::distance(poly_start, nurbs_start) < 0.15);
        assert!(v3::distance(poly_end, nurbs_end) < 0.15);

        assert_eq!(rgm_object_release(session, nurbs), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, poly), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, periodic), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn intersect_curve_plane_counts_expected_hits() {
        let session = create_session();
        let mut line_crossing = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    end: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0,
                    },
                },
                tol(),
                &mut line_crossing as *mut _,
            ),
            RgmStatus::Ok
        );

        let plane_xy = RgmPlane {
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

        let mut count = 0_u32;
        let mut points = [RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 8];
        assert_eq!(
            rgm_intersect_curve_plane(
                session,
                line_crossing,
                plane_xy,
                points.as_mut_ptr(),
                points.len() as u32,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(count, 1);
        assert!(
            v3::distance(
                points[0],
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                }
            ) < 1e-8
        );

        let mut line_parallel = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: -1.0,
                        y: 0.0,
                        z: 2.0,
                    },
                    end: RgmPoint3 {
                        x: 1.0,
                        y: 0.0,
                        z: 2.0,
                    },
                },
                tol(),
                &mut line_parallel as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_intersect_curve_plane(
                session,
                line_parallel,
                plane_xy,
                ptr::null_mut(),
                0,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(count, 0);

        let mut circle = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_circle(
                session,
                RgmCircle3 {
                    plane: RgmPlane {
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
                    },
                    radius: 1.0,
                },
                tol(),
                &mut circle as *mut _,
            ),
            RgmStatus::Ok
        );

        let tangent_plane = RgmPlane {
            origin: RgmPoint3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            x_axis: RgmVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            y_axis: RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            z_axis: RgmVec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        };
        assert_eq!(
            rgm_intersect_curve_plane(
                session,
                circle,
                tangent_plane,
                points.as_mut_ptr(),
                points.len() as u32,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(count, 1);
        assert!((points[0].x - 1.0).abs() < 5e-2);

        let secant_plane = RgmPlane {
            origin: RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            x_axis: RgmVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            y_axis: RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            z_axis: RgmVec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        };
        assert_eq!(
            rgm_intersect_curve_plane(
                session,
                circle,
                secant_plane,
                points.as_mut_ptr(),
                points.len() as u32,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(count, 2);
        assert!(points[0].x.abs() < 5e-2 && points[1].x.abs() < 5e-2);

        assert_eq!(rgm_object_release(session, circle), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line_parallel), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line_crossing), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn intersect_curve_curve_counts_expected_hits() {
        let session = create_session();
        let mut line_x = RgmObjectHandle(0);
        let mut line_y = RgmObjectHandle(0);
        let mut line_skew = RgmObjectHandle(0);

        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: -1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line_x as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: 0.0,
                        y: -1.0,
                        z: 0.0,
                    },
                    end: RgmPoint3 {
                        x: 0.0,
                        y: 1.0,
                        z: 0.0,
                    },
                },
                tol(),
                &mut line_y as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 {
                        x: -1.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    end: RgmPoint3 {
                        x: 1.0,
                        y: 0.0,
                        z: 1.0,
                    },
                },
                tol(),
                &mut line_skew as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut count = 0_u32;
        let mut points = [RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 8];
        assert_eq!(
            rgm_intersect_curve_curve(
                session,
                line_x,
                line_y,
                points.as_mut_ptr(),
                points.len() as u32,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(count, 1);
        assert!(
            v3::distance(
                points[0],
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0
                }
            ) < 1e-8
        );

        assert_eq!(
            rgm_intersect_curve_curve(
                session,
                line_y,
                line_skew,
                ptr::null_mut(),
                0,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(count, 0);

        assert_eq!(rgm_object_release(session, line_skew), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line_y), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line_x), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn threaded_sessions_are_isolated() {
        let threads: Vec<_> = (0..8)
            .map(|_| {
                std::thread::spawn(|| {
                    let session = create_session();
                    let points = sample_points();
                    let mut object = RgmObjectHandle(0);
                    let status = rgm_nurbs_interpolate_fit_points(
                        session,
                        points.as_ptr(),
                        points.len(),
                        2,
                        false,
                        tol(),
                        &mut object as *mut _,
                    );
                    assert_eq!(status, RgmStatus::Ok);
                    assert_eq!(rgm_object_release(session, object), RgmStatus::Ok);
                    assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
                })
            })
            .collect();

        for thread in threads {
            thread.join().expect("thread should complete");
        }
    }
}
