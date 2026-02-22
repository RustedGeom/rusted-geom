# Full NURBS-Core Roadmap (Curve-Complete, Surface-Ready)

## Brief Summary
Implement a full, production-grade curve stack where `pointAt`, `D0`, `D1`, `D2`, `tangentAt`, `planeAt`, `normalAt` all run on a true NURBS evaluator, and where `Line`, `Arc`, `Circle`, `Polyline`, and `Polycurve` are first-class public curve types that evaluate as a single curve from the user perspective.

This roadmap is curve-complete and conversion-complete now, with architecture deliberately prepared for NURBS surface reuse next.

## Grounding (Locked Mathematical References)
1. Use NURBS Book algorithms as primary math contract:
   1. A2.1 `FindSpan`.
   2. A2.2 `BasisFuns`.
   3. A2.3 `DersBasisFuns`.
   4. A3.1/A3.2 non-rational point/derivatives.
   5. A4.1/A4.2 rational point/derivatives.
2. Match OCCT runtime behavior patterns:
   1. Periodic parameter normalization before evaluation.
   2. Robust span location with knot-edge tolerance handling.
   3. Derivative-based differential properties (`D1`,`D2`) for normal/curvature logic.
3. Keep the current API philosophy but replace placeholder math in `/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel-ffi/src/lib.rs`.

## Public API and Type Additions (Decision-Complete)

### New FFI Types
Add to `/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel-ffi/src/lib.rs` and generated headers:
```c
typedef struct RgmLine3 {
  RgmPoint3 start;
  RgmPoint3 end;
} RgmLine3;

typedef struct RgmCircle3 {
  RgmPlane plane;      // origin=center, x/y define circle plane
  double radius;
} RgmCircle3;

typedef struct RgmArc3 {
  RgmPlane plane;      // origin=center, x/y define arc plane
  double radius;
  double start_angle;  // radians
  double sweep_angle;  // radians, signed
} RgmArc3;

typedef struct RgmPolycurveSegment {
  RgmObjectHandle curve;
  bool reversed;
} RgmPolycurveSegment;
```

### New Constructors and Conversion APIs
Add new C exports and generated TS names in ABI metadata:
1. `rgm_curve_create_line` (`createLine`).
2. `rgm_curve_create_circle` (`createCircle`).
3. `rgm_curve_create_arc` (`createArc`).
4. `rgm_curve_create_polyline` (`createPolyline`).
5. `rgm_curve_create_polycurve` (`createPolycurve`).
6. `rgm_curve_to_nurbs` (`toNurbs`).

### Existing APIs That Change Internals Only
1. Keep existing evaluation function signatures in generated TS bindings.
2. Route all of them through the new evaluator core.
3. Keep `rgm_nurbs_interpolate_fit_points` as-is (constructor policy unchanged, evaluator changes).

## Internal Object Model
Replace single-variant curve storage with explicit curve family:
1. `GeometryObject::NurbsCurve(NurbsCurveData)`.
2. `GeometryObject::Line(LineData)`.
3. `GeometryObject::Arc(ArcData)`.
4. `GeometryObject::Circle(CircleData)`.
5. `GeometryObject::Polyline(PolylineData)`.
6. `GeometryObject::Polycurve(PolycurveData)`.

Each non-NURBS primitive stores:
1. Its original definition.
2. A canonical exact NURBS representation (where exact exists).
3. Cached arc-length metadata for fast `*AtLength`.

`PolycurveData` stores:
1. Ordered segment handles with `reversed`.
2. Cumulative segment lengths.
3. Global length.
4. Optional cached concatenated single NURBS for `toNurbs`.

## Core Evaluator Design (Reusable for Surfaces Later)
Create internal modules under `/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel-ffi/src/`:
1. `math/basis.rs` for A2.x routines.
2. `math/nurbs_curve_eval.rs` for homogeneous de Boor + derivative evaluation.
3. `math/arc_length.rs` for integration/inversion.
4. `math/frame.rs` for tangent/normal/plane.
5. `math/mod.rs` as stable internal interface.

Expose one canonical function:
1. `eval_curve_u(curve_nurbs, u) -> {point, d1, d2}`.
2. All APIs call this, directly or through polycurve segment dispatch.

## Algorithm Specifications

### NURBS D0/D1/D2
1. D0:
   1. Convert control points to homogeneous `(xw,yw,zw,w)`.
   2. Evaluate with de Boor in active span.
   3. Divide by weight.
2. D1/D2:
   1. Compute basis derivatives up to order 2 via A2.3.
   2. Build homogeneous derivatives.
   3. Convert to Cartesian derivatives via A4.2 formulas.
3. Denominator/ill-conditioned checks:
   1. `|w| <= eps` returns `NumericalFailure`.

### Parameter Domain
1. Open NURBS:
   1. `u in [U[p], U[n+1]]`.
2. Periodic NURBS:
   1. Normalize into one period OCCT-style.
3. `t_norm` mapping:
   1. `t_norm in [0,1]`.
   2. Map linearly to curve domain.
   3. Seam handling deterministic (`t=1` endpoint equivalent for periodic).

### Arc-Length
1. Build true length cache from `|D1(u)|`.
2. Integrate per span with adaptive Simpson.
3. Invert length with safeguarded Newton + bisection fallback.
4. Use tolerance context with numeric floors.
5. On iteration failure return `NoConvergence`.

### Frame Contract
1. `tangentAt` is normalized `D1`.
2. `normalAt`:
   1. Use derivative-based principal normal when curvature is valid.
   2. Use stable world-up fallback when curvature is near zero.
3. `planeAt`:
   1. `x_axis = tangent`.
   2. `z_axis = normal`.
   3. `y_axis = normalize(cross(z_axis, x_axis))`.
