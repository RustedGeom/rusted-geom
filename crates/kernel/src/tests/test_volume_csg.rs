    #[test]
    fn mesh_box_volume_is_correct() {
        let session = create_session();

        let mut mesh_box = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_create_box(
                session,
                RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
                RgmVec3 { x: 2.0, y: 3.0, z: 4.0 },
                &mut mesh_box as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut volume = 0.0_f64;
        assert_eq!(
            rgm_mesh_volume(session, mesh_box, &mut volume as *mut _),
            RgmStatus::Ok
        );
        assert!((volume - 24.0).abs() < 0.5, "Box volume {volume} should be ~24.0");
    }

    #[test]
    fn csg_union_produces_mesh() {
        let session = create_session();

        let mut mesh_a = RgmObjectHandle(0);
        let mut mesh_b = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_create_box(
                session,
                RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
                RgmVec3 { x: 2.0, y: 2.0, z: 2.0 },
                &mut mesh_a as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_mesh_create_box(
                session,
                RgmPoint3 { x: 1.0, y: 0.0, z: 0.0 },
                RgmVec3 { x: 2.0, y: 2.0, z: 2.0 },
                &mut mesh_b as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut result = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_boolean(session, mesh_a, mesh_b, 0, &mut result as *mut _),
            RgmStatus::Ok
        );

        let mut vert_count = 0_u32;
        let mut tri_count = 0_u32;
        assert_eq!(
            rgm_mesh_vertex_count(session, result, &mut vert_count as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_mesh_triangle_count(session, result, &mut tri_count as *mut _),
            RgmStatus::Ok
        );
        assert!(vert_count > 0, "CSG union should produce vertices");
        assert!(tri_count > 0, "CSG union should produce triangles");
    }

    #[test]
    fn csg_intersection_volume_smaller_than_union() {
        let session = create_session();

        let mut mesh_a = RgmObjectHandle(0);
        let mut mesh_b = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_create_box(
                session,
                RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
                RgmVec3 { x: 2.0, y: 2.0, z: 2.0 },
                &mut mesh_a as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_mesh_create_box(
                session,
                RgmPoint3 { x: 0.5, y: 0.0, z: 0.0 },
                RgmVec3 { x: 2.0, y: 2.0, z: 2.0 },
                &mut mesh_b as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut union_mesh = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_boolean(session, mesh_a, mesh_b, 0, &mut union_mesh as *mut _),
            RgmStatus::Ok
        );

        let mut inter_mesh = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_boolean(session, mesh_a, mesh_b, 1, &mut inter_mesh as *mut _),
            RgmStatus::Ok
        );

        let mut union_vol = 0.0_f64;
        let mut inter_vol = 0.0_f64;
        assert_eq!(
            rgm_mesh_volume(session, union_mesh, &mut union_vol as *mut _),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_mesh_volume(session, inter_mesh, &mut inter_vol as *mut _),
            RgmStatus::Ok
        );

        assert!(
            inter_vol < union_vol,
            "Intersection volume ({inter_vol}) should be less than union volume ({union_vol})"
        );
    }

    #[test]
    fn csg_difference_produces_mesh() {
        let session = create_session();

        let mut mesh_a = RgmObjectHandle(0);
        let mut mesh_b = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_create_box(
                session,
                RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
                RgmVec3 { x: 4.0, y: 4.0, z: 4.0 },
                &mut mesh_a as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_mesh_create_box(
                session,
                RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
                RgmVec3 { x: 2.0, y: 2.0, z: 2.0 },
                &mut mesh_b as *mut _,
            ),
            RgmStatus::Ok
        );

        let mut result = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_boolean(session, mesh_a, mesh_b, 2, &mut result as *mut _),
            RgmStatus::Ok
        );

        let mut vol = 0.0_f64;
        assert_eq!(
            rgm_mesh_volume(session, result, &mut vol as *mut _),
            RgmStatus::Ok
        );
        // 4^3 - 2^3 = 64 - 8 = 56
        assert!(vol > 40.0 && vol < 70.0, "Difference volume {vol} should be ~56.0");
    }
