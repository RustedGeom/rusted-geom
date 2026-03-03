//! B-spline basis function computation.
//!
//! Implements the Cox–de Boor recursion algorithms from Piegl & Tiller,
//! *The NURBS Book*, 2nd ed. (Springer, 1997):
//!
//! * §2.2 — [`find_span`]: binary-search knot-span algorithm (Algorithm A2.1).
//! * §2.2 — [`basis_funs`]: non-zero B-spline basis values (Algorithm A2.2).
//! * §2.3 — [`ders_basis_funs`]: basis function derivatives (Algorithm A2.3).
//!
//! **Numerical stability:** denominators below [`f64::EPSILON`] are treated as
//! zero to avoid division by tiny numbers at knot coincidences.
//!
//! **Domain constraints:** `degree ≤ n`, `knots.len() ≥ n + degree + 2`, and
//! the knot vector must be non-decreasing.

use crate::RgmStatus;

#[inline]
pub(crate) fn find_span(
    n: usize,
    degree: usize,
    u: f64,
    knots: &[f64],
) -> Result<usize, RgmStatus> {
    if degree > n {
        return Err(RgmStatus::InvalidInput);
    }
    if knots.len() < n + degree + 2 {
        return Err(RgmStatus::InvalidInput);
    }

    let u_low = knots[degree];
    let u_high = knots[n + 1];

    if u <= u_low {
        return Ok(degree);
    }
    if u >= u_high {
        return Ok(n);
    }

    let mut low = degree;
    let mut high = n + 1;
    let mut mid = (low + high) / 2;

    while u < knots[mid] || u >= knots[mid + 1] {
        if u < knots[mid] {
            high = mid;
        } else {
            low = mid;
        }
        mid = (low + high) / 2;
    }

    Ok(mid)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn basis_funs(
    span: usize,
    u: f64,
    degree: usize,
    knots: &[f64],
) -> Result<Vec<f64>, RgmStatus> {
    let mut n = vec![0.0_f64; degree + 1];
    let mut left = vec![0.0_f64; degree + 1];
    let mut right = vec![0.0_f64; degree + 1];

    n[0] = 1.0;

    for j in 1..=degree {
        left[j] = u - knots[span + 1 - j];
        right[j] = knots[span + j] - u;
        let mut saved = 0.0;

        for r in 0..j {
            let denom = right[r + 1] + left[j - r];
            let temp = if denom.abs() <= f64::EPSILON {
                0.0
            } else {
                n[r] / denom
            };
            n[r] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        n[j] = saved;
    }

    Ok(n)
}

pub(crate) fn ders_basis_funs(
    span: usize,
    u: f64,
    degree: usize,
    order: usize,
    knots: &[f64],
) -> Result<Vec<Vec<f64>>, RgmStatus> {
    const MAX_STACK_DEGREE: usize = 5;

    let order = order.min(degree);

    if degree <= MAX_STACK_DEGREE {
        // Fast path: stack-allocated working arrays — avoids heap allocation on
        // every call for the common case of degree ≤ 5.
        let mut ndu = [[0.0_f64; MAX_STACK_DEGREE + 1]; MAX_STACK_DEGREE + 1];
        let mut left = [0.0_f64; MAX_STACK_DEGREE + 1];
        let mut right = [0.0_f64; MAX_STACK_DEGREE + 1];

        ndu[0][0] = 1.0;

        for j in 1..=degree {
            left[j] = u - knots[span + 1 - j];
            right[j] = knots[span + j] - u;
            let mut saved = 0.0;

            for r in 0..j {
                ndu[j][r] = right[r + 1] + left[j - r];
                let temp = if ndu[j][r].abs() <= f64::EPSILON {
                    0.0
                } else {
                    ndu[r][j - 1] / ndu[j][r]
                };
                ndu[r][j] = saved + right[r + 1] * temp;
                saved = left[j - r] * temp;
            }

            ndu[j][j] = saved;
        }

        let mut ders = vec![vec![0.0_f64; degree + 1]; order + 1];
        for (j, value) in ders[0].iter_mut().enumerate().take(degree + 1) {
            *value = ndu[j][degree];
        }

        let mut a = [[0.0_f64; MAX_STACK_DEGREE + 1]; 2];

        for r in 0..=degree {
            let mut s1 = 0_usize;
            let mut s2 = 1_usize;
            a[0][0] = 1.0;

            for k in 1..=order {
                let mut d = 0.0;
                let rk = r as isize - k as isize;
                let pk = degree as isize - k as isize;

                if r >= k {
                    let denom = ndu[(pk + 1) as usize][rk as usize];
                    a[s2][0] = if denom.abs() <= f64::EPSILON {
                        0.0
                    } else {
                        a[s1][0] / denom
                    };
                    d = a[s2][0] * ndu[rk as usize][pk as usize];
                }

                let j1 = if rk >= -1 { 1 } else { (-rk) as usize };
                let j2 = if (r as isize - 1) <= pk {
                    k - 1
                } else {
                    degree - r
                };

                for j in j1..=j2 {
                    let denom = ndu[(pk + 1) as usize][(rk + j as isize) as usize];
                    a[s2][j] = if denom.abs() <= f64::EPSILON {
                        0.0
                    } else {
                        (a[s1][j] - a[s1][j - 1]) / denom
                    };
                    d += a[s2][j] * ndu[(rk + j as isize) as usize][pk as usize];
                }

                if (r as isize) <= pk {
                    let denom = ndu[(pk + 1) as usize][r];
                    a[s2][k] = if denom.abs() <= f64::EPSILON {
                        0.0
                    } else {
                        -a[s1][k - 1] / denom
                    };
                    d += a[s2][k] * ndu[r][pk as usize];
                }

                ders[k][r] = d;
                std::mem::swap(&mut s1, &mut s2);
            }
        }

        let mut factor = degree as f64;
        for k in 1..=order {
            for j in 0..=degree {
                ders[k][j] *= factor;
            }
            factor *= (degree - k) as f64;
        }

        Ok(ders)
    } else {
        // Fallback heap path for degree > MAX_STACK_DEGREE.
        let mut ndu = vec![vec![0.0_f64; degree + 1]; degree + 1];
        let mut left = vec![0.0_f64; degree + 1];
        let mut right = vec![0.0_f64; degree + 1];

        ndu[0][0] = 1.0;

        for j in 1..=degree {
            left[j] = u - knots[span + 1 - j];
            right[j] = knots[span + j] - u;
            let mut saved = 0.0;

            for r in 0..j {
                ndu[j][r] = right[r + 1] + left[j - r];
                let temp = if ndu[j][r].abs() <= f64::EPSILON {
                    0.0
                } else {
                    ndu[r][j - 1] / ndu[j][r]
                };
                ndu[r][j] = saved + right[r + 1] * temp;
                saved = left[j - r] * temp;
            }

            ndu[j][j] = saved;
        }

        let mut ders = vec![vec![0.0_f64; degree + 1]; order + 1];
        for (j, value) in ders[0].iter_mut().enumerate().take(degree + 1) {
            *value = ndu[j][degree];
        }

        let mut a = vec![vec![0.0_f64; degree + 1]; 2];

        for r in 0..=degree {
            let mut s1 = 0_usize;
            let mut s2 = 1_usize;
            a[0][0] = 1.0;

            for k in 1..=order {
                let mut d = 0.0;
                let rk = r as isize - k as isize;
                let pk = degree as isize - k as isize;

                if r >= k {
                    let denom = ndu[(pk + 1) as usize][rk as usize];
                    a[s2][0] = if denom.abs() <= f64::EPSILON {
                        0.0
                    } else {
                        a[s1][0] / denom
                    };
                    d = a[s2][0] * ndu[rk as usize][pk as usize];
                }

                let j1 = if rk >= -1 { 1 } else { (-rk) as usize };
                let j2 = if (r as isize - 1) <= pk {
                    k - 1
                } else {
                    degree - r
                };

                for j in j1..=j2 {
                    let denom = ndu[(pk + 1) as usize][(rk + j as isize) as usize];
                    a[s2][j] = if denom.abs() <= f64::EPSILON {
                        0.0
                    } else {
                        (a[s1][j] - a[s1][j - 1]) / denom
                    };
                    d += a[s2][j] * ndu[(rk + j as isize) as usize][pk as usize];
                }

                if (r as isize) <= pk {
                    let denom = ndu[(pk + 1) as usize][r];
                    a[s2][k] = if denom.abs() <= f64::EPSILON {
                        0.0
                    } else {
                        -a[s1][k - 1] / denom
                    };
                    d += a[s2][k] * ndu[r][pk as usize];
                }

                ders[k][r] = d;
                std::mem::swap(&mut s1, &mut s2);
            }
        }

        let mut factor = degree as f64;
        for k in 1..=order {
            for j in 0..=degree {
                ders[k][j] *= factor;
            }
            factor *= (degree - k) as f64;
        }

        Ok(ders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple clamped-open uniform knot vector: degree repetitions at each end.
    fn uniform_knots(degree: usize, n: usize) -> Vec<f64> {
        // n+1 control points, degree+1 start/end clamps
        // knot count = n + degree + 2
        let mut knots = Vec::new();
        for _ in 0..=degree {
            knots.push(0.0);
        }
        let inner = n - degree;
        for i in 1..=inner {
            knots.push(i as f64 / (inner + 1) as f64);
        }
        for _ in 0..=degree {
            knots.push(1.0);
        }
        knots
    }

    #[test]
    fn test_find_span_interior() {
        // degree-3 curve with 5 control points → n=4
        let degree = 3;
        let n = 4;
        let knots = uniform_knots(degree, n);
        // mid-point should map to an interior span
        let span = find_span(n, degree, 0.5, &knots).unwrap();
        assert!(span >= degree);
        assert!(span <= n);
    }

    #[test]
    fn test_find_span_at_start() {
        let degree = 3;
        let n = 4;
        let knots = uniform_knots(degree, n);
        let span = find_span(n, degree, 0.0, &knots).unwrap();
        assert_eq!(span, degree);
    }

    #[test]
    fn test_find_span_at_end() {
        let degree = 3;
        let n = 4;
        let knots = uniform_knots(degree, n);
        let span = find_span(n, degree, 1.0, &knots).unwrap();
        assert_eq!(span, n);
    }

    #[test]
    fn test_find_span_invalid() {
        // degree > n is invalid
        assert!(find_span(2, 3, 0.5, &[0.0; 10]).is_err());
    }

    /// Partition-of-unity: B_{i,p}(u) sum to 1 for any u.
    #[test]
    fn test_basis_funs_partition_of_unity() {
        let degree = 3;
        let n = 6;
        let knots = uniform_knots(degree, n);

        for i in 0..=20 {
            let u = i as f64 / 20.0;
            let span = find_span(n, degree, u, &knots).unwrap();
            let b = basis_funs(span, u, degree, &knots).unwrap();
            let sum: f64 = b.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-13,
                "partition of unity failed at u={u}: sum={sum}"
            );
        }
    }

    /// Non-negativity: all basis function values ≥ 0.
    #[test]
    fn test_basis_funs_nonnegative() {
        let degree = 3;
        let n = 5;
        let knots = uniform_knots(degree, n);

        for i in 0..=20 {
            let u = i as f64 / 20.0;
            let span = find_span(n, degree, u, &knots).unwrap();
            let b = basis_funs(span, u, degree, &knots).unwrap();
            for &v in &b {
                assert!(v >= -1e-15, "negative basis value {v} at u={u}");
            }
        }
    }

    /// Boundary conditions: at u=0 only B_{0,p}=1, at u=1 only B_{n,p}=1.
    #[test]
    fn test_basis_funs_boundary_conditions() {
        let degree = 2;
        let n = 4;
        let knots = uniform_knots(degree, n);

        let span_start = find_span(n, degree, 0.0, &knots).unwrap();
        let b_start = basis_funs(span_start, 0.0, degree, &knots).unwrap();
        assert!((b_start[0] - 1.0).abs() < 1e-14, "B_0 should be 1 at u=0");
        for &v in b_start.iter().skip(1) {
            assert!(v.abs() < 1e-14, "B_i>0 should be 0 at u=0");
        }

        let span_end = find_span(n, degree, 1.0, &knots).unwrap();
        let b_end = basis_funs(span_end, 1.0, degree, &knots).unwrap();
        assert!(
            (b_end[degree] - 1.0).abs() < 1e-14,
            "B_n should be 1 at u=1"
        );
        for &v in b_end.iter().take(degree) {
            assert!(v.abs() < 1e-14, "B_i<n should be 0 at u=1");
        }
    }

    /// Finite-difference validation of first derivatives from `ders_basis_funs`.
    /// The first derivative should match a central-difference approximation.
    #[test]
    fn test_ders_basis_funs_first_derivative() {
        let degree = 3;
        let n = 6;
        let knots = uniform_knots(degree, n);
        let h = 1e-5;

        // Test at a few interior points
        for i in 2..18_usize {
            let u = i as f64 / 20.0;
            let u_lo = (u - h).max(0.0);
            let u_hi = (u + h).min(1.0);

            let span = find_span(n, degree, u, &knots).unwrap();
            let ders = ders_basis_funs(span, u, degree, 1, &knots).unwrap();

            let span_lo = find_span(n, degree, u_lo, &knots).unwrap();
            let b_lo = basis_funs(span_lo, u_lo, degree, &knots).unwrap();
            let span_hi = find_span(n, degree, u_hi, &knots).unwrap();
            let b_hi = basis_funs(span_hi, u_hi, degree, &knots).unwrap();

            // Finite difference approximation of derivative for each basis function
            // They may be on different spans, so only compare when spans are the same
            if span_lo == span && span_hi == span {
                for j in 0..=degree {
                    let fd = (b_hi[j] - b_lo[j]) / (u_hi - u_lo);
                    let analytic = ders[1][j];
                    assert!(
                        (fd - analytic).abs() < 1e-5,
                        "derivative mismatch at u={u}, j={j}: fd={fd}, analytic={analytic}"
                    );
                }
            }
        }
    }
}
