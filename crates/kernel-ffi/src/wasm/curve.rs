//! Curve constructor and evaluation methods on `KernelSession`.

use super::{flat_to_points, CurveHandle, KernelSession};
use crate::{
    rgm_curve_create_arc, rgm_curve_create_arc_by_3_points, rgm_curve_create_arc_by_angles,
    rgm_curve_create_circle, rgm_curve_create_line, rgm_curve_create_polycurve,
    rgm_curve_create_polyline, rgm_curve_d0_at, rgm_curve_d0_at_length, rgm_curve_d1_at,
    rgm_curve_d1_at_length, rgm_curve_d2_at, rgm_curve_d2_at_length, rgm_curve_length,
    rgm_curve_length_at, rgm_curve_normal_at, rgm_curve_normal_at_length, rgm_curve_plane_at,
    rgm_curve_plane_at_length, rgm_curve_point_at, rgm_curve_point_at_length,
    rgm_curve_tangent_at, rgm_curve_tangent_at_length, rgm_curve_to_nurbs,
    rgm_nurbs_interpolate_fit_points, rgm_point_convert_coordinate_system, RgmArc3, RgmCircle3,
    RgmLine3, RgmObjectHandle, RgmPlane, RgmPoint3, RgmPolycurveSegment, RgmVec3,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl KernelSession {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Create a line segment from (x0,y0,z0) to (x1,y1,z1).
    pub fn create_line(
        &self,
        x0: f64, y0: f64, z0: f64,
        x1: f64, y1: f64, z1: f64,
    ) -> Result<CurveHandle, JsValue> {
        let line = RgmLine3 {
            start: RgmPoint3 { x: x0, y: y0, z: z0 },
            end:   RgmPoint3 { x: x1, y: y1, z: z1 },
        };
        let tol = self.tol();
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_curve_create_line(
            self.handle(), &line as *const _, &tol as *const _, &mut out as *mut _,
        ))?;
        Ok(CurveHandle::new(self.session_id, out.0))
    }

    /// Create a full circle.
    ///
    /// `plane_*` defines the circle plane (origin + x/y/z axes).
    pub fn create_circle(
        &self,
        origin_x: f64, origin_y: f64, origin_z: f64,
        x_axis_x: f64, x_axis_y: f64, x_axis_z: f64,
        y_axis_x: f64, y_axis_y: f64, y_axis_z: f64,
        z_axis_x: f64, z_axis_y: f64, z_axis_z: f64,
        radius: f64,
    ) -> Result<CurveHandle, JsValue> {
        let circle = RgmCircle3 {
            plane: plane(origin_x, origin_y, origin_z,
                         x_axis_x, x_axis_y, x_axis_z,
                         y_axis_x, y_axis_y, y_axis_z,
                         z_axis_x, z_axis_y, z_axis_z),
            radius,
        };
        let tol = self.tol();
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_curve_create_circle(
            self.handle(), &circle as *const _, &tol as *const _, &mut out as *mut _,
        ))?;
        Ok(CurveHandle::new(self.session_id, out.0))
    }

    /// Create an arc from a plane + radius + start/sweep angles (radians).
    pub fn create_arc(
        &self,
        origin_x: f64, origin_y: f64, origin_z: f64,
        x_axis_x: f64, x_axis_y: f64, x_axis_z: f64,
        y_axis_x: f64, y_axis_y: f64, y_axis_z: f64,
        z_axis_x: f64, z_axis_y: f64, z_axis_z: f64,
        radius: f64,
        start_angle: f64,
        sweep_angle: f64,
    ) -> Result<CurveHandle, JsValue> {
        let arc = RgmArc3 {
            plane: plane(origin_x, origin_y, origin_z,
                         x_axis_x, x_axis_y, x_axis_z,
                         y_axis_x, y_axis_y, y_axis_z,
                         z_axis_x, z_axis_y, z_axis_z),
            radius,
            start_angle,
            sweep_angle,
        };
        let tol = self.tol();
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_curve_create_arc(
            self.handle(), &arc as *const _, &tol as *const _, &mut out as *mut _,
        ))?;
        Ok(CurveHandle::new(self.session_id, out.0))
    }

    /// Create an arc from a plane + radius + start/end angles (radians).
    pub fn create_arc_by_angles(
        &self,
        origin_x: f64, origin_y: f64, origin_z: f64,
        x_axis_x: f64, x_axis_y: f64, x_axis_z: f64,
        y_axis_x: f64, y_axis_y: f64, y_axis_z: f64,
        z_axis_x: f64, z_axis_y: f64, z_axis_z: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Result<CurveHandle, JsValue> {
        let p = plane(origin_x, origin_y, origin_z,
                      x_axis_x, x_axis_y, x_axis_z,
                      y_axis_x, y_axis_y, y_axis_z,
                      z_axis_x, z_axis_y, z_axis_z);
        let tol = self.tol();
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_curve_create_arc_by_angles(
            self.handle(), &p as *const _, radius, start_angle, end_angle,
            &tol as *const _, &mut out as *mut _,
        ))?;
        Ok(CurveHandle::new(self.session_id, out.0))
    }

    /// Create an arc through three 3-D points.
    pub fn create_arc_by_3_points(
        &self,
        x0: f64, y0: f64, z0: f64,
        x1: f64, y1: f64, z1: f64,
        x2: f64, y2: f64, z2: f64,
    ) -> Result<CurveHandle, JsValue> {
        let start = RgmPoint3 { x: x0, y: y0, z: z0 };
        let mid   = RgmPoint3 { x: x1, y: y1, z: z1 };
        let end   = RgmPoint3 { x: x2, y: y2, z: z2 };
        let tol = self.tol();
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_curve_create_arc_by_3_points(
            self.handle(),
            &start as *const _, &mid as *const _, &end as *const _,
            &tol as *const _, &mut out as *mut _,
        ))?;
        Ok(CurveHandle::new(self.session_id, out.0))
    }

    /// Create a polyline from a flat `[x,y,z, …]` array.
    pub fn create_polyline(
        &self,
        points: Vec<f64>,
        closed: bool,
    ) -> Result<CurveHandle, JsValue> {
        let pts = flat_to_points(&points);
        let tol = self.tol();
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_curve_create_polyline(
            self.handle(), pts.as_ptr(), pts.len(), closed,
            &tol as *const _, &mut out as *mut _,
        ))?;
        Ok(CurveHandle::new(self.session_id, out.0))
    }

    /// Create a polycurve from a list of (object_id_as_f64, reversed) pairs.
    ///
    /// Pass a flat array: `[id0, rev0_as_f64, id1, rev1_as_f64, …]`.
    /// `rev` is `0.0` for forward, `1.0` for reversed.
    pub fn create_polycurve(&self, segments: Vec<f64>) -> Result<CurveHandle, JsValue> {
        let segs: Vec<RgmPolycurveSegment> = segments
            .chunks_exact(2)
            .map(|c| RgmPolycurveSegment {
                curve:    RgmObjectHandle(c[0] as u64),
                reversed: c[1] != 0.0,
            })
            .collect();
        let tol = self.tol();
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_curve_create_polycurve(
            self.handle(), segs.as_ptr(), segs.len(),
            &tol as *const _, &mut out as *mut _,
        ))?;
        Ok(CurveHandle::new(self.session_id, out.0))
    }

    /// Fit a NURBS curve through the given 3-D points.
    ///
    /// `points` is a flat `[x,y,z, …]` array.
    pub fn interpolate_nurbs_fit_points(
        &self,
        points: Vec<f64>,
        degree: u32,
        closed: bool,
    ) -> Result<CurveHandle, JsValue> {
        let pts = flat_to_points(&points);
        let tol = self.tol();
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_nurbs_interpolate_fit_points(
            self.handle(), pts.as_ptr(), pts.len(), degree, closed,
            &tol as *const _, &mut out as *mut _,
        ))?;
        Ok(CurveHandle::new(self.session_id, out.0))
    }

    /// Convert any curve to its NURBS representation.
    pub fn curve_to_nurbs(&self, curve: &CurveHandle) -> Result<CurveHandle, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_curve_to_nurbs(
            self.handle(), RgmObjectHandle(curve.object_id), &mut out as *mut _,
        ))?;
        Ok(CurveHandle::new(self.session_id, out.0))
    }

    // ── Evaluation ────────────────────────────────────────────────────────────

    /// Evaluate a curve at normalised parameter `t ∈ [0, 1]`.  Returns `[x, y, z]`.
    pub fn curve_point_at(&self, curve: &CurveHandle, t: f64) -> Result<Vec<f64>, JsValue> {
        let mut pt = RgmPoint3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_curve_point_at(
            self.handle(), RgmObjectHandle(curve.object_id), t, &mut pt as *mut _,
        ))?;
        Ok(vec![pt.x, pt.y, pt.z])
    }

    /// Evaluate position at a given arc-length distance from the start.  Returns `[x, y, z]`.
    pub fn curve_point_at_length(
        &self,
        curve: &CurveHandle,
        distance: f64,
    ) -> Result<Vec<f64>, JsValue> {
        let mut pt = RgmPoint3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_curve_point_at_length(
            self.handle(), RgmObjectHandle(curve.object_id), distance, &mut pt as *mut _,
        ))?;
        Ok(vec![pt.x, pt.y, pt.z])
    }

    /// Total arc-length of a curve.
    pub fn curve_length(&self, curve: &CurveHandle) -> Result<f64, JsValue> {
        let mut length = 0.0f64;
        super::error::check(rgm_curve_length(
            self.handle(), RgmObjectHandle(curve.object_id), &mut length as *mut _,
        ))?;
        Ok(length)
    }

    /// Arc-length from the start to normalised parameter `t`.
    pub fn curve_length_at(&self, curve: &CurveHandle, t: f64) -> Result<f64, JsValue> {
        let mut length = 0.0f64;
        super::error::check(rgm_curve_length_at(
            self.handle(), RgmObjectHandle(curve.object_id), t, &mut length as *mut _,
        ))?;
        Ok(length)
    }

    /// First derivative (velocity) at normalised `t`.  Returns `[dx, dy, dz]`.
    pub fn curve_d1_at(&self, curve: &CurveHandle, t: f64) -> Result<Vec<f64>, JsValue> {
        let mut v = RgmVec3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_curve_d1_at(
            self.handle(), RgmObjectHandle(curve.object_id), t, &mut v as *mut _,
        ))?;
        Ok(vec![v.x, v.y, v.z])
    }

    /// Second derivative (acceleration) at normalised `t`.  Returns `[dx, dy, dz]`.
    pub fn curve_d2_at(&self, curve: &CurveHandle, t: f64) -> Result<Vec<f64>, JsValue> {
        let mut v = RgmVec3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_curve_d2_at(
            self.handle(), RgmObjectHandle(curve.object_id), t, &mut v as *mut _,
        ))?;
        Ok(vec![v.x, v.y, v.z])
    }

    /// Unit tangent at normalised `t`.  Returns `[tx, ty, tz]`.
    pub fn curve_tangent_at(&self, curve: &CurveHandle, t: f64) -> Result<Vec<f64>, JsValue> {
        let mut v = RgmVec3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_curve_tangent_at(
            self.handle(), RgmObjectHandle(curve.object_id), t, &mut v as *mut _,
        ))?;
        Ok(vec![v.x, v.y, v.z])
    }

    /// Unit normal at normalised `t`.  Returns `[nx, ny, nz]`.
    pub fn curve_normal_at(&self, curve: &CurveHandle, t: f64) -> Result<Vec<f64>, JsValue> {
        let mut v = RgmVec3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_curve_normal_at(
            self.handle(), RgmObjectHandle(curve.object_id), t, &mut v as *mut _,
        ))?;
        Ok(vec![v.x, v.y, v.z])
    }

    /// Frenet frame (plane) at normalised `t`.
    /// Returns flat `[ox,oy,oz, xx,xy,xz, yx,yy,yz, zx,zy,zz]` (origin + 3 axes).
    pub fn curve_plane_at(&self, curve: &CurveHandle, t: f64) -> Result<Vec<f64>, JsValue> {
        let mut p = RgmPlane {
            origin: RgmPoint3 { x: 0., y: 0., z: 0. },
            x_axis: RgmVec3 { x: 1., y: 0., z: 0. },
            y_axis: RgmVec3 { x: 0., y: 1., z: 0. },
            z_axis: RgmVec3 { x: 0., y: 0., z: 1. },
        };
        super::error::check(rgm_curve_plane_at(
            self.handle(), RgmObjectHandle(curve.object_id), t, &mut p as *mut _,
        ))?;
        Ok(plane_to_flat(&p))
    }

    /// Position at arc-length distance.  Returns `[x, y, z]`.
    pub fn curve_d0_at_length(
        &self,
        curve: &CurveHandle,
        distance: f64,
    ) -> Result<Vec<f64>, JsValue> {
        let mut pt = RgmPoint3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_curve_d0_at_length(
            self.handle(), RgmObjectHandle(curve.object_id), distance, &mut pt as *mut _,
        ))?;
        Ok(vec![pt.x, pt.y, pt.z])
    }

    /// First derivative at arc-length distance.  Returns `[dx, dy, dz]`.
    pub fn curve_d1_at_length(
        &self,
        curve: &CurveHandle,
        distance: f64,
    ) -> Result<Vec<f64>, JsValue> {
        let mut v = RgmVec3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_curve_d1_at_length(
            self.handle(), RgmObjectHandle(curve.object_id), distance, &mut v as *mut _,
        ))?;
        Ok(vec![v.x, v.y, v.z])
    }

    /// Second derivative at arc-length distance.  Returns `[dx, dy, dz]`.
    pub fn curve_d2_at_length(
        &self,
        curve: &CurveHandle,
        distance: f64,
    ) -> Result<Vec<f64>, JsValue> {
        let mut v = RgmVec3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_curve_d2_at_length(
            self.handle(), RgmObjectHandle(curve.object_id), distance, &mut v as *mut _,
        ))?;
        Ok(vec![v.x, v.y, v.z])
    }

    /// Unit tangent at arc-length distance.  Returns `[tx, ty, tz]`.
    pub fn curve_tangent_at_length(
        &self,
        curve: &CurveHandle,
        distance: f64,
    ) -> Result<Vec<f64>, JsValue> {
        let mut v = RgmVec3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_curve_tangent_at_length(
            self.handle(), RgmObjectHandle(curve.object_id), distance, &mut v as *mut _,
        ))?;
        Ok(vec![v.x, v.y, v.z])
    }

    /// Unit normal at arc-length distance.  Returns `[nx, ny, nz]`.
    pub fn curve_normal_at_length(
        &self,
        curve: &CurveHandle,
        distance: f64,
    ) -> Result<Vec<f64>, JsValue> {
        let mut v = RgmVec3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_curve_normal_at_length(
            self.handle(), RgmObjectHandle(curve.object_id), distance, &mut v as *mut _,
        ))?;
        Ok(vec![v.x, v.y, v.z])
    }

    /// Frenet frame at arc-length distance.
    /// Returns flat `[ox,oy,oz, xx,xy,xz, yx,yy,yz, zx,zy,zz]`.
    pub fn curve_plane_at_length(
        &self,
        curve: &CurveHandle,
        distance: f64,
    ) -> Result<Vec<f64>, JsValue> {
        let mut p = RgmPlane {
            origin: RgmPoint3 { x: 0., y: 0., z: 0. },
            x_axis: RgmVec3 { x: 1., y: 0., z: 0. },
            y_axis: RgmVec3 { x: 0., y: 1., z: 0. },
            z_axis: RgmVec3 { x: 0., y: 0., z: 1. },
        };
        super::error::check(rgm_curve_plane_at_length(
            self.handle(), RgmObjectHandle(curve.object_id), distance, &mut p as *mut _,
        ))?;
        Ok(plane_to_flat(&p))
    }

    /// Position at normalised parameter `t`.  Alias for `curve_point_at`.
    pub fn curve_d0_at(&self, curve: &CurveHandle, t: f64) -> Result<Vec<f64>, JsValue> {
        let mut pt = RgmPoint3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_curve_d0_at(
            self.handle(), RgmObjectHandle(curve.object_id), t, &mut pt as *mut _,
        ))?;
        Ok(vec![pt.x, pt.y, pt.z])
    }

    /// Convert a point between coordinate systems.
    ///
    /// `source` and `target` use the `RgmAlignmentCoordinateSystem` enum values
    /// (0 = EastingNorthing, 1 = NorthingEasting).
    pub fn convert_coordinate_system(
        &self,
        x: f64, y: f64, z: f64,
        source: i32,
        target: i32,
    ) -> Result<Vec<f64>, JsValue> {
        let mut pt = RgmPoint3 { x: 0., y: 0., z: 0. };
        super::error::check(rgm_point_convert_coordinate_system(
            self.handle(), x, y, z, source, target, &mut pt as *mut _,
        ))?;
        Ok(vec![pt.x, pt.y, pt.z])
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn plane(
    ox: f64, oy: f64, oz: f64,
    xx: f64, xy: f64, xz: f64,
    yx: f64, yy: f64, yz: f64,
    zx: f64, zy: f64, zz: f64,
) -> RgmPlane {
    RgmPlane {
        origin: RgmPoint3 { x: ox, y: oy, z: oz },
        x_axis: RgmVec3   { x: xx, y: xy, z: xz },
        y_axis: RgmVec3   { x: yx, y: yy, z: yz },
        z_axis: RgmVec3   { x: zx, y: zy, z: zz },
    }
}

pub(super) fn plane_to_flat(p: &RgmPlane) -> Vec<f64> {
    vec![
        p.origin.x, p.origin.y, p.origin.z,
        p.x_axis.x, p.x_axis.y, p.x_axis.z,
        p.y_axis.x, p.y_axis.y, p.y_axis.z,
        p.z_axis.x, p.z_axis.y, p.z_axis.z,
    ]
}
