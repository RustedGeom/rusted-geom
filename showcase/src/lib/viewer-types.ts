import type { MeshHandle, RgmPoint3, RgmVec3 } from "@rusted-geom/bindings-web";

export type LogLevel = "info" | "debug" | "error";
export type GizmoMode = "translate" | "rotate" | "scale";
export type KernelStatus = "booting" | "ready" | "computing" | "error";

export interface LogEntry {
  id: number;
  level: LogLevel;
  time: string;
  message: string;
}

export interface ProbeUiState {
  tNorm: number;
  x: number;
  y: number;
  z: number;
  probeLength: number;
  totalLength: number;
}

export interface SurfaceProbeUiState {
  u: number;
  v: number;
  point: RgmPoint3;
  du: RgmVec3;
  dv: RgmVec3;
  normal: RgmVec3;
  hasD2: boolean;
  duu: RgmVec3;
  duv: RgmVec3;
  dvv: RgmVec3;
}

export interface MeshVisual {
  vertices: RgmPoint3[];
  indices: number[];
  color: string;
  opacity: number;
  wireframe?: boolean;
  name: string;
}

export interface OverlayCurveVisual {
  points: RgmPoint3[];
  color: string;
  width: number;
  opacity: number;
  name: string;
}

export interface SegmentOverlayVisual {
  points: RgmPoint3[];
  color: string;
  opacity: number;
  width?: number;
  name: string;
}

export interface TransformTarget {
  key: string;
  label: string;
  handle: MeshHandle;
  color: string;
  opacity: number;
  wireframe: boolean;
}

export interface ViewerPerformance {
  loadMs: number;
  intersectionMs: number;
}

export interface CameraSnapshot {
  position: RgmPoint3;
  target: RgmPoint3;
  up: RgmPoint3;
  fov: number;
}

export type ExampleKey =
  | "nurbs"
  | "line"
  | "polyline"
  | "polycurve"
  | "arc"
  | "circle"
  | "intersectCurveCurve"
  | "intersectCurvePlane"
  | "meshLarge"
  | "meshTransform"
  | "meshIntersectMeshMesh"
  | "meshIntersectMeshPlane"
  | "meshBoolean"
  | "surfaceLarge"
  | "surfaceTransform"
  | "surfaceUvEval"
  | "surfaceIntersectSurface"
  | "surfaceIntersectPlane"
  | "surfaceIntersectCurve"
  | "trimEditWorkflow"
  | "trimValidationFailures"
  | "trimMultiLoopSurgery"
  | "brepShellAssembly"
  | "brepSolidAssembly"
  | "brepSolidRoundtripAudit"
  | "brepSolidFaceSurgery"
  | "brepFaceBridgeRoundtrip"
  | "brepNativeRoundtrip";
