//! Surface constructor and evaluation methods on `KernelSession`.

use super::{flat_to_points, BrepHandle, MeshHandle, SurfaceHandle, KernelSession};
use crate::{
    rgm_loft, rgm_loft_typed, rgm_sweep, rgm_surface_bake_transform, rgm_surface_create_nurbs, rgm_surface_d1_at,
    rgm_surface_d2_at, rgm_surface_frame_at, rgm_surface_normal_at, rgm_surface_point_at,
    rgm_surface_rotate, rgm_surface_scale, rgm_surface_tessellate_to_mesh, rgm_surface_translate,
    RgmNurbsSurfaceDesc, RgmObjectHandle, RgmPoint3, RgmSurfaceEvalFrame,
    RgmSurfaceTessellationOptions, RgmUv2, RgmVec3,
};
use wasm_bindgen::prelude::*;

/// Result of evaluating a surface at (u, v).
#[wasm_bindgen]
pub struct SurfaceEvalResult {
    /// World-space point on the surface.
    pub px: f64, pub py: f64, pub pz: f64,
    /// First partial derivative with respect to u.
    pub du_x: f64, pub du_y: f64, pub du_z: f64,
    /// First partial derivative with respect to v.
    pub dv_x: f64, pub dv_y: f64, pub dv_z: f64,
    /// Unit surface normal.
    pub nx: f64, pub ny: f64, pub nz: f64,
}

