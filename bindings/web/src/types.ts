// Pure TypeScript data shape interfaces for the rusted-geom geometry API.
// These are convenience types — they are not WASM-bindgen classes.

export interface RgmPoint3 { x: number; y: number; z: number }
export interface RgmVec3 { x: number; y: number; z: number }
export interface RgmVec2 { x: number; y: number }
export interface RgmUv2 { u: number; v: number }

export interface RgmLine3 {
  start: RgmPoint3;
  end: RgmPoint3;
}

export interface RgmPlane {
  origin: RgmPoint3;
  x_axis: RgmVec3;
  y_axis: RgmVec3;
  z_axis: RgmVec3;
}

export interface RgmCircle3 {
  plane: RgmPlane;
  radius: number;
}

export interface RgmArc3 {
  plane: RgmPlane;
  radius: number;
  start_angle: number;
  sweep_angle: number;
}

export interface RgmNurbsSurfaceDesc {
  degree_u: number;
  degree_v: number;
  periodic_u: boolean;
  periodic_v: boolean;
  control_u_count: number;
  control_v_count: number;
}

export interface RgmSurfaceTessellationOptions {
  min_u_segments: number;
  min_v_segments: number;
  max_u_segments: number;
  max_v_segments: number;
  chord_tol: number;
  normal_tol_rad: number;
}

export interface RgmToleranceContext {
  abs_tol: number;
  rel_tol: number;
  angle_tol: number;
}

export enum RgmBoundsMode {
  Fast = 0,
  Optimal = 1,
}

export interface RgmBoundsOptions {
  mode: RgmBoundsMode;
  sample_budget: number;
  padding: number;
}

export interface RgmAabb3 {
  min: RgmPoint3;
  max: RgmPoint3;
}

export interface RgmObb3 {
  center: RgmPoint3;
  x_axis: RgmVec3;
  y_axis: RgmVec3;
  z_axis: RgmVec3;
  half_extents: RgmVec3;
}

export interface RgmBounds3 {
  world_aabb: RgmAabb3;
  world_obb: RgmObb3;
  local_aabb: RgmAabb3;
}

export interface RgmSurfaceEvalFrame {
  point: RgmPoint3;
  du: RgmVec3;
  dv: RgmVec3;
  normal: RgmVec3;
}

export interface RgmTrimLoopInput {
  edge_count: number;
  is_outer: boolean;
}

export interface RgmTrimEdgeInput {
  start_uv: RgmUv2;
  end_uv: RgmUv2;
  /** object_id as f64 (0 if not present) */
  curve_3d: number;
  has_curve_3d: boolean;
}

export interface RgmIntersectionBranchSummary {
  point_count: number;
  uv_a_count: number;
  uv_b_count: number;
  curve_t_count: number;
  closed: boolean;
  flags: number;
}

export enum RgmValidationSeverity {
  Info = 0,
  Warning = 1,
  Error = 2,
}

export interface RgmBrepValidationReport {
  issue_count: number;
  max_severity: RgmValidationSeverity;
  overflow: boolean;
}
