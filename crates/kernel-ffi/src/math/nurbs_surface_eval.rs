//! NURBS surface evaluation: position, first, and second derivatives.
//!
//! Implements the tensor-product rational de Boor algorithm for NURBS surfaces
//! from Piegl & Tiller, *The NURBS Book*, 2nd ed.:
//!
//! * §4.4 — [`eval_nurbs_surface_uv`]: point and derivative evaluation at a
//!   raw `(u, v)` knot parameter (Algorithm A4.6 adapted for first and second
//!   order derivatives simultaneously).
//! * Normalization helpers map the public `[0, 1]²` domain to the surface's
//!   native `[u_start, u_end] × [v_start, v_end]` domain; see
//!   [`eval_nurbs_surface_normalized`].
//!
//! **Periodicity:** each parametric direction is independently wrapped for
//! periodic surfaces.
//!
//! **Domain constraints:** the surface must pass [`validate_surface`] before
//! evaluation.  `validate_surface` checks degrees, control-point grid
//! dimensions, and knot vector consistency in both directions.

use super::basis::{ders_basis_funs, find_span};
use crate::{RgmPoint3, RgmStatus, RgmToleranceContext, RgmUv2, RgmVec3};

#[derive(Clone, Debug)]
pub(crate) struct NurbsSurfaceCore {
    pub(crate) degree_u: usize,
    pub(crate) degree_v: usize,
    pub(crate) periodic_u: bool,
    pub(crate) periodic_v: bool,
    pub(crate) control_u_count: usize,
    pub(crate) control_v_count: usize,
    pub(crate) control_points: Vec<RgmPoint3>,
    pub(crate) weights: Vec<f64>,
    pub(crate) knots_u: Vec<f64>,
    pub(crate) knots_v: Vec<f64>,
    pub(crate) u_start: f64,
    pub(crate) u_end: f64,
    pub(crate) v_start: f64,
    pub(crate) v_end: f64,
    pub(crate) tol: RgmToleranceContext,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SurfaceEvalResult {
    pub(crate) point: RgmPoint3,
    pub(crate) du: RgmVec3,
    pub(crate) dv: RgmVec3,
    pub(crate) duu: RgmVec3,
    pub(crate) dvv: RgmVec3,
    pub(crate) duv: RgmVec3,
}

#[derive(Clone, Copy, Debug)]
struct H4 {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl H4 {
    fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }
    }

    fn add_scaled(&mut self, other: H4, scale: f64) {
        self.x += other.x * scale;
        self.y += other.y * scale;
        self.z += other.z * scale;
        self.w += other.w * scale;
    }
}

fn point_to_h4(point: RgmPoint3, w: f64) -> H4 {
    H4 {
        x: point.x * w,
        y: point.y * w,
        z: point.z * w,
        w,
    }
}

fn h4_to_point(value: H4, denom_eps: f64) -> Result<RgmPoint3, RgmStatus> {
    if value.w.abs() <= denom_eps {
        return Err(RgmStatus::NumericalFailure);
    }
    Ok(RgmPoint3 {
        x: value.x / value.w,
        y: value.y / value.w,
        z: value.z / value.w,
    })
}

pub(crate) fn validate_surface(surface: &NurbsSurfaceCore) -> Result<(), RgmStatus> {
    if surface.degree_u == 0 || surface.degree_v == 0 {
        return Err(RgmStatus::InvalidInput);
    }
    if surface.control_u_count <= surface.degree_u || surface.control_v_count <= surface.degree_v {
        return Err(RgmStatus::InvalidInput);
    }
    let control_count = surface
        .control_u_count
        .checked_mul(surface.control_v_count)
        .ok_or(RgmStatus::OutOfRange)?;
    if surface.control_points.len() != control_count || surface.weights.len() != control_count {
        return Err(RgmStatus::InvalidInput);
    }

    let nu = surface.control_u_count - 1;
    let nv = surface.control_v_count - 1;
    let expected_ku = nu + surface.degree_u + 2;
    let expected_kv = nv + surface.degree_v + 2;
    if surface.knots_u.len() < expected_ku || surface.knots_v.len() < expected_kv {
        return Err(RgmStatus::InvalidInput);
    }
    if surface.u_end < surface.u_start || surface.v_end < surface.v_start {
        return Err(RgmStatus::InvalidInput);
    }
    Ok(())
}

fn normalize_periodic(value: f64, min: f64, max: f64) -> f64 {
    let period = max - min;
    if period.abs() <= f64::EPSILON {
        return min;
    }
    let mut v = value;
    while v < min {
        v += period;
    }
    while v >= max {
        v -= period;
    }
    v
}

pub(crate) fn map_normalized_to_surface_uv(
    surface: &NurbsSurfaceCore,
    uv: RgmUv2,
) -> Result<RgmUv2, RgmStatus> {
    if !(0.0..=1.0).contains(&uv.u) || !(0.0..=1.0).contains(&uv.v) {
        return Err(RgmStatus::OutOfRange);
    }

    let u_span = surface.u_end - surface.u_start;
    let v_span = surface.v_end - surface.v_start;
    let mut u = surface.u_start + uv.u * u_span;
    let mut v = surface.v_start + uv.v * v_span;

    if surface.periodic_u && (uv.u - 1.0).abs() <= f64::EPSILON {
        u = surface.u_start;
    }
    if surface.periodic_v && (uv.v - 1.0).abs() <= f64::EPSILON {
        v = surface.v_start;
    }

    Ok(RgmUv2 { u, v })
}

