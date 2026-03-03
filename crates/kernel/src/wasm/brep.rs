//! B-rep assembly and query methods on `KernelSession`.

use super::{flat_to_uv, BrepHandle, FaceHandle, MeshHandle, SurfaceHandle, KernelSession};
use crate::{
    rgm_brep_add_face, rgm_brep_add_face_from_surface, rgm_brep_add_loop_uv, rgm_brep_clone,
    rgm_brep_create_empty, rgm_brep_create_from_faces, rgm_brep_create_from_surface,
    rgm_brep_estimate_area, rgm_brep_extract_face_object, rgm_brep_face_adjacency,
    rgm_brep_face_count, rgm_brep_finalize_shell, rgm_brep_finalize_solid, rgm_brep_from_face_object,
    rgm_brep_heal, rgm_brep_is_solid, rgm_brep_load_native, rgm_brep_save_native,
    rgm_brep_shell_count, rgm_brep_solid_count, rgm_brep_state, rgm_brep_tessellate_to_mesh,
    rgm_brep_validate, RgmBrepValidationReport, RgmObjectHandle,
};
use wasm_bindgen::prelude::*;

/// Compact validation result from `brep_validate`.
#[wasm_bindgen]
pub struct BrepValidationResult {
    /// Total number of issues found.
    pub issue_count: u32,
    /// Maximum severity: 0 = Info, 1 = Warning, 2 = Error.
    pub max_severity: u32,
    /// Whether the issue array overflowed (more than 16 issues exist).
    pub overflow: bool,
}

