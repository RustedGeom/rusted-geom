
use boolmesh::{
    compute_boolean,
    prelude::{Manifold, OpType as BoolOpType},
};
use kernel_abi_meta::{rgm_export, rgm_ffi_type};
use crate::math;
use crate::math::arc_length::{build_arc_length_cache, length_from_u, u_from_length};
use crate::math::vec3 as v3;
use crate::math::frame::{
    normal as frame_normal, orthonormalize_plane_axes, plane as frame_plane, point_from_frame,
    tangent as frame_tangent,
};
use crate::math::intersections::{intersect_curve_curve_points, intersect_curve_plane_points};
use crate::math::nurbs_curve_eval::{
    eval_nurbs_normalized, eval_nurbs_u, map_normalized_to_u, validate_curve, CurveEvalResult,
    NurbsCurveCore,
};
use crate::math::nurbs_surface_eval::{
    eval_nurbs_surface_normalized, eval_nurbs_surface_uv_unchecked, validate_surface,
    NurbsSurfaceCore, SurfaceEvalResult,
};
use crate::session::objects::*;
use crate::session::store::*;
use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::f64::consts::{FRAC_PI_2, PI};

/// Return-code type for all C-ABI exports.
///
/// Every exported function returns `RgmStatus::Ok` on success.  On failure the
/// per-session error is also written via [`set_error`] so callers can retrieve a
/// human-readable description with `rgm_last_error_message`.
#[rgm_ffi_type]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RgmStatus {
    /// The operation completed successfully.
    Ok = 0,
    /// One or more arguments were null, out-of-range, or logically inconsistent.
    InvalidInput = 1,
    /// The referenced session or object handle does not exist.
    NotFound = 2,
    /// A parameter (e.g. a curve parameter `t`) is outside its valid domain.
    OutOfRange = 3,
    /// The geometry is singular or too degenerate for the requested operation
    /// (e.g. zero-length tangent vector, collinear arc points).
    DegenerateGeometry = 4,
    /// An iterative numerical solver failed to converge within its iteration limit.
    NoConvergence = 5,
    /// An internal floating-point calculation produced a non-finite result.
    NumericalFailure = 6,
    /// The requested operation is recognized but not yet implemented.
    NotImplemented = 7,
    /// An unexpected internal error occurred (mutex poisoning, allocation failure, …).
    InternalError = 255,
}

/// Opaque session handle returned by `rgm_kernel_create`.
///
/// A `RgmKernelHandle` identifies an isolated geometry session.  Sessions are
/// independent: objects in one session cannot be referenced from another.
/// Destroy with `rgm_kernel_destroy` when done; all objects in the session are
/// released automatically.
///
/// The zero value (`RgmKernelHandle(0)`) is never a valid handle.
#[rgm_ffi_type]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RgmKernelHandle(pub u64);

/// Opaque handle to a geometry object (curve, surface, mesh, face, or
/// intersection result) within a session.
///
/// Handles are session-scoped: they are invalidated when the owning session is
/// destroyed, or when `rgm_object_release` is called on them.  Passing a handle
/// from a different session is `InvalidInput`.
///
/// The zero value (`RgmObjectHandle(0)`) is never a valid handle.
#[rgm_ffi_type]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RgmObjectHandle(pub u64);

/// A point in 3-D world space.  Coordinates are in the same linear unit as the
/// session tolerance (typically metres or millimetres — see [`RgmToleranceContext`]).
#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmPoint3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A free vector (direction + magnitude) in 3-D world space.
///
/// Unlike [`RgmPoint3`], `RgmVec3` has no position; it represents a displacement
/// or direction.  The same linear unit as the session tolerance applies to the
/// magnitude.
#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A right-handed orthonormal coordinate frame.
///
/// `x_axis`, `y_axis`, and `z_axis` are expected to be unit vectors forming a
/// right-handed triad (`z = x × y`).  `origin` is the frame origin in world
/// space.  Several APIs accept a `RgmPlane` as an input frame and will
/// orthonormalize it if the axes are not perfectly unit/orthogonal.
#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmPlane {
    pub origin: RgmPoint3,
    pub x_axis: RgmVec3,
    pub y_axis: RgmVec3,
    pub z_axis: RgmVec3,
}

/// Numerical tolerances used by geometry construction and evaluation.
///
/// All three fields must be strictly positive.
///
/// | Field        | Semantic                                                   | Typical value |
/// |--------------|------------------------------------------------------------|---------------|
/// | `abs_tol`    | Maximum acceptable point-to-point distance error (linear) | `1e-6` m      |
/// | `rel_tol`    | Relative distance tolerance (fraction of characteristic length) | `1e-4` |
/// | `angle_tol`  | Maximum acceptable angular error (radians)                 | `1e-6` rad    |
///
/// Pass a `RgmToleranceContext` by const pointer to construction functions.
/// The kernel stores a copy alongside each created object so that subsequent
/// operations (arc-length caching, intersection, tessellation) use consistent
/// tolerances without requiring the caller to re-specify them.
#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmToleranceContext {
    /// Maximum acceptable linear distance error (same unit as world coordinates).
    pub abs_tol: f64,
    /// Relative distance tolerance as a fraction of characteristic geometry length.
    pub rel_tol: f64,
    /// Maximum acceptable angular error, in radians.
    pub angle_tol: f64,
}

