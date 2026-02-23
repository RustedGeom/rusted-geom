//! Curve–plane and curve–curve intersection algorithms.
//!
//! ## Curve–plane intersection ([`intersect_curve_plane_points`])
//!
//! Samples the curve at 384 uniformly-spaced parameter values, detects
//! sign changes of the signed distance to the plane, then refines each root
//! with a 56-iteration bisection/secant hybrid.  Returns deduplicated hit
//! points sorted by parameter.
//!
//! ## Curve–curve intersection ([`intersect_curve_curve_points`])
//!
//! Samples both curves at 160 points each, builds a grid of segment–segment
//! proximity candidates via [`closest_segment_parameters`] (Ericson 2005,
//! §5.1.9), then refines each candidate with an 8-iteration coordinate-descent
//! grid search.  Falls back to a midpoint seed if no grid hits are found.
//!
//! **Numerical parameters:** `point_tol = abs_tol.max(1e-9)` and
//! `param_tol = 1e-6` are hard-coded.  Both algorithms return deduplicated
//! results where duplicate detection uses both point distance and parameter
//! proximity.

use super::vec3::{add_vec, cross, distance, dot, normalize, scale as scale_vec, sub as sub_vec};
use crate::{RgmPlane, RgmPoint3, RgmStatus, RgmVec3};

#[derive(Clone, Copy, Debug)]
struct HitRecord {
    ta: f64,
    tb: f64,
    point: RgmPoint3,
}

