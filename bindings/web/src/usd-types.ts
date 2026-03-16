export class GfVec2d {
  constructor(
    public x: number,
    public y: number,
  ) {}
}

export class GfVec3d {
  constructor(
    public x: number,
    public y: number,
    public z: number,
  ) {}
}

export class GfVec3f {
  constructor(
    public x: number,
    public y: number,
    public z: number,
  ) {}
}

export type GfMatrix4d = [
  [number, number, number, number],
  [number, number, number, number],
  [number, number, number, number],
  [number, number, number, number],
];

export interface UsdGeomXform {
  xformOpOrder: string[];
  xformOpTransform?: GfMatrix4d;
}

export interface UsdGeomNurbsCurves {
  points: GfVec3f[];
  curveVertexCounts: number[];
  widths: number[];
  order: number[];
  knots: number[];
  ranges: GfVec2d[];
  pointWeights: number[];
}

export interface UsdGeomNurbsPatch {
  points: GfVec3f[];
  uVertexCount: number;
  vVertexCount: number;
  uOrder: number;
  vOrder: number;
  uKnots: number[];
  vKnots: number[];
  uRange?: GfVec2d;
  vRange?: GfVec2d;
  uForm?: string;
  vForm?: string;
  pointWeights: number[];
  trimCurveCounts: number[];
  trimCurveOrders: number[];
  trimCurveVertexCounts: number[];
  trimCurveKnots: number[];
  trimCurveRanges: GfVec2d[];
  trimCurvePoints: GfVec3d[];
}

export interface UsdGeomMesh {
  points: GfVec3f[];
  faceVertexCounts: number[];
  faceVertexIndices: number[];
  normals?: GfVec3f[];
  subdivisionScheme?: string;
}

export interface UsdStagePrim<TSchema = unknown> {
  path: string;
  schema: TSchema;
}

export interface UsdStage {
  prims: Array<UsdStagePrim>;
}

// Compatibility aliases for existing consumer code. Geometry data now uses USD value classes.
export type RgmPoint3 = GfVec3d;
export type RgmVec3 = GfVec3d;
export type RgmVec2 = GfVec2d;
export interface RgmUv2 {
  u: number;
  v: number;
}

export interface RgmLine3 {
  start: GfVec3d;
  end: GfVec3d;
}

export interface RgmPlane {
  origin: GfVec3d;
  x_axis: GfVec3d;
  y_axis: GfVec3d;
  z_axis: GfVec3d;
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
  min: GfVec3d;
  max: GfVec3d;
}

export interface RgmObb3 {
  center: GfVec3d;
  x_axis: GfVec3d;
  y_axis: GfVec3d;
  z_axis: GfVec3d;
  half_extents: GfVec3d;
}

export interface RgmBounds3 {
  world_aabb: RgmAabb3;
  world_obb: RgmObb3;
  local_aabb: RgmAabb3;
}

export interface RgmSurfaceEvalFrame {
  point: GfVec3d;
  du: GfVec3d;
  dv: GfVec3d;
  normal: GfVec3d;
}

export interface RgmTrimLoopInput {
  edge_count: number;
  is_outer: boolean;
}

export interface RgmTrimEdgeInput {
  start_uv: RgmUv2;
  end_uv: RgmUv2;
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
