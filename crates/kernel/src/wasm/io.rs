//! CAD and mesh export methods on `KernelSession`.

use super::KernelSession;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl KernelSession {
    /// Export selected objects to IGES 5.3 text format.
    ///
    /// `object_ids` is a flat array of object IDs (obtained from `.object_id()`
    /// on any handle). Curves become Entity 126, surfaces become Entity 128,
    /// faces/BReps become trimmed surfaces (Entity 144).
    /// Returns the complete IGES file as a string.
    pub fn export_iges(&self, object_ids: Vec<f64>) -> Result<String, JsValue> {
        let ids: Vec<u64> = object_ids.iter().map(|&v| v as u64).collect();
        crate::export_iges_text(self.handle(), &ids)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Export selected objects to ACIS SAT text format (version 7.0).
    ///
    /// `object_ids` is a flat array of object IDs. Curves, surfaces, and B-rep
    /// solids are exported with full topology. Returns the complete SAT file as
    /// a string.
    pub fn export_sat(&self, object_ids: Vec<f64>) -> Result<String, JsValue> {
        let ids: Vec<u64> = object_ids.iter().map(|&v| v as u64).collect();
        crate::export_sat_text(self.handle(), &ids)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Export selected mesh objects to ASCII STL format.
    ///
    /// `object_ids` is a flat array of object IDs. Only mesh objects are
    /// included; other geometry types are silently skipped.
    /// Returns the complete STL file as a string.
    pub fn export_stl(&self, object_ids: Vec<f64>) -> Result<String, JsValue> {
        let ids: Vec<u64> = object_ids.iter().map(|&v| v as u64).collect();
        crate::export_stl_text(self.handle(), &ids)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Export selected mesh objects to glTF 2.0 format.
    ///
    /// `object_ids` is a flat array of object IDs. Only mesh objects are
    /// included. The returned string is a glTF JSON file with an embedded
    /// base64 binary buffer.
    pub fn export_gltf(&self, object_ids: Vec<f64>) -> Result<String, JsValue> {
        let ids: Vec<u64> = object_ids.iter().map(|&v| v as u64).collect();
        crate::export_gltf_text(self.handle(), &ids)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Export the current session's USD stage to USDA text format.
    ///
    /// All geometry is written to the stage at insertion time, so this
    /// simply serializes the current stage contents.
    pub fn export_usda(&self) -> Result<String, JsValue> {
        let entry = crate::session::store::SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let state = entry.value().read();
        let canonical = crate::canonical_stage_for_usd_export(&state.stage);
        Ok(rusted_usd::usda::writer::write_stage(&canonical))
    }

    /// Export selected objects to USDA text format.
    ///
    /// `object_ids` is a flat array of object IDs. Only the prims registered
    /// for the given IDs are included in the output.
    pub fn export_usda_prims(&self, object_ids: Vec<f64>) -> Result<String, JsValue> {
        let ids: Vec<u64> = object_ids.iter().map(|&v| v as u64).collect();
        crate::export_usda_text(self.handle(), &ids)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Import a USDA text document into this session.
    ///
    /// Parses the USDA and reconstructs geometry objects for each recognised
    /// prim (NurbsCurves → curve, NurbsPatch → surface, Mesh → mesh).
    /// The new objects are appended to the session; existing objects are not
    /// modified.
    pub fn import_usda(&self, text: String) -> Result<(), JsValue> {
        use rusted_usd::schema::generated::SchemaData;

        let parsed_stage = rusted_usd::usda::parser::parse_usda(&text)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        let entry = crate::session::store::SESSIONS
            .get(&self.session_id)
            .ok_or_else(|| JsValue::from_str("session not found"))?;
        let mut state = entry.value().write();

        for prim in parsed_stage.all_prims() {
            let parent_is_xform = prim
                .path
                .parent()
                .and_then(|parent| parsed_stage.get::<rusted_usd::schema::generated::UsdGeomXform>(&parent))
                .is_some();
            match &prim.schema {
                SchemaData::NurbsCurves(curves) => {
                    for i in 0..curves.curve_vertex_counts.len() {
                        if let Ok(core) = crate::nurbs_core_from_curves_prim(curves, i) {
                            let world = crate::world_transform_for_path(&parsed_stage, &prim.path);
                            let core = crate::transform_curve_core(&core, world);
                            let closed = core.periodic;
                            let arc = crate::math::arc_length::ArcLengthCache::default();
                            let curve_data = crate::session::objects::CurveData::NurbsCurve(
                                crate::session::objects::NurbsCurveData {
                                    core,
                                    closed,
                                    fit_points: Vec::new(),
                                    arc_length: arc,
                                },
                            );
                            crate::session::store::insert_curve(&mut state, curve_data);
                        }
                    }
                }
                SchemaData::NurbsPatch(patch) => {
                    if let Ok(core) = crate::nurbs_core_from_patch_prim(patch) {
                        let transform = crate::world_transform_for_path(&parsed_stage, &prim.path);
                        let surface_data = crate::session::objects::SurfaceData {
                            core,
                            transform,
                        };
                        crate::session::store::insert_surface(&mut state, surface_data);
                    }
                }
                SchemaData::Mesh(mesh) => {
                    if parent_is_xform && prim.path.name() != "Geom" {
                        continue;
                    }
                    let mut mesh_data = crate::mesh_data_from_prim(mesh);
                    mesh_data.transform = crate::world_transform_for_path(&parsed_stage, &prim.path);
                    crate::session::store::insert_mesh(&mut state, mesh_data);
                }
                _ => {}
            }
        }

        Ok(())
    }
}
