//! Intersection operation methods on `KernelSession`.

use super::{points_to_flat, uv_to_flat, CurveHandle, IntersectionHandle, KernelSession, SurfaceHandle};
use crate::{
    rgm_intersect_curve_curve, rgm_intersect_curve_plane, rgm_intersect_surface_curve,
    rgm_intersect_surface_plane, rgm_intersect_surface_surface, rgm_intersection_branch_count,
    rgm_intersection_branch_summary, rgm_intersection_branch_to_nurbs,
    rgm_intersection_copy_branch_curve_t, rgm_intersection_copy_branch_points,
    rgm_intersection_copy_branch_uv_on_surface_a, rgm_intersection_copy_branch_uv_on_surface_b,
    RgmIntersectionBranchSummary, RgmObjectHandle, RgmPoint3, RgmUv2,
};
use wasm_bindgen::prelude::*;

/// Summary data for a single intersection branch.
#[wasm_bindgen]
pub struct BranchSummary {
    /// Number of 3-D sample points on this branch.
    pub point_count: u32,
    /// Number of UV coordinates on surface A.
    pub uv_a_count: u32,
    /// Number of UV coordinates on surface B.
    pub uv_b_count: u32,
    /// Number of curve parameter values.
    pub curve_t_count: u32,
    /// Whether the intersection branch is a closed loop.
    pub closed: bool,
    /// Internal flags.
    pub flags: u32,
}

#[wasm_bindgen]
impl KernelSession {
    // ── Curve intersections ───────────────────────────────────────────────────

    /// Intersect two curves.  Returns a flat `[x,y,z, …]` array of hit points.
    pub fn intersect_curve_curve(
        &self,
        curve_a: &CurveHandle,
        curve_b: &CurveHandle,
    ) -> Result<Vec<f64>, JsValue> {
        copy_points_result(|buf, cap, out_count| {
            rgm_intersect_curve_curve(
                self.handle(),
                RgmObjectHandle(curve_a.object_id),
                RgmObjectHandle(curve_b.object_id),
                buf, cap, out_count,
            )
        })
    }

    /// Intersect a curve with a plane.
    ///
    /// `plane_flat`: `[ox,oy,oz, xx,xy,xz, yx,yy,yz, zx,zy,zz]`.
    /// Returns a flat `[x,y,z, …]` array of hit points.
    pub fn intersect_curve_plane(
        &self,
        curve: &CurveHandle,
        plane_flat: Vec<f64>,
    ) -> Result<Vec<f64>, JsValue> {
        let p = parse_plane_js(&plane_flat)?;
        copy_points_result(|buf, cap, out_count| {
            rgm_intersect_curve_plane(
                self.handle(),
                RgmObjectHandle(curve.object_id),
                &p, buf, cap, out_count,
            )
        })
    }

    // ── Surface intersections (return IntersectionHandle) ─────────────────────