#[rgm_ffi_type]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RgmAlignmentCoordinateSystem {
    EastingNorthing = 0,
    NorthingEasting = 1,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmLine3 {
    pub start: RgmPoint3,
    pub end: RgmPoint3,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmCircle3 {
    pub plane: RgmPlane,
    pub radius: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmArc3 {
    pub plane: RgmPlane,
    pub radius: f64,
    pub start_angle: f64,
    pub sweep_angle: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmPolycurveSegment {
    pub curve: RgmObjectHandle,
    pub reversed: bool,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmPoint2 {
    pub x: f64,
    pub y: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmVec2 {
    pub x: f64,
    pub y: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmUv2 {
    pub u: f64,
    pub v: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmNurbsSurfaceDesc {
    pub degree_u: u32,
    pub degree_v: u32,
    pub periodic_u: bool,
    pub periodic_v: bool,
    pub control_u_count: u32,
    pub control_v_count: u32,
}

/// Surface evaluation result at a single (u, v) parameter.
///
/// All vectors are expressed in world space after the surface transform is
/// applied.
///
/// * `point`  — The 3-D position on the surface.
/// * `du`     — First partial derivative with respect to `u` (un-normalized).
/// * `dv`     — First partial derivative with respect to `v` (un-normalized).
/// * `normal` — Unit surface normal (`du × dv`, normalized).  Returned as a
///              zero vector if the surface is degenerate at this point.
#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmSurfaceEvalFrame {
    pub point: RgmPoint3,
    pub du: RgmVec3,
    pub dv: RgmVec3,
    pub normal: RgmVec3,
}

/// One edge of a trim loop, specified by start/end parameter-space UV
/// coordinates and an optional 3-D curve that lies on the surface.
///
/// `start_uv` and `end_uv` are normalized UV coordinates in `[0, 1]²`.
/// If `has_curve_3d` is `true`, `curve_3d` must be a valid curve handle in the
/// same session; the kernel samples it when building the edge UV polyline.
/// If `has_curve_3d` is `false`, the `curve_3d` field is ignored and the edge
/// is interpolated linearly in UV space.
#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmTrimEdgeInput {
    pub start_uv: RgmUv2,
    pub end_uv: RgmUv2,
    pub curve_3d: RgmObjectHandle,
    pub has_curve_3d: bool,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmTrimLoopInput {
    pub edge_count: u32,
    pub is_outer: bool,
}

// S6: Named constants for entity_kind field in RgmValidationIssue, replacing magic integers.
#[rgm_ffi_type]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RgmBrepEntityKind {
    Edge = 1,
    Trim = 2,
    Loop = 3,
    Face = 4,
    Shell = 5,
    Solid = 6,
}

#[rgm_ffi_type]
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RgmValidationSeverity {
    Info = 0,
    Warning = 1,
    Error = 2,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmValidationIssue {
    pub severity: RgmValidationSeverity,
    pub code: u32,
    pub entity_kind: u32,
    pub entity_id: u32,
    pub param_u: f64,
    pub param_v: f64,
}

impl Default for RgmValidationIssue {
    fn default() -> Self {
        Self {
            severity: RgmValidationSeverity::Info,
            code: 0,
            entity_kind: 0,
            entity_id: 0,
            param_u: f64::NAN,
            param_v: f64::NAN,
        }
    }
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmBrepValidationReport {
    pub issue_count: u32,
    pub max_severity: RgmValidationSeverity,
    pub overflow: bool,
    pub issues: [RgmValidationIssue; 16],
}

impl Default for RgmBrepValidationReport {
    fn default() -> Self {
        Self {
            issue_count: 0,
            max_severity: RgmValidationSeverity::Info,
            overflow: false,
            issues: [RgmValidationIssue::default(); 16],
        }
    }
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmSurfaceTessellationOptions {
    pub min_u_segments: u32,
    pub min_v_segments: u32,
    pub max_u_segments: u32,
    pub max_v_segments: u32,
    pub chord_tol: f64,
    pub normal_tol_rad: f64,
}

#[rgm_ffi_type]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgmIntersectionBranchSummary {
    pub point_count: u32,
    pub uv_a_count: u32,
    pub uv_b_count: u32,
    pub curve_t_count: u32,
    pub closed: bool,
    pub flags: u32,
}

impl Default for RgmStatus {
    fn default() -> Self {
        Self::Ok
    }
}

