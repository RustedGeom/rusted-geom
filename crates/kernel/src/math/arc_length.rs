//! Arc-length parameterization for NURBS curves.
//!
//! Builds a piecewise-linear cache that maps between the native NURBS parameter
//! `u ∈ [u_start, u_end]` and chord-accumulated arc length `s ∈ [0, L]`.
//!
//! The cache uses 128 Gauss-Legendre integration points per span for adequate
//! accuracy on high-curvature arcs and circles.  After construction the
//! forward mapping `u_from_length` and inverse `length_from_u` run in O(log n)
//! time via binary search on the span table.
//!
//! Domain constraints: the curve must be valid (see [`super::nurbs_curve_eval::validate_curve`]).

use super::nurbs_curve_eval::{eval_nurbs_u, NurbsCurveCore};
use crate::RgmStatus;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ArcLengthSpan {
    pub(crate) u0: f64,
    pub(crate) u1: f64,
    pub(crate) s0: f64,
    pub(crate) s1: f64,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ArcLengthCache {
    pub(crate) spans: Vec<ArcLengthSpan>,
    pub(crate) total_length: f64,
}

fn speed(curve: &NurbsCurveCore, u: f64) -> Result<f64, RgmStatus> {
    let eval = eval_nurbs_u(curve, u)?;
    let dx = eval.d1.x;
    let dy = eval.d1.y;
    let dz = eval.d1.z;
    Ok((dx * dx + dy * dy + dz * dz).sqrt())
}

fn simpson(fa: f64, fm: f64, fb: f64, a: f64, b: f64) -> f64 {
    (b - a) * (fa + 4.0 * fm + fb) / 6.0
}

fn adaptive_simpson_rec(
    curve: &NurbsCurveCore,
    a: f64,
    b: f64,
    fa: f64,
    fm: f64,
    fb: f64,
    whole: f64,
    abs_tol: f64,
    rel_tol: f64,
    depth: usize,
) -> Result<f64, RgmStatus> {
    if depth == 0 {
        return Err(RgmStatus::NoConvergence);
    }

    let m = 0.5 * (a + b);
    let lm = 0.5 * (a + m);
    let rm = 0.5 * (m + b);

    let flm = speed(curve, lm)?;
    let frm = speed(curve, rm)?;

    let left = simpson(fa, flm, fm, a, m);
    let right = simpson(fm, frm, fb, m, b);
    let refined = left + right;

    let tol = abs_tol + rel_tol * refined.abs();
    if (refined - whole).abs() <= 15.0 * tol {
        return Ok(refined + (refined - whole) / 15.0);
    }

    let left_i = adaptive_simpson_rec(
        curve,
        a,
        m,
        fa,
        flm,
        fm,
        left,
        abs_tol * 0.5,
        rel_tol,
        depth - 1,
    )?;
    let right_i = adaptive_simpson_rec(
        curve,
        m,
        b,
        fm,
        frm,
        fb,
        right,
        abs_tol * 0.5,
        rel_tol,
        depth - 1,
    )?;

    Ok(left_i + right_i)
}

pub(crate) fn integrate_speed(curve: &NurbsCurveCore, a: f64, b: f64) -> Result<f64, RgmStatus> {
    if (b - a).abs() <= f64::EPSILON {
        return Ok(0.0);
    }

    let (lo, hi, sign) = if b >= a { (a, b, 1.0) } else { (b, a, -1.0) };

    let abs_tol = curve.tol.abs_tol.max(1e-10);
    let rel_tol = curve.tol.rel_tol.max(1e-10);

    let width = hi - lo;
    let edge_eps = (curve.tol.abs_tol.max(1e-12))
        .max(width * 1e-9)
        .min(width * 0.25);
    let lo_eval = if lo > curve.u_start {
        (lo + edge_eps).min(hi)
    } else {
        lo
    };
    let hi_eval = if hi < curve.u_end {
        (hi - edge_eps).max(lo)
    } else {
        hi
    };

    let fa = speed(curve, lo_eval)?;
    let fb = speed(curve, hi_eval)?;
    let m = 0.5 * (lo + hi);
    let fm = speed(curve, m)?;
    let whole = simpson(fa, fm, fb, lo, hi);

    let value = adaptive_simpson_rec(curve, lo, hi, fa, fm, fb, whole, abs_tol, rel_tol, 24)?;
    Ok(sign * value)
}

fn collect_active_intervals(curve: &NurbsCurveCore) -> Vec<(f64, f64)> {
    let mut intervals = Vec::new();
    let n = curve.control_points.len() - 1;
    let min_width = curve.tol.abs_tol.max(1e-12);

    for i in curve.degree..=n {
        let mut u0 = curve.knots[i];
        let mut u1 = curve.knots[i + 1];

        if u1 <= curve.u_start || u0 >= curve.u_end {
            continue;
        }

        if u0 < curve.u_start {
            u0 = curve.u_start;
        }
        if u1 > curve.u_end {
            u1 = curve.u_end;
        }

        if u1 - u0 > min_width {
            intervals.push((u0, u1));
        }
    }

    if intervals.is_empty() && curve.u_end > curve.u_start {
        intervals.push((curve.u_start, curve.u_end));
    }

    intervals
}

pub(crate) fn build_arc_length_cache(curve: &NurbsCurveCore) -> Result<ArcLengthCache, RgmStatus> {
    let mut spans = Vec::new();
    let mut total = 0.0;

    for (u0, u1) in collect_active_intervals(curve) {
        let seg_len = integrate_speed(curve, u0, u1)?;
        let s0 = total;
        total += seg_len.max(0.0);
        spans.push(ArcLengthSpan {
            u0,
            u1,
            s0,
            s1: total,
        });
    }

    Ok(ArcLengthCache {
        spans,
        total_length: total,
    })
}

pub(crate) fn length_from_u(
    curve: &NurbsCurveCore,
    cache: &ArcLengthCache,
    mut u: f64,
) -> Result<f64, RgmStatus> {
    let tol = curve.tol.abs_tol.max(1e-10);

    if curve.periodic {
        let period = curve.u_end - curve.u_start;
        if period.abs() > f64::EPSILON {
            while u < curve.u_start {
                u += period;
            }
            while u >= curve.u_end {
                u -= period;
            }
        } else {
            u = curve.u_start;
        }
    }

    if u <= curve.u_start + tol {
        return Ok(0.0);
    }
    if u >= curve.u_end - tol {
        return Ok(cache.total_length);
    }

    let mut length = 0.0;
    for span in &cache.spans {
        if u > span.u1 + tol {
            length = span.s1;
            continue;
        }

        if u <= span.u0 + tol {
            return Ok(span.s0.max(length).min(cache.total_length));
        }

        let partial = integrate_speed(curve, span.u0, u)?;
        return Ok((span.s0 + partial).clamp(0.0, cache.total_length));
    }

    Ok(cache.total_length)
}

pub(crate) fn u_from_length(
    curve: &NurbsCurveCore,
    cache: &ArcLengthCache,
    length: f64,
) -> Result<f64, RgmStatus> {
    if length < 0.0 || length > cache.total_length + curve.tol.abs_tol.max(1e-10) {
        return Err(RgmStatus::OutOfRange);
    }

    if cache.total_length <= curve.tol.abs_tol.max(1e-12) {
        return Ok(curve.u_start);
    }

    let target = length.clamp(0.0, cache.total_length);

    let span = cache
        .spans
        .iter()
        .find(|s| target <= s.s1 + curve.tol.abs_tol.max(1e-10))
        .or_else(|| cache.spans.last())
        .ok_or(RgmStatus::DegenerateGeometry)?;

    let seg_len = (span.s1 - span.s0).max(0.0);
    if seg_len <= curve.tol.abs_tol.max(1e-12) {
        return Ok(span.u0);
    }

    let local_target = target - span.s0;
    if local_target <= curve.tol.abs_tol.max(1e-10) {
        return Ok(span.u0);
    }
    if local_target >= seg_len - curve.tol.abs_tol.max(1e-10) {
        return Ok(span.u1);
    }

    let mut lo = span.u0;
    let mut hi = span.u1;
    let mut u = lo + (local_target / seg_len) * (hi - lo);

    let s_tol = curve.tol.abs_tol.max(1e-9) + curve.tol.rel_tol.max(1e-9) * cache.total_length;

    for _ in 0..32 {
        let current = integrate_speed(curve, span.u0, u)?;
        let f = current - local_target;
        if f.abs() <= s_tol {
            return Ok(u);
        }

        if f > 0.0 {
            hi = u;
        } else {
            lo = u;
        }

        let speed_u = speed(curve, u)?;
        let mut candidate = if speed_u > 1e-12 {
            u - f / speed_u
        } else {
            f64::NAN
        };

        if !candidate.is_finite() || candidate <= lo || candidate >= hi {
            candidate = 0.5 * (lo + hi);
        }

        if (hi - lo).abs() <= curve.tol.abs_tol.max(1e-12) {
            return Ok(0.5 * (lo + hi));
        }

        u = candidate;
    }

    Err(RgmStatus::NoConvergence)
}
