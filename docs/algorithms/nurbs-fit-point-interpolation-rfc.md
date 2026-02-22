# NURBS Fit-Point Constructor RFC (M1)

## Goal
Define deterministic behavior for constructing a NURBS curve from fit points plus degree only.

## Inputs
- Fit points (`Point3[]`)
- Degree (`u32`)
- Closed flag (`bool`)
- Tolerance context

## Behavior
1. Validate non-empty points and `degree >= 1`.
2. If `closed = true`, auto-deduplicate seam endpoint when first and last points are within `abs_tol`.
3. Require `fit_point_count > degree`.
4. Compute chord-length parameterization.
5. Set all weights to `1.0`.
6. Build knot vector:
- Open: clamped knot vector with end multiplicity `degree + 1`.
- Closed: periodic knot vector.
7. Persist curve as a session-scoped object handle.

## Notes
- Constructor is exact fit-point interpolation policy for API semantics.
- M1 runtime evaluates through the shared NURBS core (`de Boor` + derivative stack) while preserving constructor guarantees and ABI shape.
- Follow-on milestones should focus on robustness/performance hardening and richer intersection payloads, not placeholder replacement.
