//! Canonical 3-D vector and point arithmetic for [`RgmVec3`] and [`RgmPoint3`].
//!
//! All functions are `pub(crate)` and zero-cost (inlined by the compiler).
//! This module is the single source of truth for vector math; the private
//! duplicate implementations previously scattered across `math/frame.rs`,
//! `math/intersections.rs`, and `kernel_impl/curve_geometry.rs` are removed
//! in favour of `use crate::math::vec3 as v3`.

use crate::{RgmPoint3, RgmVec3};

/// Dot product of two vectors.
#[inline]
pub(crate) fn dot(a: RgmVec3, b: RgmVec3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

/// Cross product of two vectors.
#[inline]
pub(crate) fn cross(a: RgmVec3, b: RgmVec3) -> RgmVec3 {
    RgmVec3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

/// Euclidean length (L2 norm) of a vector.
#[inline]
pub(crate) fn norm(v: RgmVec3) -> f64 {
    dot(v, v).sqrt()
}

/// Returns a unit vector, or `None` if `v` is near-zero.
///
/// A vector is considered near-zero when its length is at or below
/// [`f64::EPSILON`].
#[inline]
pub(crate) fn normalize(v: RgmVec3) -> Option<RgmVec3> {
    let n = norm(v);
    if n <= f64::EPSILON {
        return None;
    }
    Some(scale(v, 1.0 / n))
}

/// Multiply a vector by a scalar.
#[inline]
pub(crate) fn scale(v: RgmVec3, s: f64) -> RgmVec3 {
    RgmVec3 {
        x: v.x * s,
        y: v.y * s,
        z: v.z * s,
    }
}

/// Negate a vector (unary minus).
#[inline]
pub(crate) fn neg(v: RgmVec3) -> RgmVec3 {
    RgmVec3 {
        x: -v.x,
        y: -v.y,
        z: -v.z,
    }
}

/// Add two vectors component-wise.
#[inline]
pub(crate) fn add(a: RgmVec3, b: RgmVec3) -> RgmVec3 {
    RgmVec3 {
        x: a.x + b.x,
        y: a.y + b.y,
        z: a.z + b.z,
    }
}

/// Subtract two points, returning the displacement vector `a − b`.
#[inline]
pub(crate) fn sub(a: RgmPoint3, b: RgmPoint3) -> RgmVec3 {
    RgmVec3 {
        x: a.x - b.x,
        y: a.y - b.y,
        z: a.z - b.z,
    }
}

/// Displace a point by a vector: `p + v`.
#[inline]
pub(crate) fn add_vec(p: RgmPoint3, v: RgmVec3) -> RgmPoint3 {
    RgmPoint3 {
        x: p.x + v.x,
        y: p.y + v.y,
        z: p.z + v.z,
    }
}

/// Euclidean distance between two points.
#[inline]
pub(crate) fn distance(a: RgmPoint3, b: RgmPoint3) -> f64 {
    norm(sub(a, b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RgmPoint3, RgmVec3};

    fn vec(x: f64, y: f64, z: f64) -> RgmVec3 {
        RgmVec3 { x, y, z }
    }

    fn pt(x: f64, y: f64, z: f64) -> RgmPoint3 {
        RgmPoint3 { x, y, z }
    }

    #[test]
    fn test_dot_orthogonal() {
        assert_eq!(dot(vec(1.0, 0.0, 0.0), vec(0.0, 1.0, 0.0)), 0.0);
    }

    #[test]
    fn test_dot_parallel() {
        assert_eq!(dot(vec(2.0, 0.0, 0.0), vec(3.0, 0.0, 0.0)), 6.0);
    }

    #[test]
    fn test_cross_unit_axes() {
        let x = vec(1.0, 0.0, 0.0);
        let y = vec(0.0, 1.0, 0.0);
        let z = cross(x, y);
        assert!((z.x - 0.0).abs() < 1e-15);
        assert!((z.y - 0.0).abs() < 1e-15);
        assert!((z.z - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_cross_anticommutative() {
        let a = vec(1.0, 2.0, 3.0);
        let b = vec(4.0, 5.0, 6.0);
        let ab = cross(a, b);
        let ba = cross(b, a);
        assert!((ab.x + ba.x).abs() < 1e-15);
        assert!((ab.y + ba.y).abs() < 1e-15);
        assert!((ab.z + ba.z).abs() < 1e-15);
    }

    #[test]
    fn test_norm() {
        assert!((norm(vec(3.0, 4.0, 0.0)) - 5.0).abs() < 1e-14);
        assert!((norm(vec(1.0, 1.0, 1.0)) - 3_f64.sqrt()).abs() < 1e-14);
    }

    #[test]
    fn test_normalize_unit() {
        let v = normalize(vec(3.0, 0.0, 0.0)).unwrap();
        assert!((v.x - 1.0).abs() < 1e-15);
        assert!(v.y.abs() < 1e-15);
        assert!(v.z.abs() < 1e-15);
        assert!((norm(v) - 1.0).abs() < 1e-14);
    }

    #[test]
    fn test_normalize_zero_vector() {
        // Zero vector must return None, not panic or produce NaN.
        assert!(normalize(vec(0.0, 0.0, 0.0)).is_none());
    }

    #[test]
    fn test_normalize_near_zero() {
        assert!(normalize(vec(f64::EPSILON * 0.5, 0.0, 0.0)).is_none());
    }

    #[test]
    fn test_distance() {
        let a = pt(1.0, 0.0, 0.0);
        let b = pt(4.0, 0.0, 0.0);
        assert!((distance(a, b) - 3.0).abs() < 1e-14);
    }

    #[test]
    fn test_distance_self() {
        let a = pt(1.0, 2.0, 3.0);
        assert!(distance(a, a).abs() < 1e-15);
    }

    #[test]
    fn test_scale() {
        let v = scale(vec(1.0, 2.0, 3.0), 2.0);
        assert_eq!(v.x, 2.0);
        assert_eq!(v.y, 4.0);
        assert_eq!(v.z, 6.0);
    }

    #[test]
    fn test_neg() {
        let v = neg(vec(1.0, -2.0, 3.0));
        assert_eq!(v.x, -1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, -3.0);
    }

    #[test]
    fn test_add() {
        let v = add(vec(1.0, 2.0, 3.0), vec(4.0, 5.0, 6.0));
        assert_eq!(v.x, 5.0);
        assert_eq!(v.y, 7.0);
        assert_eq!(v.z, 9.0);
    }

    #[test]
    fn test_sub() {
        let v = sub(pt(4.0, 5.0, 6.0), pt(1.0, 2.0, 3.0));
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 3.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_add_vec() {
        let p = add_vec(pt(1.0, 2.0, 3.0), vec(10.0, 20.0, 30.0));
        assert_eq!(p.x, 11.0);
        assert_eq!(p.y, 22.0);
        assert_eq!(p.z, 33.0);
    }
}
