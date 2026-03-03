    #[test]
    fn runtime_contract_curve_session_flow_matches_bindings_runtime() {
        let session = create_session();
        let points = runtime_curve_points();
        let mut curve = RgmObjectHandle(0);
        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                points.as_ptr(),
                points.len(),
                2,
                false,
                tol(),
                &mut curve as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut sampled = Vec::with_capacity(32);
        for idx in 0..32 {
            let t = idx as f64 / 31.0;
            let mut point = RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            assert_eq!(
                rgm_curve_point_at(session, curve, t, &mut point as *mut _),
                RgmStatus::Ok
            );
            sampled.push(point);
        }
        assert_eq!(sampled.len(), 32);
        assert!((sampled[0].x - 0.0).abs() < 1e-6);
        assert!((sampled[31].x - 3.0).abs() < 1e-6);

        let mut probe = RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        assert_eq!(
            rgm_curve_point_at(session, curve, 0.37, &mut probe as *mut _),
            RgmStatus::Ok
        );
        assert!(probe.x >= 0.0 && probe.x <= 3.0);

        let mut total_length = 0.0_f64;
        let mut length_at = 0.0_f64;
        assert_eq!(
            rgm_curve_length(session, curve, &mut total_length as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_length_at(session, curve, 0.37, &mut length_at as *mut _),
            RgmStatus::Ok
        );
        assert!(total_length > 0.0);
        assert!(length_at > 0.0 && length_at < total_length);

        let mut circle = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_circle(
                session,
                RgmCircle3 {
                    plane: RgmPlane {
                        origin: RgmPoint3 {
                            x: 1.25,
                            y: -0.8,
                            z: 0.4,
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
                    radius: 3.6,
                },
                tol(),
                &mut circle as *mut _,
            ),
            RgmStatus::Ok
        );

        for t in [0.0, 0.11, 0.3, 0.5, 0.77, 1.0] {
            let mut p = RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            assert_eq!(
                rgm_curve_point_at(session, circle, t, &mut p as *mut _),
                RgmStatus::Ok
            );
            let dx = p.x - 1.25;
            let dy = p.y + 0.8;
            let dz = p.z - 0.4;
            let radius = (dx * dx + dy * dy + dz * dz).sqrt();
            assert!((radius - 3.6).abs() < 1e-3);
        }

        let mut line_a = RgmObjectHandle(0);
        let mut line_b = RgmObjectHandle(0);
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
                &mut line_a as *mut _,
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
                &mut line_b as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut curve_hits = [RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 8];
        let mut curve_hit_count = 0_u32;
        assert_eq!(
            rgm_intersect_curve_curve(
                session,
                line_a,
                line_b,
                curve_hits.as_mut_ptr(),
                curve_hits.len() as u32,
                &mut curve_hit_count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(curve_hit_count, 1);
        assert!(curve_hits[0].x.abs() < 1e-3);
        assert!(curve_hits[0].y.abs() < 1e-3);

        let mut plane_hits = [RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 8];
        let mut plane_hit_count = 0_u32;
        assert_eq!(
            rgm_intersect_curve_plane(
                session,
                line_a,
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
                        y: 1.0,
                        z: 0.0,
                    },
                },
                plane_hits.as_mut_ptr(),
                plane_hits.len() as u32,
                &mut plane_hit_count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert!(plane_hit_count >= 1);

        assert_eq!(rgm_object_release(session, line_b), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, line_a), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, circle), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, curve), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
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

