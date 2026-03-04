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
}