pub(crate) fn eval_nurbs_surface_uv(
    surface: &NurbsSurfaceCore,
    uv_input: RgmUv2,
) -> Result<SurfaceEvalResult, RgmStatus> {
    validate_surface(surface)?;
    let mut u = uv_input.u;
    let mut v = uv_input.v;
    let eps = surface.tol.abs_tol.max(1e-12);

    if surface.periodic_u {
        u = normalize_periodic(u, surface.u_start, surface.u_end);
    } else if u < surface.u_start - eps || u > surface.u_end + eps {
        return Err(RgmStatus::OutOfRange);
    } else {
        u = u.clamp(surface.u_start, surface.u_end);
    }

    if surface.periodic_v {
        v = normalize_periodic(v, surface.v_start, surface.v_end);
    } else if v < surface.v_start - eps || v > surface.v_end + eps {
        return Err(RgmStatus::OutOfRange);
    } else {
        v = v.clamp(surface.v_start, surface.v_end);
    }

    let nu = surface.control_u_count - 1;
    let nv = surface.control_v_count - 1;
    let span_u = find_span(nu, surface.degree_u, u, &surface.knots_u)?;
    let span_v = find_span(nv, surface.degree_v, v, &surface.knots_v)?;
    let ders_u = ders_basis_funs(span_u, u, surface.degree_u, 2, &surface.knots_u)?;
    let ders_v = ders_basis_funs(span_v, v, surface.degree_v, 2, &surface.knots_v)?;

    let mut skl = [[H4::zero(); 3]; 3];
    for iu in 0..=surface.degree_u {
        let ctrl_u = span_u - surface.degree_u + iu;
        for iv in 0..=surface.degree_v {
            let ctrl_v = span_v - surface.degree_v + iv;
            let idx = ctrl_u * surface.control_v_count + ctrl_v;
            let pw = point_to_h4(surface.control_points[idx], surface.weights[idx]);
            for ku in 0..=2 {
                let bu = if ku <= surface.degree_u {
                    ders_u[ku][iu]
                } else {
                    0.0
                };
                for kv in 0..=2 {
                    let bv = if kv <= surface.degree_v {
                        ders_v[kv][iv]
                    } else {
                        0.0
                    };
                    skl[ku][kv].add_scaled(pw, bu * bv);
                }
            }
        }
    }

    let denom_eps = surface.tol.abs_tol.max(1e-14);
    let point = h4_to_point(skl[0][0], denom_eps)?;
    let w00 = skl[0][0].w;
    if w00.abs() <= denom_eps {
        return Err(RgmStatus::NumericalFailure);
    }

    let du = RgmVec3 {
        x: (skl[1][0].x - skl[1][0].w * point.x) / w00,
        y: (skl[1][0].y - skl[1][0].w * point.y) / w00,
        z: (skl[1][0].z - skl[1][0].w * point.z) / w00,
    };
    let dv = RgmVec3 {
        x: (skl[0][1].x - skl[0][1].w * point.x) / w00,
        y: (skl[0][1].y - skl[0][1].w * point.y) / w00,
        z: (skl[0][1].z - skl[0][1].w * point.z) / w00,
    };
    let duu = RgmVec3 {
        x: (skl[2][0].x - 2.0 * skl[1][0].w * du.x - skl[2][0].w * point.x) / w00,
        y: (skl[2][0].y - 2.0 * skl[1][0].w * du.y - skl[2][0].w * point.y) / w00,
        z: (skl[2][0].z - 2.0 * skl[1][0].w * du.z - skl[2][0].w * point.z) / w00,
    };
    let dvv = RgmVec3 {
        x: (skl[0][2].x - 2.0 * skl[0][1].w * dv.x - skl[0][2].w * point.x) / w00,
        y: (skl[0][2].y - 2.0 * skl[0][1].w * dv.y - skl[0][2].w * point.y) / w00,
        z: (skl[0][2].z - 2.0 * skl[0][1].w * dv.z - skl[0][2].w * point.z) / w00,
    };
    let duv = RgmVec3 {
        x: (skl[1][1].x - skl[1][0].w * dv.x - skl[0][1].w * du.x - skl[1][1].w * point.x) / w00,
        y: (skl[1][1].y - skl[1][0].w * dv.y - skl[0][1].w * du.y - skl[1][1].w * point.y) / w00,
        z: (skl[1][1].z - skl[1][0].w * dv.z - skl[0][1].w * du.z - skl[1][1].w * point.z) / w00,
    };

    Ok(SurfaceEvalResult {
        point,
        du,
        dv,
        duu,
        dvv,
        duv,
    })
}

pub(crate) fn eval_nurbs_surface_normalized(
    surface: &NurbsSurfaceCore,
    uv_norm: RgmUv2,
) -> Result<SurfaceEvalResult, RgmStatus> {
    let uv = map_normalized_to_surface_uv(surface, uv_norm)?;
    eval_nurbs_surface_uv(surface, uv)
}
