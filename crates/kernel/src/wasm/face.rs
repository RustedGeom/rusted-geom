//! Face (trimmed surface) constructor and manipulation methods on `KernelSession`.

use super::{flat_to_uv, FaceHandle, MeshHandle, SurfaceHandle, KernelSession};
use crate::{
    rgm_face_add_loop, rgm_face_add_loop_edges, rgm_face_create_from_surface, rgm_face_heal,
    rgm_face_remove_loop, rgm_face_reverse_loop, rgm_face_split_trim_edge,
    rgm_face_tessellate_to_mesh, rgm_face_validate, RgmObjectHandle, RgmTrimEdgeInput,
    RgmTrimLoopInput, RgmUv2,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl KernelSession {
    /// Create a trimmed face from a surface (initially untrimmed).
    pub fn create_face_from_surface(
        &self,
        surface: &SurfaceHandle,
    ) -> Result<FaceHandle, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_face_create_from_surface(
            self.handle(), RgmObjectHandle(surface.object_id), &mut out,
        ))?;
        Ok(FaceHandle::new(self.session_id, out.0))
    }

    /// Add a trim loop to a face from UV points.
    ///
    /// `points_uv` is a flat `[u,v, …]` array.  `is_outer` = true for the outer boundary.
    pub fn face_add_loop(
        &self,
        face: &FaceHandle,
        points_uv: Vec<f64>,
        is_outer: bool,
    ) -> Result<(), JsValue> {
        let uvs = flat_to_uv(&points_uv);
        super::error::check(rgm_face_add_loop(
            self.handle(), RgmObjectHandle(face.object_id),
            uvs.as_ptr(), uvs.len(), is_outer,
        ))
    }

    /// Add a trim loop from structured edges.
    ///
    /// `loop_is_outer`: whether this loop is the outer boundary.
    /// `edges_flat`: flat array of `[u0,v0, u1,v1, obj_id_as_f64, has_curve_as_f64]` per edge.
    /// If `has_curve_as_f64 == 0.0` the edge is linear in UV space.
    pub fn face_add_loop_edges(
        &self,
        face: &FaceHandle,
        loop_is_outer: bool,
        edges_flat: Vec<f64>,
    ) -> Result<(), JsValue> {
        let edges: Vec<RgmTrimEdgeInput> = edges_flat
            .chunks_exact(6)
            .map(|c| RgmTrimEdgeInput {
                start_uv:    RgmUv2 { u: c[0], v: c[1] },
                end_uv:      RgmUv2 { u: c[2], v: c[3] },
                curve_3d:    crate::RgmObjectHandle(c[4] as u64),
                has_curve_3d: c[5] != 0.0,
            })
            .collect();
        let loop_input = RgmTrimLoopInput {
            edge_count: edges.len() as u32,
            is_outer:   loop_is_outer,
        };
        super::error::check(rgm_face_add_loop_edges(
            self.handle(), RgmObjectHandle(face.object_id),
            &loop_input, edges.as_ptr(), edges.len(),
        ))
    }

    /// Remove a trim loop by index.
    pub fn face_remove_loop(
        &self,
        face: &FaceHandle,
        loop_index: u32,
    ) -> Result<(), JsValue> {
        super::error::check(rgm_face_remove_loop(
            self.handle(), RgmObjectHandle(face.object_id), loop_index,
        ))
    }

    /// Reverse a trim loop by index.
    pub fn face_reverse_loop(
        &self,
        face: &FaceHandle,
        loop_index: u32,
    ) -> Result<(), JsValue> {
        super::error::check(rgm_face_reverse_loop(
            self.handle(), RgmObjectHandle(face.object_id), loop_index,
        ))
    }

    /// Split a trim edge at parameter `split_t ∈ (0, 1)`.
    pub fn face_split_trim_edge(
        &self,
        face: &FaceHandle,
        loop_index: u32,
        edge_index: u32,
        split_t: f64,
    ) -> Result<(), JsValue> {
        super::error::check(rgm_face_split_trim_edge(
            self.handle(), RgmObjectHandle(face.object_id),
            loop_index, edge_index, split_t,
        ))
    }

    /// Validate the trim topology of a face.  Returns `true` if valid.
    pub fn face_validate(&self, face: &FaceHandle) -> Result<bool, JsValue> {
        let mut valid = false;
        super::error::check(rgm_face_validate(
            self.handle(), RgmObjectHandle(face.object_id), &mut valid,
        ))?;
        Ok(valid)
    }

    /// Attempt to repair the trim topology of a face.
    pub fn face_heal(&self, face: &FaceHandle) -> Result<(), JsValue> {
        super::error::check(rgm_face_heal(
            self.handle(), RgmObjectHandle(face.object_id),
        ))
    }

    /// Tessellate a trimmed face to a mesh.
    ///
    /// `options` is the same 6-element flat array as `surface_tessellate_to_mesh`.
    pub fn face_tessellate_to_mesh(
        &self,
        face: &FaceHandle,
        options: Vec<f64>,
    ) -> Result<MeshHandle, JsValue> {
        let opts = super::surface::parse_tess_options(&options);
        let opts_ptr = opts.as_ref().map(|o| o as *const _).unwrap_or(std::ptr::null());
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_face_tessellate_to_mesh(
            self.handle(), RgmObjectHandle(face.object_id), opts_ptr, &mut out,
        ))?;
        Ok(MeshHandle::new(self.session_id, out.0))
    }
}
