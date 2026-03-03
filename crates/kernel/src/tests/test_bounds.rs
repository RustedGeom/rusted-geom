    #[test]
    fn bounds_curve_line_exactness_and_axis_alignment() {
        let session = create_session();
        let start = RgmPoint3 {
            x: -3.0,
            y: 1.25,
            z: 4.5,
        };
        let end = RgmPoint3 {
            x: 6.0,
            y: -2.75,
            z: -1.5,
        };
        let mut line = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(session, RgmLine3 { start, end }, tol(), &mut line as *mut _),
            RgmStatus::Ok
        );

        let options = RgmBoundsOptions {
            mode: RgmBoundsMode::Fast,
            sample_budget: 0,
            padding: 0.0,
        };
        let mut bounds = RgmBounds3 {
            world_aabb: RgmAabb3 { min: start, max: end },
            world_obb: RgmObb3 {
                center: start,
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
                half_extents: RgmVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            local_aabb: RgmAabb3 { min: start, max: end },
        };
        assert_eq!(
            rgm_object_compute_bounds(session, line, Some(options), &mut bounds as *mut _),
            RgmStatus::Ok
        );

        let expected_min = RgmPoint3 {
            x: start.x.min(end.x),
            y: start.y.min(end.y),
            z: start.z.min(end.z),
        };
        let expected_max = RgmPoint3 {
            x: start.x.max(end.x),
            y: start.y.max(end.y),
            z: start.z.max(end.z),
        };
        assert!((bounds.world_aabb.min.x - expected_min.x).abs() <= 2e-8);
        assert!((bounds.world_aabb.min.y - expected_min.y).abs() <= 2e-8);
        assert!((bounds.world_aabb.min.z - expected_min.z).abs() <= 2e-8);
        assert!((bounds.world_aabb.max.x - expected_max.x).abs() <= 2e-8);
        assert!((bounds.world_aabb.max.y - expected_max.y).abs() <= 2e-8);
        assert!((bounds.world_aabb.max.z - expected_max.z).abs() <= 2e-8);

        let dir = v3::normalize(point_delta(end, start)).expect("line direction");
        let alignment = v3::dot(bounds.world_obb.x_axis, dir).abs();
        assert!(alignment >= 0.999, "OBB principal axis misaligned: {alignment}");
        assert_points_inside_local_aabb(bounds, &[start, end], 1e-8);

        assert_eq!(rgm_object_release(session, line), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn bounds_surface_fast_contains_dense_samples() {
        let session = create_session();
        let surface = create_warped_surface(session, 7, 6, 9.0, 7.0, 0.9);
        let options = RgmBoundsOptions {
            mode: RgmBoundsMode::Fast,
            sample_budget: 0,
            padding: 0.0,
        };
        let mut bounds = RgmBounds3 {
            world_aabb: RgmAabb3 {
                min: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            world_obb: RgmObb3 {
                center: RgmPoint3 {
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
                half_extents: RgmVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            local_aabb: RgmAabb3 {
                min: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        };
        assert_eq!(
            rgm_object_compute_bounds(session, surface, Some(options), &mut bounds as *mut _),
            RgmStatus::Ok
        );

        for iu in 0..=30 {
            let u = iu as f64 / 30.0;
            for iv in 0..=30 {
                let v = iv as f64 / 30.0;
                let mut point = RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                };
                assert_eq!(
                    rgm_surface_point_at(
                        session,
                        surface,
                        &RgmUv2 { u, v } as *const _,
                        &mut point as *mut _,
                    ),
                    RgmStatus::Ok
                );
                assert!(
                    point_inside_aabb(bounds.world_aabb, point, 1e-7),
                    "surface sample out of fast AABB at u={u:.3}, v={v:.3}: ({:.6},{:.6},{:.6})",
                    point.x,
                    point.y,
                    point.z
                );
            }
        }

        assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn bounds_mesh_transform_matches_transformed_local_aabb() {
        let session = create_session();
        let mut mesh = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_create_box(
                session,
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                RgmVec3 {
                    x: 4.0,
                    y: 2.0,
                    z: 3.0,
                },
                &mut mesh as *mut _,
            ),
            RgmStatus::Ok
        );
        let mut translated = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_translate(
                session,
                mesh,
                RgmVec3 {
                    x: 1.5,
                    y: -0.75,
                    z: 2.25,
                },
                &mut translated as *mut _,
            ),
            RgmStatus::Ok
        );
        let mut rotated = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_rotate(
                session,
                translated,
                RgmVec3 {
                    x: 0.3,
                    y: 1.0,
                    z: 0.4,
                },
                0.62,
                RgmPoint3 {
                    x: 0.2,
                    y: -0.3,
                    z: 0.0,
                },
                &mut rotated as *mut _,
            ),
            RgmStatus::Ok
        );
        let mut scaled = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_scale(
                session,
                rotated,
                RgmVec3 {
                    x: 1.25,
                    y: 0.85,
                    z: 1.4,
                },
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                &mut scaled as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut bounds = RgmBounds3 {
            world_aabb: RgmAabb3 {
                min: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            world_obb: RgmObb3 {
                center: RgmPoint3 {
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
                half_extents: RgmVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            local_aabb: RgmAabb3 {
                min: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        };
        assert_eq!(
            rgm_object_compute_bounds(
                session,
                scaled,
                Some(RgmBoundsOptions {
                    mode: RgmBoundsMode::Fast,
                    sample_budget: 256,
                    padding: 0.0,
                }),
                &mut bounds as *mut _,
            ),
            RgmStatus::Ok
        );

        let mesh_data = debug_get_mesh(session, scaled).expect("mesh exists");
        let local_aabb =
            crate::math::bounds::aabb_from_points(&mesh_data.vertices).expect("local mesh aabb");
        let corners = crate::math::bounds::aabb_corners(local_aabb);
        let mut env_min = RgmPoint3 {
            x: f64::INFINITY,
            y: f64::INFINITY,
            z: f64::INFINITY,
        };
        let mut env_max = RgmPoint3 {
            x: f64::NEG_INFINITY,
            y: f64::NEG_INFINITY,
            z: f64::NEG_INFINITY,
        };
        let mut world_points = Vec::with_capacity(mesh_data.vertices.len());
        for vertex in &mesh_data.vertices {
            world_points.push(matrix_apply_point(mesh_data.transform, *vertex));
        }
        for corner in corners {
            let world = matrix_apply_point(mesh_data.transform, corner);
            env_min.x = env_min.x.min(world.x);
            env_min.y = env_min.y.min(world.y);
            env_min.z = env_min.z.min(world.z);
            env_max.x = env_max.x.max(world.x);
            env_max.y = env_max.y.max(world.y);
            env_max.z = env_max.z.max(world.z);
        }
        assert!((bounds.world_aabb.min.x - env_min.x).abs() <= 1e-8);
        assert!((bounds.world_aabb.min.y - env_min.y).abs() <= 1e-8);
        assert!((bounds.world_aabb.min.z - env_min.z).abs() <= 1e-8);
        assert!((bounds.world_aabb.max.x - env_max.x).abs() <= 1e-8);
        assert!((bounds.world_aabb.max.y - env_max.y).abs() <= 1e-8);
        assert!((bounds.world_aabb.max.z - env_max.z).abs() <= 1e-8);
        for point in &world_points {
            assert!(point_inside_aabb(bounds.world_aabb, *point, 1e-8));
        }

        assert_eq!(rgm_object_release(session, scaled), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, rotated), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, translated), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, mesh), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn bounds_brep_fast_contains_tessellated_vertices() {
        let session = create_session();
        let surface = create_warped_surface(session, 10, 9, 8.0, 7.0, 0.65);
        let mut brep = RgmObjectHandle(0);
        assert_eq!(
            rgm_brep_create_empty(session, &mut brep as *mut _),
            RgmStatus::Ok
        );
        let mut face_id = 0_u32;
        assert_eq!(
            rgm_brep_add_face_from_surface(session, brep, surface, &mut face_id as *mut _),
            RgmStatus::Ok
        );
        let outer = [
            RgmUv2 { u: 0.05, v: 0.05 },
            RgmUv2 { u: 0.95, v: 0.07 },
            RgmUv2 { u: 0.94, v: 0.94 },
            RgmUv2 { u: 0.06, v: 0.92 },
        ];
        let mut outer_loop = 0_u32;
        assert_eq!(
            rgm_brep_add_loop_uv(
                session,
                brep,
                face_id,
                outer.as_ptr(),
                outer.len(),
                true,
                &mut outer_loop as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut bounds = RgmBounds3 {
            world_aabb: RgmAabb3 {
                min: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            world_obb: RgmObb3 {
                center: RgmPoint3 {
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
                half_extents: RgmVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            local_aabb: RgmAabb3 {
                min: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        };
        assert_eq!(
            rgm_object_compute_bounds(
                session,
                brep,
                Some(RgmBoundsOptions {
                    mode: RgmBoundsMode::Fast,
                    sample_budget: 0,
                    padding: 0.0,
                }),
                &mut bounds as *mut _,
            ),
            RgmStatus::Ok
        );

        let tess_options = RgmSurfaceTessellationOptions {
            min_u_segments: 14,
            min_v_segments: 14,
            max_u_segments: 32,
            max_v_segments: 32,
            chord_tol: 2e-4,
            normal_tol_rad: 0.1,
        };
        let mut mesh = RgmObjectHandle(0);
        assert_eq!(
            rgm_brep_tessellate_to_mesh(
                session,
                brep,
                &tess_options as *const _,
                &mut mesh as *mut _,
            ),
            RgmStatus::Ok
        );
        let mesh_data = debug_get_mesh(session, mesh).expect("tess mesh exists");
        let mut sampled_world = Vec::with_capacity(mesh_data.vertices.len());
        for vertex in &mesh_data.vertices {
            let world = matrix_apply_point(mesh_data.transform, *vertex);
            sampled_world.push(world);
            assert!(
                point_inside_aabb(bounds.world_aabb, world, 1e-6),
                "BREP fast bounds must contain tess vertex ({:.6}, {:.6}, {:.6})",
                world.x,
                world.y,
                world.z
            );
        }
        assert_points_inside_local_aabb(bounds, &sampled_world, 1e-5);

        assert_eq!(rgm_object_release(session, mesh), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, brep), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn bounds_optimal_obb_volume_not_worse_than_fast() {
        let session = create_session();
        let mut torus = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_create_torus(
                session,
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                3.8,
                1.15,
                48,
                36,
                &mut torus as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut fast = RgmBounds3 {
            world_aabb: RgmAabb3 {
                min: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            world_obb: RgmObb3 {
                center: RgmPoint3 {
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
                half_extents: RgmVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            local_aabb: RgmAabb3 {
                min: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        };
        let mut optimal = fast;
        assert_eq!(
            rgm_object_compute_bounds(
                session,
                torus,
                Some(RgmBoundsOptions {
                    mode: RgmBoundsMode::Fast,
                    sample_budget: 512,
                    padding: 0.0,
                }),
                &mut fast as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_object_compute_bounds(
                session,
                torus,
                Some(RgmBoundsOptions {
                    mode: RgmBoundsMode::Optimal,
                    sample_budget: 4096,
                    padding: 0.0,
                }),
                &mut optimal as *mut _,
            ),
            RgmStatus::Ok
        );
        assert!(obb_volume(optimal) <= obb_volume(fast) + 1e-6);

        assert_eq!(rgm_object_release(session, torus), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn bounds_local_aabb_consistency_for_transformed_mesh() {
        let session = create_session();
        let mut sphere = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_create_uv_sphere(
                session,
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                2.8,
                32,
                20,
                &mut sphere as *mut _,
            ),
            RgmStatus::Ok
        );
        let mut rotated = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_rotate(
                session,
                sphere,
                RgmVec3 {
                    x: 0.6,
                    y: 1.0,
                    z: 0.2,
                },
                0.8,
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                &mut rotated as *mut _,
            ),
            RgmStatus::Ok
        );
        let mut moved = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_translate(
                session,
                rotated,
                RgmVec3 {
                    x: 1.7,
                    y: -0.9,
                    z: 0.6,
                },
                &mut moved as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut bounds = RgmBounds3 {
            world_aabb: RgmAabb3 {
                min: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            world_obb: RgmObb3 {
                center: RgmPoint3 {
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
                half_extents: RgmVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            local_aabb: RgmAabb3 {
                min: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        };
        assert_eq!(
            rgm_object_compute_bounds(
                session,
                moved,
                Some(RgmBoundsOptions {
                    mode: RgmBoundsMode::Optimal,
                    sample_budget: 4096,
                    padding: 0.0,
                }),
                &mut bounds as *mut _,
            ),
            RgmStatus::Ok
        );

        let mesh_data = debug_get_mesh(session, moved).expect("mesh exists");
        let mut world_points = Vec::with_capacity(mesh_data.vertices.len());
        for vertex in &mesh_data.vertices {
            world_points.push(matrix_apply_point(mesh_data.transform, *vertex));
        }
        assert_points_inside_local_aabb(bounds, &world_points, 1e-6);

        assert_eq!(rgm_object_release(session, moved), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, rotated), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, sphere), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn bounds_errors_for_unsupported_object_and_empty_brep() {
        let session = create_session();
        let surface = create_bilinear_surface(session, 0.0, 0.0, 0.0, 0.0);
        let mut face = RgmObjectHandle(0);
        assert_eq!(
            rgm_face_create_from_surface(session, surface, &mut face as *mut _),
            RgmStatus::Ok
        );
        let mut bounds = RgmBounds3 {
            world_aabb: RgmAabb3 {
                min: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            world_obb: RgmObb3 {
                center: RgmPoint3 {
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
                half_extents: RgmVec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            local_aabb: RgmAabb3 {
                min: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                max: RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        };
        assert_eq!(
            rgm_object_compute_bounds(session, face, None, &mut bounds as *mut _),
            RgmStatus::InvalidInput
        );

        let mut empty_brep = RgmObjectHandle(0);
        assert_eq!(
            rgm_brep_create_empty(session, &mut empty_brep as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_object_compute_bounds(session, empty_brep, None, &mut bounds as *mut _),
            RgmStatus::InvalidInput
        );

        assert_eq!(rgm_object_release(session, empty_brep), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, face), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