#[wasm_bindgen]
impl KernelSession {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Create a NURBS surface.
    ///
    /// - `control_points`: flat `[x,y,z, …]` array, row-major (u outer, v inner).
    /// - `weights`: one per control point.
    /// - `knots_u`, `knots_v`: knot vectors.
    pub fn create_nurbs_surface(
        &self,
        degree_u: u32,
        degree_v: u32,
        control_u_count: u32,
        control_v_count: u32,
        periodic_u: bool,
        periodic_v: bool,
        control_points: Vec<f64>,
        weights: Vec<f64>,
        knots_u: Vec<f64>,
        knots_v: Vec<f64>,
    ) -> Result<SurfaceHandle, JsValue> {
        let desc = RgmNurbsSurfaceDesc {
            degree_u,
            degree_v,
            periodic_u,
            periodic_v,
            control_u_count,
            control_v_count,
        };
        let pts = flat_to_points(&control_points);
        let tol = self.tol();
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_surface_create_nurbs(
            self.handle(),
            &desc as *const _,
            pts.as_ptr(),
            pts.len(),
            weights.as_ptr(),
            weights.len(),
            knots_u.as_ptr(),
            knots_u.len(),
            knots_v.as_ptr(),
            knots_v.len(),
            &tol as *const _,
            &mut out as *mut _,
        ))?;
        Ok(SurfaceHandle::new(self.session_id, out.0))
    }

    // ── Evaluation ────────────────────────────────────────────────────────────

    /// Evaluate the world-space position at normalised (u, v) ∈ [0,1]².
    /// Returns `[x, y, z]`.
    pub fn surface_point_at(
        &self,
        surface: &SurfaceHandle,
        u: f64, v: f64,
    ) -> Result<Vec<f64>, JsValue> {
        let uv = RgmUv2 { u, v };
        let mut pt = RgmPoint3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_surface_point_at(
            self.handle(), RgmObjectHandle(surface.object_id), &uv, &mut pt,
        ))?;
        Ok(vec![pt.x, pt.y, pt.z])
    }

    /// First partial derivatives at normalised (u, v).
    /// Returns `[du_x,du_y,du_z, dv_x,dv_y,dv_z]`.
    pub fn surface_d1_at(
        &self,
        surface: &SurfaceHandle,
        u: f64, v: f64,
    ) -> Result<Vec<f64>, JsValue> {
        let uv = RgmUv2 { u, v };
        let mut du = RgmVec3 { x: 0., y: 0., z: 0. };
        let mut dv = RgmVec3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_surface_d1_at(
            self.handle(), RgmObjectHandle(surface.object_id), &uv, &mut du, &mut dv,
        ))?;
        Ok(vec![du.x, du.y, du.z, dv.x, dv.y, dv.z])
    }

    /// Second partial derivatives at normalised (u, v).
    /// Returns `[duu_x,duu_y,duu_z, duv_x,duv_y,duv_z, dvv_x,dvv_y,dvv_z]`.
    pub fn surface_d2_at(
        &self,
        surface: &SurfaceHandle,
        u: f64, v: f64,
    ) -> Result<Vec<f64>, JsValue> {
        let uv = RgmUv2 { u, v };
        let mut duu = RgmVec3 { x: 0., y: 0., z: 0. };
        let mut duv = RgmVec3 { x: 0., y: 0., z: 0. };
        let mut dvv = RgmVec3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_surface_d2_at(
            self.handle(), RgmObjectHandle(surface.object_id), &uv,
            &mut duu, &mut duv, &mut dvv,
        ))?;
        Ok(vec![duu.x, duu.y, duu.z, duv.x, duv.y, duv.z, dvv.x, dvv.y, dvv.z])
    }

    /// Unit surface normal at normalised (u, v).  Returns `[nx, ny, nz]`.
    pub fn surface_normal_at(
        &self,
        surface: &SurfaceHandle,
        u: f64, v: f64,
    ) -> Result<Vec<f64>, JsValue> {
        let uv = RgmUv2 { u, v };
        let mut n = RgmVec3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_surface_normal_at(
            self.handle(), RgmObjectHandle(surface.object_id), &uv, &mut n,
        ))?;
        Ok(vec![n.x, n.y, n.z])
    }

    /// Full evaluation frame (position + derivatives + normal) at (u, v).
    pub fn surface_frame_at(
        &self,
        surface: &SurfaceHandle,
        u: f64, v: f64,
    ) -> Result<SurfaceEvalResult, JsValue> {
        let uv = RgmUv2 { u, v };
        let mut f = RgmSurfaceEvalFrame {
            point:  crate::RgmPoint3 { x: 0., y: 0., z: 0. },
            du:     crate::RgmVec3   { x: 0., y: 0., z: 0. },
            dv:     crate::RgmVec3   { x: 0., y: 0., z: 0. },
            normal: crate::RgmVec3   { x: 0., y: 0., z: 0. },
        };
        super::error::check(rgm_surface_frame_at(
            self.handle(), RgmObjectHandle(surface.object_id), &uv, &mut f,
        ))?;
        Ok(SurfaceEvalResult {
            px: f.point.x, py: f.point.y, pz: f.point.z,
            du_x: f.du.x,  du_y: f.du.y,  du_z: f.du.z,
            dv_x: f.dv.x,  dv_y: f.dv.y,  dv_z: f.dv.z,
            nx: f.normal.x, ny: f.normal.y, nz: f.normal.z,
        })
    }

    // ── Transforms ────────────────────────────────────────────────────────────

    /// Translate a surface by `(dx, dy, dz)`.  Returns a new handle.
    pub fn surface_translate(
        &self,
        surface: &SurfaceHandle,
        dx: f64, dy: f64, dz: f64,
    ) -> Result<SurfaceHandle, JsValue> {
        let delta = RgmVec3 { x: dx, y: dy, z: dz };
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_surface_translate(
            self.handle(), RgmObjectHandle(surface.object_id),
            &delta, &mut out,
        ))?;
        Ok(SurfaceHandle::new(self.session_id, out.0))
    }

    /// Rotate a surface around `axis` by `angle_rad` about `pivot`.  Returns a new handle.
    pub fn surface_rotate(
        &self,
        surface: &SurfaceHandle,
        axis_x: f64, axis_y: f64, axis_z: f64,
        angle_rad: f64,
        pivot_x: f64, pivot_y: f64, pivot_z: f64,
    ) -> Result<SurfaceHandle, JsValue> {
        let axis  = RgmVec3   { x: axis_x,  y: axis_y,  z: axis_z  };
        let pivot = crate::RgmPoint3 { x: pivot_x, y: pivot_y, z: pivot_z };
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_surface_rotate(
            self.handle(), RgmObjectHandle(surface.object_id),
            &axis, angle_rad, &pivot, &mut out,
        ))?;
        Ok(SurfaceHandle::new(self.session_id, out.0))
    }

    /// Scale a surface non-uniformly about `pivot`.  Returns a new handle.
    pub fn surface_scale(
        &self,
        surface: &SurfaceHandle,
        sx: f64, sy: f64, sz: f64,
        pivot_x: f64, pivot_y: f64, pivot_z: f64,
    ) -> Result<SurfaceHandle, JsValue> {
        let scale = RgmVec3   { x: sx, y: sy, z: sz };
        let pivot = crate::RgmPoint3 { x: pivot_x, y: pivot_y, z: pivot_z };
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_surface_scale(
            self.handle(), RgmObjectHandle(surface.object_id),
            &scale, &pivot, &mut out,
        ))?;
        Ok(SurfaceHandle::new(self.session_id, out.0))
    }

    /// Bake the transform of a surface (apply it to control points).  Returns a new handle.
    pub fn surface_bake_transform(&self, surface: &SurfaceHandle) -> Result<SurfaceHandle, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_surface_bake_transform(
            self.handle(), RgmObjectHandle(surface.object_id), &mut out,
        ))?;
        Ok(SurfaceHandle::new(self.session_id, out.0))
    }

    // ── Sweep / Loft ──────────────────────────────────────────────────────────

    /// Sweep a profile curve along a path curve.
    ///
    /// - `cap_faces = false`: returns a `SurfaceHandle` (via `JsValue`)
    /// - `cap_faces = true`: validates profile is closed, returns a `BrepHandle`.
    ///   Errors if the profile is open.
    pub fn sweep(
        &self,
        path: &super::CurveHandle,
        profile: &super::CurveHandle,
        n_stations: u32,
        cap_faces: bool,
    ) -> Result<JsValue, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_sweep(
            self.handle(),
            RgmObjectHandle(path.object_id),
            RgmObjectHandle(profile.object_id),
            n_stations,
            cap_faces,
            &mut out,
        ))?;
        if cap_faces {
            Ok(JsValue::from(BrepHandle::new(self.session_id, out.0)))
        } else {
            Ok(JsValue::from(SurfaceHandle::new(self.session_id, out.0)))
        }
    }

    /// Loft through multiple section curves.
    ///
    /// `section_ids` is a flat array of curve object IDs (as f64).
    ///
    /// - `cap_faces = false`: returns a `SurfaceHandle` (via `JsValue`)
    /// - `cap_faces = true`: validates all sections are closed, returns `BrepHandle`.
    ///   Errors if any section is open.
    pub fn loft(
        &self,
        section_ids: Vec<f64>,
        n_samples: u32,
        cap_faces: bool,
    ) -> Result<JsValue, JsValue> {
        let handles: Vec<RgmObjectHandle> = section_ids
            .iter()
            .map(|&id| RgmObjectHandle(id as u64))
            .collect();
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_loft(
            self.handle(),
            handles.as_ptr(),
            handles.len(),
            n_samples,
            cap_faces,
            &mut out,
        ))?;
        if cap_faces {
            Ok(JsValue::from(BrepHandle::new(self.session_id, out.0)))
        } else {
            Ok(JsValue::from(SurfaceHandle::new(self.session_id, out.0)))
        }
    }

    /// Loft with an explicit loft type.
    ///
    /// `loft_type`: `"normal"` (default), `"loose"`, `"tight"`, `"straight"`.
    pub fn loft_typed(
        &self,
        section_ids: Vec<f64>,
        n_samples: u32,
        cap_faces: bool,
        loft_type: String,
    ) -> Result<JsValue, JsValue> {
        let lt: u32 = match loft_type.as_str() {
            "normal" | "" => 0,
            "loose" => 1,
            "tight" => 2,
            "straight" => 3,
            _ => return Err(JsValue::from_str(&format!("Unknown loft type: {loft_type}"))),
        };
        let handles: Vec<RgmObjectHandle> = section_ids
            .iter()
            .map(|&id| RgmObjectHandle(id as u64))
            .collect();
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_loft_typed(
            self.handle(),
            handles.as_ptr(),
            handles.len(),
            n_samples,
            cap_faces,
            lt,
            &mut out,
        ))?;
        if cap_faces {
            Ok(JsValue::from(BrepHandle::new(self.session_id, out.0)))
        } else {
            Ok(JsValue::from(SurfaceHandle::new(self.session_id, out.0)))
        }
    }

    // ── Tessellation ──────────────────────────────────────────────────────────

    /// Tessellate a surface to a mesh.
    ///
    /// `options` is an optional flat array:
    /// `[min_u, min_v, max_u, max_v, chord_tol, normal_tol_rad]` (6 values).
    /// Pass an empty `Vec` to use defaults.
    pub fn surface_tessellate_to_mesh(
        &self,
        surface: &SurfaceHandle,
        options: Vec<f64>,
    ) -> Result<MeshHandle, JsValue> {
        let opts = parse_tess_options(&options);
        let opts_ptr = opts.as_ref().map(|o| o as *const _).unwrap_or(std::ptr::null());
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_surface_tessellate_to_mesh(
            self.handle(), RgmObjectHandle(surface.object_id), opts_ptr, &mut out,
        ))?;
        Ok(MeshHandle::new(self.session_id, out.0))
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub(crate) fn parse_tess_options(opts: &[f64]) -> Option<RgmSurfaceTessellationOptions> {
    if opts.len() >= 6 {
        Some(RgmSurfaceTessellationOptions {
            min_u_segments: opts[0] as u32,
            min_v_segments: opts[1] as u32,
            max_u_segments: opts[2] as u32,
            max_v_segments: opts[3] as u32,
            chord_tol:       opts[4],
            normal_tol_rad:  opts[5],
        })
    } else {
        None
    }
}