    /// Intersect two surfaces.  Returns an intersection handle.
    pub fn intersect_surface_surface(
        &self,
        surface_a: &SurfaceHandle,
        surface_b: &SurfaceHandle,
    ) -> Result<IntersectionHandle, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_intersect_surface_surface(
            self.handle(),
            RgmObjectHandle(surface_a.object_id),
            RgmObjectHandle(surface_b.object_id),
            &mut out,
        ))?;
        Ok(IntersectionHandle::new(self.session_id, out.0))
    }

    /// Intersect a surface with a plane.
    pub fn intersect_surface_plane(
        &self,
        surface: &SurfaceHandle,
        plane_flat: Vec<f64>,
    ) -> Result<IntersectionHandle, JsValue> {
        let p = parse_plane_js(&plane_flat)?;
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_intersect_surface_plane(
            self.handle(),
            RgmObjectHandle(surface.object_id),
            &p, &mut out,
        ))?;
        Ok(IntersectionHandle::new(self.session_id, out.0))
    }

    /// Intersect a surface with a curve.
    pub fn intersect_surface_curve(
        &self,
        surface: &SurfaceHandle,
        curve: &CurveHandle,
    ) -> Result<IntersectionHandle, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_intersect_surface_curve(
            self.handle(),
            RgmObjectHandle(surface.object_id),
            RgmObjectHandle(curve.object_id),
            &mut out,
        ))?;
        Ok(IntersectionHandle::new(self.session_id, out.0))
    }

    // ── Intersection data access ───────────────────────────────────────────────

    /// Number of branches in an intersection result.
    pub fn intersection_branch_count(
        &self,
        intersection: &IntersectionHandle,
    ) -> Result<u32, JsValue> {
        let mut count = 0u32;
        super::error::check(rgm_intersection_branch_count(
            self.handle(), RgmObjectHandle(intersection.object_id), &mut count,
        ))?;
        Ok(count)
    }

    /// Summary of a single branch.
    pub fn intersection_branch_summary(
        &self,
        intersection: &IntersectionHandle,
        branch_index: u32,
    ) -> Result<BranchSummary, JsValue> {
        let mut s = RgmIntersectionBranchSummary {
            point_count:   0,
            uv_a_count:    0,
            uv_b_count:    0,
            curve_t_count: 0,
            closed:        false,
            flags:         0,
        };
        super::error::check(rgm_intersection_branch_summary(
            self.handle(), RgmObjectHandle(intersection.object_id), branch_index, &mut s,
        ))?;
        Ok(BranchSummary {
            point_count:   s.point_count,
            uv_a_count:    s.uv_a_count,
            uv_b_count:    s.uv_b_count,
            curve_t_count: s.curve_t_count,
            closed:        s.closed,
            flags:         s.flags,
        })
    }

    /// Copy 3-D points from an intersection branch.
    /// Returns a flat `[x,y,z, …]` array.
    pub fn intersection_branch_copy_points(
        &self,
        intersection: &IntersectionHandle,
        branch_index: u32,
    ) -> Result<Vec<f64>, JsValue> {
        copy_points_result(|buf, cap, out_count| {
            rgm_intersection_copy_branch_points(
                self.handle(), RgmObjectHandle(intersection.object_id),
                branch_index, buf, cap, out_count,
            )
        })
    }

    /// Copy UV coordinates on surface A from an intersection branch.
    /// Returns a flat `[u,v, …]` array.
    pub fn intersection_branch_copy_uv_a(
        &self,
        intersection: &IntersectionHandle,
        branch_index: u32,
    ) -> Result<Vec<f64>, JsValue> {
        copy_uv_result(|buf, cap, out_count| {
            rgm_intersection_copy_branch_uv_on_surface_a(
                self.handle(), RgmObjectHandle(intersection.object_id),
                branch_index, buf, cap, out_count,
            )
        })
    }

    /// Copy UV coordinates on surface B from an intersection branch.
    /// Returns a flat `[u,v, …]` array.
    pub fn intersection_branch_copy_uv_b(
        &self,
        intersection: &IntersectionHandle,
        branch_index: u32,
    ) -> Result<Vec<f64>, JsValue> {
        copy_uv_result(|buf, cap, out_count| {
            rgm_intersection_copy_branch_uv_on_surface_b(
                self.handle(), RgmObjectHandle(intersection.object_id),
                branch_index, buf, cap, out_count,
            )
        })
    }

    /// Copy curve parameter values from an intersection branch.
    /// Returns an array of `f64` values.
    pub fn intersection_branch_copy_curve_t(
        &self,
        intersection: &IntersectionHandle,
        branch_index: u32,
    ) -> Result<Vec<f64>, JsValue> {
        // Phase 1: count
        let mut count = 0u32;
        rgm_intersection_copy_branch_curve_t(
            self.handle(), RgmObjectHandle(intersection.object_id),
            branch_index, std::ptr::null_mut(), 0, &mut count,
        );
        if count == 0 {
            return Ok(Vec::new());
        }
        // Phase 2: fill
        let mut buf = vec![0.0f64; count as usize];
        let mut actual = 0u32;
        super::error::check(rgm_intersection_copy_branch_curve_t(
            self.handle(), RgmObjectHandle(intersection.object_id),
            branch_index, buf.as_mut_ptr(), count, &mut actual,
        ))?;
        buf.truncate(actual as usize);
        Ok(buf)
    }

    /// Convert an intersection branch to a NURBS curve.
    pub fn intersection_branch_to_nurbs(
        &self,
        intersection: &IntersectionHandle,
        branch_index: u32,
    ) -> Result<CurveHandle, JsValue> {
        let tol = self.tol();
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_intersection_branch_to_nurbs(
            self.handle(), RgmObjectHandle(intersection.object_id),
            branch_index, &tol, &mut out,
        ))?;
        Ok(CurveHandle::new(self.session_id, out.0))
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Two-phase copy pattern for `RgmPoint3` output arrays.
fn copy_points_result<F>(mut f: F) -> Result<Vec<f64>, JsValue>
where
    F: FnMut(*mut RgmPoint3, u32, *mut u32) -> crate::RgmStatus,
{
    let mut count = 0u32;
    f(std::ptr::null_mut(), 0, &mut count);
    if count == 0 {
        return Ok(Vec::new());
    }
    let mut pts = vec![RgmPoint3 { x: 0., y: 0., z: 0. }; count as usize];
    let mut actual = 0u32;
    super::error::check(f(pts.as_mut_ptr(), count, &mut actual))?;
    pts.truncate(actual as usize);
    Ok(points_to_flat(&pts))
}

/// Two-phase copy pattern for `RgmUv2` output arrays.
fn copy_uv_result<F>(mut f: F) -> Result<Vec<f64>, JsValue>
where
    F: FnMut(*mut RgmUv2, u32, *mut u32) -> crate::RgmStatus,
{
    let mut count = 0u32;
    f(std::ptr::null_mut(), 0, &mut count);
    if count == 0 {
        return Ok(Vec::new());
    }
    let mut uvs = vec![RgmUv2 { u: 0., v: 0. }; count as usize];
    let mut actual = 0u32;
    super::error::check(f(uvs.as_mut_ptr(), count, &mut actual))?;
    uvs.truncate(actual as usize);
    Ok(uv_to_flat(&uvs))
}

fn parse_plane_js(flat: &[f64]) -> Result<crate::RgmPlane, JsValue> {
    if flat.len() < 12 {
        return Err(JsValue::from_str(
            "plane array must have 12 values [ox,oy,oz, xx,xy,xz, yx,yy,yz, zx,zy,zz]",
        ));
    }
    Ok(crate::RgmPlane {
        origin: crate::RgmPoint3 { x: flat[0],  y: flat[1],  z: flat[2]  },
        x_axis: crate::RgmVec3   { x: flat[3],  y: flat[4],  z: flat[5]  },
        y_axis: crate::RgmVec3   { x: flat[6],  y: flat[7],  z: flat[8]  },
        z_axis: crate::RgmVec3   { x: flat[9],  y: flat[10], z: flat[11] },
    })
}
