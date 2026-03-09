import type { MeshHandle, RgmPoint3, RgmVec3 } from "@rustedgeom/kernel";

export type LogLevel = "info" | "debug" | "error";
export type GizmoMode = "translate" | "rotate" | "scale";
export type KernelStatus = "booting" | "ready" | "computing" | "error";
export type CameraMode = "perspective" | "orthographic";
export type SceneUpAxis = "y" | "z";
export type ViewPresetName = "top" | "bottom" | "left" | "right" | "front" | "back";

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
  tangent?: { x: number; y: number; z: number };
  normal?: { x: number; y: number; z: number };
  binormal?: { x: number; y: number; z: number };
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
  boundsMs: number;
}

export interface CameraSnapshot {
  position: RgmPoint3;
  target: RgmPoint3;
  up: RgmPoint3;
  fov: number;
  mode?: CameraMode;
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
  | "bboxMeshBooleanAssembly"
  | "surfaceLarge"
  | "surfaceTransform"
  | "surfaceUvEval"
  | "surfaceIntersectSurface"
  | "surfaceIntersectPlane"
  | "surfaceIntersectCurve"
  | "bboxSurfaceWarped"
  | "bboxCurveNonTrivial"
  | "sweepSurface"
  | "loftSurface"
  | "meshVolume"
  | "closestPointCurve"
  | "closestPointSurface"
  | "landxmlViewer";

export type LandXmlExampleKey = "landxmlViewer";

const VALID_EXAMPLE_KEYS = new Set<ExampleKey>([
  "nurbs", "line", "polyline", "polycurve", "arc", "circle",
  "intersectCurveCurve", "intersectCurvePlane",
  "meshLarge", "meshTransform", "meshIntersectMeshMesh", "meshIntersectMeshPlane",
  "meshBoolean", "bboxMeshBooleanAssembly",
  "surfaceLarge", "surfaceTransform", "surfaceUvEval",
  "surfaceIntersectSurface", "surfaceIntersectPlane", "surfaceIntersectCurve",
  "bboxSurfaceWarped",
  "bboxCurveNonTrivial",
  "sweepSurface",
  "loftSurface",
  "meshVolume",
  "closestPointCurve",
  "closestPointSurface",
  "landxmlViewer",
]);

export function isValidExampleKey(value: unknown): value is ExampleKey {
  return typeof value === "string" && VALID_EXAMPLE_KEYS.has(value as ExampleKey);
}

export interface LandXmlAlignmentInfo {
  index: number;
  name: string;
  profileCount: number;
  profileNames: string[];
  staStart: number;
  staEnd: number;
}

export interface LandXmlProbeUiState {
  station: number;
  stationNorm: number;
  alignmentIndex: number;
  profileIndex: number;
  alignmentPoint: { x: number; y: number; z: number };
  profilePoint: { x: number; y: number; z: number };
  tangent: { x: number; y: number; z: number };
  grade: number;
}

export const LANDXML_FILE_LIST: readonly string[] = [
  "12DExample.xml",
  "C3DDesignExample.xml",
  "C3DDesignExample3.xml",
  "C3DFeatureLineCoordGeom.xml",
  "C3DFeatureLineLocation.xml",
  "C3DProfileExample.xml",
  "C3DProfileExample2.xml",
  "C3DSpiralDoubleRadius.xml",
  "CircCurveExample.xml",
  "FeatureLineCoordinateTest.xml",
  "ImperialUnitsExample.xml",
  "OpenRoadBreakingAlignment.xml",
  "OpenRoadBreaklines.xml",
  "OpenRoadExample2.xml",
  "OpenRoadExample3.xml",
  "OpenRoadExampleEmptyAlignment.xml",
  "OpenRoadExampleWalls.xml",
  "OpenRoadProfile.xml",
  "OpenRoadProfileFirstPVIAtStartOfParaCurve.xml",
  "OpenRoadSpiralDoubleRadius.xml",
  "OpenRoadTin.xml",
] as const;