4. Keep `normalAt == planeAt.z_axis` invariant.

## Primitive-to-NURBS Exact Conversion Rules

### Line
1. Degree 1 clamped.
2. Control points `[start, end]`.
3. Weights `[1,1]`.
4. Knots `[0,0,1,1]`.

### Polyline
1. Degree 1 clamped multi-span.
2. Controls = polyline vertices.
3. Weights all 1.
4. Open/closed behavior explicit via constructor flag.

### Arc
1. Exact rational degree-2 construction.
2. Split into spans where `|sweep_segment| <= pi/2`.
3. For each span, middle weight `cos(delta/2)`.
4. Concatenate spans with correct knot multiplicities at joints.

### Circle
1. Exact rational degree-2 as 4 quarter arcs.
2. Standard alternating weights `1` and `sqrt(2)/2`.
3. Closed periodic domain.

### Polycurve
1. Treated as one global curve for evaluation.
2. Piecewise exact mapping:
   1. Global parameter/length selects segment via cumulative length partition.
   2. Segment-local evaluation uses underlying segment evaluator.
3. Joint policy:
   1. D0 continuous at segment boundaries if endpoints match.
   2. D1/D2 are one-sided at C0 joints with deterministic side selection.
4. `toNurbs` on polycurve:
   1. Convert each segment to NURBS.
   2. Degree-elevate segments to common degree.
   3. Reparameterize segment domains into contiguous global domain.
   4. Concatenate by knot/control merging preserving piecewise exactness.
   5. Return one NURBS curve handle.

## Evaluation Routing Rules
1. Existing eval APIs dispatch by object type.
2. For `NurbsCurve`, evaluate directly.
3. For `Line/Arc/Circle/Polyline`, evaluate via stored canonical NURBS.
4. For `Polycurve`, evaluate by piecewise segment selection and underlying segment NURBS eval.
5. `toNurbs` returns canonical NURBS representation:
   1. For primitive objects: exact.
   2. For polycurve: concatenated exact piecewise NURBS.

## Error Semantics
1. `InvalidInput` for malformed definitions or null pointers.
2. `OutOfRange` for invalid normalized parameter or length.
3. `DegenerateGeometry` for undefined tangent/frame after fallback.
4. `NumericalFailure` for invalid rational denominator or unstable numeric state.
5. `NoConvergence` for integration/inversion failure.

## Phased Execution Plan

### Phase 1: Core Math Replacement
1. Implement basis/de Boor/rational derivatives.
2. Replace placeholder evaluator and hardcoded `D2`.
3. Add finite-difference derivative validation tests.

### Phase 2: True Length System
1. Implement arc-length integration cache.
2. Implement length inversion for all `*AtLength`.
3. Remove old `length/total_length` linear-parameter shortcut behavior.

### Phase 3: Primitive Curve Types + Constructors
1. Add new FFI types and constructors for line/arc/circle/polyline.
2. Store canonical exact NURBS alongside primitive definitions.
3. Ensure all evaluation APIs work uniformly across primitive and NURBS handles.

### Phase 4: Polycurve as Single-Curve Evaluation Object
1. Add polycurve segment type and constructor.
2. Implement global parameter and length mapping across segments.
3. Implement deterministic joint behavior for derivatives.
4. Implement `toNurbs` concatenation path.

### Phase 5: ABI/Bindings/Tooling Updates
1. Update metadata annotations in `/Users/cesarecaoduro/GitHub/rusted-geom/crates/kernel-ffi/src/lib.rs`.
2. Regenerate:
   1. `/Users/cesarecaoduro/GitHub/rusted-geom/bindings/web/src/generated/*`
3. Update ABI baseline at `/Users/cesarecaoduro/GitHub/rusted-geom/abi/baseline/rgm_abi.json`.

### Phase 6: Hardening and Readiness Gate
1. Stress tests across random knot vectors and high degree.
2. Robustness tests near repeated knots and seam boundaries.
3. Performance baseline and regression guard.

### Phase 7: Surface-Ready Foundation (No Surface API Yet)
1. Ensure curve math modules are dimension-agnostic where useful.
2. Define internal interfaces reusable for tensor-product surface evaluator.
3. Add TODO contracts and module boundaries so surface implementation is a direct follow-on.

## Test Cases and Acceptance Scenarios

### Core Accuracy
1. `D0 == pointAt`.
2. `D1` and `D2` match finite differences within strict tolerances.
3. Rational curves preserve expected exact geometry.

### Shape Families
1. Line:
   1. Constant tangent.
   2. Zero second derivative.
2. Circle:
   1. Constant radius.
   2. Seam continuity.
3. Arc:
   1. Endpoints and sweep consistency.
4. Polyline:
   1. Exact vertices at knot boundaries.
   2. One-sided derivative at corners.
5. Polycurve:
   1. Global parameterization monotonicity.
   2. Correct segment dispatch.
   3. `toNurbs` roundtrip evaluation equivalence.

### Length APIs
1. Monotonicity of length mapping.
2. Endpoint correctness for `pointAtLength(0)` and `pointAtLength(total)`.
3. Nonlinear parameter/length difference on curved examples.

### Error Paths
1. Out-of-range normalized parameter and length.
2. Degenerate geometry frame failures.
3. No-convergence synthetic stress cases.

## Explicit Assumptions and Defaults
1. Angle units are radians.
2. Tolerance defaults use incoming `RgmToleranceContext` with minimum numeric floors.
3. Primitive constructors validate geometry strictly.
4. Polycurve is globally evaluable as one curve by piecewise exact semantics.
5. Surface APIs are not added in this roadmap, but architecture is prepared so the next roadmap can add NURBS surfaces with minimal refactor.