#[wasm_bindgen]
impl KernelSession {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Create an empty B-rep.
    pub fn brep_create_empty(&self) -> Result<BrepHandle, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_brep_create_empty(self.handle(), &mut out))?;
        Ok(BrepHandle::new(self.session_id, out.0))
    }

    /// Create a B-rep from a list of face object IDs (as f64 values).
    pub fn brep_create_from_faces(&self, face_ids: Vec<f64>) -> Result<BrepHandle, JsValue> {
        let handles: Vec<RgmObjectHandle> = face_ids.iter().map(|&id| RgmObjectHandle(id as u64)).collect();
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_brep_create_from_faces(
            self.handle(), handles.as_ptr(), handles.len(), &mut out,
        ))?;
        Ok(BrepHandle::new(self.session_id, out.0))
    }

    /// Create a B-rep from a single surface.
    pub fn brep_create_from_surface(&self, surface: &SurfaceHandle) -> Result<BrepHandle, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_brep_create_from_surface(
            self.handle(), RgmObjectHandle(surface.object_id), &mut out,
        ))?;
        Ok(BrepHandle::new(self.session_id, out.0))
    }

    // ── Building ──────────────────────────────────────────────────────────────

    /// Add a face object to a B-rep.  Returns the internal face ID.
    pub fn brep_add_face(&self, brep: &BrepHandle, face: &FaceHandle) -> Result<u32, JsValue> {
        let mut face_id = 0u32;
        super::error::check(rgm_brep_add_face(
            self.handle(), RgmObjectHandle(brep.object_id),
            RgmObjectHandle(face.object_id), &mut face_id,
        ))?;
        Ok(face_id)
    }

    /// Add a surface directly to a B-rep as a new face.  Returns the internal face ID.
    pub fn brep_add_face_from_surface(
        &self,
        brep: &BrepHandle,
        surface: &SurfaceHandle,
    ) -> Result<u32, JsValue> {
        let mut face_id = 0u32;
        super::error::check(rgm_brep_add_face_from_surface(
            self.handle(), RgmObjectHandle(brep.object_id),
            RgmObjectHandle(surface.object_id), &mut face_id,
        ))?;
        Ok(face_id)
    }

    /// Add a UV trim loop to a face in a B-rep.
    ///
    /// `points_uv` is a flat `[u,v, …]` array.
    /// Returns the internal loop ID.
    pub fn brep_add_loop_uv(
        &self,
        brep: &BrepHandle,
        face_id: u32,
        points_uv: Vec<f64>,
        is_outer: bool,
    ) -> Result<u32, JsValue> {
        let uvs = flat_to_uv(&points_uv);
        let mut loop_id = 0u32;
        super::error::check(rgm_brep_add_loop_uv(
            self.handle(), RgmObjectHandle(brep.object_id),
            face_id, uvs.as_ptr(), uvs.len(), is_outer, &mut loop_id,
        ))?;
        Ok(loop_id)
    }

    /// Finalize an open shell in the B-rep.  Returns the shell ID.
    pub fn brep_finalize_shell(&self, brep: &BrepHandle) -> Result<u32, JsValue> {
        let mut shell_id = 0u32;
        super::error::check(rgm_brep_finalize_shell(
            self.handle(), RgmObjectHandle(brep.object_id), &mut shell_id,
        ))?;
        Ok(shell_id)
    }

    /// Finalize a closed solid in the B-rep.  Returns the solid ID.
    pub fn brep_finalize_solid(&self, brep: &BrepHandle) -> Result<u32, JsValue> {
        let mut solid_id = 0u32;
        super::error::check(rgm_brep_finalize_solid(
            self.handle(), RgmObjectHandle(brep.object_id), &mut solid_id,
        ))?;
        Ok(solid_id)
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// Validate the topology of a B-rep.
    pub fn brep_validate(&self, brep: &BrepHandle) -> Result<BrepValidationResult, JsValue> {
        let mut report = RgmBrepValidationReport::default();
        super::error::check(rgm_brep_validate(
            self.handle(), RgmObjectHandle(brep.object_id), &mut report,
        ))?;
        Ok(BrepValidationResult {
            issue_count:  report.issue_count,
            max_severity: report.max_severity as u32,
            overflow:     report.overflow,
        })
    }

    /// Attempt to heal a B-rep.  Returns the number of issues fixed.
    pub fn brep_heal(&self, brep: &BrepHandle) -> Result<u32, JsValue> {
        let mut fixed = 0u32;
        super::error::check(rgm_brep_heal(
            self.handle(), RgmObjectHandle(brep.object_id), &mut fixed,
        ))?;
        Ok(fixed)
    }

    /// Clone a B-rep.
    pub fn brep_clone(&self, brep: &BrepHandle) -> Result<BrepHandle, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_brep_clone(
            self.handle(), RgmObjectHandle(brep.object_id), &mut out,
        ))?;
        Ok(BrepHandle::new(self.session_id, out.0))
    }

    /// Number of faces in a B-rep.
    pub fn brep_face_count(&self, brep: &BrepHandle) -> Result<u32, JsValue> {
        let mut count = 0u32;
        super::error::check(rgm_brep_face_count(
            self.handle(), RgmObjectHandle(brep.object_id), &mut count,
        ))?;
        Ok(count)
    }

    /// Number of shells in a B-rep.
    pub fn brep_shell_count(&self, brep: &BrepHandle) -> Result<u32, JsValue> {
        let mut count = 0u32;
        super::error::check(rgm_brep_shell_count(
            self.handle(), RgmObjectHandle(brep.object_id), &mut count,
        ))?;
        Ok(count)
    }

    /// Number of solids in a B-rep.
    pub fn brep_solid_count(&self, brep: &BrepHandle) -> Result<u32, JsValue> {
        let mut count = 0u32;
        super::error::check(rgm_brep_solid_count(
            self.handle(), RgmObjectHandle(brep.object_id), &mut count,
        ))?;
        Ok(count)
    }

    /// Whether the B-rep represents a closed solid.
    pub fn brep_is_solid(&self, brep: &BrepHandle) -> Result<bool, JsValue> {
        let mut is_solid = false;
        super::error::check(rgm_brep_is_solid(
            self.handle(), RgmObjectHandle(brep.object_id), &mut is_solid,
        ))?;
        Ok(is_solid)
    }

    /// Adjacent face IDs for a given face.  Returns a `Vec<u32>`.
    pub fn brep_face_adjacency(
        &self,
        brep: &BrepHandle,
        face_id: u32,
    ) -> Result<Vec<u32>, JsValue> {
        // Phase 1: count
        let mut count = 0u32;
        rgm_brep_face_adjacency(
            self.handle(), RgmObjectHandle(brep.object_id),
            face_id, std::ptr::null_mut(), 0, &mut count,
        );
        if count == 0 {
            return Ok(Vec::new());
        }
        // Phase 2: fill
        let mut ids = vec![0u32; count as usize];
        let mut actual = 0u32;
        super::error::check(rgm_brep_face_adjacency(
            self.handle(), RgmObjectHandle(brep.object_id),
            face_id, ids.as_mut_ptr(), count, &mut actual,
        ))?;
        ids.truncate(actual as usize);
        Ok(ids)
    }

    /// Tessellate a B-rep to a mesh.
    pub fn brep_tessellate_to_mesh(
        &self,
        brep: &BrepHandle,
        options: Vec<f64>,
    ) -> Result<MeshHandle, JsValue> {
        let opts = super::surface::parse_tess_options(&options);
        let opts_ptr = opts.as_ref().map(|o| o as *const _).unwrap_or(std::ptr::null());
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_brep_tessellate_to_mesh(
            self.handle(), RgmObjectHandle(brep.object_id), opts_ptr, &mut out,
        ))?;
        Ok(MeshHandle::new(self.session_id, out.0))
    }

    /// B-rep state integer (0 = empty, 1 = has faces, 2 = shell, 3 = solid).
    pub fn brep_state(&self, brep: &BrepHandle) -> Result<u32, JsValue> {
        let mut state = 0u32;
        super::error::check(rgm_brep_state(
            self.handle(), RgmObjectHandle(brep.object_id), &mut state,
        ))?;
        Ok(state)
    }

    /// Estimated surface area of a B-rep.
    pub fn brep_estimate_area(&self, brep: &BrepHandle) -> Result<f64, JsValue> {
        let mut area = 0.0f64;
        super::error::check(rgm_brep_estimate_area(
            self.handle(), RgmObjectHandle(brep.object_id), &mut area,
        ))?;
        Ok(area)
    }

    // ── Conversion ────────────────────────────────────────────────────────────

    /// Promote a face object into a B-rep.
    pub fn brep_from_face_object(&self, face: &FaceHandle) -> Result<BrepHandle, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_brep_from_face_object(
            self.handle(), RgmObjectHandle(face.object_id), &mut out,
        ))?;
        Ok(BrepHandle::new(self.session_id, out.0))
    }

    /// Extract a face from a B-rep as a standalone face object.
    pub fn brep_extract_face_object(
        &self,
        brep: &BrepHandle,
        face_id: u32,
    ) -> Result<FaceHandle, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_brep_extract_face_object(
            self.handle(), RgmObjectHandle(brep.object_id), face_id, &mut out,
        ))?;
        Ok(FaceHandle::new(self.session_id, out.0))
    }

    // ── Serialisation ─────────────────────────────────────────────────────────

    /// Serialise a B-rep to native binary format.  Returns the bytes as `Vec<u8>`.
    pub fn brep_save_native(&self, brep: &BrepHandle) -> Result<Vec<u8>, JsValue> {
        // Phase 1: count bytes
        let mut byte_count = 0u32;
        rgm_brep_save_native(
            self.handle(), RgmObjectHandle(brep.object_id),
            std::ptr::null_mut(), 0, &mut byte_count,
        );
        if byte_count == 0 {
            return Ok(Vec::new());
        }
        // Phase 2: fill
        let mut buf = vec![0u8; byte_count as usize];
        let mut actual = 0u32;
        super::error::check(rgm_brep_save_native(
            self.handle(), RgmObjectHandle(brep.object_id),
            buf.as_mut_ptr(), byte_count, &mut actual,
        ))?;
        buf.truncate(actual as usize);
        Ok(buf)
    }

    /// Deserialise a B-rep from native binary bytes.
    pub fn brep_load_native(&self, bytes: Vec<u8>) -> Result<BrepHandle, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_brep_load_native(
            self.handle(), bytes.as_ptr(), bytes.len(), &mut out,
        ))?;
        Ok(BrepHandle::new(self.session_id, out.0))
    }
}