fn signed_distance(point: RgmPoint3, origin: RgmPoint3, normal: RgmVec3) -> f64 {
    dot(sub_vec(point, origin), normal)
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn unique_hit(records: &[HitRecord], candidate: HitRecord, point_tol: f64, param_tol: f64) -> bool {
    !records.iter().any(|record| {
        (record.ta - candidate.ta).abs() <= param_tol
            || (record.tb - candidate.tb).abs() <= param_tol
            || distance(record.point, candidate.point) <= point_tol
    })
}

fn normalize_plane(plane: RgmPlane) -> Result<RgmVec3, RgmStatus> {
    if let Some(normal) = normalize(plane.z_axis) {
        return Ok(normal);
    }
    let fallback = cross(plane.x_axis, plane.y_axis);
    normalize(fallback).ok_or(RgmStatus::InvalidInput)
}

fn refine_plane_root<F>(
    eval_distance: &mut F,
    mut lo: f64,
    mut hi: f64,
    mut vlo: f64,
    mut vhi: f64,
    dist_tol: f64,
    param_tol: f64,
) -> Result<f64, RgmStatus>
where
    F: FnMut(f64) -> Result<f64, RgmStatus>,
{
    for _ in 0..56 {
        let midpoint = 0.5 * (lo + hi);
        let mut candidate = midpoint;
        let denom = vhi - vlo;
        if denom.abs() > 1e-15 {
            let secant = lo - vlo * (hi - lo) / denom;
            if secant.is_finite() && secant > lo && secant < hi {
                candidate = secant;
            }
        }

        let vc = eval_distance(candidate)?;
        if vc.abs() <= dist_tol || (hi - lo).abs() <= param_tol {
            return Ok(candidate);
        }

        if (vlo > 0.0 && vc > 0.0) || (vlo < 0.0 && vc < 0.0) {
            lo = candidate;
            vlo = vc;
        } else {
            hi = candidate;
            vhi = vc;
        }
    }

    Ok(0.5 * (lo + hi))
}

pub(crate) fn intersect_curve_plane_points<F>(
    mut eval_point: F,
    plane: RgmPlane,
    abs_tol: f64,
) -> Result<Vec<RgmPoint3>, RgmStatus>
where
    F: FnMut(f64) -> Result<RgmPoint3, RgmStatus>,
{
    let normal = normalize_plane(plane)?;
    let point_tol = abs_tol.max(1e-9);
    let dist_tol = point_tol * 2.0;
    let param_tol = 1e-6;
    let sample_count = 384_usize;

    let mut ts = Vec::with_capacity(sample_count + 1);
    let mut values = Vec::with_capacity(sample_count + 1);

    for i in 0..=sample_count {
        let t = i as f64 / sample_count as f64;
        let point = eval_point(t)?;
        let value = signed_distance(point, plane.origin, normal);
        ts.push(t);
        values.push(value);
    }

    if values.iter().all(|value| value.abs() <= dist_tol) {
        return Ok(vec![eval_point(0.0)?]);
    }

    let mut raw_hits = Vec::new();
    for idx in 0..sample_count {
        let t0 = ts[idx];
        let t1 = ts[idx + 1];
        let v0 = values[idx];
        let v1 = values[idx + 1];

        if v0.abs() <= dist_tol {
            raw_hits.push(t0);
        }
        if v1.abs() <= dist_tol {
            raw_hits.push(t1);
        }

        if (v0 > 0.0 && v1 < 0.0) || (v0 < 0.0 && v1 > 0.0) {
            let mut eval_distance = |t: f64| -> Result<f64, RgmStatus> {
                let point = eval_point(t)?;
                Ok(signed_distance(point, plane.origin, normal))
            };
            let root = refine_plane_root(&mut eval_distance, t0, t1, v0, v1, dist_tol, param_tol)?;
            raw_hits.push(root);
        }
    }

    raw_hits.sort_by(|a, b| a.total_cmp(b));
    let mut unique_hits: Vec<HitRecord> = Vec::new();
    for t in raw_hits {
        let t = clamp01(t);
        let point = eval_point(t)?;
        let candidate = HitRecord {
            ta: t,
            tb: t,
            point,
        };
        if unique_hit(&unique_hits, candidate, point_tol, param_tol) {
            unique_hits.push(candidate);
        }
    }

    unique_hits.sort_by(|a, b| a.ta.total_cmp(&b.ta));
    Ok(unique_hits.into_iter().map(|hit| hit.point).collect())
}

fn closest_segment_parameters(
    a0: RgmPoint3,
    a1: RgmPoint3,
    b0: RgmPoint3,
    b1: RgmPoint3,
) -> (f64, f64, RgmPoint3, RgmPoint3, f64) {
    let d1 = sub_vec(a1, a0);
    let d2 = sub_vec(b1, b0);
    let r = sub_vec(a0, b0);
    let a = dot(d1, d1);
    let e = dot(d2, d2);
    let f = dot(d2, r);
    let eps = 1e-14;

    let (s, t) = if a <= eps && e <= eps {
        (0.0, 0.0)
    } else if a <= eps {
        (0.0, clamp01(f / e))
    } else {
        let c = dot(d1, r);
        if e <= eps {
            (clamp01(-c / a), 0.0)
        } else {
            let b = dot(d1, d2);
            let denom = a * e - b * b;
            let mut s = if denom.abs() > eps {
                clamp01((b * f - c * e) / denom)
            } else {
                0.0
            };
            let mut t = (b * s + f) / e;
            if t < 0.0 {
                t = 0.0;
                s = clamp01(-c / a);
            } else if t > 1.0 {
                t = 1.0;
                s = clamp01((b - c) / a);
            }
            (s, t)
        }
    };

    let pa = add_vec(a0, scale_vec(d1, s));
    let pb = add_vec(b0, scale_vec(d2, t));
    let dist = distance(pa, pb);
    (s, t, pa, pb, dist)
}

fn midpoint(a: RgmPoint3, b: RgmPoint3) -> RgmPoint3 {
    RgmPoint3 {
        x: 0.5 * (a.x + b.x),
        y: 0.5 * (a.y + b.y),
        z: 0.5 * (a.z + b.z),
    }
}

fn refine_curve_curve_candidate<FA, FB>(
    eval_a: &mut FA,
    eval_b: &mut FB,
    mut ta: f64,
    mut tb: f64,
    mut step_a: f64,
    mut step_b: f64,
) -> Result<(f64, f64, RgmPoint3, f64), RgmStatus>
where
    FA: FnMut(f64) -> Result<RgmPoint3, RgmStatus>,
    FB: FnMut(f64) -> Result<RgmPoint3, RgmStatus>,
{
    let mut best_a = ta;
    let mut best_b = tb;
    let mut pa = eval_a(best_a)?;
    let mut pb = eval_b(best_b)?;
    let mut best_dist = distance(pa, pb);

    for _ in 0..8 {
        let mut improved = false;
        for da in [-step_a, 0.0, step_a] {
            for db in [-step_b, 0.0, step_b] {
                ta = clamp01(best_a + da);
                tb = clamp01(best_b + db);
                let candidate_a = eval_a(ta)?;
                let candidate_b = eval_b(tb)?;
                let candidate_dist = distance(candidate_a, candidate_b);
                if candidate_dist < best_dist {
                    best_dist = candidate_dist;
                    best_a = ta;
                    best_b = tb;
                    pa = candidate_a;
                    pb = candidate_b;
                    improved = true;
                }
            }
        }

        if !improved {
            step_a *= 0.5;
            step_b *= 0.5;
        }
    }

    Ok((best_a, best_b, midpoint(pa, pb), best_dist))
}

pub(crate) fn intersect_curve_curve_points<FA, FB>(
    mut eval_a: FA,
    mut eval_b: FB,
    abs_tol: f64,
) -> Result<Vec<RgmPoint3>, RgmStatus>
where
    FA: FnMut(f64) -> Result<RgmPoint3, RgmStatus>,
    FB: FnMut(f64) -> Result<RgmPoint3, RgmStatus>,
{
    let point_tol = abs_tol.max(1e-9);
    let pair_tol = point_tol * 2.0;
    let samples_a = 160_usize;
    let samples_b = 160_usize;

    let mut points_a = Vec::with_capacity(samples_a + 1);
    let mut points_b = Vec::with_capacity(samples_b + 1);
    for i in 0..=samples_a {
        let t = i as f64 / samples_a as f64;
        points_a.push(eval_a(t)?);
    }
    for i in 0..=samples_b {
        let t = i as f64 / samples_b as f64;
        points_b.push(eval_b(t)?);
    }

    let mut hits: Vec<HitRecord> = Vec::new();
    let mut step_a = 1.0 / samples_a as f64;
    let mut step_b = 1.0 / samples_b as f64;

    for ia in 0..samples_a {
        let a0 = points_a[ia];
        let a1 = points_a[ia + 1];
        for ib in 0..samples_b {
            let b0 = points_b[ib];
            let b1 = points_b[ib + 1];
            let (sa, sb, _pa, _pb, dist) = closest_segment_parameters(a0, a1, b0, b1);
            if dist > pair_tol {
                continue;
            }

            let ta = (ia as f64 + sa) / samples_a as f64;
            let tb = (ib as f64 + sb) / samples_b as f64;
            let (ta, tb, point, refined_dist) =
                refine_curve_curve_candidate(&mut eval_a, &mut eval_b, ta, tb, step_a, step_b)?;
            if refined_dist > pair_tol {
                continue;
            }

            let candidate = HitRecord { ta, tb, point };
            if unique_hit(&hits, candidate, pair_tol, 1e-3) {
                hits.push(candidate);
            }
        }
    }

    step_a *= 0.5;
    step_b *= 0.5;
    if hits.is_empty() {
        let (ta, tb, point, refined_dist) =
            refine_curve_curve_candidate(&mut eval_a, &mut eval_b, 0.5, 0.5, step_a, step_b)?;
        if refined_dist <= pair_tol {
            hits.push(HitRecord { ta, tb, point });
        }
    }

    hits.sort_by(|a, b| a.ta.total_cmp(&b.ta));
    Ok(hits.into_iter().map(|hit| hit.point).collect())
}
