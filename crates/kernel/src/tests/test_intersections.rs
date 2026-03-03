    #[test]
    fn runtime_contract_pointer_style_intersection_exports_match_bindings_runtime() {
        let session = create_session();
        let mut line = RgmObjectHandle(0);
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
                &mut line as *mut _,
            ),
            RgmStatus::Ok
        );

        let plane = RgmPlane {
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

        let mut points = [RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 8];
        let mut count = 0_u32;
        assert_eq!(
            rgm_intersect_curve_plane(
                session,
                line,
                plane,
                points.as_mut_ptr(),
                points.len() as u32,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(count, 1);
        assert!(points[0].x.abs() < 1e-6);
        assert!(points[0].y.abs() < 1e-6);
        assert!(points[0].z.abs() < 1e-6);

        assert_eq!(
            rgm_intersect_curve_curve(
                session,
                line,
                line,
                points.as_mut_ptr(),
                points.len() as u32,
                &mut count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert!(count >= 1);

        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
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
            total_hits >= 3,
            "viewer surface-curve should produce multiple hits (elapsed={elapsed:?}, branches={}, hits={total_hits})",
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

