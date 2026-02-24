    #[test]
    fn runtime_contract_mesh_flow_matches_bindings_runtime() {
        let session = create_session();

        let mut mesh_box = RgmObjectHandle(0);
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
                    y: 3.0,
                    z: 2.0,
                },
                &mut mesh_box as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut vertex_count = 0_u32;
        let mut triangle_count = 0_u32;
        assert_eq!(
            rgm_mesh_vertex_count(session, mesh_box, &mut vertex_count as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_mesh_triangle_count(session, mesh_box, &mut triangle_count as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(vertex_count, 8);
        assert_eq!(triangle_count, 12);

        let mut translated = RgmObjectHandle(0);
        let mut transformed = RgmObjectHandle(0);
        let mut baked = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_translate(
                session,
                mesh_box,
                RgmVec3 {
                    x: 0.8,
                    y: -0.4,
                    z: 1.2,
                },
                &mut translated as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_mesh_rotate(
                session,
                translated,
                RgmVec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.2,
                },
                0.7,
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                &mut transformed as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_mesh_bake_transform(session, transformed, &mut baked as *mut _),
            RgmStatus::Ok
        );

        let mut baked_vertices = vec![
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            16
        ];
        let mut baked_indices = vec![0_u32; 72];
        let mut baked_vertex_count = 0_u32;
        let mut baked_index_count = 0_u32;
        assert_eq!(
            rgm_mesh_copy_vertices(
                session,
                baked,
                baked_vertices.as_mut_ptr(),
                baked_vertices.len() as u32,
                &mut baked_vertex_count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_mesh_copy_indices(
                session,
                baked,
                baked_indices.as_mut_ptr(),
                baked_indices.len() as u32,
                &mut baked_index_count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(baked_vertex_count, 8);
        assert_eq!(baked_index_count, 36);

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
                1.1,
                28,
                20,
                &mut torus as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut mesh_plane_hits = vec![
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            16_384
        ];
        let mut mesh_plane_hit_count = 0_u32;
        assert_eq!(
            rgm_intersect_mesh_plane(
                session,
                torus,
                RgmPlane {
                    origin: RgmPoint3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.2,
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
                mesh_plane_hits.as_mut_ptr(),
                mesh_plane_hits.len() as u32,
                &mut mesh_plane_hit_count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert!(mesh_plane_hit_count > 0);
        assert_eq!(mesh_plane_hit_count % 2, 0);

        let mut sphere = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_create_uv_sphere(
                session,
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                4.2,
                24,
                16,
                &mut sphere as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut mesh_mesh_hits = vec![
            RgmPoint3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            32_768
        ];
        let mut mesh_mesh_hit_count = 0_u32;
        assert_eq!(
            rgm_intersect_mesh_mesh(
                session,
                sphere,
                torus,
                mesh_mesh_hits.as_mut_ptr(),
                mesh_mesh_hits.len() as u32,
                &mut mesh_mesh_hit_count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert!(mesh_mesh_hit_count > 0);
        assert_eq!(mesh_mesh_hit_count % 2, 0);

        let mut boolean_host = RgmObjectHandle(0);
        let mut inner_torus = RgmObjectHandle(0);
        let mut boolean_diff = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_create_box(
                session,
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                RgmVec3 {
                    x: 8.8,
                    y: 8.8,
                    z: 8.8,
                },
                &mut boolean_host as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_mesh_create_torus(
                session,
                RgmPoint3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                2.4,
                0.8,
                32,
                24,
                &mut inner_torus as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_mesh_boolean(
                session,
                boolean_host,
                inner_torus,
                2,
                &mut boolean_diff as *mut _,
            ),
            RgmStatus::Ok
        );
        let mut boolean_triangles = 0_u32;
        assert_eq!(
            rgm_mesh_triangle_count(session, boolean_diff, &mut boolean_triangles as *mut _),
            RgmStatus::Ok
        );
        assert!(boolean_triangles > 0);

        assert_eq!(rgm_object_release(session, boolean_diff), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, inner_torus), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, boolean_host), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, sphere), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, torus), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, baked), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, transformed), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, translated), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, mesh_box), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

