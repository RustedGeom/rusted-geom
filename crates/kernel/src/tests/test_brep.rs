    #[test]
    fn brep_native_roundtrip_and_queries_work() {
        let session = create_session();
        let surface = create_warped_surface(session, 14, 11, 10.0, 8.0, 0.6);

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
            RgmUv2 { u: 0.05, v: 0.06 },
            RgmUv2 { u: 0.95, v: 0.07 },
            RgmUv2 { u: 0.94, v: 0.93 },
            RgmUv2 { u: 0.06, v: 0.94 },
        ];
        let mut outer_loop_id = 0_u32;
        assert_eq!(
            rgm_brep_add_loop_uv(
                session,
                brep,
                face_id,
                outer.as_ptr(),
                outer.len(),
                true,
                &mut outer_loop_id as *mut _,
            ),
            RgmStatus::Ok
        );

        let hole = [
            RgmUv2 { u: 0.24, v: 0.24 },
            RgmUv2 { u: 0.44, v: 0.24 },
            RgmUv2 { u: 0.44, v: 0.44 },
            RgmUv2 { u: 0.24, v: 0.44 },
        ];
        let mut hole_loop_id = 0_u32;
        assert_eq!(
            rgm_brep_add_loop_uv(
                session,
                brep,
                face_id,
                hole.as_ptr(),
                hole.len(),
                false,
                &mut hole_loop_id as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut shell_id = 0_u32;
        assert_eq!(
            rgm_brep_finalize_shell(session, brep, &mut shell_id as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(shell_id, 0);

        let mut solid_id = 0_u32;
        assert_eq!(
            rgm_brep_finalize_solid(session, brep, &mut solid_id as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(solid_id, 0);

        let mut state = u32::MAX;
        assert_eq!(
            rgm_brep_state(session, brep, &mut state as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(state, 1);

        let mut face_count = 0_u32;
        assert_eq!(
            rgm_brep_face_count(session, brep, &mut face_count as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(face_count, 1);

        let mut shell_count = 0_u32;
        assert_eq!(
            rgm_brep_shell_count(session, brep, &mut shell_count as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(shell_count, 1);

        let mut solid_count = 0_u32;
        assert_eq!(
            rgm_brep_solid_count(session, brep, &mut solid_count as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(solid_count, 1);

        let mut is_solid = false;
        assert_eq!(
            rgm_brep_is_solid(session, brep, &mut is_solid as *mut _),
            RgmStatus::Ok
        );
        assert!(is_solid);

        let mut report = RgmBrepValidationReport::default();
        assert_eq!(
            rgm_brep_validate(session, brep, &mut report as *mut _),
            RgmStatus::Ok
        );
        assert!(report.issue_count <= 16);

        let mut fixed_count = 0_u32;
        assert_eq!(
            rgm_brep_heal(session, brep, &mut fixed_count as *mut _),
            RgmStatus::Ok
        );

        let mut area = 0.0_f64;
        assert_eq!(
            rgm_brep_estimate_area(session, brep, &mut area as *mut _),
            RgmStatus::Ok
        );
        assert!(area.is_finite());
        assert!(area > 0.0);

        let tess_options = RgmSurfaceTessellationOptions {
            min_u_segments: 16,
            min_v_segments: 16,
            max_u_segments: 40,
            max_v_segments: 36,
            chord_tol: 2e-4,
            normal_tol_rad: 0.09,
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
        let mut tri_count = 0_u32;
        assert_eq!(
            rgm_mesh_triangle_count(session, mesh, &mut tri_count as *mut _),
            RgmStatus::Ok
        );
        assert!(tri_count > 0);

        let mut byte_count = 0_u32;
        assert_eq!(
            rgm_brep_save_native(
                session,
                brep,
                ptr::null_mut(),
                0,
                &mut byte_count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert!(byte_count > 0);

        let mut bytes = vec![0_u8; byte_count as usize];
        assert_eq!(
            rgm_brep_save_native(
                session,
                brep,
                bytes.as_mut_ptr(),
                byte_count,
                &mut byte_count as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut loaded = RgmObjectHandle(0);
        assert_eq!(
            rgm_brep_load_native(
                session,
                bytes.as_ptr(),
                byte_count as usize,
                &mut loaded as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut loaded_face_count = 0_u32;
        assert_eq!(
            rgm_brep_face_count(session, loaded, &mut loaded_face_count as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(loaded_face_count, face_count);

        let mut loaded_solid_count = 0_u32;
        assert_eq!(
            rgm_brep_solid_count(session, loaded, &mut loaded_solid_count as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(loaded_solid_count, solid_count);

        assert_eq!(rgm_object_release(session, mesh), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, loaded), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, brep), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

    #[test]
    fn brep_bridge_clone_and_adjacency_exports_work() {
        let session = create_session();
        let surface = create_bilinear_surface(session, 0.0, 0.0, 0.0, 0.0);

        let mut face_a = RgmObjectHandle(0);
        let mut face_b = RgmObjectHandle(0);
        assert_eq!(
            rgm_face_create_from_surface(session, surface, &mut face_a as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_face_create_from_surface(session, surface, &mut face_b as *mut _),
            RgmStatus::Ok
        );
        add_outer_rect_loop(session, face_a);
        add_outer_rect_loop(session, face_b);

        let faces = [face_a, face_b];
        let mut brep = RgmObjectHandle(0);
        assert_eq!(
            rgm_brep_create_from_faces(session, faces.as_ptr(), faces.len(), &mut brep as *mut _),
            RgmStatus::Ok
        );

        let mut adjacency_count = 0_u32;
        assert_eq!(
            rgm_brep_face_adjacency(
                session,
                brep,
                0,
                ptr::null_mut(),
                0,
                &mut adjacency_count as *mut _,
            ),
            RgmStatus::Ok
        );
        assert!(adjacency_count <= 8);

        let mut adjacency = [0_u32; 8];
        assert_eq!(
            rgm_brep_face_adjacency(
                session,
                brep,
                0,
                adjacency.as_mut_ptr(),
                adjacency.len() as u32,
                &mut adjacency_count as *mut _,
            ),
            RgmStatus::Ok
        );
        let adjacency_len = adjacency_count as usize;
        for &face_id in adjacency.iter().take(adjacency_len) {
            assert!(face_id < 2);
            assert_ne!(face_id, 0);
        }

        let mut shell_id = 0_u32;
        assert_eq!(
            rgm_brep_finalize_shell(session, brep, &mut shell_id as *mut _),
            RgmStatus::Ok
        );

        let mut solid_id = 0_u32;
        assert_eq!(
            rgm_brep_finalize_solid(session, brep, &mut solid_id as *mut _),
            RgmStatus::Ok
        );

        let mut cloned = RgmObjectHandle(0);
        assert_eq!(
            rgm_brep_clone(session, brep, &mut cloned as *mut _),
            RgmStatus::Ok
        );

        let mut solid_count = 0_u32;
        assert_eq!(
            rgm_brep_solid_count(session, cloned, &mut solid_count as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(solid_count, 1);

        let mut cloned_face_count = 0_u32;
        assert_eq!(
            rgm_brep_face_count(session, cloned, &mut cloned_face_count as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(cloned_face_count, 2);

        let mut extracted_face = RgmObjectHandle(0);
        assert_eq!(
            rgm_brep_extract_face_object(session, cloned, 0, &mut extracted_face as *mut _),
            RgmStatus::Ok
        );
        let mut extracted_valid = false;
        assert_eq!(
            rgm_face_validate(session, extracted_face, &mut extracted_valid as *mut _),
            RgmStatus::Ok
        );
        assert!(extracted_valid);

        let mut bridged = RgmObjectHandle(0);
        assert_eq!(
            rgm_brep_from_face_object(session, extracted_face, &mut bridged as *mut _),
            RgmStatus::Ok
        );
        let mut bridged_count = 0_u32;
        assert_eq!(
            rgm_brep_face_count(session, bridged, &mut bridged_count as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(bridged_count, 1);

        assert_eq!(rgm_object_release(session, bridged), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, extracted_face), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, cloned), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, brep), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, face_b), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, face_a), RgmStatus::Ok);
        assert_eq!(rgm_object_release(session, surface), RgmStatus::Ok);
        assert_eq!(rgm_kernel_destroy(session), RgmStatus::Ok);
    }

