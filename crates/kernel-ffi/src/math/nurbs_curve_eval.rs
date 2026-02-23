//! NURBS curve evaluation: position and derivatives.
//!
//! Implements the rational de Boor (NURBS) point evaluation and derivative
//! algorithms from Piegl & Tiller, *The NURBS Book*, 2nd ed.:
//!
//! * §4.1 — [`eval_nurbs_u`]: weighted B-spline evaluation at a raw knot
//!   parameter `u` (Algorithm A4.2 extended to compute first and second
//!   derivatives simultaneously).
//! * Normalization helpers map the public `[0, 1]` domain to the curve's
//!   native `[u_start, u_end]` domain; see [`eval_nurbs_normalized`].
//!
//! **Periodicity:** for periodic curves the raw parameter is wrapped back into
//! `[u_start, u_end]` before evaluation using knot-range modular arithmetic.
//!
//! **Domain constraints:** the curve must pass [`validate_curve`] before any
//! evaluation call.  Validation checks degree, control-point count, and knot
//! vector monotonicity.

use super::basis::{basis_funs, ders_basis_funs, find_span};
use crate::{RgmPoint3, RgmStatus, RgmToleranceContext, RgmVec3};

#[derive(Clone, Debug)]
pub(crate) struct NurbsCurveCore {
    pub(crate) degree: usize,
    pub(crate) periodic: bool,
    pub(crate) control_points: Vec<RgmPoint3>,
    pub(crate) weights: Vec<f64>,
    pub(crate) knots: Vec<f64>,
    pub(crate) u_start: f64,
    pub(crate) u_end: f64,
    pub(crate) tol: RgmToleranceContext,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CurveEvalResult {
    pub(crate) point: RgmPoint3,
    pub(crate) d1: RgmVec3,
    pub(crate) d2: RgmVec3,
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

    fn blend(a: H4, b: H4, alpha: f64) -> H4 {
        let beta = 1.0 - alpha;
        H4 {
            x: beta * a.x + alpha * b.x,
            y: beta * a.y + alpha * b.y,
            z: beta * a.z + alpha * b.z,
            w: beta * a.w + alpha * b.w,
        }
    }
}

fn to_h4(point: RgmPoint3, weight: f64) -> H4 {
    H4 {
        x: point.x * weight,
        y: point.y * weight,
        z: point.z * weight,
        w: weight,
    }
}

pub(crate) fn validate_curve(curve: &NurbsCurveCore) -> Result<(), RgmStatus> {
    if curve.control_points.is_empty() {
        return Err(RgmStatus::DegenerateGeometry);
    }
    if curve.weights.len() != curve.control_points.len() {
        return Err(RgmStatus::InvalidInput);
    }
    if curve.degree == 0 {
        return Err(RgmStatus::InvalidInput);
    }
    if curve.control_points.len() <= curve.degree {
        return Err(RgmStatus::InvalidInput);
    }
    let n = curve.control_points.len() - 1;
    let min_knots = n + curve.degree + 2;
    if curve.knots.len() < min_knots {
        return Err(RgmStatus::InvalidInput);
    }
    if curve.u_end < curve.u_start {
        return Err(RgmStatus::InvalidInput);
    }
    Ok(())
}

pub(crate) fn normalize_periodic_u(curve: &NurbsCurveCore, u: f64) -> f64 {
    if !curve.periodic {
        return u;
    }

    let period = curve.u_end - curve.u_start;
    if period.abs() <= f64::EPSILON {
        return curve.u_start;
    }

    let mut value = u;
    while value < curve.u_start {
        value += period;
    }
    while value >= curve.u_end {
        value -= period;
    }
    value
}

pub(crate) fn map_normalized_to_u(curve: &NurbsCurveCore, t_norm: f64) -> Result<f64, RgmStatus> {
    if !(0.0..=1.0).contains(&t_norm) {
        return Err(RgmStatus::OutOfRange);
    }

    let span = curve.u_end - curve.u_start;
    if span.abs() <= f64::EPSILON {
        return Ok(curve.u_start);
    }

    if curve.periodic {
        if (t_norm - 1.0).abs() <= f64::EPSILON {
            return Ok(curve.u_start);
        }
        return Ok(curve.u_start + t_norm * span);
    }

    Ok(curve.u_start + t_norm * span)
}

fn de_boor_homogeneous(curve: &NurbsCurveCore, span: usize, u: f64) -> Result<H4, RgmStatus> {
    let p = curve.degree;
    let mut d = vec![H4::zero(); p + 1];

    for (j, slot) in d.iter_mut().enumerate().take(p + 1) {
        let idx = span - p + j;
        *slot = to_h4(curve.control_points[idx], curve.weights[idx]);
    }

    for r in 1..=p {
        for j in (r..=p).rev() {
            let i = span - p + j;
            let left = curve.knots[i];
            let right = curve.knots[i + p - r + 1];
            let denom = right - left;
            let alpha = if denom.abs() <= f64::EPSILON {
                0.0
            } else {
                (u - left) / denom
            };
            d[j] = H4::blend(d[j - 1], d[j], alpha);
        }
    }

    Ok(d[p])
}

fn point_from_h4(h: H4, denom_eps: f64) -> Result<RgmPoint3, RgmStatus> {
    if h.w.abs() <= denom_eps {
        return Err(RgmStatus::NumericalFailure);
    }

    Ok(RgmPoint3 {
        x: h.x / h.w,
        y: h.y / h.w,
        z: h.z / h.w,
    })
}

pub(crate) fn eval_nurbs_u(
    curve: &NurbsCurveCore,
    u_input: f64,
) -> Result<CurveEvalResult, RgmStatus> {
    validate_curve(curve)?;

    let mut u = if curve.periodic {
        normalize_periodic_u(curve, u_input)
    } else {
        u_input
    };

    let u_eps = curve.tol.abs_tol.max(1e-12);
    if !curve.periodic {
        if u < curve.u_start - u_eps || u > curve.u_end + u_eps {
            return Err(RgmStatus::OutOfRange);
        }
        if u < curve.u_start {
            u = curve.u_start;
        } else if u > curve.u_end {
            u = curve.u_end;
        }
    }

    let n = curve.control_points.len() - 1;
    let span = find_span(n, curve.degree, u, &curve.knots)?;

    let p_h = de_boor_homogeneous(curve, span, u)?;

    let ders = ders_basis_funs(span, u, curve.degree, 2, &curve.knots)?;

    let mut c0w = H4::zero();
    let mut c1w = H4::zero();
    let mut c2w = H4::zero();

    for j in 0..=curve.degree {
        let idx = span - curve.degree + j;
        let pw = to_h4(curve.control_points[idx], curve.weights[idx]);
        c0w.add_scaled(pw, ders[0][j]);
        c1w.add_scaled(pw, ders.get(1).map(|d| d[j]).unwrap_or(0.0));
        c2w.add_scaled(pw, ders.get(2).map(|d| d[j]).unwrap_or(0.0));
    }

    let denom_eps = curve.tol.abs_tol.max(1e-14);
    let point = point_from_h4(p_h, denom_eps)?;
    if c0w.w.abs() <= denom_eps {
        return Err(RgmStatus::NumericalFailure);
    }

    let d1 = RgmVec3 {
        x: (c1w.x - c1w.w * point.x) / c0w.w,
        y: (c1w.y - c1w.w * point.y) / c0w.w,
        z: (c1w.z - c1w.w * point.z) / c0w.w,
    };

    let d2 = RgmVec3 {
        x: (c2w.x - 2.0 * c1w.w * d1.x - c2w.w * point.x) / c0w.w,
        y: (c2w.y - 2.0 * c1w.w * d1.y - c2w.w * point.y) / c0w.w,
        z: (c2w.z - 2.0 * c1w.w * d1.z - c2w.w * point.z) / c0w.w,
    };

    Ok(CurveEvalResult { point, d1, d2 })
}

pub(crate) fn eval_nurbs_normalized(
    curve: &NurbsCurveCore,
    t_norm: f64,
) -> Result<CurveEvalResult, RgmStatus> {
    let u = map_normalized_to_u(curve, t_norm)?;
    eval_nurbs_u(curve, u)
}

#[allow(dead_code)]
pub(crate) fn evaluate_point_basis(
    curve: &NurbsCurveCore,
    u_input: f64,
) -> Result<RgmPoint3, RgmStatus> {
    validate_curve(curve)?;

    let mut u = if curve.periodic {
        normalize_periodic_u(curve, u_input)
    } else {
        u_input
    };

    if !curve.periodic {
        if u < curve.u_start || u > curve.u_end {
            return Err(RgmStatus::OutOfRange);
        }
        if u == curve.u_end {
            u = curve.u_end;
        }
    }

    let n = curve.control_points.len() - 1;
    let span = find_span(n, curve.degree, u, &curve.knots)?;
    let basis = basis_funs(span, u, curve.degree, &curve.knots)?;

    let mut c0w = H4::zero();
    for (j, value) in basis.iter().enumerate().take(curve.degree + 1) {
        let idx = span - curve.degree + j;
        c0w.add_scaled(to_h4(curve.control_points[idx], curve.weights[idx]), *value);
    }

    point_from_h4(c0w, curve.tol.abs_tol.max(1e-14))
}
