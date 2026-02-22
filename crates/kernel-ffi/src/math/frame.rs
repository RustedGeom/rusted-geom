use super::nurbs_curve_eval::CurveEvalResult;
use crate::{RgmPlane, RgmPoint3, RgmStatus, RgmVec3};

fn dot(a: RgmVec3, b: RgmVec3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn cross(a: RgmVec3, b: RgmVec3) -> RgmVec3 {
    RgmVec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

fn norm(v: RgmVec3) -> f64 {
    dot(v, v).sqrt()
}

fn normalize(v: RgmVec3) -> Option<RgmVec3> {
    let n = norm(v);
    if n <= f64::EPSILON {
        return None;
    }
    Some(RgmVec3 {
        x: v.x / n,
        y: v.y / n,
        z: v.z / n,
    })
}

pub(crate) fn tangent(eval: CurveEvalResult) -> Result<RgmVec3, RgmStatus> {
    normalize(eval.d1).ok_or(RgmStatus::DegenerateGeometry)
}

pub(crate) fn normal(eval: CurveEvalResult, abs_tol: f64) -> Result<RgmVec3, RgmStatus> {
    let d1_norm_sq = dot(eval.d1, eval.d1);
    if d1_norm_sq <= abs_tol.max(1e-12).powi(2) {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let principal = RgmVec3 {
        x: eval.d2.x * d1_norm_sq - eval.d1.x * dot(eval.d1, eval.d2),
        y: eval.d2.y * d1_norm_sq - eval.d1.y * dot(eval.d1, eval.d2),
        z: eval.d2.z * d1_norm_sq - eval.d1.z * dot(eval.d1, eval.d2),
    };

    if let Some(n) = normalize(principal) {
        return Ok(n);
    }

    let t = tangent(eval)?;
    let world_up = RgmVec3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    let fallback_up = RgmVec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    let mut n = cross(world_up, t);
    if normalize(n).is_none() {
        n = cross(fallback_up, t);
    }

    normalize(n).ok_or(RgmStatus::DegenerateGeometry)
}

pub(crate) fn plane(eval: CurveEvalResult, abs_tol: f64) -> Result<RgmPlane, RgmStatus> {
    let x_axis = tangent(eval)?;
    let z_axis = normal(eval, abs_tol)?;
    let y_axis = normalize(cross(z_axis, x_axis)).ok_or(RgmStatus::DegenerateGeometry)?;

    Ok(RgmPlane {
        origin: eval.point,
        x_axis,
        y_axis,
        z_axis,
    })
}

pub(crate) fn orthonormalize_plane_axes(
    plane: crate::RgmPlane,
) -> Result<(RgmVec3, RgmVec3), RgmStatus> {
    let x = normalize(plane.x_axis).ok_or(RgmStatus::InvalidInput)?;
    let mut y = plane.y_axis;
    let proj = dot(x, y);
    y.x -= proj * x.x;
    y.y -= proj * x.y;
    y.z -= proj * x.z;
    let y = normalize(y).ok_or(RgmStatus::InvalidInput)?;
    Ok((x, y))
}

pub(crate) fn point_from_frame(
    center: RgmPoint3,
    x_axis: RgmVec3,
    y_axis: RgmVec3,
    x: f64,
    y: f64,
) -> RgmPoint3 {
    RgmPoint3 {
        x: center.x + x_axis.x * x + y_axis.x * y,
        y: center.y + x_axis.y * x + y_axis.y * y,
        z: center.z + x_axis.z * x + y_axis.z * y,
    }
}
