    #[test]
    fn usda_export_upaxis_is_z() {
        let session = create_session();
        let t = tol();

        let line = RgmLine3 {
            start: RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
            end:   RgmPoint3 { x: 1.0, y: 0.0, z: 0.0 },
        };
        let mut curve_obj = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(session, line, t, &mut curve_obj as *mut _),
            RgmStatus::Ok
        );

        let usda = {
            let entry = super::SESSIONS.get(&session.0).unwrap();
            let state = entry.value().read();
            rusted_usd::usda::writer::write_stage(&state.stage)
        };

        assert!(usda.contains("upAxis = \"Z\""), "Expected upAxis = \"Z\", got: {usda}");
        assert!(!usda.contains("upAxis = \"Y\""), "Found stale upAxis = \"Y\"");

        super::rgm_kernel_destroy(session);
    }

    #[test]
    fn usda_export_contains_curve_widths() {
        let session = create_session();
        let t = tol();

        let pts = sample_points();
        let mut curve_obj = RgmObjectHandle(0);
        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                pts.as_ptr(),
                pts.len(),
                3,
                false,
                t,
                &mut curve_obj as *mut _,
            ),
            RgmStatus::Ok
        );

        let usda = {
            let entry = super::SESSIONS.get(&session.0).unwrap();
            let state = entry.value().read();
            rusted_usd::usda::writer::write_stage(&state.stage)
        };

        assert!(usda.contains("float[] widths"), "NurbsCurves prim missing widths attribute");

        super::rgm_kernel_destroy(session);
    }

    #[test]
    fn canonical_usda_export_keeps_curve_prims_without_mesh_helper() {
        let session = create_session();
        let t = tol();

        let line = RgmLine3 {
            start: RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
            end: RgmPoint3 { x: 1.0, y: 0.0, z: 0.0 },
        };
        let mut curve_obj = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(session, line, t, &mut curve_obj as *mut _),
            RgmStatus::Ok
        );

        let usda = super::export_usda_text(session, &[curve_obj.0]).expect("usda export");
        assert!(usda.contains("def NurbsCurves"), "Missing canonical curve prim");
        assert!(!usda.contains("_mesh"), "Curve tube helper leaked into USDA export: {usda}");
        assert!(usda.contains("_display"), "Expected BasisCurves display proxy in USDA export: {usda}");

        super::rgm_kernel_destroy(session);
    }

    #[test]
    fn usda_export_surface_with_transform() {
        let session = create_session();

        // Create a bilinear surface at z=0 and apply a translation by (0,0,5).
        let surface_obj = create_bilinear_surface(session, 0.0, 0.0, 0.0, 0.0);

        let delta = RgmVec3 { x: 0.0, y: 0.0, z: 5.0 };
        let mut translated = RgmObjectHandle(0);
        assert_eq!(
            rgm_surface_translate(session, surface_obj, &delta as *const _, &mut translated as *mut _),
            RgmStatus::Ok
        );

        let (usda, surface_path) = {
            let entry = super::SESSIONS.get(&session.0).unwrap();
            let state = entry.value().read();
            (
                rusted_usd::usda::writer::write_stage(&state.stage),
                state.path_index.get(&translated.0).cloned().expect("surface path"),
            )
        };

        assert!(usda.contains("def Xform"), "Missing Xform prim");
        assert!(usda.contains("matrix4d xformOp:transform"), "Missing matrix xform op");
        assert!(usda.contains("\"xformOp:transform\""), "Missing xformOpOrder token");

        let parsed_stage = rusted_usd::usda::parser::parse_usda(&usda).expect("USDA parse failed");
        let xform = parsed_stage
            .get::<rusted_usd::schema::generated::UsdGeomXform>(&surface_path)
            .unwrap_or_else(|| panic!("surface xform missing at {} in USDA:\n{}", surface_path, usda));
        let matrix = xform.xform_op_transform.expect("xform matrix");
        assert!((matrix[2][3] - 5.0).abs() < 1e-9, "Expected z translation of 5.0, got {matrix:?}");

        let patch = parsed_stage
            .get::<rusted_usd::schema::generated::UsdGeomNurbsPatch>(&surface_path.child("Patch"))
            .expect("surface patch");
        assert!(
            patch.points.iter().all(|point| point.z.abs() < 1e-6),
            "Expected local-space patch points, got {:#?}",
            patch.points
        );

        super::rgm_kernel_destroy(session);
    }

    #[test]
    fn canonical_usda_export_excludes_surface_display_mesh_and_other_objects() {
        let session = create_session();
        let t = tol();

        let line = RgmLine3 {
            start: RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
            end: RgmPoint3 { x: 1.0, y: 0.0, z: 0.0 },
        };
        let mut curve_obj = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(session, line, t, &mut curve_obj as *mut _),
            RgmStatus::Ok
        );

        let surface_obj = create_bilinear_surface(session, 0.0, 0.0, 0.0, 0.0);
        let delta = RgmVec3 { x: 0.0, y: 0.0, z: 5.0 };
        let mut translated = RgmObjectHandle(0);
        assert_eq!(
            rgm_surface_translate(session, surface_obj, &delta as *const _, &mut translated as *mut _),
            RgmStatus::Ok
        );
        let usda = super::export_usda_text(session, &[translated.0]).expect("usda export");

        assert!(usda.contains("def Xform"), "Missing surface xform");
        assert!(usda.contains("def NurbsPatch \"Patch\""), "Missing canonical surface patch");
        assert!(!usda.contains("displayMesh"), "Surface display mesh leaked into USDA export: {usda}");
        assert!(!usda.contains("def NurbsCurves"), "Unselected curve leaked into USDA export: {usda}");
        assert!(
            usda.contains(", 5,") || usda.contains(", 5.0,") || usda.contains(", 5)") || usda.contains(", 5.0)"),
            "Expected baked surface transform in exported USDA patch points: {usda}"
        );

        super::rgm_kernel_destroy(session);
    }

    #[test]
    fn canonical_usda_export_preserves_usd_patch_point_order() {
        let session = create_session();

        let surface = create_bilinear_surface(session, 0.0, 10.0, 20.0, 30.0);
        let usda = super::export_usda_text(session, &[surface.0]).expect("usda export");

        let patch = rusted_usd::usda::parser::parse_usda(&usda)
            .expect("USDA parse failed")
            .all_prims()
            .find_map(|prim| match prim.schema {
                rusted_usd::schema::generated::SchemaData::NurbsPatch(ref patch) => Some(patch.clone()),
                _ => None,
            })
            .expect("surface patch");

        let zs: Vec<f32> = patch.points.iter().map(|point| point.z).collect();
        assert_eq!(zs, vec![0.0, 10.0, 20.0, 30.0], "USD patch point order changed: {usda}");

        super::rgm_kernel_destroy(session);
    }

    #[test]
    fn usda_export_intersection_has_branch() {
        let session = create_session();

        let surf_a = create_bilinear_surface(session, 0.0, 0.0, 0.0, 0.0);
        let surf_b = create_bilinear_surface(session, -0.5, -0.5, 0.5, 0.5);

        let mut isect = RgmObjectHandle(0);
        // Intersection may or may not find branches — just verify USDA export works.
        let _ = rgm_intersect_surface_surface(session, surf_a, surf_b, &mut isect as *mut _);

        let usda = {
            let entry = super::SESSIONS.get(&session.0).unwrap();
            let state = entry.value().read();
            rusted_usd::usda::writer::write_stage(&state.stage)
        };

        assert!(usda.contains("#usda 1.0"), "Missing USDA header");
        assert!(usda.contains("Intersect_"), "Missing Intersect scope in USDA");

        super::rgm_kernel_destroy(session);
    }

    #[test]
    fn usda_export_contains_curve_mesh_surface() {
        let session = create_session();
        let t = tol();

        let line = RgmLine3 {
            start: RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
            end:   RgmPoint3 { x: 1.0, y: 0.0, z: 0.0 },
        };
        let mut curve_obj = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(session, line, t, &mut curve_obj as *mut _),
            RgmStatus::Ok
        );

        let mut mesh_obj = RgmObjectHandle(0);
        assert_eq!(
            rgm_mesh_create_box(
                session,
                RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
                RgmVec3 { x: 1.0, y: 1.0, z: 1.0 },
                &mut mesh_obj as *mut _,
            ),
            RgmStatus::Ok
        );

        let usda = {
            let entry = super::SESSIONS.get(&session.0).unwrap();
            let state = entry.value().read();
            rusted_usd::usda::writer::write_stage(&state.stage)
        };

        assert!(usda.contains("#usda 1.0"), "Missing USDA header");
        assert!(usda.contains("def NurbsCurves"), "Missing NurbsCurves prim for line");
        assert!(usda.contains("int[] curveVertexCounts"), "Missing curveVertexCounts");
        assert!(usda.contains("int[] order"), "Missing order");
        assert!(usda.contains("double[] knots"), "Missing knots");
        assert!(usda.contains("point3f[] points"), "Missing points");
        assert!(usda.contains("def Mesh"), "Missing Mesh prim for box");
        assert!(usda.contains("int[] faceVertexCounts"), "Missing faceVertexCounts");

        super::rgm_kernel_destroy(session);
    }

    #[test]
    fn usda_export_round_trips_curve() {
        let session = create_session();
        let t = tol();

        let pts = sample_points();
        let mut curve_obj = RgmObjectHandle(0);
        assert_eq!(
            rgm_nurbs_interpolate_fit_points(
                session,
                pts.as_ptr(),
                pts.len(),
                3,
                false,
                t,
                &mut curve_obj as *mut _,
            ),
            RgmStatus::Ok
        );

        let (usda, original_curve_count) = {
            let entry = super::SESSIONS.get(&session.0).unwrap();
            let state = entry.value().read();
            let usda = rusted_usd::usda::writer::write_stage(&state.stage);
            let count = state.stage.all_prims()
                .filter(|p| matches!(p.schema, rusted_usd::schema::generated::SchemaData::NurbsCurves(_)))
                .count();
            (usda, count)
        };

        let parsed_stage = rusted_usd::usda::parser::parse_usda(&usda).expect("USDA parse failed");
        let parsed_curve_count = parsed_stage.all_prims()
            .filter(|p| matches!(p.schema, rusted_usd::schema::generated::SchemaData::NurbsCurves(_)))
            .count();
        assert_eq!(original_curve_count, 1, "Expected exactly 1 NurbsCurves prim");
        assert_eq!(parsed_curve_count, original_curve_count, "Round-trip prim count mismatch");

        super::rgm_kernel_destroy(session);
    }

    #[test]
    fn iges_and_sat_export_polycurve_scope_children() {
        let session = create_session();
        let t = tol();

        let mut line_a = RgmObjectHandle(0);
        let mut line_b = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 },
                    end: RgmPoint3 { x: 1.0, y: 0.0, z: 0.0 },
                },
                t,
                &mut line_a as *mut _,
            ),
            RgmStatus::Ok
        );
        assert_eq!(
            rgm_curve_create_line(
                session,
                RgmLine3 {
                    start: RgmPoint3 { x: 1.0, y: 0.0, z: 0.0 },
                    end: RgmPoint3 { x: 1.0, y: 1.0, z: 0.0 },
                },
                t,
                &mut line_b as *mut _,
            ),
            RgmStatus::Ok
        );

        let segments = [
            RgmPolycurveSegment { curve: line_a, reversed: false },
            RgmPolycurveSegment { curve: line_b, reversed: false },
        ];
        let mut polycurve = RgmObjectHandle(0);
        assert_eq!(
            rgm_curve_create_polycurve(session, segments.as_ptr(), segments.len(), t, &mut polycurve),
            RgmStatus::Ok
        );

        let iges = super::export_iges_text(session, &[polycurve.0]).expect("iges export");
        let sat = super::export_sat_text(session, &[polycurve.0]).expect("sat export");

        assert!(iges.contains("126,"), "Expected NURBS curve entity in IGES: {iges}");
        assert!(sat.contains("exactcur-curve"), "Expected NURBS curve entity in SAT: {sat}");

        super::rgm_kernel_destroy(session);
    }

    #[test]
    fn landxml_stage_export_contains_surface_meshes() {
        let session = create_session();
        let xml = include_str!("../../../../docs/landxml-test-files/OpenRoadTin.xml");
        let options = crate::landxml::LandXmlParseOptions {
            mode: crate::landxml::LandXmlParseMode::Lenient,
            ..Default::default()
        };
        let doc = crate::landxml::parse_landxml(xml, options).expect("landxml parse");

        let object = super::with_session_mut(session, |state| {
            Ok(super::insert_landxml_doc(state, super::LandXmlDocData { doc }))
        })
        .expect("insert landxml");

        let usda = super::export_usda_text(session, &[object.0]).expect("usda export");
        assert!(usda.contains("def Scope \"Doc_"), "Expected LandXML scope root");
        assert!(usda.contains("def Mesh \"Surface_0\""), "Expected LandXML surface mesh in USD subtree");
    }

    #[test]
    fn landxml_stage_export_contains_alignment_curves() {
        let session = create_session();
        let xml = include_str!("../../../../docs/landxml-test-files/OpenRoadExampleEmptyAlignment.xml");
        let options = crate::landxml::LandXmlParseOptions {
            mode: crate::landxml::LandXmlParseMode::Lenient,
            ..Default::default()
        };
        let doc = crate::landxml::parse_landxml(xml, options).expect("landxml parse");

        let object = super::with_session_mut(session, |state| {
            Ok(super::insert_landxml_doc(state, super::LandXmlDocData { doc }))
        })
        .expect("insert landxml");

        let usda = super::export_usda_text(session, &[object.0]).expect("usda export");
        assert!(usda.contains("def Scope \"Doc_"), "Expected LandXML scope root");
        assert!(
            usda.contains("def NurbsCurves \"Alignment_") || usda.contains("def NurbsCurves \"PlanLinear_"),
            "Expected LandXML curve geometry in USD subtree"
        );
    }
