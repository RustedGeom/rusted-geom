//! Bounding-box computation methods on `KernelSession`.

use super::KernelSession;
use crate::{
    rgm_object_compute_bounds, RgmBounds3, RgmBoundsMode, RgmBoundsOptions, RgmObjectHandle,
};
use wasm_bindgen::prelude::*;

/// Bounding box result for a geometry object.
///
/// All fields are world-space coordinates after the object transform is applied.
#[wasm_bindgen]
pub struct Bounds3 {
    // ── World AABB ─────────────────────────────────────────────────────────────
    pub aabb_min_x: f64,
    pub aabb_min_y: f64,
    pub aabb_min_z: f64,
    pub aabb_max_x: f64,
    pub aabb_max_y: f64,
    pub aabb_max_z: f64,
    // ── World OBB centre ──────────────────────────────────────────────────────
    pub obb_center_x: f64,
    pub obb_center_y: f64,
    pub obb_center_z: f64,
    // ── OBB half-extents ──────────────────────────────────────────────────────
    pub obb_half_x: f64,
    pub obb_half_y: f64,
    pub obb_half_z: f64,
    // ── OBB local X axis ──────────────────────────────────────────────────────
    pub obb_ax_x: f64,
    pub obb_ax_y: f64,
    pub obb_ax_z: f64,
    // ── OBB local Y axis ──────────────────────────────────────────────────────
    pub obb_ay_x: f64,
    pub obb_ay_y: f64,
    pub obb_ay_z: f64,
    // ── OBB local Z axis ──────────────────────────────────────────────────────
    pub obb_az_x: f64,
    pub obb_az_y: f64,
    pub obb_az_z: f64,
    // ── Local-frame AABB (in OBB coordinate frame) ────────────────────────────
    pub local_aabb_min_x: f64,
    pub local_aabb_min_y: f64,
    pub local_aabb_min_z: f64,
    pub local_aabb_max_x: f64,
    pub local_aabb_max_y: f64,
    pub local_aabb_max_z: f64,
}

#[wasm_bindgen]
impl KernelSession {
    /// Compute bounding boxes (AABB + OBB) for any geometry object.
    ///
    /// - `object_id`: the raw object ID obtained from a handle's `.object_id()` method.
    /// - `mode`: `0` = Fast (control-point hull), `1` = Optimal (PCA + OBB refinement).
    /// - `sample_budget`: number of evaluation samples (0 = use kernel default).
    /// - `padding`: uniform padding applied to all box dimensions.
    pub fn compute_bounds(
        &self,
        object_id: f64,
        mode: u32,
        sample_budget: u32,
        padding: f64,
    ) -> Result<Bounds3, JsValue> {
        let bounds_mode = if mode == 0 { RgmBoundsMode::Fast } else { RgmBoundsMode::Optimal };
        let opts = RgmBoundsOptions { mode: bounds_mode, sample_budget, padding };
        let mut out = RgmBounds3 {
            world_aabb: crate::RgmAabb3 {
                min: crate::RgmPoint3 { x: 0., y: 0., z: 0. },
                max: crate::RgmPoint3 { x: 0., y: 0., z: 0. },
            },
            world_obb: crate::RgmObb3 {
                center:       crate::RgmPoint3 { x: 0., y: 0., z: 0. },
                x_axis:       crate::RgmVec3   { x: 1., y: 0., z: 0. },
                y_axis:       crate::RgmVec3   { x: 0., y: 1., z: 0. },
                z_axis:       crate::RgmVec3   { x: 0., y: 0., z: 1. },
                half_extents: crate::RgmVec3   { x: 0., y: 0., z: 0. },
            },
            local_aabb: crate::RgmAabb3 {
                min: crate::RgmPoint3 { x: 0., y: 0., z: 0. },
                max: crate::RgmPoint3 { x: 0., y: 0., z: 0. },
            },
        };
        super::error::check(rgm_object_compute_bounds(
            self.handle(),
            RgmObjectHandle(object_id as u64),
            &opts as *const _,
            &mut out as *mut _,
        ))?;
        Ok(Bounds3 {
            aabb_min_x: out.world_aabb.min.x,
            aabb_min_y: out.world_aabb.min.y,
            aabb_min_z: out.world_aabb.min.z,
            aabb_max_x: out.world_aabb.max.x,
            aabb_max_y: out.world_aabb.max.y,
            aabb_max_z: out.world_aabb.max.z,

            obb_center_x: out.world_obb.center.x,
            obb_center_y: out.world_obb.center.y,
            obb_center_z: out.world_obb.center.z,

            obb_half_x: out.world_obb.half_extents.x,
            obb_half_y: out.world_obb.half_extents.y,
            obb_half_z: out.world_obb.half_extents.z,

            obb_ax_x: out.world_obb.x_axis.x,
            obb_ax_y: out.world_obb.x_axis.y,
            obb_ax_z: out.world_obb.x_axis.z,

            obb_ay_x: out.world_obb.y_axis.x,
            obb_ay_y: out.world_obb.y_axis.y,
            obb_ay_z: out.world_obb.y_axis.z,

            obb_az_x: out.world_obb.z_axis.x,
            obb_az_y: out.world_obb.z_axis.y,
            obb_az_z: out.world_obb.z_axis.z,

            local_aabb_min_x: out.local_aabb.min.x,
            local_aabb_min_y: out.local_aabb.min.y,
            local_aabb_min_z: out.local_aabb.min.z,
            local_aabb_max_x: out.local_aabb.max.x,
            local_aabb_max_y: out.local_aabb.max.y,
            local_aabb_max_z: out.local_aabb.max.z,
        })
    }
}
