    #[test]
    fn runtime_contract_surface_face_flow_matches_bindings_runtime() {
        let session = create_session();
        let points = runtime_surface_points();
        let weights = [1.0_f64; 9];
        let knots_u = clamped_uniform_knots(3, 2);
        let knots_v = clamped_uniform_knots(3, 2);

        let mut surface = RgmObjectHandle(0);
        assert_eq!(
            rgm_surface_create_nurbs(
                session,
                &RgmNurbsSurfaceDesc {
                    degree_u: 2,
                    degree_v: 2,
                    periodic_u: false,
                    periodic_v: false,
                    control_u_count: 3,
                    control_v_count: 3,
                } as *const _,
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

        let uv = RgmUv2 { u: 0.5, v: 0.5 };
        let mut frame = RgmSurfaceEvalFrame {
            point: RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            du: RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            dv: RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            normal: RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        };
        let mut d0 = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut d1u = RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut d1v = RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut duu = RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut duv = RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut dvv = RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };

        assert_eq!(
            rgm_surface_frame_at(session, surface, &uv as *const _, &mut frame as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_surface_point_at(session, surface, &uv as *const _, &mut d0 as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_surface_d1_at(
                session,
                surface,
                &uv as *const _,
                &mut d1u as *mut _,
                &mut d1v as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_surface_d2_at(
                session,
                surface,
                &uv as *const _,
                &mut duu as *mut _,
                &mut duv as *mut _,
                &mut dvv as *mut _,
            ),
            RgmStatus::Ok
        );

        assert!((frame.point.x - d0.x).abs() < 1e-7);
        assert!((frame.point.y - d0.y).abs() < 1e-7);
        assert!((frame.point.z - d0.z).abs() < 1e-7);
        assert!((frame.du.x - d1u.x).abs() < 1e-7);
        assert!((frame.dv.y - d1v.y).abs() < 1e-7);
        assert!(duu.x.is_finite());
        assert!(duv.y.is_finite());
        assert!(dvv.z.is_finite());
        assert!(frame.point.x.is_finite());
        assert!(frame.normal.z.is_finite());

        let mut face = RgmObjectHandle(0);
        assert_eq!(
            rgm_face_create_from_surface(session, surface, &mut face as *mut _),
            RgmStatus::Ok
        );
        let outer_loop = [
            RgmUv2 { u: 0.05, v: 0.05 },
            RgmUv2 { u: 0.95, v: 0.05 },
            RgmUv2 { u: 0.95, v: 0.95 },
            RgmUv2 { u: 0.05, v: 0.95 },
        ];
        assert_eq!(
            rgm_face_add_loop(session, face, outer_loop.as_ptr(), outer_loop.len(), true),
            RgmStatus::Ok
        );

        let mut trim_circle = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_circle(
                session,
                RgmCircle3 {
                    plane: RgmPlane {
                        origin: RgmPoint3 {
                            x: 0.5,
                            y: 0.5,
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
                    radius: 0.18,
                },
                tol(),
                &mut trim_circle as *mut _,
            ),
            RgmStatus::Ok
        );

        let loop_input = RgmTrimLoopInput {
            edge_count: 1,
            is_outer: false,
        };
        let trim_edges = [RgmTrimEdgeInput {
            start_uv: RgmUv2 { u: 0.68, v: 0.5 },
            end_uv: RgmUv2 { u: 0.68, v: 0.5 },
            curve_3d: trim_circle,
            has_curve_3d: true,
        }];
        assert_eq!(
            rgm_face_add_loop_edges(
                session,
                face,
                &loop_input as *const _,
                trim_edges.as_ptr(),
                trim_edges.len(),
            ),
            RgmStatus::Ok
        );

        let mut valid = false;
        assert_eq!(
            rgm_face_validate(session, face, &mut valid as *mut _),
            RgmStatus::Ok
        );
        assert!(valid);

        let tess_opts = RgmSurfaceTessellationOptions {
            min_u_segments: 28,
            min_v_segments: 28,
            max_u_segments: 48,
            max_v_segments: 48,
            chord_tol: 1e-4,
            normal_tol_rad: 0.08,
        };
        let mut face_mesh = RgmObjectHandle(0);
        assert_eq!(
            rgm_face_tessellate_to_mesh(
                session,
                face,
                &tess_opts as *const _,
                &mut face_mesh as *mut _,
            ),
            RgmStatus::Ok
        );
        let mut tri_count = 0_u32;
        assert_eq!(
            rgm_mesh_triangle_count(session, face_mesh, &mut tri_count as *mut _),
            RgmStatus::Ok
        );
        assert!(tri_count > 0);

        let mut intersection = RgmObjectHandle(0);
        assert_eq!(
            rgm_intersect_surface_plane(
                session,
                surface,
                &RgmPlane {
                    origin: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.1,
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
                } as *const _,
                &mut intersection as *mut _,
            ),
            RgmStatus::Ok
        );
        let mut branch_count = 0_u32;
        assert_eq!(
            rgm_intersection_branch_count(session, intersection, &mut branch_count as *mut _),
            RgmStatus::Ok
        );

        assert_eq!(rgm_object_release(session, intersection), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, face_mesh), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, trim_circle), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, face), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
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

