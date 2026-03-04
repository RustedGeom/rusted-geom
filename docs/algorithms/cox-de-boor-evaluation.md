# Cox-de Boor B-Spline Evaluation

Implementation reference for the B-spline basis function algorithms in `crates/kernel/src/math/basis.rs`.

## Overview

rusted-geom evaluates NURBS curves and surfaces using the Cox-de Boor recursion from
*The NURBS Book* (Piegl & Tiller, 1997). Three core routines form the foundation of
all curve and surface evaluation.

## Algorithm A2.1 -- Find Span

Given a parameter value `u`, a knot vector `U`, and the number of basis functions `n`,
`find_span` returns the knot span index `i` such that `U[i] <= u < U[i+1]`.

Special cases:

- If `u == U[n+1]` (the upper domain boundary), the span is clamped to `n`.
- Binary search is used for the general case, giving O(log n) lookup.

## Algorithm A2.2 -- Basis Functions

`basis_funs` computes the `p+1` non-zero basis functions `N_{i-p,p}(u), ..., N_{i,p}(u)`
at parameter `u` using a triangular recursion table. The implementation uses two
temporary arrays (`left` and `right`) sized `p+1` and fills the output in-place.

Complexity: O(p^2) per evaluation.

## Algorithm A2.3 -- Derivatives of Basis Functions

`ders_basis_funs` extends A2.2 to compute basis function derivatives up to order `n`.
The result is a 2D table `ders[k][j]` where `k` is the derivative order and `j` indexes
the `p+1` non-zero functions. Uses the derivative recursion formula with pre-computed
`ndu` and `a` coefficient arrays.

## Curve Evaluation (A4.2)

`crates/kernel/src/math/nurbs_curve_eval.rs` implements rational de Boor evaluation for
NURBS curves. For a curve `C(t)` with `n+1` control points, weights, and knot vector:

1. Map the normalised `t` to the knot domain.
2. Call `find_span` to locate the active knot interval.
3. Call `basis_funs` (or `ders_basis_funs` for derivatives) to compute the non-zero basis values.
4. Accumulate the weighted sum: `C(t) = sum(N_i * w_i * P_i) / sum(N_i * w_i)`.

Derivatives use the quotient rule on the numerator `A(t)` and denominator `w(t)`.

## Surface Evaluation (A4.6)

`crates/kernel/src/math/nurbs_surface_eval.rs` evaluates a tensor-product rational
B-spline surface `S(u,v)`. The algorithm:

1. Evaluate basis functions in `u` and `v` independently.
2. Compute the weighted sum over the (degree_u+1) x (degree_v+1) active control points.
3. For frame evaluation, compute partial derivatives `dS/du` and `dS/dv` using `ders_basis_funs`
   in each direction and the product rule.
4. The surface normal is `normalize(dS/du x dS/dv)`.

## References

- Piegl, L. & Tiller, W. (1997). *The NURBS Book*, 2nd edition. Springer.
  - Algorithm A2.1: Section 2.3
  - Algorithm A2.2: Section 2.5
  - Algorithm A2.3: Section 2.5
  - Algorithm A4.2: Section 4.3
  - Algorithm A4.6: Section 4.3
