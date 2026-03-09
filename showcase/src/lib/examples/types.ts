import type {
  CurveHandle,
  IntersectionHandle,
  MeshHandle,
  SurfaceHandle,
} from "@rustedgeom/kernel";
import type { RgmPlane, RgmPoint3 } from "@rustedgeom/kernel";
import type { CurvePreset } from "@/lib/preset-schema";
import type {
  MeshVisual,
  OverlayCurveVisual,
  SegmentOverlayVisual,
  TransformTarget,
} from "@/lib/viewer-types";

export type AnyHandle =
  | CurveHandle
  | MeshHandle
  | SurfaceHandle
  | IntersectionHandle;

export interface BuiltExample {
  kind: "curve" | "mesh";
  curveHandle: CurveHandle | null;
  ownedHandles: AnyHandle[];
  exportHandles?: AnyHandle[];
  name: string;
  degreeLabel: string;
  renderDegree: number;
  renderSamples: number;
  meshVisual: MeshVisual | null;
  overlayMeshes: MeshVisual[];
  overlayCurves: OverlayCurveVisual[];
  segmentOverlays: SegmentOverlayVisual[];
  intersectionPoints: RgmPoint3[];
  planeVisual: RgmPlane | null;
  interactiveMeshHandle: MeshHandle | null;
  transformTargets: TransformTarget[];
  defaultTransformTargetKey: string | null;
  surfaceProbeHandle?: SurfaceHandle | null;
  surfaceProbeD1Scale?: number;
  surfaceProbeD2Scale?: number;
  booleanState?:
    | {
        baseHandle: MeshHandle;
        toolHandle: MeshHandle;
        resultHandle: MeshHandle;
      }
    | null;
  intersectionMs: number;
  boundsMs?: number;
  logs: string[];
}

export interface ExampleOptions {
  nurbsPreset?: CurvePreset | null;
}
