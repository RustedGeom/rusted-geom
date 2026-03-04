"use client";

import {
  KernelSession,
  loadKernel,
  type BrepHandle,
  type CurveHandle,
  type FaceHandle,
  type IntersectionHandle,
  type LandXmlDocHandle,
  type MeshHandle,
  type SurfaceHandle,
} from "@rustedgeom/kernel";
import type {
  RgmArc3,
  RgmCircle3,
  RgmLine3,
  RgmNurbsSurfaceDesc,
  RgmPlane,
  RgmPoint3,
  RgmTrimEdgeInput,
  RgmTrimLoopInput,
  RgmUv2,
  RgmVec3,
} from "@rustedgeom/kernel";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { TransformControls } from "three/examples/jsm/controls/TransformControls.js";
import { Line2 } from "three/examples/jsm/lines/Line2.js";
import { LineGeometry } from "three/examples/jsm/lines/LineGeometry.js";
import { LineMaterial } from "three/examples/jsm/lines/LineMaterial.js";
import { LineSegments2 } from "three/examples/jsm/lines/LineSegments2.js";
import { LineSegmentsGeometry } from "three/examples/jsm/lines/LineSegmentsGeometry.js";

import {
  parseCurvePreset,
  parseViewerSession,
  type CurvePreset,
  type ViewerSessionFile,
} from "@/lib/preset-schema";
import { parseExampleSelection } from "@/lib/examples";
import { useTheme } from "@/lib/use-theme";
import { useKeyboardShortcuts } from "@/lib/use-keyboard-shortcut";
import type {
  CameraMode,
  CameraSnapshot,
  ExampleKey,
  GizmoMode,
  KernelStatus,
  LogEntry,
  LogLevel,
  MeshVisual,
  OverlayCurveVisual,
  ProbeUiState,
  SceneUpAxis,
  SegmentOverlayVisual,
  SurfaceProbeUiState,
  TransformTarget,
  ViewerPerformance,
  ViewPresetName,
} from "@/lib/viewer-types";
import { LANDXML_FILE_LIST, type LandXmlExampleKey, type LandXmlAlignmentInfo, type LandXmlProbeUiState } from "@/lib/viewer-types";
import type { Bounds3 } from "@rustedgeom/kernel";
import { ViewerToolbar } from "./viewer/toolbar/ViewerToolbar";
import { InspectorPanel } from "./viewer/inspector/InspectorPanel";
import { KernelConsole } from "./viewer/console/KernelConsole";
import { ExampleBrowser } from "./viewer/ExampleBrowser";

// ── New API helpers ───────────────────────────────────────────────────────────

/** Convert a flat [x,y,z,...] array to RgmPoint3[]. */
function flatToPoints(flat: ArrayLike<number>): RgmPoint3[] {
  const pts: RgmPoint3[] = [];
  for (let i = 0; i < flat.length; i += 3) {
    pts.push({ x: flat[i], y: flat[i + 1], z: flat[i + 2] });
  }
  return pts;
}

/** Convert an RgmPlane to flat [ox,oy,oz, xx,xy,xz, yx,yy,yz, zx,zy,zz]. */
function flattenPlane(p: RgmPlane): Float64Array {
  return new Float64Array([
    p.origin.x, p.origin.y, p.origin.z,
    p.x_axis.x, p.x_axis.y, p.x_axis.z,
    p.y_axis.x, p.y_axis.y, p.y_axis.z,
    p.z_axis.x, p.z_axis.y, p.z_axis.z,
  ]);
}

/** Convert RgmPoint3[] to flat [x,y,z,...]. */
function pointsToFlat(pts: RgmPoint3[]): Float64Array {
  return new Float64Array(pts.flatMap((p) => [p.x, p.y, p.z]));
}

/** Sample n points along a curve, returns RgmPoint3[]. */
function samplePolyline(session: KernelSession, curve: CurveHandle, n: number): RgmPoint3[] {
  const pts: RgmPoint3[] = [];
  for (let i = 0; i < n; i++) {
    const t = n === 1 ? 0.0 : i / (n - 1);
    const a = session.curve_point_at(curve, t);
    pts.push({ x: a[0], y: a[1], z: a[2] });
  }
  return pts;
}

/** Get mesh vertices+indices in the MeshVisual format. */
function meshToBuffers(session: KernelSession, mesh: MeshHandle): { vertices: RgmPoint3[]; indices: number[] } {
  const vflat = session.mesh_copy_vertices(mesh);
  const iflat = session.mesh_copy_indices(mesh);
  return {
    vertices: flatToPoints(vflat),
    indices: Array.from(iflat),
  };
}

const DEFAULT_CAMERA_POSITION = new THREE.Vector3(10, -11, 8);
const DEFAULT_CAMERA_TARGET = new THREE.Vector3(0, 0, 0);
const MIN_RENDER_SAMPLES = 2048;
const MAX_RENDER_SAMPLES = 12000;
const MOBILE_MEDIA_QUERY = "(max-width: 880px)";


type AnyHandle = CurveHandle | MeshHandle | SurfaceHandle | FaceHandle | IntersectionHandle | BrepHandle;
type BrepFaceId = number;

interface BuiltExample {
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


function toPoint3(vector: THREE.Vector3): RgmPoint3 {
  return { x: vector.x, y: vector.y, z: vector.z };
}

function fromPoint3(point: RgmPoint3): THREE.Vector3 {
  return new THREE.Vector3(point.x, point.y, point.z);
}

function downloadJson(filename: string, payload: unknown): void {
  const blob = new Blob([JSON.stringify(payload, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}

function downloadDataUrl(filename: string, dataUrl: string): void {
  const anchor = document.createElement("a");
  anchor.href = dataUrl;
  anchor.download = filename;
  anchor.click();
}

function downloadText(filename: string, text: string): void {
  const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}

function nowStamp(): string {
  const d = new Date();
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  const ss = String(d.getSeconds()).padStart(2, "0");
  const ms = String(d.getMilliseconds()).padStart(3, "0");
  return `${hh}:${mm}:${ss}.${ms}`;
}

function fileSafeStamp(): string {
  const d = new Date();
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  const ss = String(d.getSeconds()).padStart(2, "0");
  return `${year}-${month}-${day}_${hh}-${mm}-${ss}`;
}

function formatLogsAsText(entries: LogEntry[]): string {
  if (entries.length === 0) {
    return "[empty] Kernel console has no log entries.\n";
  }

  return `${entries
    .map((entry) => `[${entry.time}] ${entry.level.toUpperCase()} ${entry.message}`)
    .join("\n")}\n`;
}


function dist(a: RgmPoint3, b: RgmPoint3): number {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  const dz = a.z - b.z;
  return Math.sqrt(dx * dx + dy * dy + dz * dz);
}

function addScaled(point: RgmPoint3, vector: RgmVec3, scale: number): RgmPoint3 {
  return {
    x: point.x + vector.x * scale,
    y: point.y + vector.y * scale,
    z: point.z + vector.z * scale,
  };
}

function scaleVec(vector: RgmVec3, scale: number): RgmVec3 {
  return {
    x: vector.x * scale,
    y: vector.y * scale,
    z: vector.z * scale,
  };
}

function crossVec(a: RgmVec3, b: RgmVec3): RgmVec3 {
  return {
    x: a.y * b.z - a.z * b.y,
    y: a.z * b.x - a.x * b.z,
    z: a.x * b.y - a.y * b.x,
  };
}

function normalizedVec(vector: RgmVec3): RgmVec3 | null {
  const len = magnitude(vector);
  if (!Number.isFinite(len) || len <= 1e-12) {
    return null;
  }
  return {
    x: vector.x / len,
    y: vector.y / len,
    z: vector.z / len,
  };
}

function buildArrowSegments(origin: RgmPoint3, vector: RgmVec3, scale: number): RgmPoint3[] {
  const scaled = scaleVec(vector, scale);
  const stemLength = magnitude(scaled);
  if (!Number.isFinite(stemLength) || stemLength <= 1e-10) {
    return [];
  }

  const tip = addScaled(origin, vector, scale);
  const dir = normalizedVec(scaled);
  if (!dir) {
    return [];
  }

  const upRef = Math.abs(dir.z) < 0.9 ? { x: 0, y: 0, z: 1 } : { x: 0, y: 1, z: 0 };
  let side = normalizedVec(crossVec(dir, upRef));
  if (!side) {
    side = normalizedVec(crossVec(dir, { x: 1, y: 0, z: 0 }));
  }
  if (!side) {
    return [origin, tip];
  }

  const headLength = stemLength * 0.34;
  const headWidth = headLength * 0.6;
  const base = addScaled(tip, dir, -headLength);
  const wingA = addScaled(base, side, headWidth);
  const wingB = addScaled(base, side, -headWidth);

  return [origin, tip, tip, wingA, tip, wingB];
}

function magnitude(vector: RgmVec3): number {
  return Math.sqrt(vector.x * vector.x + vector.y * vector.y + vector.z * vector.z);
}

function formatPoint(point: RgmPoint3): string {
  return `(${point.x.toFixed(4)}, ${point.y.toFixed(4)}, ${point.z.toFixed(4)})`;
}

function formatVec(vector: RgmVec3): string {
  return `(${vector.x.toFixed(4)}, ${vector.y.toFixed(4)}, ${vector.z.toFixed(4)})`;
}

function chordParams(points: RgmPoint3[]): number[] {
  if (points.length <= 1) {
    return points.map(() => 0);
  }

  const cumulative = new Array(points.length).fill(0);
  let total = 0;
  for (let i = 1; i < points.length; i += 1) {
    total += dist(points[i - 1], points[i]);
    cumulative[i] = total;
  }

  if (total <= Number.EPSILON) {
    return points.map((_, idx) => idx / Math.max(1, points.length - 1));
  }

  return cumulative.map((v) => v / total);
}

function clampedOpenKnots(pointCount: number, degree: number, params: number[]): number[] {
  const knotCount = pointCount + degree + 1;
  const knots = new Array(knotCount).fill(0);

  for (let k = 0; k <= degree; k += 1) {
    knots[k] = 0;
    knots[knotCount - 1 - k] = 1;
  }

  if (pointCount > degree + 1) {
    const n = pointCount - 1;
    const interiorCount = n - degree;
    for (let j = 1; j <= interiorCount; j += 1) {
      let sum = 0;
      for (let i = j; i < j + degree; i += 1) {
        sum += params[i];
      }
      knots[j + degree] = sum / degree;
    }
  }

  return knots;
}

function periodicKnots(controlCount: number, degree: number): number[] {
  return Array.from({ length: controlCount + degree + 1 }, (_, idx) => idx);
}

function clampedUniformKnots(controlCount: number, degree: number): Float64Array {
  const knotCount = controlCount + degree + 1;
  const knots = new Float64Array(knotCount);
  const interior = controlCount - degree - 1;
  for (let i = 0; i <= degree; i += 1) {
    knots[i] = 0;
    knots[knotCount - 1 - i] = 1;
  }
  for (let i = 1; i <= interior; i += 1) {
    knots[degree + i] = i / (interior + 1);
  }
  return knots;
}

function buildWarpedSurfaceNet(
  uCount: number,
  vCount: number,
  spanU: number,
  spanV: number,
  warpScale: number,
): { desc: RgmNurbsSurfaceDesc; points: Float64Array; weights: Float64Array; knotsU: Float64Array; knotsV: Float64Array } {
  const rawPoints: RgmPoint3[] = [];
  const rawWeights: number[] = [];
  const halfU = spanU * 0.5;
  const halfV = spanV * 0.5;
  for (let iu = 0; iu < uCount; iu += 1) {
    const u = iu / Math.max(1, uCount - 1);
    const x = -halfU + u * spanU;
    for (let iv = 0; iv < vCount; iv += 1) {
      const v = iv / Math.max(1, vCount - 1);
      const y = -halfV + v * spanV;
      const z =
        Math.sin((u * 2.0 + v * 1.2) * Math.PI) * warpScale +
        Math.cos((u * 0.8 - v * 1.6) * Math.PI) * (warpScale * 0.6);
      rawPoints.push({ x, y, z });
      rawWeights.push(1.0 + 0.08 * Math.sin((u + v) * Math.PI));
    }
  }

  return {
    desc: {
      degree_u: 3,
      degree_v: 3,
      periodic_u: false,
      periodic_v: false,
      control_u_count: uCount,
      control_v_count: vCount,
    },
    points: pointsToFlat(rawPoints),
    weights: new Float64Array(rawWeights),
    knotsU: clampedUniformKnots(uCount, 3),
    knotsV: clampedUniformKnots(vCount, 3),
  };
}

function rectangleLoopUV(uMin: number, uMax: number, vMin: number, vMax: number): RgmUv2[] {
  return [
    { u: uMin, v: vMin },
    { u: uMax, v: vMin },
    { u: uMax, v: vMax },
    { u: uMin, v: vMax },
  ];
}

const CURVED_SOLID_FACE_FRAMES: Array<{
  normal: THREE.Vector3;
  axisU: THREE.Vector3;
  axisV: THREE.Vector3;
}> = [
  {
    normal: new THREE.Vector3(1, 0, 0),
    axisU: new THREE.Vector3(0, 1, 0),
    axisV: new THREE.Vector3(0, 0, 1),
  },
  {
    normal: new THREE.Vector3(-1, 0, 0),
    axisU: new THREE.Vector3(0, -1, 0),
    axisV: new THREE.Vector3(0, 0, 1),
  },
  {
    normal: new THREE.Vector3(0, 1, 0),
    axisU: new THREE.Vector3(-1, 0, 0),
    axisV: new THREE.Vector3(0, 0, 1),
  },
  {
    normal: new THREE.Vector3(0, -1, 0),
    axisU: new THREE.Vector3(1, 0, 0),
    axisV: new THREE.Vector3(0, 0, 1),
  },
  {
    normal: new THREE.Vector3(0, 0, 1),
    axisU: new THREE.Vector3(1, 0, 0),
    axisV: new THREE.Vector3(0, 1, 0),
  },
  {
    normal: new THREE.Vector3(0, 0, -1),
    axisU: new THREE.Vector3(1, 0, 0),
    axisV: new THREE.Vector3(0, -1, 0),
  },
];

function spherifyCubePoint(point: THREE.Vector3): THREE.Vector3 {
  const x2 = point.x * point.x;
  const y2 = point.y * point.y;
  const z2 = point.z * point.z;
  return new THREE.Vector3(
    point.x * Math.sqrt(Math.max(0, 1.0 - y2 * 0.5 - z2 * 0.5 + (y2 * z2) / 3.0)),
    point.y * Math.sqrt(Math.max(0, 1.0 - z2 * 0.5 - x2 * 0.5 + (z2 * x2) / 3.0)),
    point.z * Math.sqrt(Math.max(0, 1.0 - x2 * 0.5 - y2 * 0.5 + (x2 * y2) / 3.0)),
  );
}

function buildSkewedBoxSurfaces(
  session: KernelSession,
  center: RgmPoint3,
  size: RgmVec3,
  skew: number,
): SurfaceHandle[] {
  const controlCount = 9;
  const knots = clampedUniformKnots(controlCount, 3);
  const desc: RgmNurbsSurfaceDesc = {
    degree_u: 3,
    degree_v: 3,
    periodic_u: false,
    periodic_v: false,
    control_u_count: controlCount,
    control_v_count: controlCount,
  };
  const half = {
    x: Math.max(0.6, size.x * 0.5),
    y: Math.max(0.6, size.y * 0.5),
    z: Math.max(0.6, size.z * 0.5),
  };
  const surfaces: SurfaceHandle[] = [];

  for (const frame of CURVED_SOLID_FACE_FRAMES) {
    const controlPoints: RgmPoint3[] = [];
    const weights = new Float64Array(controlCount * controlCount).fill(1.0);
    for (let iu = 0; iu < controlCount; iu += 1) {
      const su = (iu / (controlCount - 1)) * 2.0 - 1.0;
      for (let iv = 0; iv < controlCount; iv += 1) {
        const sv = (iv / (controlCount - 1)) * 2.0 - 1.0;
        const cube = frame.normal
          .clone()
          .addScaledVector(frame.axisU, su)
          .addScaledVector(frame.axisV, sv);
        const rounded = spherifyCubePoint(cube);
        const lon = Math.atan2(rounded.y, rounded.x);
        const lat = Math.atan2(rounded.z, Math.max(1e-9, Math.hypot(rounded.x, rounded.y)));
        const radialScale =
          1.0 +
          0.14 * Math.sin(3.0 * lon + 0.65 * Math.sin(2.0 * lat)) +
          0.08 * Math.cos(2.0 * lat - 0.45 * lon) +
          Math.abs(skew) * 0.12 * Math.sin(4.0 * lon + lat);
        const sculpted = rounded.multiplyScalar(radialScale);
        const twist = skew * 0.2 * sculpted.z;
        const cosT = Math.cos(twist);
        const sinT = Math.sin(twist);
        const tx = sculpted.x * cosT - sculpted.y * sinT;
        const ty = sculpted.x * sinT + sculpted.y * cosT;
        const tz = sculpted.z;

        controlPoints.push({
          x:
            center.x +
            tx * half.x +
            skew * ty * tz * 0.16 * half.x,
          y:
            center.y +
            ty * half.y +
            skew * tx * tz * 0.14 * half.y,
          z:
            center.z +
            tz * half.z +
            skew * tx * ty * 0.1 * half.z,
        });
      }
    }

    const _flatPts = pointsToFlat(controlPoints);
    const surface = session.create_nurbs_surface(
      desc.degree_u,
      desc.degree_v,
      desc.control_u_count,
      desc.control_v_count,
      desc.periodic_u,
      desc.periodic_v,
      _flatPts,
      weights,
      knots,
      knots,
    );
    surfaces.push(surface);
  }

  return surfaces;
}

function preview(values: number[], max = 12): string {
  if (values.length <= max) {
    return `[${values.map((v) => Number(v.toFixed(6))).join(", ")}]`;
  }
  const head = values.slice(0, Math.floor(max / 2)).map((v) => Number(v.toFixed(6)));
  const tail = values.slice(values.length - Math.floor(max / 2)).map((v) => Number(v.toFixed(6)));
  return `[${head.join(", ")}, ..., ${tail.join(", ")}]`;
}

function validationSeverityLabel(value: number): string {
  if (value === 2) return "error";
  if (value === 1) return "warning";
  return "info";
}

function validationReportLogLines(
  report: { issue_count: number; max_severity: number; overflow: boolean },
  prefix: string,
): string[] {
  return [
    `${prefix}: issue_count=${report.issue_count} max=${validationSeverityLabel(report.max_severity)} overflow=${report.overflow}`,
  ];
}

function constructionLogLines(preset: CurvePreset): string[] {
  const tol = Math.max(0, preset.tolerance.abs_tol);
  let fitPoints = [...preset.points];
  const lines: string[] = [];

  lines.push(
    `Preset "${preset.name}": degree=${preset.degree}, closed=${preset.closed}, fitPoints=${fitPoints.length}, sampleCount=${preset.sampleCount}`,
  );

  if (
    preset.closed &&
    fitPoints.length > 1 &&
    dist(fitPoints[0], fitPoints[fitPoints.length - 1]) <= tol
  ) {
    fitPoints = fitPoints.slice(0, -1);
    lines.push("Closed preset: removed duplicated end point (within abs_tol)");
  }

  if (fitPoints.length <= preset.degree) {
    lines.push("Invalid construction: fitPoints <= degree");
    return lines;
  }

  if (preset.closed) {
    const baseCount = fitPoints.length;
    const controlCount = baseCount + preset.degree;
    const knots = periodicKnots(controlCount, preset.degree);
    lines.push(
      `Periodic construction: controlCount=${controlCount} (base ${baseCount} + degree ${preset.degree}), weights=all 1`,
    );
    lines.push(`Uniform periodic knots: ${preview(knots)}`);
  } else {
    const params = chordParams(fitPoints);
    const knots = clampedOpenKnots(fitPoints.length, preset.degree, params);
    lines.push(
      `Open clamped construction: controlCount=${fitPoints.length}, weights=all 1, params=chord-length`,
    );
    lines.push(`Chord params: ${preview(params)}`);
    lines.push(`Clamped knots: ${preview(knots)}`);
  }

  return lines;
}

function renderSampleCountForPreset(preset: CurvePreset): number {
  const degreeBoost = (preset.degree + 1) * Math.max(6, preset.points.length) * 80;
  return Math.max(
    MIN_RENDER_SAMPLES,
    Math.min(MAX_RENDER_SAMPLES, Math.max(preset.sampleCount, degreeBoost)),
  );
}

function curveColorForDegree(degree: number): string {
  if (degree <= 1) {
    return "#ffc670";
  }
  if (degree === 2) {
    return "#98f0ff";
  }
  if (degree === 3) {
    return "#7dc4ff";
  }
  return "#a0b6ff";
}

function curveWidthForDegree(degree: number): number {
  if (degree <= 1) {
    return 2.1;
  }
  if (degree === 2) {
    return 2.6;
  }
  if (degree === 3) {
    return 3.0;
  }
  return 3.4;
}

function fallbackTolerance(): { abs_tol: number; rel_tol: number; angle_tol: number } {
  return {
    abs_tol: 1e-9,
    rel_tol: 1e-9,
    angle_tol: 1e-9,
  };
}

function shouldShowProbeForExample(example: ExampleKey): boolean {
  return (
    example !== "intersectCurveCurve" &&
    example !== "intersectCurvePlane" &&
    example !== "meshLarge" &&
    example !== "meshTransform" &&
    example !== "meshIntersectMeshMesh" &&
    example !== "meshIntersectMeshPlane" &&
    example !== "meshBoolean" &&
    example !== "bboxMeshBooleanAssembly" &&
    example !== "surfaceLarge" &&
    example !== "surfaceTransform" &&
    example !== "surfaceUvEval" &&
    example !== "surfaceIntersectSurface" &&
    example !== "surfaceIntersectPlane" &&
    example !== "surfaceIntersectCurve" &&
    example !== "bboxSurfaceWarped" &&
    example !== "trimEditWorkflow" &&
    example !== "trimValidationFailures" &&
    example !== "trimMultiLoopSurgery" &&
    example !== "brepShellAssembly" &&
    example !== "brepSolidAssembly" &&
    example !== "brepSolidRoundtripAudit" &&
    example !== "brepSolidFaceSurgery" &&
    example !== "brepFaceBridgeRoundtrip" &&
    example !== "brepNativeRoundtrip" &&
    example !== "bboxBrepSolidLifecycle" &&
    example !== "landxmlViewer"
  );
}

function isBrepExample(example: ExampleKey): boolean {
  return example.startsWith("brep");
}

function isMeshOnlyExample(example: ExampleKey): boolean {
  return (
    example === "meshLarge" ||
    example === "meshTransform" ||
    example === "meshIntersectMeshMesh" ||
    example === "meshIntersectMeshPlane" ||
    example === "meshBoolean" ||
    example === "bboxMeshBooleanAssembly"
  );
}

function createWideLine(
  points: RgmPoint3[],
  color: string,
  width: number,
  opacity: number,
  viewport: HTMLDivElement | null,
): Line2 {
  const positions = points.flatMap((point) => [point.x, point.y, point.z]);
  const geometry = new LineGeometry();
  geometry.setPositions(positions);
  const material = new LineMaterial({
    color,
    transparent: opacity < 1,
    opacity,
    linewidth: width,
    worldUnits: false,
    depthTest: false,
    depthWrite: false,
  });
  material.resolution.set(viewport?.clientWidth ?? 1, viewport?.clientHeight ?? 1);
  const line = new Line2(geometry, material);
  line.computeLineDistances();
  return line;
}

function createMeshGeometry(
  vertices: RgmPoint3[],
  indices: number[],
  origin?: THREE.Vector3,
): THREE.BufferGeometry {
  const geometry = new THREE.BufferGeometry();
  const positions = new Float32Array(vertices.length * 3);
  for (let idx = 0; idx < vertices.length; idx += 1) {
    positions[idx * 3] = vertices[idx].x - (origin?.x ?? 0);
    positions[idx * 3 + 1] = vertices[idx].y - (origin?.y ?? 0);
    positions[idx * 3 + 2] = vertices[idx].z - (origin?.z ?? 0);
  }
  geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
  geometry.setIndex(indices);
  geometry.computeVertexNormals();
  geometry.computeBoundingSphere();
  return geometry;
}

function createSegmentLines(
  points: RgmPoint3[],
  color: string,
  opacity: number,
  width: number,
  viewport: HTMLDivElement | null,
): LineSegments2 | null {
  const segmentCount = Math.floor(points.length / 2);
  if (segmentCount === 0) {
    return null;
  }

  const positions = new Float32Array(segmentCount * 6);
  for (let idx = 0; idx < segmentCount; idx += 1) {
    const a = points[idx * 2];
    const b = points[idx * 2 + 1];
    const base = idx * 6;
    positions[base] = a.x;
    positions[base + 1] = a.y;
    positions[base + 2] = a.z;
    positions[base + 3] = b.x;
    positions[base + 4] = b.y;
    positions[base + 5] = b.z;
  }

  const geometry = new LineSegmentsGeometry();
  geometry.setPositions(Array.from(positions));
  const material = new LineMaterial({
    color,
    transparent: opacity < 1,
    opacity,
    linewidth: width,
    worldUnits: false,
    depthWrite: false,
    depthTest: false,
  });
  material.resolution.set(viewport?.clientWidth ?? 1, viewport?.clientHeight ?? 1);
  return new LineSegments2(geometry, material);
}

function normalizedVector(input: RgmPoint3, fallback: THREE.Vector3): THREE.Vector3 {
  const vector = new THREE.Vector3(input.x, input.y, input.z);
  if (vector.lengthSq() <= Number.EPSILON) {
    return fallback.clone();
  }
  return vector.normalize();
}

function buildPlaneFrame(plane: RgmPlane): {
  origin: THREE.Vector3;
  xAxis: THREE.Vector3;
  yAxis: THREE.Vector3;
  normal: THREE.Vector3;
} {
  const origin = new THREE.Vector3(plane.origin.x, plane.origin.y, plane.origin.z);
  let normal = normalizedVector(plane.z_axis, new THREE.Vector3(0, 0, 1));
  let xAxis = normalizedVector(plane.x_axis, new THREE.Vector3(1, 0, 0));
  xAxis = xAxis.clone().sub(normal.clone().multiplyScalar(xAxis.dot(normal)));
  if (xAxis.lengthSq() <= 1e-12) {
    xAxis = new THREE.Vector3(0, 1, 0).cross(normal);
  }
  xAxis.normalize();
  let yAxis = normal.clone().cross(xAxis).normalize();
  if (yAxis.lengthSq() <= 1e-12) {
    yAxis = normalizedVector(plane.y_axis, new THREE.Vector3(0, 1, 0));
    yAxis = yAxis.clone().sub(normal.clone().multiplyScalar(yAxis.dot(normal)));
    if (yAxis.lengthSq() <= 1e-12) {
      yAxis = new THREE.Vector3(0, 0, 1).cross(xAxis);
    }
    yAxis.normalize();
  }
  normal = xAxis.clone().cross(yAxis).normalize();

  return { origin, xAxis, yAxis, normal };
}

function centroidOfPoints(points: RgmPoint3[]): THREE.Vector3 {
  if (points.length === 0) {
    return new THREE.Vector3();
  }
  const centroid = new THREE.Vector3();
  for (const point of points) {
    centroid.add(new THREE.Vector3(point.x, point.y, point.z));
  }
  return centroid.multiplyScalar(1 / points.length);
}

function projectedPointOnPlane(
  point: THREE.Vector3,
  origin: THREE.Vector3,
  normal: THREE.Vector3,
): THREE.Vector3 {
  const delta = point.clone().sub(origin);
  return point.clone().sub(normal.clone().multiplyScalar(delta.dot(normal)));
}

function planeVisualSize(points: RgmPoint3[]): number {
  if (points.length < 2) {
    return 12;
  }
  const box = new THREE.Box3();
  for (const point of points) {
    box.expandByPoint(new THREE.Vector3(point.x, point.y, point.z));
  }
  const diagonal = box.getSize(new THREE.Vector3()).length();
  return Math.max(10, diagonal * 1.6);
}

// ── Bounds3 (new flat API) helpers ────────────────────────────────────────────

function aabbExtentsFromBounds3(bounds: Bounds3): RgmVec3 {
  return {
    x: Math.max(0, bounds.aabb_max_x - bounds.aabb_min_x),
    y: Math.max(0, bounds.aabb_max_y - bounds.aabb_min_y),
    z: Math.max(0, bounds.aabb_max_z - bounds.aabb_min_z),
  };
}

function pointInsideBounds3Aabb(bounds: Bounds3, point: RgmPoint3, eps = 1e-7): boolean {
  return (
    point.x >= bounds.aabb_min_x - eps &&
    point.x <= bounds.aabb_max_x + eps &&
    point.y >= bounds.aabb_min_y - eps &&
    point.y <= bounds.aabb_max_y + eps &&
    point.z >= bounds.aabb_min_z - eps &&
    point.z <= bounds.aabb_max_z + eps
  );
}

function obbExtents(bounds: Bounds3): RgmVec3 {
  return { x: bounds.obb_half_x * 2, y: bounds.obb_half_y * 2, z: bounds.obb_half_z * 2 };
}

function extentsVolume(extents: RgmVec3): number {
  return extents.x * extents.y * extents.z;
}

function formatExtents(extents: RgmVec3): string {
  return `${extents.x.toFixed(3)} × ${extents.y.toFixed(3)} × ${extents.z.toFixed(3)}`;
}

function obbLocalToWorld(bounds: Bounds3, x: number, y: number, z: number): RgmPoint3 {
  return {
    x:
      bounds.obb_center_x +
      bounds.obb_ax_x * x +
      bounds.obb_ay_x * y +
      bounds.obb_az_x * z,
    y:
      bounds.obb_center_y +
      bounds.obb_ax_y * x +
      bounds.obb_ay_y * y +
      bounds.obb_az_y * z,
    z:
      bounds.obb_center_z +
      bounds.obb_ax_z * x +
      bounds.obb_ay_z * y +
      bounds.obb_az_z * z,
  };
}

function aabbWireSegments(bounds: Bounds3): RgmPoint3[] {
  const corners: RgmPoint3[] = [
    { x: bounds.aabb_min_x, y: bounds.aabb_min_y, z: bounds.aabb_min_z },
    { x: bounds.aabb_max_x, y: bounds.aabb_min_y, z: bounds.aabb_min_z },
    { x: bounds.aabb_max_x, y: bounds.aabb_max_y, z: bounds.aabb_min_z },
    { x: bounds.aabb_min_x, y: bounds.aabb_max_y, z: bounds.aabb_min_z },
    { x: bounds.aabb_min_x, y: bounds.aabb_min_y, z: bounds.aabb_max_z },
    { x: bounds.aabb_max_x, y: bounds.aabb_min_y, z: bounds.aabb_max_z },
    { x: bounds.aabb_max_x, y: bounds.aabb_max_y, z: bounds.aabb_max_z },
    { x: bounds.aabb_min_x, y: bounds.aabb_max_y, z: bounds.aabb_max_z },
  ];
  const edgeIndices = [
    [0, 1], [1, 2], [2, 3], [3, 0],
    [4, 5], [5, 6], [6, 7], [7, 4],
    [0, 4], [1, 5], [2, 6], [3, 7],
  ] as const;
  const segments: RgmPoint3[] = [];
  for (const [a, b] of edgeIndices) {
    segments.push(corners[a], corners[b]);
  }
  return segments;
}

function obbWireSegments(bounds: Bounds3): RgmPoint3[] {
  const hx = bounds.obb_half_x;
  const hy = bounds.obb_half_y;
  const hz = bounds.obb_half_z;
  const corners: RgmPoint3[] = [
    obbLocalToWorld(bounds, -hx, -hy, -hz),
    obbLocalToWorld(bounds, hx, -hy, -hz),
    obbLocalToWorld(bounds, hx, hy, -hz),
    obbLocalToWorld(bounds, -hx, hy, -hz),
    obbLocalToWorld(bounds, -hx, -hy, hz),
    obbLocalToWorld(bounds, hx, -hy, hz),
    obbLocalToWorld(bounds, hx, hy, hz),
    obbLocalToWorld(bounds, -hx, hy, hz),
  ];
  const edgeIndices = [
    [0, 1], [1, 2], [2, 3], [3, 0],
    [4, 5], [5, 6], [6, 7], [7, 4],
    [0, 4], [1, 5], [2, 6], [3, 7],
  ] as const;
  const segments: RgmPoint3[] = [];
  for (const [a, b] of edgeIndices) {
    segments.push(corners[a], corners[b]);
  }
  return segments;
}

function localAabbWireSegments(bounds: Bounds3): RgmPoint3[] {
  const mnX = bounds.local_aabb_min_x;
  const mnY = bounds.local_aabb_min_y;
  const mnZ = bounds.local_aabb_min_z;
  const mxX = bounds.local_aabb_max_x;
  const mxY = bounds.local_aabb_max_y;
  const mxZ = bounds.local_aabb_max_z;
  const corners: RgmPoint3[] = [
    obbLocalToWorld(bounds, mnX, mnY, mnZ),
    obbLocalToWorld(bounds, mxX, mnY, mnZ),
    obbLocalToWorld(bounds, mxX, mxY, mnZ),
    obbLocalToWorld(bounds, mnX, mxY, mnZ),
    obbLocalToWorld(bounds, mnX, mnY, mxZ),
    obbLocalToWorld(bounds, mxX, mnY, mxZ),
    obbLocalToWorld(bounds, mxX, mxY, mxZ),
    obbLocalToWorld(bounds, mnX, mxY, mxZ),
  ];
  const edgeIndices = [
    [0, 1], [1, 2], [2, 3], [3, 0],
    [4, 5], [5, 6], [6, 7], [7, 4],
    [0, 4], [1, 5], [2, 6], [3, 7],
  ] as const;
  const segments: RgmPoint3[] = [];
  for (const [a, b] of edgeIndices) {
    segments.push(corners[a], corners[b]);
  }
  return segments;
}

function obbAxisSegments(bounds: Bounds3, axisScale = 1.12): SegmentOverlayVisual[] {
  const c: RgmPoint3 = { x: bounds.obb_center_x, y: bounds.obb_center_y, z: bounds.obb_center_z };
  const hx = Math.max(1e-6, bounds.obb_half_x * axisScale);
  const hy = Math.max(1e-6, bounds.obb_half_y * axisScale);
  const hz = Math.max(1e-6, bounds.obb_half_z * axisScale);
  const xAxis: RgmVec3 = { x: bounds.obb_ax_x, y: bounds.obb_ax_y, z: bounds.obb_ax_z };
  const yAxis: RgmVec3 = { x: bounds.obb_ay_x, y: bounds.obb_ay_y, z: bounds.obb_ay_z };
  const zAxis: RgmVec3 = { x: bounds.obb_az_x, y: bounds.obb_az_y, z: bounds.obb_az_z };
  return [
    {
      points: [c, addScaled(c, xAxis, hx)],
      color: "#ff7070",
      opacity: 0.96,
      width: 2.9,
      name: "obb-x-axis",
    },
    {
      points: [c, addScaled(c, yAxis, hy)],
      color: "#7cff9a",
      opacity: 0.96,
      width: 2.9,
      name: "obb-y-axis",
    },
    {
      points: [c, addScaled(c, zAxis, hz)],
      color: "#77b4ff",
      opacity: 0.96,
      width: 2.9,
      name: "obb-z-axis",
    },
  ];
}

function boundsOverlaySegments(bounds: Bounds3): SegmentOverlayVisual[] {
  return [
    {
      points: aabbWireSegments(bounds),
      color: "#ffc658",
      opacity: 0.96,
      width: 3.1,
      name: "world-aabb",
    },
    {
      points: obbWireSegments(bounds),
      color: "#67d9ff",
      opacity: 0.9,
      width: 2.6,
      name: "world-obb",
    },
    {
      points: localAabbWireSegments(bounds),
      color: "#a9ffb2",
      opacity: 0.86,
      width: 2.4,
      name: "local-aabb",
    },
    ...obbAxisSegments(bounds),
  ];
}

function boundsToBox3(b: Bounds3): THREE.Box3 {
  return new THREE.Box3(
    new THREE.Vector3(b.aabb_min_x, b.aabb_min_y, b.aabb_min_z),
    new THREE.Vector3(b.aabb_max_x, b.aabb_max_y, b.aabb_max_z),
  );
}

function computeSceneBounds(
  session: KernelSession,
  handles: Array<{ objectId: number }>,
): THREE.Box3 {
  const union = new THREE.Box3();
  for (const { objectId } of handles) {
    try {
      const b = session.compute_bounds(objectId, 0, 0, 0);
      union.union(boundsToBox3(b));
    } catch {
      // handle may have been freed
    }
  }
  return union;
}

/**
 * After changing camera.up, OrbitControls' internal quaternion (computed once
 * at construction) becomes stale.  This syncs it to the current camera.up so
 * spherical-coordinate math inside controls.update() uses the right frame.
 */
function syncControlsUpAxis(
  controls: OrbitControls,
  camera: THREE.Camera,
): void {
  const c = controls as unknown as {
    _quat: THREE.Quaternion;
    _quatInverse: THREE.Quaternion;
  };
  c._quat.setFromUnitVectors(camera.up, new THREE.Vector3(0, 1, 0));
  c._quatInverse.copy(c._quat).invert();
}

function downloadTextFile(text: string, filename: string, mimeType: string): void {
  const blob = new Blob([text], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

function zoomToFit(
  camera: THREE.PerspectiveCamera | THREE.OrthographicCamera,
  controls: OrbitControls,
  box: THREE.Box3,
  fog?: THREE.Fog | null,
): void {
  const center = box.getCenter(new THREE.Vector3());
  const size = box.getSize(new THREE.Vector3());
  const maxDim = Math.max(size.x, size.y, size.z);
  if (maxDim < 1e-10) return;

  const dir = new THREE.Vector3().subVectors(camera.position, controls.target);
  if (dir.lengthSq() < 1e-10) dir.set(1, 0.55, 1);
  dir.normalize();

  const sphere = box.getBoundingSphere(new THREE.Sphere());

  if (camera instanceof THREE.PerspectiveCamera) {
    const fovRad = THREE.MathUtils.degToRad(camera.fov);
    const aspect = camera.aspect;
    const fitH = maxDim / (2 * Math.tan(fovRad / 2));
    const fitW = maxDim / (2 * Math.tan(fovRad / 2) * aspect);
    const distance = Math.max(fitH, fitW) * 1.15;
    camera.position.copy(center).addScaledVector(dir, distance);
    camera.near = Math.max(0.01, distance * 0.001);
    camera.far = Math.max(distance * 10, sphere.radius * 20);
  } else {
    const aspect = (camera.right - camera.left) / (camera.top - camera.bottom) || 1;
    const halfH = maxDim * 0.575;
    const halfW = halfH * aspect;
    camera.left = -halfW;
    camera.right = halfW;
    camera.top = halfH;
    camera.bottom = -halfH;
    camera.near = 0.001;
    camera.far = Math.max(sphere.radius * 20, 10000);
    camera.position.copy(center).addScaledVector(dir, sphere.radius * 2);
  }
  camera.updateProjectionMatrix();
  controls.target.copy(center);

  if (fog) {
    const r = sphere.radius;
    fog.near = r > 500 ? 1e9 : Math.max(34, r * 0.5);
    fog.far = r > 500 ? 1e9 : Math.max(138, r * 3.5);
  }
  controls.update();
}

function getViewPresets(upAxis: SceneUpAxis) {
  if (upAxis === "z") {
    return {
      top: { dir: new THREE.Vector3(0, 0, -1), up: new THREE.Vector3(0, 1, 0) },
      bottom: { dir: new THREE.Vector3(0, 0, 1), up: new THREE.Vector3(0, -1, 0) },
      front: { dir: new THREE.Vector3(0, -1, 0), up: new THREE.Vector3(0, 0, 1) },
      back: { dir: new THREE.Vector3(0, 1, 0), up: new THREE.Vector3(0, 0, 1) },
      left: { dir: new THREE.Vector3(1, 0, 0), up: new THREE.Vector3(0, 0, 1) },
      right: { dir: new THREE.Vector3(-1, 0, 0), up: new THREE.Vector3(0, 0, 1) },
    };
  }
  return {
    top: { dir: new THREE.Vector3(0, -1, 0), up: new THREE.Vector3(0, 0, -1) },
    bottom: { dir: new THREE.Vector3(0, 1, 0), up: new THREE.Vector3(0, 0, 1) },
    front: { dir: new THREE.Vector3(0, 0, -1), up: new THREE.Vector3(0, 1, 0) },
    back: { dir: new THREE.Vector3(0, 0, 1), up: new THREE.Vector3(0, 1, 0) },
    left: { dir: new THREE.Vector3(1, 0, 0), up: new THREE.Vector3(0, 1, 0) },
    right: { dir: new THREE.Vector3(-1, 0, 0), up: new THREE.Vector3(0, 1, 0) },
  };
}

function isAsyncExample(key: ExampleKey): key is LandXmlExampleKey {
  return key === "landxmlViewer";
}

interface LandXmlStats {
  surfCount: number;
  alignCount: number;
  vertCount: number;
  featureLineCount: number;
  breaklineCount: number;
  unit: string;
  warnCount: number;
  parseMs: number;
}

const HORIZ_COLORS: Record<number, string> = { 0: "#2d7bff", 1: "#ff3b30", 2: "#20d66b" };
const HORIZ_LABELS: Record<number, string> = { 0: "Line", 1: "Arc", 2: "Spiral" };
const RESULTANT_3D_COLOR = "#ffef00";
const PLAN_LINEAR_COLORS: Record<number, string> = { 0: "#9ec4ff", 1: "#ffb454" };
const PLAN_LINEAR_LABELS: Record<number, string> = { 0: "FeatureLine", 1: "Breakline" };
interface LandXmlCurveData {
  horizCurves: OverlayCurveVisual[];
  raw3dCurves: OverlayCurveVisual[];
}

interface LandXmlContext {
  docHandle: LandXmlDocHandle;
  centroidX: number;
  centroidY: number;
  centroidZ: number;
  alignments: LandXmlAlignmentInfo[];
}

function applyDatumAndExaggeration(
  data: LandXmlCurveData,
  datumOffset: number,
  vertExag: number,
  scaleFactor = 1,
): OverlayCurveVisual[] {
  const s = scaleFactor;
  const transformed3d = data.raw3dCurves.map((c) => ({
    ...c,
    points: c.points.map((p) => ({
      x: p.x * s,
      y: p.y * s,
      z: (p.z - datumOffset) * vertExag * s,
    })),
  }));
  const scaledHoriz = data.horizCurves.map((c) => ({
    ...c,
    points: c.points.map((p) => ({ x: p.x * s, y: p.y * s, z: p.z * s })),
  }));
  return [...scaledHoriz, ...transformed3d];
}

async function buildLandXmlExample(
  session: KernelSession,
  filename: string,
  signal: AbortSignal,
): Promise<{
  built: BuiltExample;
  stats: LandXmlStats;
  curveData: LandXmlCurveData;
  zRange: { min: number; max: number };
  defaultDatum: number;
  context: LandXmlContext;
}> {
  const t0 = performance.now();
  const response = await fetch("/landxml/" + filename, { signal });
  if (!response.ok) throw new Error(`Failed to fetch ${filename}: ${response.status}`);
  const xmlText = await response.text();
  if (signal.aborted) throw new DOMException("Aborted", "AbortError");

  const doc = session.landxml_parse(xmlText, 1, 0, 0);
  const parseMs = performance.now() - t0;

  const surfCount = session.landxml_surface_count(doc);
  const alignCount = session.landxml_alignment_count(doc);
  const warnCount = session.landxml_warning_count(doc);
  const unit = session.landxml_linear_unit(doc);

  let centroidX = 0;
  let centroidY = 0;
  let centroidZ = 0;
  let totalVertCount = 0;

  if (surfCount > 0) {
    const rawVerts = session.landxml_surface_copy_vertices(doc, 0);
    const nv = rawVerts.length / 3;
    for (let i = 0; i < rawVerts.length; i += 3) {
      centroidX += rawVerts[i];
      centroidY += rawVerts[i + 1];
      centroidZ += rawVerts[i + 2];
    }
    if (nv > 0) {
      centroidX /= nv;
      centroidY /= nv;
      centroidZ /= nv;
    }
    totalVertCount += nv;
  } else if (alignCount > 0) {
    let ptCount = 0;
    for (let a = 0; a < alignCount; a++) {
      const packed = session.landxml_sample_horiz_2d_segments(doc, a);
      const segCount = packed[0];
      let idx = 1;
      for (let s = 0; s < segCount; s++) {
        idx++; // skip seg type
        const n = packed[idx++];
        for (let j = 0; j < n; j++) {
          centroidX += packed[idx];
          centroidY += packed[idx + 1];
          idx += 3;
          ptCount++;
        }
      }
    }
    if (ptCount > 0) {
      centroidX /= ptCount;
      centroidY /= ptCount;
    }
  }

  let meshVisual: MeshVisual | null = null;
  const overlayMeshes: MeshVisual[] = [];

  for (let si = 0; si < surfCount; si++) {
    const rawV = session.landxml_surface_copy_vertices(doc, si);
    const rawI = session.landxml_surface_copy_indices(doc, si);
    const name = session.landxml_surface_name(doc, si);

    const verts: RgmPoint3[] = [];
    for (let i = 0; i < rawV.length; i += 3) {
      verts.push({
        x: rawV[i] - centroidX,
        y: rawV[i + 1] - centroidY,
        z: rawV[i + 2] - centroidZ,
      });
    }
    totalVertCount += verts.length;

    const mv: MeshVisual = {
      vertices: verts,
      indices: Array.from(rawI),
      color: si === 0 ? "#8cb4d0" : "#a0c8a0",
      opacity: 0.85,
      wireframe: false,
      name,
    };
    if (si === 0) {
      meshVisual = mv;
    } else {
      overlayMeshes.push(mv);
    }
  }

  const horizCurves: OverlayCurveVisual[] = [];
  const raw3dCurves: OverlayCurveVisual[] = [];
  const debugLogs: string[] = [];
  let zMin = Infinity;
  let zMax = -Infinity;

  if (alignCount > 0) {
    for (let a = 0; a < alignCount; a++) {
      const aName = session.landxml_alignment_name(doc, a);

      const packed2d = session.landxml_sample_horiz_2d_segments(doc, a);
      const segCount = packed2d[0];
      let idx = 1;
      for (let s = 0; s < segCount; s++) {
        const segType = packed2d[idx++];
        const nPts = packed2d[idx++];
        const pts: RgmPoint3[] = [];
        for (let j = 0; j < nPts; j++) {
          pts.push({
            x: packed2d[idx] - centroidX,
            y: packed2d[idx + 1] - centroidY,
            z: 0,
          });
          idx += 3;
        }
        if (pts.length >= 2) {
          horizCurves.push({
            points: pts,
            color: HORIZ_COLORS[segType] ?? "#cccccc",
            width: 3,
            opacity: 1.0,
            name: `${aName} — ${HORIZ_LABELS[segType] ?? "?"} #${s + 1}`,
          });
        }
      }

      const profCount = session.landxml_alignment_profile_count(doc, a);
      if (profCount === 0) {
        debugLogs.push(`${aName}: no profiles found`);
      }
      for (let p = 0; p < profCount; p++) {
        try {
          const profName = session.landxml_alignment_profile_name(doc, a, p);
          const packed3d = session.landxml_sample_alignment_3d(doc, a, p, 500);
          const nPts3d = packed3d[0];
          const pts3d: RgmPoint3[] = [];
          for (let j = 0; j < nPts3d; j++) {
            const z = packed3d[1 + j * 3 + 2] - centroidZ;
            pts3d.push({
              x: packed3d[1 + j * 3] - centroidX,
              y: packed3d[1 + j * 3 + 1] - centroidY,
              z,
            });
            if (z < zMin) zMin = z;
            if (z > zMax) zMax = z;
          }
          if (pts3d.length >= 2) {
            raw3dCurves.push({
              points: pts3d,
              color: RESULTANT_3D_COLOR,
              width: 2,
              opacity: 1.0,
              name: `${aName} — 3D [${profName}]`,
            });
            debugLogs.push(`${aName} / ${profName}: ${nPts3d} pts, Z range [${zMin.toFixed(1)}, ${zMax.toFixed(1)}]`);
          } else {
            debugLogs.push(`${aName} / ${profName}: 3D sampling returned ${nPts3d} pts — skipped`);
          }
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          debugLogs.push(`${aName} profile ${p}: 3D eval failed — ${msg}`);
        }
      }
    }
  }

  const planLinearCount = session.landxml_plan_linear_count(doc);
  let featureLineCount = 0;
  let breaklineCount = 0;
  for (let i = 0; i < planLinearCount; i++) {
    const name = session.landxml_plan_linear_name(doc, i);
    const kind = session.landxml_plan_linear_kind(doc, i);
    const rawPts = session.landxml_plan_linear_copy_points(doc, i);
    const pts: RgmPoint3[] = [];
    for (let j = 0; j < rawPts.length; j += 3) {
      pts.push({
        x: rawPts[j] - centroidX,
        y: rawPts[j + 1] - centroidY,
        z: rawPts[j + 2] - centroidZ,
      });
    }
    if (pts.length >= 2) {
      horizCurves.push({
        points: pts,
        color: PLAN_LINEAR_COLORS[kind] ?? "#cccccc",
        width: kind === 1 ? 2 : 1.5,
        opacity: kind === 1 ? 0.88 : 0.74,
        name: `${PLAN_LINEAR_LABELS[kind] ?? "?"}: ${name}`,
      });
    }
    if (kind === 1) breaklineCount++;
    else featureLineCount++;
  }

  const alignmentInfos: LandXmlAlignmentInfo[] = [];
  for (let a = 0; a < alignCount; a++) {
    const aName = session.landxml_alignment_name(doc, a);
    const profCount = session.landxml_alignment_profile_count(doc, a);
    const profNames: string[] = [];
    for (let p = 0; p < profCount; p++) {
      profNames.push(session.landxml_alignment_profile_name(doc, a, p));
    }
    const staInfo = session.landxml_alignment_station_range(doc, a);
    alignmentInfos.push({
      index: a,
      name: aName,
      profileCount: profCount,
      profileNames: profNames,
      staStart: staInfo[0],
      staEnd: staInfo[1],
    });
  }

  const curveData: LandXmlCurveData = { horizCurves, raw3dCurves };
  const defaultDatum = 0;
  const overlayCurves = applyDatumAndExaggeration(curveData, defaultDatum, 1, 1);

  const context: LandXmlContext = {
    docHandle: doc,
    centroidX,
    centroidY,
    centroidZ,
    alignments: alignmentInfos,
  };

  const logs = [
    `LandXML parsed in ${parseMs.toFixed(1)}ms`,
    `Surfaces: ${surfCount}, Alignments: ${alignCount}, Vertices: ${totalVertCount}`,
    `FeatureLines: ${featureLineCount}, Breaklines: ${breaklineCount}`,
    `Units: ${unit}, Warnings: ${warnCount}`,
    ...debugLogs,
  ];

  const built: BuiltExample = {
    kind: "mesh",
    curveHandle: null,
    ownedHandles: [],
    name: filename,
    degreeLabel: `LandXML 1.2`,
    renderDegree: 0,
    renderSamples: 0,
    meshVisual,
    overlayMeshes,
    overlayCurves,
    segmentOverlays: [],
    intersectionPoints: [],
    planeVisual: null,
    interactiveMeshHandle: null,
    transformTargets: [],
    defaultTransformTargetKey: null,
    intersectionMs: 0,
    boundsMs: 0,
    logs,
  };

  return {
    built,
    stats: {
      surfCount,
      alignCount,
      vertCount: totalVertCount,
      featureLineCount,
      breaklineCount,
      unit,
      warnCount,
      parseMs,
    },
    curveData,
    zRange: { min: isFinite(zMin) ? zMin : 0, max: isFinite(zMax) ? zMax : 0 },
    defaultDatum,
    context,
  };
}

export function KernelViewer() {
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const sessionFileInputRef = useRef<HTMLInputElement | null>(null);

  const sessionRef = useRef<KernelSession | null>(null);
  const curveHandleRef = useRef<CurveHandle | null>(null);
  const ownedCurveHandlesRef = useRef<AnyHandle[]>([]);
  const nurbsPresetRef = useRef<CurvePreset | null>(null);

  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const perspCameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const orthoCameraRef = useRef<THREE.OrthographicCamera | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | THREE.OrthographicCamera | null>(null);
  const controlsRef = useRef<OrbitControls | null>(null);
  const activeHandlesRef = useRef<Array<{ objectId: number }>>([]);
  const exportHandlesRef = useRef<Array<{ objectId: number }>>([]);
  const meshToHandleMap = useRef(new WeakMap<THREE.Object3D, { objectId: number }>());
  const lineRef = useRef<Line2 | null>(null);
  const overlayLineRefs = useRef<Line2[]>([]);
  const segmentOverlayRefs = useRef<LineSegments2[]>([]);
  const meshRef = useRef<THREE.Mesh<THREE.BufferGeometry, THREE.MeshStandardMaterial> | null>(null);
  const meshWireRef = useRef<
    THREE.LineSegments<THREE.WireframeGeometry, THREE.LineBasicMaterial> | null
  >(null);
  const overlayMeshRefs = useRef<
    Array<{
      mesh: THREE.Mesh<THREE.BufferGeometry, THREE.MeshStandardMaterial>;
      wire: THREE.LineSegments<THREE.WireframeGeometry, THREE.LineBasicMaterial> | null;
    }>
  >([]);
  const transformControlsRef = useRef<TransformControls | null>(null);
  const transformControlsHelperRef = useRef<THREE.Object3D | null>(null);
  const isTransformDraggingRef = useRef(false);
  const dragStartTransformRef = useRef<{
    position: THREE.Vector3;
    quaternion: THREE.Quaternion;
    scale: THREE.Vector3;
  } | null>(null);
  const interactiveMeshHandleRef = useRef<MeshHandle | null>(null);
  const transformTargetsRef = useRef<TransformTarget[]>([]);
  const meshPlaneMeshHandleRef = useRef<MeshHandle | null>(null);
  const meshPlanePlaneRef = useRef<RgmPlane | null>(null);
  const booleanBaseMeshHandleRef = useRef<MeshHandle | null>(null);
  const booleanToolMeshHandleRef = useRef<MeshHandle | null>(null);
  const booleanResultMeshHandleRef = useRef<MeshHandle | null>(null);
  const planeGroupRef = useRef<THREE.Group | null>(null);
  const liveIntersectionTimerRef = useRef<number | null>(null);
  const previewMeshHandleRef = useRef<MeshHandle | null>(null);
  const suppressAutoFitRef = useRef(false);
  const intersectionMarkerRefs = useRef<
    THREE.Mesh<THREE.SphereGeometry, THREE.MeshStandardMaterial>[]
  >([]);
  const planeMeshRef = useRef<THREE.Mesh<THREE.BufferGeometry, THREE.MeshStandardMaterial> | null>(
    null,
  );
  const planeWireRef = useRef<THREE.LineSegments<THREE.BufferGeometry, THREE.LineBasicMaterial> | null>(
    null,
  );
  const planeNormalRef = useRef<THREE.ArrowHelper | null>(null);
  const probeRef = useRef<THREE.Mesh<THREE.SphereGeometry, THREE.MeshStandardMaterial> | null>(
    null,
  );
  const gridRef = useRef<THREE.GridHelper | null>(null);
  const axesRef = useRef<THREE.AxesHelper | null>(null);
  const probeTNormRef = useRef(0.35);
  const probePointRef = useRef<RgmPoint3 | null>(null);
  const totalLengthRef = useRef(0);
  const surfaceProbeHandleRef = useRef<SurfaceHandle | null>(null);
  const surfaceProbeD1ScaleRef = useRef(0.2);
  const surfaceProbeD2ScaleRef = useRef(0.1);
  const surfaceProbeUvRef = useRef<RgmUv2>({ u: 0.47, v: 0.63 });
  const updateSurfaceProbeForUvRef = useRef<(nextU: number, nextV: number, logCommit: boolean) => void>(
    () => {},
  );
  const logSequenceRef = useRef(1);

  const [preset, setPreset] = useState<CurvePreset | null>(null);
  const [activeExample, setActiveExample] = useState<ExampleKey>("nurbs");
  const [activeCurveName, setActiveCurveName] = useState("NURBS");
  const [activeDegreeLabel, setActiveDegreeLabel] = useState("");
  const [activeRenderDegree, setActiveRenderDegree] = useState(3);
  const [sampledPoints, setSampledPoints] = useState<RgmPoint3[]>([]);
  const [meshVisual, setMeshVisual] = useState<MeshVisual | null>(null);
  const [overlayMeshes, setOverlayMeshes] = useState<MeshVisual[]>([]);
  const [overlayCurves, setOverlayCurves] = useState<OverlayCurveVisual[]>([]);
  const [segmentOverlays, setSegmentOverlays] = useState<SegmentOverlayVisual[]>([]);
  const [intersectionPoints, setIntersectionPoints] = useState<RgmPoint3[]>([]);
  const [intersectionPlane, setIntersectionPlane] = useState<RgmPlane | null>(null);
  const [gizmoMode, setGizmoMode] = useState<GizmoMode>("translate");
  const [transformTargetsUi, setTransformTargetsUi] = useState<Array<{ key: string; label: string }>>(
    [],
  );
  const [transformTargetKey, setTransformTargetKey] = useState<string>("");
  const [meshPlaneTarget, setMeshPlaneTarget] = useState<"mesh" | "plane">("mesh");
  const [perfStats, setPerfStats] = useState<ViewerPerformance>({
    loadMs: 0,
    intersectionMs: 0,
    boundsMs: 0,
  });
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [, setStatusMessage] = useState("Booting kernel runtime...");
  const [, setErrorMessage] = useState<string | null>(null);
  const [capabilities, setCapabilities] = useState({ igesImport: false, igesExport: true });
  const [showGrid, setShowGrid] = useState(true);
  const [showAxes, setShowAxes] = useState(false);
  const [orbitEnabled, setOrbitEnabled] = useState(true);
  const [cameraMode, setCameraMode] = useState<CameraMode>("perspective");
  const cameraModeRef = useRef<CameraMode>("perspective");
  const [sceneUpAxis, setSceneUpAxis] = useState<SceneUpAxis>("z");
  const followCameraRef = useRef(false);
  const [probeUiState, setProbeUiState] = useState<ProbeUiState>({
    tNorm: 0.35,
    x: 0,
    y: 0,
    z: 0,
    probeLength: 0,
    totalLength: 0,
  });
  const [surfaceProbeUiState, setSurfaceProbeUiState] = useState<SurfaceProbeUiState>({
    u: surfaceProbeUvRef.current.u,
    v: surfaceProbeUvRef.current.v,
    point: { x: 0, y: 0, z: 0 },
    du: { x: 0, y: 0, z: 0 },
    dv: { x: 0, y: 0, z: 0 },
    normal: { x: 0, y: 0, z: 1 },
    hasD2: false,
    duu: { x: 0, y: 0, z: 0 },
    duv: { x: 0, y: 0, z: 0 },
    dvv: { x: 0, y: 0, z: 0 },
  });
  const [isInspectorOpen, setIsInspectorOpen] = useState(true);
  const [isConsoleOpen, setIsConsoleOpen] = useState(true);
  const [isMobileLayout, setIsMobileLayout] = useState(false);
  const [kernelStatus, setKernelStatus] = useState<KernelStatus>("booting");
  const [isExampleBrowserOpen, setIsExampleBrowserOpen] = useState(false);
  const [consoleFilter, setConsoleFilter] = useState<LogLevel | "all">("all");
  const [activeLandXmlFile, setActiveLandXmlFile] = useState<string>("12DExample.xml");
  const [landXmlStats, setLandXmlStats] = useState<LandXmlStats | null>(null);
  const [landXmlDatumOffset, setLandXmlDatumOffset] = useState(0);
  const [landXmlVertExag, setLandXmlVertExag] = useState(1);
  const [landXmlScaleFactor, setLandXmlScaleFactor] = useState(1);
  const [landXmlZRange, setLandXmlZRange] = useState<{ min: number; max: number }>({ min: 0, max: 0 });
  const landXmlCurveDataRef = useRef<LandXmlCurveData | null>(null);
  const landXmlContextRef = useRef<LandXmlContext | null>(null);
  const landXmlRawMeshRef = useRef<{ meshVisual: MeshVisual | null; overlayMeshes: MeshVisual[] } | null>(null);
  const [landXmlAlignments, setLandXmlAlignments] = useState<LandXmlAlignmentInfo[]>([]);
  const [landXmlProbeAlignIdx, setLandXmlProbeAlignIdx] = useState(0);
  const [landXmlProbeProfileIdx, setLandXmlProbeProfileIdx] = useState(0);
  const [landXmlProbeUiState, setLandXmlProbeUiState] = useState<LandXmlProbeUiState>({
    station: 0,
    stationNorm: 0.5,
    alignmentIndex: 0,
    profileIndex: 0,
    alignmentPoint: { x: 0, y: 0, z: 0 },
    profilePoint: { x: 0, y: 0, z: 0 },
    tangent: { x: 1, y: 0, z: 0 },
    grade: 0,
  });
  const pendingAsyncExampleRef = useRef<AbortController | null>(null);

  const { isDarkMode, toggleDarkMode } = useTheme();
  const isDarkModeRef = useRef(isDarkMode);

  const appendLog = useCallback((level: LogLevel, message: string): void => {
    setLogs((previous) => {
      const next = [
        ...previous,
        {
          id: logSequenceRef.current,
          level,
          time: nowStamp(),
          message,
        },
      ];
      logSequenceRef.current += 1;
      if (next.length > 500) {
        return next.slice(next.length - 500);
      }
      return next;
    });
  }, []);

  const clearLogs = useCallback(() => {
    setLogs([]);
  }, []);

  const releaseOwnedCurveHandles = useCallback((): void => {
    // Handles auto-release when GC'd or when session.free() is called.
    ownedCurveHandlesRef.current = [];
    curveHandleRef.current = null;
    interactiveMeshHandleRef.current = null;
    meshPlaneMeshHandleRef.current = null;
    meshPlanePlaneRef.current = null;
    booleanBaseMeshHandleRef.current = null;
    booleanToolMeshHandleRef.current = null;
    booleanResultMeshHandleRef.current = null;
    surfaceProbeHandleRef.current = null;
    surfaceProbeD1ScaleRef.current = 0.2;
    surfaceProbeD2ScaleRef.current = 0.1;
    transformTargetsRef.current = [];
    setTransformTargetsUi([]);
    if (previewMeshHandleRef.current !== null) {
      previewMeshHandleRef.current = null;
    }
    if (liveIntersectionTimerRef.current !== null) {
      window.clearTimeout(liveIntersectionTimerRef.current);
      liveIntersectionTimerRef.current = null;
    }
  }, []);

  const buildExampleCurve = useCallback(
    (
      session: KernelSession,
      example: ExampleKey,
      nurbsPresetOverride?: CurvePreset,
    ): BuiltExample => {
      const tol = nurbsPresetOverride?.tolerance ?? nurbsPresetRef.current?.tolerance ?? fallbackTolerance();
      const asCurve = (
        curveHandle: CurveHandle,
        ownedHandles: AnyHandle[],
        name: string,
        degreeLabel: string,
        renderDegree: number,
        renderSamples: number,
        logs: string[],
        overlayCurves: OverlayCurveVisual[] = [],
        intersectionPoints: RgmPoint3[] = [],
        planeVisual: RgmPlane | null = null,
      ): BuiltExample => ({
        kind: "curve",
        curveHandle,
        ownedHandles,
        name,
        degreeLabel,
        renderDegree,
        renderSamples,
        meshVisual: null,
        overlayMeshes: [],
        overlayCurves,
        segmentOverlays: [],
        intersectionPoints,
        planeVisual,
        interactiveMeshHandle: null,
        transformTargets: [],
        defaultTransformTargetKey: null,
        booleanState: null,
        intersectionMs: 0,
        boundsMs: 0,
        logs,
      });

      if (example === "nurbs") {
        const presetToUse = nurbsPresetOverride ?? nurbsPresetRef.current;
        if (!presetToUse) {
          throw new Error("NURBS preset is not loaded");
        }
        const _fp0 = pointsToFlat(presetToUse.points);
        const handle = session.interpolate_nurbs_fit_points(_fp0, presetToUse.degree, presetToUse.closed);
        return asCurve(
          handle,
          [handle],
          presetToUse.name,
          `NURBS p=${presetToUse.degree}`,
          presetToUse.degree,
          renderSampleCountForPreset(presetToUse),
          constructionLogLines(presetToUse),
        );
      }

      if (example === "line") {
        const line: RgmLine3 = {
          start: { x: -7.8, y: -2.9, z: 1.6 },
          end: { x: 8.1, y: 3.4, z: -2.3 },
        };
        const handle = session.create_line(line.start.x, line.start.y, line.start.z, line.end.x, line.end.y, line.end.z);
        return asCurve(handle, [handle], "Skew 3D Line Span", "Line (p=1)", 1, 320, [
            `Line start=(${line.start.x}, ${line.start.y}, ${line.start.z})`,
            `Line end=(${line.end.x}, ${line.end.y}, ${line.end.z})`,
          ]);
      }

      if (example === "polyline") {
        const points: RgmPoint3[] = [
          { x: -8.0, y: -2.1, z: 0.4 },
          { x: -6.7, y: 1.2, z: 1.9 },
          { x: -5.2, y: 2.6, z: -0.6 },
          { x: -3.4, y: 0.5, z: -2.3 },
          { x: -1.7, y: -1.6, z: -0.1 },
          { x: 0.4, y: 0.2, z: 2.4 },
          { x: 2.8, y: 2.9, z: 1.2 },
          { x: 5.1, y: 1.7, z: -1.8 },
          { x: 7.4, y: -1.1, z: -0.5 },
        ];
        const handle = session.create_polyline(pointsToFlat(points), false);
        return asCurve(
          handle,
          [handle],
          "Spatial Polyline Traverse",
          "Polyline (p=1)",
          1,
          1200,
          [`Polyline vertices=${points.length} closed=false`],
        );
      }

      if (example === "arc") {
        const arc: RgmArc3 = {
          plane: {
            origin: { x: -1.3, y: 0.9, z: 0.7 },
            x_axis: { x: 0.8944271909999159, y: 0.0, z: 0.4472135954999579 },
            y_axis: { x: 0.0, y: 1.0, z: 0.0 },
            z_axis: { x: -0.4472135954999579, y: 0.0, z: 0.8944271909999159 },
          },
          radius: 4.25,
          start_angle: -0.55,
          sweep_angle: 2.35,
        };
        const handle = session.create_arc(arc.plane.origin.x, arc.plane.origin.y, arc.plane.origin.z, arc.plane.x_axis.x, arc.plane.x_axis.y, arc.plane.x_axis.z, arc.plane.y_axis.x, arc.plane.y_axis.y, arc.plane.y_axis.z, arc.plane.z_axis.x, arc.plane.z_axis.y, arc.plane.z_axis.z, arc.radius, arc.start_angle, arc.sweep_angle);
        return asCurve(
          handle,
          [handle],
          "Tilted Rational Arc",
          "Arc (rational p=2)",
          2,
          1800,
          [`Arc radius=${arc.radius} start=${arc.start_angle} sweep=${arc.sweep_angle}`],
        );
      }

      if (example === "circle") {
        const circle: RgmCircle3 = {
          plane: {
            origin: { x: 1.6, y: -0.7, z: 1.3 },
            x_axis: { x: 0.7071067811865476, y: 0.7071067811865476, z: 0.0 },
            y_axis: { x: -0.4082482904638631, y: 0.4082482904638631, z: 0.8164965809277261 },
            z_axis: { x: 0.5773502691896258, y: -0.5773502691896258, z: 0.5773502691896258 },
          },
          radius: 3.6,
        };
        const handle = session.create_circle(circle.plane.origin.x, circle.plane.origin.y, circle.plane.origin.z, circle.plane.x_axis.x, circle.plane.x_axis.y, circle.plane.x_axis.z, circle.plane.y_axis.x, circle.plane.y_axis.y, circle.plane.y_axis.z, circle.plane.z_axis.x, circle.plane.z_axis.y, circle.plane.z_axis.z, circle.radius);
        return asCurve(
          handle,
          [handle],
          "Tilted Rational Circle",
          "Circle (rational p=2 periodic)",
          2,
          2400,
          [`Circle radius=${circle.radius}`],
        );
      }

      if (example === "bboxCurveNonTrivial") {
        const builtHandles: AnyHandle[] = [];
        try {
          const lineA = session.create_line(-8.2, -2.6, 1.4, -4.8, 0.9, 3.0);
          builtHandles.push(lineA);
          const arcA = session.create_arc(-4.8, 0.9, 3.0, 0.7, 0.2, 0.68, -0.12, 0.97, -0.2, -0.71, 0.11, 0.69, 2.4, 0.0, 1.6);
          builtHandles.push(arcA);
          const lineB = session.create_line(-2.1, 3.1, 1.8, 3.8, 1.4, -2.4);
          builtHandles.push(lineB);
          const arcB = session.create_arc(3.8, 1.4, -2.4, 0.54, 0.84, 0.02, -0.22, 0.12, 0.97, 0.81, -0.53, 0.23, 2.1, 0.0, -1.18);
          builtHandles.push(arcB);

          const _pcSegs = [lineA, false, arcA, false, lineB, false, arcB, false].reduce<number[]>((acc, val, i) => {
            if (i % 2 === 0) acc.push((val as CurveHandle).object_id());
            else acc.push(val ? 1.0 : 0.0);
            return acc;
          }, []);
          const polycurve = session.create_polycurve(new Float64Array(_pcSegs));
          builtHandles.push(polycurve);

          const fastStart = performance.now();
          const fast = session.compute_bounds(polycurve.object_id(), 0, 0, 0.0);
          const fastMs = performance.now() - fastStart;
          const optimalStart = performance.now();
          const optimal = session.compute_bounds(polycurve.object_id(), 1, 2048, 0.0);
          const optimalMs = performance.now() - optimalStart;

          const fastSegments = boundsOverlaySegments(fast).map((segment) => ({
            ...segment,
            color: segment.name === "world-aabb" ? "#ff9d5a" : "#b38cff",
            opacity: 0.46,
            width: (segment.width ?? 2.4) - 0.55,
            name: `fast-${segment.name}`,
          }));
          const optimalSegments = boundsOverlaySegments(optimal).map((segment) => ({
            ...segment,
            opacity: Math.min(1, segment.opacity + 0.08),
            name: `optimal-${segment.name}`,
          }));

          const fastObbExt = obbExtents(fast);
          const optimalObbExt = obbExtents(optimal);
          const base = asCurve(
            polycurve,
            builtHandles,
            "Bounds Curve: Mixed Polycurve",
            "Curve bounds (Fast vs Optimal + local frame)",
            3,
            3600,
            [],
          );
          return {
            ...base,
            segmentOverlays: [...fastSegments, ...optimalSegments],
            boundsMs: fastMs + optimalMs,
            logs: [
              `mode=Fast time=${fastMs.toFixed(2)}ms obb_extents=${formatExtents(fastObbExt)} volume=${extentsVolume(fastObbExt).toFixed(3)}`,
              `mode=Optimal time=${optimalMs.toFixed(2)}ms obb_extents=${formatExtents(optimalObbExt)} volume=${extentsVolume(optimalObbExt).toFixed(3)}`,
              `world_aabb_extents=${formatExtents(aabbExtentsFromBounds3(optimal))}`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "intersectCurveCurve") {
        const builtHandles: AnyHandle[] = [];
        try {
          const transform = new THREE.Matrix4().makeRotationFromEuler(
            new THREE.Euler(0.68, -0.41, 0.52, "XYZ"),
          );
          transform.setPosition(new THREE.Vector3(1.4, -0.9, 0.7));
          const rotationOnly = new THREE.Matrix3().setFromMatrix4(transform);
          const transformPoint = (x: number, y: number, z: number): RgmPoint3 =>
            toPoint3(new THREE.Vector3(x, y, z).applyMatrix4(transform));
          const transformAxis = (x: number, y: number, z: number): RgmPoint3 =>
            toPoint3(new THREE.Vector3(x, y, z).applyMatrix3(rotationOnly).normalize());

          const circlePrimary: RgmCircle3 = {
            plane: {
              origin: transformPoint(0, 0, 0),
              x_axis: transformAxis(1, 0, 0),
              y_axis: transformAxis(0, 1, 0),
              z_axis: transformAxis(0, 0, 1),
            },
            radius: 4.8,
          };
          const circleSecondary: RgmCircle3 = {
            plane: {
              origin: transformPoint(0, 0, 0),
              x_axis: transformAxis(0, 1, 0),
              y_axis: transformAxis(0, 0, 1),
              z_axis: transformAxis(1, 0, 0),
            },
            radius: 4.8,
          };

          const primaryHandle = session.create_circle(circlePrimary.plane.origin.x, circlePrimary.plane.origin.y, circlePrimary.plane.origin.z, circlePrimary.plane.x_axis.x, circlePrimary.plane.x_axis.y, circlePrimary.plane.x_axis.z, circlePrimary.plane.y_axis.x, circlePrimary.plane.y_axis.y, circlePrimary.plane.y_axis.z, circlePrimary.plane.z_axis.x, circlePrimary.plane.z_axis.y, circlePrimary.plane.z_axis.z, circlePrimary.radius);
          builtHandles.push(primaryHandle);
          const secondaryHandle = session.create_circle(circleSecondary.plane.origin.x, circleSecondary.plane.origin.y, circleSecondary.plane.origin.z, circleSecondary.plane.x_axis.x, circleSecondary.plane.x_axis.y, circleSecondary.plane.x_axis.z, circleSecondary.plane.y_axis.x, circleSecondary.plane.y_axis.y, circleSecondary.plane.y_axis.z, circleSecondary.plane.z_axis.x, circleSecondary.plane.z_axis.y, circleSecondary.plane.z_axis.z, circleSecondary.radius);
          builtHandles.push(secondaryHandle);

          const secondarySamples = samplePolyline(session, secondaryHandle, 2400);
          const hits = flatToPoints(session.intersect_curve_curve(primaryHandle, secondaryHandle));
          const hitLogs = hits.map(
            (point, idx) =>
              `Curve-curve hit ${idx + 1}: (${point.x.toFixed(4)}, ${point.y.toFixed(4)}, ${point.z.toFixed(4)})`,
          );

          return asCurve(
            primaryHandle,
            builtHandles,
            "Dual Tilted Circle Intersection",
            "Intersection (curve-curve)",
            2,
            2400,
            [
              "Primary: rational circle in tilted plane",
              "Secondary: orthogonal tilted circle transformed in world space",
              `Intersection count=${hits.length}`,
              ...hitLogs,
            ],
            [
              {
                points: secondarySamples,
                color: "#f8ae63",
                width: 2.4,
                opacity: 0.95,
                name: "secondary curve",
              },
            ],
            hits,
            null,
          );
        } catch (error) {
          throw error;
        }
      }

      if (example === "intersectCurvePlane") {
        const planeNormal = new THREE.Vector3(0.46, -0.37, 0.81).normalize();
        let planeXAxis = new THREE.Vector3(0.93, 0.15, -0.34).normalize();
        planeXAxis = planeXAxis
          .clone()
          .sub(planeNormal.clone().multiplyScalar(planeXAxis.dot(planeNormal)))
          .normalize();
        const planeYAxis = new THREE.Vector3().crossVectors(planeNormal, planeXAxis).normalize();
        const planeOrigin = new THREE.Vector3(-0.8, 0.5, 0.2);
        const plane: RgmPlane = {
          origin: toPoint3(planeOrigin),
          x_axis: toPoint3(planeXAxis),
          y_axis: toPoint3(planeYAxis),
          z_axis: toPoint3(planeNormal),
        };

        const fitPoints: RgmPoint3[] = [];
        for (let idx = 0; idx < 11; idx += 1) {
          const along = -7.2 + idx * 1.45;
          const across = Math.sin(idx * 0.92) * 2.9;
          const normalOffset = (idx % 2 === 0 ? 1 : -1) * (1.2 + 0.3 * Math.cos(idx * 0.55));
          const point = planeOrigin
            .clone()
            .add(planeXAxis.clone().multiplyScalar(along))
            .add(planeYAxis.clone().multiplyScalar(across))
            .add(planeNormal.clone().multiplyScalar(normalOffset));
          fitPoints.push(toPoint3(point));
        }

        const _fp1 = pointsToFlat(fitPoints);
        const curveHandle = session.interpolate_nurbs_fit_points(_fp1, 3, false);
        const hits = flatToPoints(session.intersect_curve_plane(curveHandle, flattenPlane(plane)));
        const hitLogs = hits.map(
          (point, idx) =>
            `Curve-plane hit ${idx + 1}: (${point.x.toFixed(4)}, ${point.y.toFixed(4)}, ${point.z.toFixed(4)})`,
        );

        return asCurve(
          curveHandle,
          [curveHandle],
          "NURBS vs Tilted Plane",
          "Intersection (curve-plane)",
          3,
          3600,
          [
            `Curve control points=${fitPoints.length}`,
            "Plane is intentionally oblique to world axes",
            `Intersection count=${hits.length}`,
            ...hitLogs,
          ],
          [],
          hits,
          plane,
        );
      }

      if (example === "meshLarge") {
        const mesh = session.create_torus_mesh(0, 0, 0, 6.0, 1.35, 240, 160);
        const buffers = meshToBuffers(session, mesh);
        return {
          kind: "mesh",
          curveHandle: null,
          ownedHandles: [mesh],
          name: "Dense Torus Benchmark",
          degreeLabel: "Mesh (high-resolution indexed)",
          renderDegree: 0,
          renderSamples: 0,
          meshVisual: {
            vertices: buffers.vertices,
            indices: buffers.indices,
            color: "#5f9de0",
            opacity: 0.88,
            wireframe: true,
            name: "dense torus",
          },
          overlayMeshes: [],
          overlayCurves: [],
          segmentOverlays: [],
          intersectionPoints: [],
          planeVisual: null,
          interactiveMeshHandle: null,
          transformTargets: [],
          defaultTransformTargetKey: null,
          intersectionMs: 0,
          logs: [
            `mesh vertices=${session.mesh_vertex_count(mesh)}`,
            `mesh triangles=${session.mesh_triangle_count(mesh)}`,
          ],
        };
      }

      if (example === "meshTransform") {
        const built: AnyHandle[] = [];
        try {
          const base = session.create_box_mesh(0.0, 0.0, -1.0, 7.2, 2.6, 1.2);
          built.push(base);
          const rotor = session.create_torus_mesh(0, 0, 0, 2.0, 0.52, 108, 72);
          built.push(rotor);

          const baseBuffers = meshToBuffers(session, base);
          const rotorBuffers = meshToBuffers(session, rotor);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            name: "Interactive Rotor Transform",
            degreeLabel: "Mesh transform gizmo (kernel-linked)",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: rotorBuffers.vertices,
              indices: rotorBuffers.indices,
              color: "#7ec9ff",
              opacity: 0.98,
              wireframe: true,
              name: "interactive rotor",
            },
            overlayMeshes: [
              {
                vertices: baseBuffers.vertices,
                indices: baseBuffers.indices,
                color: "#6d86a8",
                opacity: 0.35,
                wireframe: true,
                name: "fixture",
              },
            ],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: rotor,
            transformTargets: [
              {
                key: "fixture",
                label: "Fixture mesh",
                handle: base,
                color: "#6d86a8",
                opacity: 0.82,
                wireframe: true,
              },
              {
                key: "rotor",
                label: "Rotor mesh",
                handle: rotor,
                color: "#7ec9ff",
                opacity: 0.98,
                wireframe: true,
              },
            ],
            defaultTransformTargetKey: "rotor",
            intersectionMs: 0,
            logs: [
              `base triangles=${session.mesh_triangle_count(base)}`,
              `rotor triangles=${session.mesh_triangle_count(rotor)}`,
              "Use target selector + gizmo mode to transform either fixture or rotor.",
              "Each drag commit updates the kernel mesh and refreshes geometry from kernel buffers.",
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "meshIntersectMeshMesh") {
        const built: AnyHandle[] = [];
        try {
          const sphere = session.create_uv_sphere_mesh(0, 0, 0, 4.6, 56, 40);
          built.push(sphere);
          const torus = session.create_torus_mesh(0.5, 0.2, 0.1, 4.2, 1.15, 92, 64);
          built.push(torus);
          const intersectionStart = performance.now();
          const hits = flatToPoints(session.intersect_mesh_mesh(sphere, torus));
          const intersectionMs = performance.now() - intersectionStart;
          const sphereBuffers = meshToBuffers(session, sphere);
          const torusBuffers = meshToBuffers(session, torus);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            name: "Sphere vs Torus Intersection",
            degreeLabel: "Mesh-mesh intersection segments",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: sphereBuffers.vertices,
              indices: sphereBuffers.indices,
              color: "#79a9de",
              opacity: 0.25,
              wireframe: false,
              name: "sphere",
            },
            overlayMeshes: [
              {
                vertices: torusBuffers.vertices,
                indices: torusBuffers.indices,
                color: "#f2b977",
                opacity: 0.28,
                wireframe: false,
                name: "torus",
              },
            ],
            overlayCurves: [],
            segmentOverlays: [
              {
                points: hits,
                color: "#ffe46b",
                opacity: 0.98,
                name: "mesh-mesh-hit",
              },
            ],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs,
            logs: [
              `mesh-mesh segment pairs=${Math.floor(hits.length / 2)}`,
              `raw points=${hits.length}`,
              `intersection solve=${intersectionMs.toFixed(2)}ms`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "meshIntersectMeshPlane") {
        const mesh = session.create_torus_mesh(0.4, -0.2, 0.7, 5.1, 1.3, 128, 72);
        const planeNormal = new THREE.Vector3(0.42, -0.33, 0.84).normalize();
        let planeXAxis = new THREE.Vector3(0.9, 0.2, -0.32).normalize();
        planeXAxis = planeXAxis
          .clone()
          .sub(planeNormal.clone().multiplyScalar(planeXAxis.dot(planeNormal)))
          .normalize();
        const planeYAxis = new THREE.Vector3().crossVectors(planeNormal, planeXAxis).normalize();
        const planeOrigin = new THREE.Vector3(-0.5, 0.3, 0.2);
        const plane: RgmPlane = {
          origin: toPoint3(planeOrigin),
          x_axis: toPoint3(planeXAxis),
          y_axis: toPoint3(planeYAxis),
          z_axis: toPoint3(planeNormal),
        };
        const intersectionStart = performance.now();
        const hits = flatToPoints(session.intersect_mesh_plane(mesh, flattenPlane(plane)));
        const intersectionMs = performance.now() - intersectionStart;
        const meshBuffers = meshToBuffers(session, mesh);
        return {
          kind: "mesh",
          curveHandle: null,
          ownedHandles: [mesh],
          name: "Oblique Plane Section",
          degreeLabel: "Mesh-plane intersection segments",
          renderDegree: 0,
          renderSamples: 0,
          meshVisual: {
            vertices: meshBuffers.vertices,
            indices: meshBuffers.indices,
            color: "#74a9d8",
            opacity: 0.3,
            wireframe: false,
            name: "section target",
          },
          overlayMeshes: [],
          overlayCurves: [],
          segmentOverlays: [
            {
              points: hits,
              color: "#ffef7f",
              opacity: 0.99,
              name: "mesh-plane-hit",
            },
          ],
          intersectionPoints: [],
          planeVisual: plane,
          interactiveMeshHandle: mesh,
          transformTargets: [],
          defaultTransformTargetKey: null,
          intersectionMs,
          logs: [
            `mesh triangles=${session.mesh_triangle_count(mesh)}`,
            `mesh-plane segment pairs=${Math.floor(hits.length / 2)}`,
            `intersection solve=${intersectionMs.toFixed(2)}ms`,
          ],
        };
      }

      if (example === "meshBoolean") {
        const built: AnyHandle[] = [];
        try {
          const outer = session.create_box_mesh(0, 0, 0, 9.0, 9.0, 9.0);
          built.push(outer);
          const inner = session.create_torus_mesh(2.2, 0.0, 0.0, 2.8, 0.95, 72, 52);
          built.push(inner);
          const result = session.mesh_boolean(outer, inner, 2);
          built.push(result);
          const outerBuffers = meshToBuffers(session, outer);
          const innerBuffers = meshToBuffers(session, inner);
          const resultBuffers = meshToBuffers(session, result);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            name: "Interactive CSG Difference (A - B)",
            degreeLabel: "Move A/B with gizmo, then recompute boolean difference",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: innerBuffers.vertices,
              indices: innerBuffers.indices,
              color: "#f7ba74",
              opacity: 0.16,
              wireframe: false,
              name: "subtracted solid (B): torus (active target)",
            },
            overlayMeshes: [
              {
                vertices: outerBuffers.vertices,
                indices: outerBuffers.indices,
                color: "#8aa2ba",
                opacity: 0.08,
                wireframe: false,
                name: "base solid (A): box",
              },
              {
                vertices: resultBuffers.vertices,
                indices: resultBuffers.indices,
                color: "#8ac6ff",
                opacity: 0.95,
                wireframe: false,
                name: "boolean result (A - B)",
              },
            ],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: inner,
            transformTargets: [
              {
                key: "base",
                label: "Base solid (A): box",
                handle: outer,
                color: "#8aa2ba",
                opacity: 0.14,
                wireframe: false,
              },
              {
                key: "tool",
                label: "Subtracted solid (B): torus",
                handle: inner,
                color: "#f7ba74",
                opacity: 0.16,
                wireframe: false,
              },
            ],
            defaultTransformTargetKey: "tool",
            booleanState: {
              baseHandle: outer,
              toolHandle: inner,
              resultHandle: result,
            },
            intersectionMs: 0,
            logs: [
              "CSG difference: result = A - B (box minus torus)",
              "Choose target A/B in Controls -> Gizmo, then drag in viewport.",
              "Each drag commit recomputes boolean result from current source solids.",
              `outer triangles=${session.mesh_triangle_count(outer)}`,
              `inner triangles=${session.mesh_triangle_count(inner)}`,
              `result triangles=${session.mesh_triangle_count(result)}`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "bboxMeshBooleanAssembly") {
        const built: AnyHandle[] = [];
        try {
          const outer = session.create_box_mesh(0, 0, 0, 9.6, 8.6, 8.2);
          built.push(outer);
          const torus = session.create_torus_mesh(1.8, 0.0, 0.3, 2.7, 0.9, 78, 54);
          built.push(torus);
          const sphere = session.create_uv_sphere_mesh(-1.1, 1.0, -0.6, 2.25, 40, 30);
          built.push(sphere);
          const cutA = session.mesh_boolean(outer, torus, 2);
          built.push(cutA);
          const rotated = session.mesh_rotate(cutA, 0.34, 1.0, 0.18, 0.74, 0, 0, 0);
          built.push(rotated);
          const moved = session.mesh_translate(rotated, 1.2, -0.4, 0.9);
          built.push(moved);

          const firstFastStart = performance.now();
          const fastFirst = session.compute_bounds(moved.object_id(), 0, 0, 0.0);
          const fastFirstMs = performance.now() - firstFastStart;
          const cachedFastStart = performance.now();
          const fastCached = session.compute_bounds(moved.object_id(), 0, 0, 0.0);
          const fastCachedMs = performance.now() - cachedFastStart;
          const optimalStart = performance.now();
          const optimal = session.compute_bounds(moved.object_id(), 1, 8192, 0.0);
          const optimalMs = performance.now() - optimalStart;

          const movedBuffers = meshToBuffers(session, moved);
          const outerBuffers = meshToBuffers(session, outer);
          const torusBuffers = meshToBuffers(session, torus);
          const sphereBuffers = meshToBuffers(session, sphere);
          const speedup = fastCachedMs > 1e-9 ? fastFirstMs / fastCachedMs : 0;
          const optimalObbExt = obbExtents(optimal);

          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [moved],
            name: "Bounds Mesh: Boolean Assembly",
            degreeLabel: "boolean difference + transform, with Fast cache and Optimal OBB",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: movedBuffers.vertices,
              indices: movedBuffers.indices,
              color: "#8ac6ff",
              opacity: 0.86,
              wireframe: false,
              name: "boolean assembly result",
            },
            overlayMeshes: [
              {
                vertices: outerBuffers.vertices,
                indices: outerBuffers.indices,
                color: "#7f8fa6",
                opacity: 0.08,
                wireframe: false,
                name: "source box",
              },
              {
                vertices: torusBuffers.vertices,
                indices: torusBuffers.indices,
                color: "#f3b06f",
                opacity: 0.08,
                wireframe: false,
                name: "source torus",
              },
              {
                vertices: sphereBuffers.vertices,
                indices: sphereBuffers.indices,
                color: "#99d9a7",
                opacity: 0.07,
                wireframe: false,
                name: "fixture sphere (reference)",
              },
            ],
            overlayCurves: [],
            segmentOverlays: boundsOverlaySegments(optimal),
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            boundsMs: fastFirstMs + fastCachedMs + optimalMs,
            logs: [
              `mode=Fast first=${fastFirstMs.toFixed(2)}ms cached=${fastCachedMs.toFixed(2)}ms speedup=${speedup.toFixed(2)}x`,
              `mode=Fast world_aabb_extents=${formatExtents(aabbExtentsFromBounds3(fastFirst))}`,
              `mode=Optimal time=${optimalMs.toFixed(2)}ms obb_extents=${formatExtents(optimalObbExt)} volume=${extentsVolume(optimalObbExt).toFixed(3)}`,
              "local_aabb_extents=n/a (not exposed in Bounds3)",
              "robust boolean path: result = (box - torus), sphere rendered as reference fixture only",
              `fast-repeat world_aabb delta_z=${Math.abs(fastFirst.aabb_max_z - fastCached.aabb_max_z).toExponential(2)}`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "surfaceLarge") {
        const built: AnyHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(28, 24, 18, 14, 1.6);
          const surface = session.create_nurbs_surface(net.desc.degree_u, net.desc.degree_v, net.desc.control_u_count, net.desc.control_v_count, net.desc.periodic_u, net.desc.periodic_v, net.points, net.weights, net.knotsU, net.knotsV);
          built.push(surface);
          const mesh = session.surface_tessellate_to_mesh(surface, new Float64Array([72, 56, 96, 72, 1e-4, 0.04]));
          built.push(mesh);
          const buffers = meshToBuffers(session, mesh);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            name: "Large Untrimmed NURBS Surface",
            degreeLabel: "Surface tessellation (kernel)",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: buffers.vertices,
              indices: buffers.indices,
              color: "#77addf",
              opacity: 0.92,
              wireframe: false,
              name: "surface",
            },
            overlayMeshes: [],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            logs: [
              `control net=${net.desc.control_u_count}x${net.desc.control_v_count}`,
              `triangles=${session.mesh_triangle_count(mesh)}`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "surfaceTransform") {
        const built: AnyHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(16, 14, 12, 10, 1.1);
          const surface = session.create_nurbs_surface(net.desc.degree_u, net.desc.degree_v, net.desc.control_u_count, net.desc.control_v_count, net.desc.periodic_u, net.desc.periodic_v, net.points, net.weights, net.knotsU, net.knotsV);
          built.push(surface);
          const moved = session.surface_translate(surface, 1.4, -0.7, 0.9);
          built.push(moved);
          const rotated = session.surface_rotate(moved, 0.4, 1.0, 0.2, 0.68, 0, 0, 0);
          built.push(rotated);
          const scaled = session.surface_scale(rotated, 1.15, 0.82, 1.3, 0.5, -0.2, 0.1);
          built.push(scaled);
          const baseMesh = session.surface_tessellate_to_mesh(surface, new Float64Array(0));
          const transformedMesh = session.surface_tessellate_to_mesh(scaled, new Float64Array(0));
          built.push(baseMesh, transformedMesh);
          const baseBuffers = meshToBuffers(session, baseMesh);
          const transformedBuffers = meshToBuffers(session, transformedMesh);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [surface, scaled],
            name: "Surface Transform Chain",
            degreeLabel: "translate -> rotate -> scale (kernel surface ops)",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: transformedBuffers.vertices,
              indices: transformedBuffers.indices,
              color: "#8ac6ff",
              opacity: 0.9,
              wireframe: true,
              name: "transformed surface",
            },
            overlayMeshes: [
              {
                vertices: baseBuffers.vertices,
                indices: baseBuffers.indices,
                color: "#f7c88a",
                opacity: 0.24,
                wireframe: true,
                name: "original surface",
              },
            ],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            logs: [
              `base triangles=${session.mesh_triangle_count(baseMesh)}`,
              `transformed triangles=${session.mesh_triangle_count(transformedMesh)}`,
              "Transform APIs used: surfaceTranslate, surfaceRotate, surfaceScale",
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "bboxSurfaceWarped") {
        const built: AnyHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(18, 16, 13, 11, 1.25);
          const base = session.create_nurbs_surface(net.desc.degree_u, net.desc.degree_v, net.desc.control_u_count, net.desc.control_v_count, net.desc.periodic_u, net.desc.periodic_v, net.points, net.weights, net.knotsU, net.knotsV);
          built.push(base);
          const moved = session.surface_translate(base, 0.8, -0.5, 0.7);
          built.push(moved);
          const rotated = session.surface_rotate(moved, 0.28, 1.0, 0.33, 0.64, 0.0, 0.0, 0.0);
          built.push(rotated);
          const scaled = session.surface_scale(rotated, 1.1, 0.84, 1.22, 0.2, -0.1, 0.0);
          built.push(scaled);

          const fastStart = performance.now();
          const fast = session.compute_bounds(scaled.object_id(), 0, 0, 0.0);
          const fastMs = performance.now() - fastStart;
          const optimalStart = performance.now();
          const optimal = session.compute_bounds(scaled.object_id(), 1, 4096, 0.0);
          const optimalMs = performance.now() - optimalStart;

          let outsideFast = 0;
          let outsideOptimal = 0;
          for (let iu = 0; iu <= 20; iu += 1) {
            const u = iu / 20;
            for (let iv = 0; iv <= 20; iv += 1) {
              const v = iv / 20;
              const sample = ((_spt) => ({ x: _spt[0], y: _spt[1], z: _spt[2] }))(session.surface_point_at(scaled, u, v));
              if (!pointInsideBounds3Aabb(fast, sample, 1e-6)) {
                outsideFast += 1;
              }
              if (!pointInsideBounds3Aabb(optimal, sample, 1e-6)) {
                outsideOptimal += 1;
              }
            }
          }

          const mesh = session.surface_tessellate_to_mesh(scaled, new Float64Array([24, 22, 52, 48, 2.0e-4, 0.08]));
          built.push(mesh);
          const buffers = meshToBuffers(session, mesh);
          const fastObb = obbExtents(fast);
          const optimalObb = obbExtents(optimal);

          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [scaled],
            name: "Bounds Surface: Warped Rational",
            degreeLabel: "transformed warped surface, Fast vs Optimal bounds",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: buffers.vertices,
              indices: buffers.indices,
              color: "#81b8e2",
              opacity: 0.78,
              wireframe: false,
              name: "warped transformed surface",
            },
            overlayMeshes: [],
            overlayCurves: [],
            segmentOverlays: boundsOverlaySegments(optimal),
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            boundsMs: fastMs + optimalMs,
            logs: [
              `mode=Fast time=${fastMs.toFixed(2)}ms obb_extents=${formatExtents(fastObb)} volume=${extentsVolume(fastObb).toFixed(3)}`,
              `mode=Optimal time=${optimalMs.toFixed(2)}ms obb_extents=${formatExtents(optimalObb)} volume=${extentsVolume(optimalObb).toFixed(3)}`,
              `world_aabb_extents=${formatExtents(aabbExtentsFromBounds3(optimal))}`,
              "local_aabb_extents=n/a (not exposed in Bounds3)",
              `containment sampled_uv=441 outside_fast=${outsideFast} outside_optimal=${outsideOptimal}`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "surfaceUvEval") {
        const built: AnyHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(22, 19, 16, 14, 1.55);
          const weights = new Float64Array(net.weights.map((base, idx) =>
            Math.max(0.22, base * (1 + 0.2 * Math.sin(idx * 0.37) + 0.08 * Math.cos(idx * 0.19))),
          ));
          const surfaceBase = session.create_nurbs_surface(net.desc.degree_u, net.desc.degree_v, net.desc.control_u_count, net.desc.control_v_count, net.desc.periodic_u, net.desc.periodic_v, net.points, weights, net.knotsU, net.knotsV);
          built.push(surfaceBase);

          const surfaceRot = session.surface_rotate(surfaceBase, 0.48, 1.0, 0.31, 0.62, 0.3, -0.1, 0.2);
          built.push(surfaceRot);

          const surface = session.surface_translate(surfaceRot, 0.9, -0.6, 0.5);
          built.push(surface);

          const mesh = session.surface_tessellate_to_mesh(surface, new Float64Array([30, 26, 54, 48, 1.8e-4, 0.075]));
          built.push(mesh);
          const buffers = meshToBuffers(session, mesh);

          let minX = Number.POSITIVE_INFINITY;
          let minY = Number.POSITIVE_INFINITY;
          let minZ = Number.POSITIVE_INFINITY;
          let maxX = Number.NEGATIVE_INFINITY;
          let maxY = Number.NEGATIVE_INFINITY;
          let maxZ = Number.NEGATIVE_INFINITY;
          for (const vertex of buffers.vertices) {
            minX = Math.min(minX, vertex.x);
            minY = Math.min(minY, vertex.y);
            minZ = Math.min(minZ, vertex.z);
            maxX = Math.max(maxX, vertex.x);
            maxY = Math.max(maxY, vertex.y);
            maxZ = Math.max(maxZ, vertex.z);
          }
          const span = Math.sqrt(
            (maxX - minX) * (maxX - minX) +
              (maxY - minY) * (maxY - minY) +
              (maxZ - minZ) * (maxZ - minZ),
          );
          const d1Scale = Math.max(0.08, span * 0.02);
          const d2Scale = d1Scale * 0.45;
          const logs: string[] = [
            `control net=${net.desc.control_u_count}x${net.desc.control_v_count} degree=(${net.desc.degree_u},${net.desc.degree_v})`,
            `weights range=[${Math.min(...weights).toFixed(4)}, ${Math.max(...weights).toFixed(4)}]`,
            `triangles=${session.mesh_triangle_count(mesh)}`,
            "Use the Surface Probe sliders to move a UV probe and inspect D0/D1 (+D2 when available).",
            "Arrow colors: du=orange, dv=cyan, duu=peach, duv=violet, dvv=blue.",
          ];

          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [surface],
            name: "Surface UV Differential Evaluation",
            degreeLabel: "D0/D1 and D2 (if available) at normalized UV samples",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: buffers.vertices,
              indices: buffers.indices,
              color: "#7fb0d8",
              opacity: 0.33,
              wireframe: false,
              name: "evaluation surface",
            },
            overlayMeshes: [],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            surfaceProbeHandle: surface,
            surfaceProbeD1Scale: d1Scale,
            surfaceProbeD2Scale: d2Scale,
            intersectionMs: 0,
            logs,
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "surfaceIntersectSurface") {
        const built: AnyHandle[] = [];
        try {
          const a = buildWarpedSurfaceNet(16, 15, 12, 10, 1.0);
          const b = buildWarpedSurfaceNet(15, 16, 11, 11, 1.25);
          const surfaceA = session.create_nurbs_surface(a.desc.degree_u, a.desc.degree_v, a.desc.control_u_count, a.desc.control_v_count, a.desc.periodic_u, a.desc.periodic_v, a.points, a.weights, a.knotsU, a.knotsV);
          const surfaceB0 = session.create_nurbs_surface(b.desc.degree_u, b.desc.degree_v, b.desc.control_u_count, b.desc.control_v_count, b.desc.periodic_u, b.desc.periodic_v, b.points, b.weights, b.knotsU, b.knotsV);
          built.push(surfaceA, surfaceB0);
          const surfaceB = session.surface_rotate(session.surface_translate(surfaceB0, 0.6, 0.3, -0.1), 0.3, 1.0, 0.2, 0.72, 0, 0, 0);
          built.push(surfaceB);
          const meshA = session.surface_tessellate_to_mesh(surfaceA, new Float64Array([18, 18, 42, 42, 2.5e-4, 0.1]));
          const meshB = session.surface_tessellate_to_mesh(surfaceB, new Float64Array([18, 18, 42, 42, 2.5e-4, 0.1]));
          built.push(meshA, meshB);
          const intersectionStart = performance.now();
          const inter = session.intersect_surface_surface(surfaceA, surfaceB);
          const intersectionMs = performance.now() - intersectionStart;
          built.push(inter);

          const branchCount = session.intersection_branch_count(inter);
          const segmentPts: RgmPoint3[] = [];
          for (let bi = 0; bi < branchCount; bi += 1) {
            const branch = flatToPoints(session.intersection_branch_copy_points(inter, bi));
            for (let i = 1; i < branch.length; i += 1) {
              segmentPts.push(branch[i - 1], branch[i]);
            }
          }
          const buffersA = meshToBuffers(session, meshA);
          const buffersB = meshToBuffers(session, meshB);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [surfaceA, surfaceB],
            name: "Surface vs Surface",
            degreeLabel: "surface-surface intersection branches",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: buffersA.vertices,
              indices: buffersA.indices,
              color: "#7aafd7",
              opacity: 0.3,
              wireframe: false,
              name: "surface A",
            },
            overlayMeshes: [
              {
                vertices: buffersB.vertices,
                indices: buffersB.indices,
                color: "#f0b775",
                opacity: 0.3,
                wireframe: false,
                name: "surface B",
              },
            ],
            overlayCurves: [],
            segmentOverlays: [
              {
                points: segmentPts,
                color: "#fff07b",
                opacity: 0.98,
                name: "surface-surface branches",
              },
            ],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs,
            logs: [
              `branch count=${branchCount}`,
              `segment pairs=${Math.floor(segmentPts.length / 2)}`,
              `intersection solve=${intersectionMs.toFixed(2)}ms`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "surfaceIntersectPlane") {
        const built: AnyHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(18, 16, 13, 11, 1.35);
          const surface = session.create_nurbs_surface(net.desc.degree_u, net.desc.degree_v, net.desc.control_u_count, net.desc.control_v_count, net.desc.periodic_u, net.desc.periodic_v, net.points, net.weights, net.knotsU, net.knotsV);
          built.push(surface);
          const mesh = session.surface_tessellate_to_mesh(surface, new Float64Array([18, 18, 42, 42, 2.5e-4, 0.1]));
          built.push(mesh);
          const plane: RgmPlane = {
            origin: { x: 0.2, y: -0.4, z: 0.25 },
            x_axis: { x: 1.0, y: 0.1, z: -0.1 },
            y_axis: { x: -0.1, y: 0.94, z: 0.32 },
            z_axis: { x: 0.12, y: -0.31, z: 0.94 },
          };
          const intersectionStart = performance.now();
          const inter = session.intersect_surface_plane(surface, flattenPlane(plane));
          const intersectionMs = performance.now() - intersectionStart;
          built.push(inter);
          const branchCount = session.intersection_branch_count(inter);
          const segments: RgmPoint3[] = [];
          for (let bi = 0; bi < branchCount; bi += 1) {
            const branch = flatToPoints(session.intersection_branch_copy_points(inter, bi));
            for (let i = 1; i < branch.length; i += 1) {
              segments.push(branch[i - 1], branch[i]);
            }
          }
          const buffers = meshToBuffers(session, mesh);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [surface],
            name: "Surface vs Plane",
            degreeLabel: "surface-plane section branches",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: buffers.vertices,
              indices: buffers.indices,
              color: "#79aed9",
              opacity: 0.35,
              wireframe: false,
              name: "surface",
            },
            overlayMeshes: [],
            overlayCurves: [],
            segmentOverlays: [
              {
                points: segments,
                color: "#fff07f",
                opacity: 0.99,
                name: "surface-plane branches",
              },
            ],
            intersectionPoints: [],
            planeVisual: plane,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs,
            logs: [
              `branch count=${branchCount}`,
              `segment pairs=${Math.floor(segments.length / 2)}`,
              `intersection solve=${intersectionMs.toFixed(2)}ms`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "surfaceIntersectCurve") {
        const built: AnyHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(16, 16, 12, 12, 1.2);
          const surface = session.create_nurbs_surface(net.desc.degree_u, net.desc.degree_v, net.desc.control_u_count, net.desc.control_v_count, net.desc.periodic_u, net.desc.periodic_v, net.points, net.weights, net.knotsU, net.knotsV);
          built.push(surface);
          const _fp2 = pointsToFlat([
              { x: -6.2, y: -3.4, z: -2.0 },
              { x: -3.1, y: -0.2, z: 2.5 },
              { x: -0.5, y: 2.8, z: -1.8 },
              { x: 2.2, y: 1.1, z: 2.2 },
              { x: 4.8, y: -1.6, z: -2.3 },
              { x: 6.1, y: 2.3, z: 1.9 },
            ]);
          const curveHandle = session.interpolate_nurbs_fit_points(_fp2, 3, false);
          built.push(curveHandle);
          const mesh = session.surface_tessellate_to_mesh(surface, new Float64Array([18, 18, 42, 42, 2.5e-4, 0.1]));
          built.push(mesh);
          const intersectionStart = performance.now();
          const inter = session.intersect_surface_curve(surface, curveHandle);
          const intersectionMs = performance.now() - intersectionStart;
          built.push(inter);
          const hits: RgmPoint3[] = [];
          const branchCount = session.intersection_branch_count(inter);
          for (let bi = 0; bi < branchCount; bi += 1) {
            hits.push(...flatToPoints(session.intersection_branch_copy_points(inter, bi)));
          }
          const curveSamples = samplePolyline(session, curveHandle, 3600);
          const buffers = meshToBuffers(session, mesh);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [surface, curveHandle],
            name: "Surface vs Curve",
            degreeLabel: "surface-curve intersection points",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: buffers.vertices,
              indices: buffers.indices,
              color: "#7baed7",
              opacity: 0.32,
              wireframe: false,
              name: "surface",
            },
            overlayMeshes: [],
            overlayCurves: [
              {
                points: curveSamples,
                color: "#f8b36e",
                width: 2.2,
                opacity: 0.98,
                name: "curve",
              },
            ],
            segmentOverlays: [],
            intersectionPoints: hits,
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs,
            logs: [
              `intersection points=${hits.length}`,
              `intersection solve=${intersectionMs.toFixed(2)}ms`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "trimEditWorkflow") {
        const built: AnyHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(14, 12, 10, 9, 0.95);
          const surface = session.create_nurbs_surface(net.desc.degree_u, net.desc.degree_v, net.desc.control_u_count, net.desc.control_v_count, net.desc.periodic_u, net.desc.periodic_v, net.points, net.weights, net.knotsU, net.knotsV);
          built.push(surface);
          const face = session.create_face_from_surface(surface);
          built.push(face);
          session.face_add_loop(face, new Float64Array((rectangleLoopUV(0.05, 0.95, 0.08, 0.92)).flatMap((p: {u:number,v:number}) => [p.u, p.v])), true);
          session.face_add_loop(face, new Float64Array((rectangleLoopUV(0.35, 0.65, 0.35, 0.65)).flatMap((p: {u:number,v:number}) => [p.u, p.v])), false);
          session.face_split_trim_edge(face, 0, 1, 0.42);
          session.face_reverse_loop(face, 1);
          session.face_heal(face);
          const valid = session.face_validate(face);
          const mesh = session.face_tessellate_to_mesh(face, new Float64Array(0));
          built.push(mesh);
          const buffers = meshToBuffers(session, mesh);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [face],
            name: "Trim Edit Workflow",
            degreeLabel: "add loop -> split edge -> reverse loop -> heal",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: buffers.vertices,
              indices: buffers.indices,
              color: "#81b6df",
              opacity: 0.9,
              wireframe: true,
              name: "edited trimmed face",
            },
            overlayMeshes: [],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            logs: [`face valid=${valid}`, `triangles=${session.mesh_triangle_count(mesh)}`],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "trimValidationFailures") {
        const built: AnyHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(12, 10, 9, 8, 0.7);
          const surface = session.create_nurbs_surface(net.desc.degree_u, net.desc.degree_v, net.desc.control_u_count, net.desc.control_v_count, net.desc.periodic_u, net.desc.periodic_v, net.points, net.weights, net.knotsU, net.knotsV);
          built.push(surface);
          const face = session.create_face_from_surface(surface);
          built.push(face);
          session.face_add_loop(face, new Float64Array((rectangleLoopUV(0.1, 0.92, 0.1, 0.9)).flatMap((p: {u:number,v:number}) => [p.u, p.v])), true);
          session.face_add_loop(face, new Float64Array((rectangleLoopUV(0.22, 0.48, 0.22, 0.48)).flatMap((p: {u:number,v:number}) => [p.u, p.v])), true);
          const before = session.face_validate(face);
          session.face_heal(face);
          const after = session.face_validate(face);
          const mesh = session.face_tessellate_to_mesh(face, new Float64Array(0));
          built.push(mesh);
          const buffers = meshToBuffers(session, mesh);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [face],
            name: "Trim Validation Failure + Heal",
            degreeLabel: "intentionally invalid topology diagnostics",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: buffers.vertices,
              indices: buffers.indices,
              color: "#88bade",
              opacity: 0.86,
              wireframe: true,
              name: "validation face",
            },
            overlayMeshes: [],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            logs: [
              `valid before heal=${before}`,
              `valid after heal=${after}`,
              "Two outer loops were added intentionally to trigger a validation failure.",
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "trimMultiLoopSurgery") {
        const built: AnyHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(18, 14, 12, 10, 1.05);
          const surface = session.create_nurbs_surface(net.desc.degree_u, net.desc.degree_v, net.desc.control_u_count, net.desc.control_v_count, net.desc.periodic_u, net.desc.periodic_v, net.points, net.weights, net.knotsU, net.knotsV);
          built.push(surface);
          const face = session.create_face_from_surface(surface);
          built.push(face);

          session.face_add_loop(face, new Float64Array([
              { u: 0.06, v: 0.08 },
              { u: 0.90, v: 0.07 },
              { u: 0.95, v: 0.32 },
              { u: 0.88, v: 0.78 },
              { u: 0.58, v: 0.94 },
              { u: 0.20, v: 0.88 },
              { u: 0.05, v: 0.54 },
            ].flatMap((p: {u:number,v:number}) => [p.u, p.v])), true);

          session.face_add_loop(face, new Float64Array((rectangleLoopUV(0.20, 0.42, 0.24, 0.52)).flatMap((p: {u:number,v:number}) => [p.u, p.v])), false);

          const edgeLoopPoints: RgmUv2[] = [
            { u: 0.62, v: 0.30 },
            { u: 0.76, v: 0.36 },
            { u: 0.78, v: 0.52 },
            { u: 0.66, v: 0.63 },
            { u: 0.52, v: 0.58 },
            { u: 0.50, v: 0.40 },
          ];
          const edgeLoopInput: RgmTrimLoopInput = {
            edge_count: edgeLoopPoints.length,
            is_outer: false,
          };
          const edgeLoopEdges: RgmTrimEdgeInput[] = edgeLoopPoints.map((start, idx) => {
            const end = edgeLoopPoints[(idx + 1) % edgeLoopPoints.length];
            return {
              start_uv: start,
              end_uv: end,
              curve_3d: 0,
              has_curve_3d: false,
            };
          });
          session.face_add_loop_edges(face, edgeLoopInput.is_outer, new Float64Array(edgeLoopEdges.flatMap(e => [e.start_uv.u, e.start_uv.v, e.end_uv.u, e.end_uv.v, e.has_curve_3d ? Number(e.curve_3d) : 0.0, e.has_curve_3d ? 1.0 : 0.0])));

          session.face_split_trim_edge(face, 0, 2, 0.41);
          session.face_split_trim_edge(face, 2, 4, 0.57);
          session.face_reverse_loop(face, 1);

          const validBefore = session.face_validate(face);
          session.face_heal(face);
          const validAfter = session.face_validate(face);

          const mesh = session.face_tessellate_to_mesh(face, new Float64Array([20, 20, 46, 46, 2.0e-4, 0.08]));
          built.push(mesh);
          const buffers = meshToBuffers(session, mesh);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [face],
            name: "Trim Multi-Loop Surgery",
            degreeLabel: "mixed loop APIs + split/reverse/heal on one face",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: buffers.vertices,
              indices: buffers.indices,
              color: "#93bde6",
              opacity: 0.88,
              wireframe: true,
              name: "multi-loop trimmed face",
            },
            overlayMeshes: [],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            logs: [
              `loops added=3 (1 outer + 2 inner)`,
              `split ops=2 reverse ops=1`,
              `valid before heal=${validBefore}`,
              `valid after heal=${validAfter}`,
              `triangles=${session.mesh_triangle_count(mesh)}`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "bboxBrepSolidLifecycle") {
        const built: AnyHandle[] = [];
        try {
          const surfaces = buildSkewedBoxSurfaces(
            session,
            { x: 0.2, y: -0.1, z: 0.0 },
            { x: 4.6, y: 3.2, z: 2.4 },
            0.07,
          );
          built.push(...surfaces);

          const brep = session.brep_create_empty();
          built.push(brep);
          const faceIds: BrepFaceId[] = [];
          for (const surface of surfaces) {
            const faceId = session.brep_add_face_from_surface(brep, surface);
            faceIds.push(faceId);
            session.brep_add_loop_uv(brep, faceId, new Float64Array((rectangleLoopUV(0.0, 1.0, 0.0, 1.0)).flatMap((p: {u:number,v:number}) => [p.u, p.v])), true)
          }

          const preStart = performance.now();
          const preFast = session.compute_bounds(brep.object_id(), 0, 0, 0.0);
          const preMs = performance.now() - preStart;

          const reportBefore = session.brep_validate(brep);
          const fixed = session.brep_heal(brep);
          const shellId = session.brep_finalize_shell(brep);
          const solidId = session.brep_finalize_solid(brep);
          const reportAfter = session.brep_validate(brep);

          const postFastStart = performance.now();
          const postFast = session.compute_bounds(brep.object_id(), 0, 0, 0.0);
          const postFastMs = performance.now() - postFastStart;
          const postOptimalStart = performance.now();
          const postOptimal = session.compute_bounds(brep.object_id(), 1, 6144, 0.0);
          const postOptimalMs = performance.now() - postOptimalStart;

          const mesh = session.brep_tessellate_to_mesh(brep, new Float64Array([12, 12, 34, 34, 1.6e-4, 0.08]));
          built.push(mesh);
          const buffers = meshToBuffers(session, mesh);
          const preExt = obbExtents(preFast);
          const postFastExt = obbExtents(postFast);
          const postOptimalExt = obbExtents(postOptimal);

          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [brep],
            name: "Bounds BREP: Solid Lifecycle",
            degreeLabel: "pre-shell vs post-solid bounds, Fast/Optimal comparison",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: buffers.vertices,
              indices: buffers.indices,
              color: "#7cb2dc",
              opacity: 0.58,
              wireframe: false,
              name: "brep lifecycle solid",
            },
            overlayMeshes: [],
            overlayCurves: [],
            segmentOverlays: boundsOverlaySegments(postOptimal),
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            boundsMs: preMs + postFastMs + postOptimalMs,
            logs: [
              `pre-finalize fast=${preMs.toFixed(2)}ms obb_extents=${formatExtents(preExt)} volume=${extentsVolume(preExt).toFixed(3)}`,
              `post-finalize fast=${postFastMs.toFixed(2)}ms obb_extents=${formatExtents(postFastExt)} volume=${extentsVolume(postFastExt).toFixed(3)}`,
              `post-finalize optimal=${postOptimalMs.toFixed(2)}ms obb_extents=${formatExtents(postOptimalExt)} volume=${extentsVolume(postOptimalExt).toFixed(3)}`,
              `world_aabb_extents=${formatExtents(aabbExtentsFromBounds3(postOptimal))}`,
              `shell_id=${shellId} solid_id=${solidId} face_count=${faceIds.length} heal_fixed=${fixed}`,
              ...validationReportLogLines(reportBefore, "validate(before finalize)"),
              ...validationReportLogLines(reportAfter, "validate(after finalize)"),
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "brepShellAssembly") {
        const built: AnyHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(20, 14, 14, 10, 0.85);
          const surface = session.create_nurbs_surface(net.desc.degree_u, net.desc.degree_v, net.desc.control_u_count, net.desc.control_v_count, net.desc.periodic_u, net.desc.periodic_v, net.points, net.weights, net.knotsU, net.knotsV);
          built.push(surface);

          const leftFace = session.create_face_from_surface(surface);
          const rightFace = session.create_face_from_surface(surface);
          built.push(leftFace, rightFace);

          session.face_add_loop(leftFace, new Float64Array((rectangleLoopUV(0.02, 0.52, 0.06, 0.94)).flatMap((p: {u:number,v:number}) => [p.u, p.v])), true);
          session.face_add_loop(rightFace, new Float64Array((rectangleLoopUV(0.48, 0.98, 0.06, 0.94)).flatMap((p: {u:number,v:number}) => [p.u, p.v])), true);

          const brep = session.brep_create_empty();
          built.push(brep);
          const leftFaceId = session.brep_add_face(brep, leftFace);
          const rightFaceId = session.brep_add_face(brep, rightFace);

          session.brep_add_loop_uv(brep, leftFaceId, new Float64Array(([
              { u: 0.14, v: 0.18 },
              { u: 0.34, v: 0.2 },
              { u: 0.36, v: 0.42 },
              { u: 0.22, v: 0.58 },
              { u: 0.1, v: 0.46 },
            ]).flatMap((p: {u:number,v:number}) => [p.u, p.v])), false)

          const before = session.brep_validate(brep);
          const fixed = session.brep_heal(brep);
          const shellId = session.brep_finalize_shell(brep);
          const after = session.brep_validate(brep);
          const state = session.brep_state(brep);
          const faceCount = session.brep_face_count(brep);
          const leftAdj = session.brep_face_adjacency(brep, leftFaceId);
          const rightAdj = session.brep_face_adjacency(brep, rightFaceId);
          const area = session.brep_estimate_area(brep);

          const brepMesh = session.brep_tessellate_to_mesh(brep, new Float64Array([18, 18, 50, 44, 1.8e-4, 0.08]));
          built.push(brepMesh);
          const brepBuffers = meshToBuffers(session, brepMesh);

          const leftExtractedFace = session.brep_extract_face_object(brep, leftFaceId);
          built.push(leftExtractedFace);
          const leftMesh = session.face_tessellate_to_mesh(leftExtractedFace, new Float64Array(0));
          built.push(leftMesh);
          const leftBuffers = meshToBuffers(session, leftMesh);

          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [brep],
            name: "BREP Shell Assembly + Adjacency",
            degreeLabel: "face->brep assembly, loop edit, validate/heal/finalize, adjacency query",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: brepBuffers.vertices,
              indices: brepBuffers.indices,
              color: "#7db2db",
              opacity: 0.54,
              wireframe: false,
              name: "assembled brep shell",
            },
            overlayMeshes: [
              {
                vertices: leftBuffers.vertices,
                indices: leftBuffers.indices,
                color: "#f3b36f",
                opacity: 0.14,
                wireframe: false,
                name: "extracted left face",
              },
            ],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            logs: [
              `shell_id=${shellId} state=${state} face_count=${faceCount}`,
              `heal fixed_count=${fixed} area_estimate=${area.toFixed(5)}`,
              `left adjacency=[${leftAdj.join(", ")}] right adjacency=[${rightAdj.join(", ")}]`,
              ...validationReportLogLines(before, "validate(before)"),
              ...validationReportLogLines(after, "validate(after)"),
              `triangles=${session.mesh_triangle_count(brepMesh)}`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "brepSolidAssembly") {
        const built: AnyHandle[] = [];
        try {
          const surfaces = buildSkewedBoxSurfaces(
            session,
            { x: 0.0, y: 0.0, z: 0.0 },
            { x: 4.0, y: 2.8, z: 2.0 },
            0.0,
          );
          built.push(...surfaces);

          const brep = session.brep_create_empty();
          built.push(brep);

          const faceIds: BrepFaceId[] = [];
          for (const surface of surfaces) {
            const faceId = session.brep_add_face_from_surface(brep, surface);
            faceIds.push(faceId);
            session.brep_add_loop_uv(brep, faceId, new Float64Array(([
                { u: 0.0, v: 0.0 },
                { u: 1.0, v: 0.0 },
                { u: 1.0, v: 1.0 },
                { u: 0.0, v: 1.0 },
              ]).flatMap((p: {u:number,v:number}) => [p.u, p.v])), true)
          }

          const reportBefore = session.brep_validate(brep);
          const fixed = session.brep_heal(brep);
          const shellId = session.brep_finalize_shell(brep);
          const solidId = session.brep_finalize_solid(brep);
          const reportAfter = session.brep_validate(brep);

          const shellCount = session.brep_shell_count(brep);
          const solidCount = session.brep_solid_count(brep);
          const isSolid = session.brep_is_solid(brep);
          const area = session.brep_estimate_area(brep);
          const state = session.brep_state(brep);
          const adjacency = session.brep_face_adjacency(brep, faceIds[0]);

          const mesh = session.brep_tessellate_to_mesh(brep, new Float64Array([10, 10, 30, 30, 1.5e-4, 0.08]));
          built.push(mesh);
          const buffers = meshToBuffers(session, mesh);

          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [brep],
            name: "BREP Solid Assembly Lifecycle",
            degreeLabel: "6 surfaces -> shell -> solid container + topology diagnostics",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: buffers.vertices,
              indices: buffers.indices,
              color: "#7bb1da",
              opacity: 0.56,
              wireframe: false,
              name: "solid brep assembly",
            },
            overlayMeshes: [],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            logs: [
              `shell_id=${shellId} solid_id=${solidId} state=${state}`,
              `shell_count=${shellCount} solid_count=${solidCount} is_solid=${isSolid}`,
              `face_count=${faceIds.length} adjacency(face0)=[${adjacency.join(", ")}]`,
              `heal fixed_count=${fixed} area_estimate=${area.toFixed(5)}`,
              ...validationReportLogLines(reportBefore, "validate(before)"),
              ...validationReportLogLines(reportAfter, "validate(after)"),
              `triangles=${session.mesh_triangle_count(mesh)}`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "brepSolidRoundtripAudit") {
        const built: AnyHandle[] = [];
        try {
          const surfaces = buildSkewedBoxSurfaces(
            session,
            { x: 0.4, y: -0.2, z: 0.1 },
            { x: 5.2, y: 3.0, z: 2.6 },
            0.08,
          );
          built.push(...surfaces);

          const source = session.brep_create_empty();
          built.push(source);
          const sourceFaceIds: BrepFaceId[] = [];
          for (const surface of surfaces) {
            const faceId = session.brep_add_face_from_surface(source, surface);
            sourceFaceIds.push(faceId);
            session.brep_add_loop_uv(source, faceId, new Float64Array((rectangleLoopUV(0.0, 1.0, 0.0, 1.0)).flatMap((p: {u:number,v:number}) => [p.u, p.v])), true);
          }

          const reportBefore = session.brep_validate(source);
          const fixed = session.brep_heal(source);
          const shellId = session.brep_finalize_shell(source);
          const solidId = session.brep_finalize_solid(source);
          const reportAfter = session.brep_validate(source);
          const sourceArea = session.brep_estimate_area(source);

          const clone = session.brep_clone(source);
          built.push(clone);
          const cloneArea = session.brep_estimate_area(clone);
          const bytes = session.brep_save_native(clone);
          const loaded = session.brep_load_native(bytes);
          built.push(loaded);
          const loadedArea = session.brep_estimate_area(loaded);
          const loadedState = session.brep_state(loaded);
          const loadedSolid = session.brep_is_solid(loaded);
          const loadedShellCount = session.brep_shell_count(loaded);
          const loadedSolidCount = session.brep_solid_count(loaded);
          const loadedFaceCount = session.brep_face_count(loaded);
          const loadedReport = session.brep_validate(loaded);

          const adjacencySignature = sourceFaceIds
            .slice(0, 3)
            .map((faceId) => `f${faceId}:[${session.brep_face_adjacency(loaded, faceId).join(",")}]`)
            .join(" ");

          const loadedMesh = session.brep_tessellate_to_mesh(loaded, new Float64Array([10, 10, 28, 28, 1.5e-4, 0.08]));
          built.push(loadedMesh);
          const loadedBuffers = meshToBuffers(session, loadedMesh);

          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [loaded],
            name: "BREP Solid Roundtrip Audit",
            degreeLabel: "solid -> clone -> native bytes -> load, then invariant checks",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: loadedBuffers.vertices,
              indices: loadedBuffers.indices,
              color: "#7cb3dd",
              opacity: 0.56,
              wireframe: false,
              name: "loaded solid",
            },
            overlayMeshes: [],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            logs: [
              `shell_id=${shellId} solid_id=${solidId} face_count=${sourceFaceIds.length}`,
              `heal fixed_count=${fixed} serialized_bytes=${bytes.length}`,
              `loaded state=${loadedState} is_solid=${loadedSolid} shells=${loadedShellCount} solids=${loadedSolidCount} faces=${loadedFaceCount}`,
              `area source=${sourceArea.toFixed(6)} clone=${cloneArea.toFixed(6)} loaded=${loadedArea.toFixed(6)} delta=${Math.abs(sourceArea - loadedArea).toExponential(2)}`,
              `adjacency signature: ${adjacencySignature}`,
              ...validationReportLogLines(reportBefore, "source validate(before)"),
              ...validationReportLogLines(reportAfter, "source validate(after)"),
              ...validationReportLogLines(loadedReport, "loaded validate"),
              `loaded triangles=${session.mesh_triangle_count(loadedMesh)}`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "brepSolidFaceSurgery") {
        const built: AnyHandle[] = [];
        try {
          const surfaces = buildSkewedBoxSurfaces(
            session,
            { x: -0.6, y: 0.3, z: 0.0 },
            { x: 4.6, y: 3.4, z: 2.2 },
            0.03,
          );
          built.push(...surfaces);

          const original = session.brep_create_empty();
          built.push(original);
          for (const surface of surfaces) {
            const faceId = session.brep_add_face_from_surface(original, surface);
            session.brep_add_loop_uv(original, faceId, new Float64Array((rectangleLoopUV(0.0, 1.0, 0.0, 1.0)).flatMap((p: {u:number,v:number}) => [p.u, p.v])), true);
          }
          session.brep_finalize_shell(original);
          session.brep_finalize_solid(original);

          const extractedFaces: FaceHandle[] = [];
          const originalFaceCount = session.brep_face_count(original);
          for (let idx = 0; idx < originalFaceCount; idx += 1) {
            const face = session.brep_extract_face_object(original, idx as unknown as BrepFaceId);
            extractedFaces.push(face);
            built.push(face);
          }

          const surgeryFace = extractedFaces[0];
          const surgeryValidBefore = session.face_validate(surgeryFace);
          session.face_add_loop(surgeryFace, new Float64Array([
              { u: 0.22, v: 0.18 },
              { u: 0.46, v: 0.2 },
              { u: 0.5, v: 0.42 },
              { u: 0.3, v: 0.54 },
              { u: 0.18, v: 0.4 },
            ].flatMap((p: {u:number,v:number}) => [p.u, p.v])), false);
          session.face_split_trim_edge(surgeryFace, 1, 1, 0.53);
          session.face_reverse_loop(surgeryFace, 1);
          session.face_heal(surgeryFace);
          const surgeryValidAfter = session.face_validate(surgeryFace);

          const rebuilt = session.brep_create_from_faces(new Float64Array((extractedFaces).map((f: FaceHandle) => f.object_id())));
          built.push(rebuilt);
          const rebuiltReportBefore = session.brep_validate(rebuilt);
          session.brep_finalize_shell(rebuilt);
          const rebuiltSolidId = session.brep_finalize_solid(rebuilt);
          const rebuiltReportAfter = session.brep_validate(rebuilt);
          const rebuiltIsSolid = session.brep_is_solid(rebuilt);
          const rebuiltSolidCount = session.brep_solid_count(rebuilt);
          const rebuiltArea = session.brep_estimate_area(rebuilt);
          const originalArea = session.brep_estimate_area(original);

          const originalMesh = session.brep_tessellate_to_mesh(original, new Float64Array([10, 10, 30, 30, 1.6e-4, 0.08]));
          built.push(originalMesh);
          const originalBuffers = meshToBuffers(session, originalMesh);

          const rebuiltMesh = session.brep_tessellate_to_mesh(rebuilt, new Float64Array([10, 10, 30, 30, 1.6e-4, 0.08]));
          built.push(rebuiltMesh);
          const rebuiltBuffers = meshToBuffers(session, rebuiltMesh);

          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [original, rebuilt],
            name: "BREP Solid Face Surgery Rebuild",
            degreeLabel: "extract faces, edit one face trim topology, then rebuild a second solid",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: rebuiltBuffers.vertices,
              indices: rebuiltBuffers.indices,
              color: "#80b6de",
              opacity: 0.58,
              wireframe: false,
              name: "rebuilt surgical solid",
            },
            overlayMeshes: [
              {
                vertices: originalBuffers.vertices,
                indices: originalBuffers.indices,
                color: "#f2b271",
                opacity: 0.1,
                wireframe: false,
                name: "original reference solid",
              },
            ],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            logs: [
              `original faces extracted=${originalFaceCount} rebuilt_solid_id=${rebuiltSolidId}`,
              `surgery face valid before=${surgeryValidBefore} after=${surgeryValidAfter}`,
              `rebuilt is_solid=${rebuiltIsSolid} solid_count=${rebuiltSolidCount}`,
              `area original=${originalArea.toFixed(6)} rebuilt=${rebuiltArea.toFixed(6)} delta=${Math.abs(originalArea - rebuiltArea).toExponential(2)}`,
              ...validationReportLogLines(rebuiltReportBefore, "rebuilt validate(before finalize)"),
              ...validationReportLogLines(rebuiltReportAfter, "rebuilt validate(after finalize)"),
              `rebuilt triangles=${session.mesh_triangle_count(rebuiltMesh)}`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "brepFaceBridgeRoundtrip") {
        const built: AnyHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(18, 16, 12, 11, 1.1);
          const surface = session.create_nurbs_surface(net.desc.degree_u, net.desc.degree_v, net.desc.control_u_count, net.desc.control_v_count, net.desc.periodic_u, net.desc.periodic_v, net.points, net.weights, net.knotsU, net.knotsV);
          built.push(surface);

          const sourceFace = session.create_face_from_surface(surface);
          built.push(sourceFace);
          session.face_add_loop(sourceFace, new Float64Array([
              { u: 0.06, v: 0.08 },
              { u: 0.88, v: 0.06 },
              { u: 0.94, v: 0.34 },
              { u: 0.88, v: 0.9 },
              { u: 0.22, v: 0.94 },
              { u: 0.06, v: 0.6 },
            ].flatMap((p: {u:number,v:number}) => [p.u, p.v])), true);
          session.face_add_loop(sourceFace, new Float64Array((rectangleLoopUV(0.24, 0.46, 0.22, 0.5)).flatMap((p: {u:number,v:number}) => [p.u, p.v])), false);
          session.face_split_trim_edge(sourceFace, 0, 2, 0.44);
          session.face_reverse_loop(sourceFace, 1);
          session.face_heal(sourceFace);

          const sourceValid = session.face_validate(sourceFace);
          const sourceMesh = session.face_tessellate_to_mesh(sourceFace, new Float64Array([18, 18, 46, 42, 2.0e-4, 0.08]));
          built.push(sourceMesh);
          const sourceBuffers = meshToBuffers(session, sourceMesh);

          const brep = session.brep_from_face_object(sourceFace);
          built.push(brep);
          const cloned = session.brep_clone(brep);
          built.push(cloned);
          const report = session.brep_validate(cloned);
          const clonedArea = session.brep_estimate_area(cloned);
          const clonedFaceCount = session.brep_face_count(cloned);
          const rootFaceId = 0 as BrepFaceId;
          const clonedAdj = session.brep_face_adjacency(cloned, rootFaceId);

          const extractedFace = session.brep_extract_face_object(cloned, rootFaceId);
          built.push(extractedFace);
          const extractedValid = session.face_validate(extractedFace);
          const extractedMesh = session.face_tessellate_to_mesh(extractedFace, new Float64Array([18, 18, 46, 42, 2.0e-4, 0.08]));
          built.push(extractedMesh);
          const extractedBuffers = meshToBuffers(session, extractedMesh);

          const brepMesh = session.brep_tessellate_to_mesh(cloned, new Float64Array(0));
          built.push(brepMesh);
          const brepBuffers = meshToBuffers(session, brepMesh);

          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [cloned],
            name: "BREP Face Bridge Roundtrip",
            degreeLabel: "face -> brep -> clone -> extract face, then compare tessellations",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: brepBuffers.vertices,
              indices: brepBuffers.indices,
              color: "#79afd9",
              opacity: 0.56,
              wireframe: false,
              name: "cloned brep mesh",
            },
            overlayMeshes: [
              {
                vertices: sourceBuffers.vertices,
                indices: sourceBuffers.indices,
                color: "#f4b472",
                opacity: 0.1,
                wireframe: false,
                name: "source face mesh",
              },
              {
                vertices: extractedBuffers.vertices,
                indices: extractedBuffers.indices,
                color: "#8fe4b8",
                opacity: 0.1,
                wireframe: false,
                name: "extracted face mesh",
              },
            ],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            logs: [
              `source face valid=${sourceValid} extracted valid=${extractedValid}`,
              `clone face_count=${clonedFaceCount} adjacency(face0)=[${clonedAdj.join(", ")}]`,
              `clone area_estimate=${clonedArea.toFixed(5)}`,
              ...validationReportLogLines(report, "clone validate"),
              `source triangles=${session.mesh_triangle_count(sourceMesh)}`,
              `extracted triangles=${session.mesh_triangle_count(extractedMesh)}`,
              `brep triangles=${session.mesh_triangle_count(brepMesh)}`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example === "brepNativeRoundtrip") {
        const built: AnyHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(17, 13, 11, 9, 0.9);
          const surface = session.create_nurbs_surface(net.desc.degree_u, net.desc.degree_v, net.desc.control_u_count, net.desc.control_v_count, net.desc.periodic_u, net.desc.periodic_v, net.points, net.weights, net.knotsU, net.knotsV);
          built.push(surface);

          const brep = session.brep_create_empty();
          built.push(brep);
          const faceId = session.brep_add_face_from_surface(brep, surface);
          session.brep_add_loop_uv(brep, faceId, new Float64Array(([
              { u: 0.06, v: 0.09 },
              { u: 0.92, v: 0.08 },
              { u: 0.95, v: 0.91 },
              { u: 0.08, v: 0.94 },
            ]).flatMap((p: {u:number,v:number}) => [p.u, p.v])), true)
          session.brep_add_loop_uv(brep, faceId, new Float64Array((rectangleLoopUV(0.2, 0.38, 0.22, 0.46)).flatMap((p: {u:number,v:number}) => [p.u, p.v])), false);
          session.brep_add_loop_uv(brep, faceId, new Float64Array(([
              { u: 0.6, v: 0.3 },
              { u: 0.76, v: 0.34 },
              { u: 0.72, v: 0.56 },
              { u: 0.54, v: 0.5 },
            ]).flatMap((p: {u:number,v:number}) => [p.u, p.v])), false)
          const shellId = session.brep_finalize_shell(brep);
          const reportBefore = session.brep_validate(brep);
          const bytes = session.brep_save_native(brep);
          const areaBefore = session.brep_estimate_area(brep);

          const loaded = session.brep_load_native(bytes);
          built.push(loaded);
          const reportAfter = session.brep_validate(loaded);
          const areaAfter = session.brep_estimate_area(loaded);
          const loadedState = session.brep_state(loaded);
          const loadedFaceCount = session.brep_face_count(loaded);

          const loadedMesh = session.brep_tessellate_to_mesh(loaded, new Float64Array([16, 16, 42, 36, 2.2e-4, 0.085]));
          built.push(loadedMesh);
          const loadedBuffers = meshToBuffers(session, loadedMesh);

          const originalMesh = session.brep_tessellate_to_mesh(brep, new Float64Array(0));
          built.push(originalMesh);
          const originalBuffers = meshToBuffers(session, originalMesh);

          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
            exportHandles: [brep, loaded],
            name: "BREP Native Save/Load Roundtrip",
            degreeLabel: "finalized brep -> bytes -> loaded brep, then validate and compare",
            renderDegree: 0,
            renderSamples: 0,
            meshVisual: {
              vertices: loadedBuffers.vertices,
              indices: loadedBuffers.indices,
              color: "#7eb5dc",
              opacity: 0.56,
              wireframe: false,
              name: "loaded brep",
            },
            overlayMeshes: [
              {
                vertices: originalBuffers.vertices,
                indices: originalBuffers.indices,
                color: "#f2b472",
                opacity: 0.1,
                wireframe: false,
                name: "original brep",
              },
            ],
            overlayCurves: [],
            segmentOverlays: [],
            intersectionPoints: [],
            planeVisual: null,
            interactiveMeshHandle: null,
            transformTargets: [],
            defaultTransformTargetKey: null,
            intersectionMs: 0,
            logs: [
              `shell_id=${shellId} serialized_bytes=${bytes.length}`,
              `loaded state=${loadedState} face_count=${loadedFaceCount}`,
              `area before=${areaBefore.toFixed(6)} area after=${areaAfter.toFixed(6)} delta=${Math.abs(areaBefore - areaAfter).toExponential(2)}`,
              ...validationReportLogLines(reportBefore, "original validate"),
              ...validationReportLogLines(reportAfter, "loaded validate"),
              `loaded triangles=${session.mesh_triangle_count(loadedMesh)}`,
            ],
          };
        } catch (error) {
          throw error;
        }
      }

      if (example !== "polycurve") {
        throw new Error(`Unsupported example value: ${example}`);
      }

      const lineA: RgmLine3 = {
        start: { x: -6.2, y: -2.1, z: 0.3 },
        end: { x: -2.0, y: 1.0, z: 1.1 },
      };
      const arcA: RgmArc3 = {
        plane: {
          origin: { x: -4.0, y: 1.0, z: 1.1 },
          x_axis: { x: 1.0, y: 0.0, z: 0.0 },
          y_axis: { x: 0.0, y: 1.0, z: 0.0 },
          z_axis: { x: 0.0, y: 0.0, z: 1.0 },
        },
        radius: 2.0,
        start_angle: 0.0,
        sweep_angle: Math.PI / 2,
      };
      const lineB: RgmLine3 = {
        start: { x: -4.0, y: 3.0, z: 1.1 },
        end: { x: 0.7, y: 1.8, z: -1.6 },
      };
      const arcB: RgmArc3 = {
        plane: {
          origin: { x: -0.38, y: 0.36, z: -1.6 },
          x_axis: { x: 0.6, y: 0.8, z: 0.0 },
          y_axis: { x: 0.0, y: 0.0, z: 1.0 },
          z_axis: { x: 0.8, y: -0.6, z: 0.0 },
        },
        radius: 1.8,
        start_angle: 0.0,
        sweep_angle: -1.2,
      };

      const builtHandles: AnyHandle[] = [];
      try {
        const hLineA = session.create_line(lineA.start.x, lineA.start.y, lineA.start.z, lineA.end.x, lineA.end.y, lineA.end.z);
        builtHandles.push(hLineA);
        const hArcA = session.create_arc(arcA.plane.origin.x, arcA.plane.origin.y, arcA.plane.origin.z, arcA.plane.x_axis.x, arcA.plane.x_axis.y, arcA.plane.x_axis.z, arcA.plane.y_axis.x, arcA.plane.y_axis.y, arcA.plane.y_axis.z, arcA.plane.z_axis.x, arcA.plane.z_axis.y, arcA.plane.z_axis.z, arcA.radius, arcA.start_angle, arcA.sweep_angle);
        builtHandles.push(hArcA);
        const hLineB = session.create_line(lineB.start.x, lineB.start.y, lineB.start.z, lineB.end.x, lineB.end.y, lineB.end.z);
        builtHandles.push(hLineB);
        const hArcB = session.create_arc(arcB.plane.origin.x, arcB.plane.origin.y, arcB.plane.origin.z, arcB.plane.x_axis.x, arcB.plane.x_axis.y, arcB.plane.x_axis.z, arcB.plane.y_axis.x, arcB.plane.y_axis.y, arcB.plane.y_axis.z, arcB.plane.z_axis.x, arcB.plane.z_axis.y, arcB.plane.z_axis.z, arcB.radius, arcB.start_angle, arcB.sweep_angle);
        builtHandles.push(hArcB);

        const segments: { curve: CurveHandle; reversed: boolean }[] = [
          { curve: hLineA, reversed: false },
          { curve: hArcA, reversed: false },
          { curve: hLineB, reversed: false },
          { curve: hArcB, reversed: false },
        ];
        const poly = session.create_polycurve(new Float64Array(segments.reduce<number[]>((acc, s, _i) => { acc.push(s.curve.object_id()); acc.push(s.reversed ? 1.0 : 0.0); return acc; }, [])));
        builtHandles.unshift(poly);

        return asCurve(
          poly,
          builtHandles,
          "Mixed Polycurve Ribbon",
          "Polycurve (line+arc+line+arc)",
          3,
          2800,
          [`Polycurve segments=${segments.length}`],
        );
      } catch (error) {
        throw error;
      }
    },
    [],
  );

  const updateCurveForExample = useCallback(
    (example: ExampleKey, successMessage: string, nurbsPresetOverride?: CurvePreset): void => {
      const session = sessionRef.current;
      if (!session) {
        throw new Error("Kernel session is not ready");
      }

      appendLog("info", `Building ${example} example`);
      releaseOwnedCurveHandles();

      const loadStart = performance.now();
      const built = buildExampleCurve(session, example, nurbsPresetOverride);
      curveHandleRef.current = built.curveHandle;
      ownedCurveHandlesRef.current = built.ownedHandles;
      surfaceProbeHandleRef.current = example === "surfaceUvEval" ? (built.surfaceProbeHandle ?? null) : null;

      activeHandlesRef.current = built.ownedHandles
        .filter(h => typeof h.object_id === "function")
        .map(h => ({ objectId: h.object_id() }));

      const exportSrc = built.exportHandles ?? built.ownedHandles;
      exportHandlesRef.current = exportSrc
        .filter(h => typeof h.object_id === "function")
        .map(h => ({ objectId: h.object_id() }));
      surfaceProbeD1ScaleRef.current = built.surfaceProbeD1Scale ?? 0.2;
      surfaceProbeD2ScaleRef.current = built.surfaceProbeD2Scale ?? 0.1;

      for (const line of built.logs) {
        appendLog("debug", line);
      }

      let curveSamples: RgmPoint3[] = [];
      let totalLength = 0;
      if (built.kind === "curve" && built.curveHandle !== null) {
        curveSamples = samplePolyline(session, built.curveHandle, built.renderSamples);
        totalLength = session.curve_length(built.curveHandle);
        totalLengthRef.current = totalLength;
        const _epArr = session.curve_point_at(built.curveHandle, probeTNormRef.current);
        const evaluatedProbe = { x: _epArr[0], y: _epArr[1], z: _epArr[2] };
        const probeLength = session.curve_length_at(built.curveHandle, probeTNormRef.current);

        probePointRef.current = evaluatedProbe;
        if (probeRef.current) {
          probeRef.current.position.set(evaluatedProbe.x, evaluatedProbe.y, evaluatedProbe.z);
          probeRef.current.scale.setScalar(1);
          probeRef.current.visible = shouldShowProbeForExample(example);
        }
        setProbeUiState({
          tNorm: probeTNormRef.current,
          x: evaluatedProbe.x,
          y: evaluatedProbe.y,
          z: evaluatedProbe.z,
          probeLength,
          totalLength,
        });
      } else {
        totalLengthRef.current = 0;
        probePointRef.current = null;
        if (probeRef.current) {
          probeRef.current.visible = false;
        }
        setProbeUiState((previous) => ({
          ...previous,
          x: 0,
          y: 0,
          z: 0,
          probeLength: 0,
          totalLength: 0,
        }));
      }

      if (nurbsPresetOverride) {
        nurbsPresetRef.current = nurbsPresetOverride;
        setPreset(nurbsPresetOverride);
      }

      if (cameraRef.current) cameraRef.current.up.set(0, 0, 1);
      if (controlsRef.current && cameraRef.current) syncControlsUpAxis(controlsRef.current, cameraRef.current);
      if (gridRef.current) gridRef.current.rotation.x = Math.PI / 2;
      setSceneUpAxis("z");
      landXmlCurveDataRef.current = null;
      landXmlContextRef.current = null;
      setLandXmlAlignments([]);

      setActiveExample(example);
      setActiveCurveName(built.name);
      setActiveDegreeLabel(built.degreeLabel);
      setActiveRenderDegree(built.renderDegree);
      setSampledPoints(curveSamples);
      setMeshVisual(built.meshVisual);
      setOverlayMeshes(built.overlayMeshes);
      setOverlayCurves(built.overlayCurves);
      setSegmentOverlays(built.segmentOverlays);
      setIntersectionPoints(built.intersectionPoints);
      setIntersectionPlane(built.planeVisual);
      interactiveMeshHandleRef.current = built.interactiveMeshHandle;
      meshPlaneMeshHandleRef.current =
        example === "meshIntersectMeshPlane" ? built.interactiveMeshHandle : null;
      meshPlanePlaneRef.current = example === "meshIntersectMeshPlane" ? built.planeVisual : null;
      booleanBaseMeshHandleRef.current = built.booleanState?.baseHandle ?? null;
      booleanToolMeshHandleRef.current = built.booleanState?.toolHandle ?? null;
      booleanResultMeshHandleRef.current = built.booleanState?.resultHandle ?? null;
      transformTargetsRef.current =
        example === "meshTransform" || example === "meshBoolean" ? built.transformTargets : [];
      setTransformTargetsUi(
        example === "meshTransform" || example === "meshBoolean"
          ? built.transformTargets.map((target) => ({ key: target.key, label: target.label }))
          : [],
      );
      setTransformTargetKey(
        example === "meshTransform" || example === "meshBoolean"
          ? (built.defaultTransformTargetKey ?? "")
          : "",
      );
      setMeshPlaneTarget("mesh");
      dragStartTransformRef.current = null;
      if (example === "surfaceUvEval" && surfaceProbeHandleRef.current !== null) {
        updateSurfaceProbeForUvRef.current(
          surfaceProbeUvRef.current.u,
          surfaceProbeUvRef.current.v,
          false,
        );
      }
      const loadMs = performance.now() - loadStart;
      const boundsMs = built.boundsMs ?? 0;
      setPerfStats({ loadMs, intersectionMs: built.intersectionMs, boundsMs });
      const intersectionSummary =
        built.intersectionPoints.length > 0
          ? ` • intersections ${built.intersectionPoints.length}`
          : "";
      const meshSummary =
        built.kind === "mesh" && built.meshVisual
          ? ` • triangles ${Math.floor(built.meshVisual.indices.length / 3)}`
          : "";
      const perfSummary = ` • load ${loadMs.toFixed(2)}ms${
        built.intersectionMs > 0 ? ` • intersection ${built.intersectionMs.toFixed(2)}ms` : ""
      }${boundsMs > 0 ? ` • bounds ${boundsMs.toFixed(2)}ms` : ""}`;
      setStatusMessage(
        `${successMessage} • ${built.name} • ${built.degreeLabel}${intersectionSummary}${meshSummary}${perfSummary}${
          built.kind === "curve"
            ? ` • exact length ${totalLength.toFixed(6)} • render samples ${curveSamples.length}`
            : ""
        }`,
      );
      setErrorMessage(null);
      if (!suppressAutoFitRef.current) {
        window.requestAnimationFrame(() => {
          const camera = cameraRef.current;
          const controls = controlsRef.current;
          if (!camera || !controls) {
            return;
          }
          const session = sessionRef.current;
          const fog = sceneRef.current?.fog as THREE.Fog | null;
          if (session && activeHandlesRef.current.length > 0) {
            const box = computeSceneBounds(session, activeHandlesRef.current);
            if (!box.isEmpty()) {
              zoomToFit(camera, controls, box, fog);
              return;
            }
          }
          const points =
            curveSamples.length > 0
              ? curveSamples
              : [
                  ...(built.meshVisual?.vertices ?? []),
                  ...built.overlayMeshes.flatMap((visual) => visual.vertices),
                ];
          if (points.length > 0) {
            const box = new THREE.Box3();
            for (const p of points) box.expandByPoint(new THREE.Vector3(p.x, p.y, p.z));
            zoomToFit(camera, controls, box, fog);
          }
        });
      }
      appendLog(
        "info",
        `Built handles=${built.ownedHandles.length} intersections=${built.intersectionPoints.length} kind=${built.kind} load=${loadMs.toFixed(2)}ms bounds=${boundsMs.toFixed(2)}ms`,
      );
    },
    [appendLog, buildExampleCurve, releaseOwnedCurveHandles],
  );

  const updateLandXmlFile = useCallback(
    async (filename: string): Promise<void> => {
      const session = sessionRef.current;
      if (!session) return;

      if (pendingAsyncExampleRef.current) {
        pendingAsyncExampleRef.current.abort();
      }
      const controller = new AbortController();
      pendingAsyncExampleRef.current = controller;

      setKernelStatus("computing");
      appendLog("info", `Loading LandXML: ${filename}`);
      releaseOwnedCurveHandles();

      try {
        const { built, stats, curveData, zRange, defaultDatum, context } = await buildLandXmlExample(session, filename, controller.signal);
        if (controller.signal.aborted) return;

        for (const line of built.logs) {
          appendLog("debug", line);
        }

        landXmlCurveDataRef.current = curveData;
        landXmlContextRef.current = context;
        setLandXmlZRange(zRange);
        setLandXmlDatumOffset(defaultDatum);
        setLandXmlVertExag(1);
        setLandXmlScaleFactor(1);
        setLandXmlAlignments(context.alignments);

        const defaultAlign = 0;
        const defaultProf = 0;
        setLandXmlProbeAlignIdx(defaultAlign);
        setLandXmlProbeProfileIdx(defaultProf);

        setActiveExample("landxmlViewer");
        setActiveCurveName(built.name);
        setActiveDegreeLabel(built.degreeLabel);
        setActiveRenderDegree(built.renderDegree);
        setSampledPoints([]);
        landXmlRawMeshRef.current = { meshVisual: built.meshVisual, overlayMeshes: built.overlayMeshes };
        setMeshVisual(built.meshVisual);
        setOverlayMeshes(built.overlayMeshes);
        setOverlayCurves(built.overlayCurves);
        setSegmentOverlays([]);
        setIntersectionPoints([]);
        setIntersectionPlane(null);
        setLandXmlStats(stats);
        setPerfStats({ loadMs: stats.parseMs, intersectionMs: 0, boundsMs: 0 });

        totalLengthRef.current = 0;
        probePointRef.current = null;
        if (probeRef.current) probeRef.current.visible = false;

        setKernelStatus("ready");
        setStatusMessage(
          `Loaded ${filename} — Surfaces: ${stats.surfCount} · Alignments: ${stats.alignCount} · Features: ${stats.featureLineCount + stats.breaklineCount} · Vertices: ${stats.vertCount.toLocaleString()} · ${stats.parseMs.toFixed(1)}ms`,
        );

        window.requestAnimationFrame(() => {
          const camera = cameraRef.current;
          const controls = controlsRef.current;
          const scene = sceneRef.current;
          if (!camera || !controls) return;

          camera.up.set(0, 0, 1);
          syncControlsUpAxis(controls, camera);
          setSceneUpAxis("z");

          const grid = gridRef.current;
          if (grid) {
            grid.rotation.x = Math.PI / 2;
          }

          const fog = scene?.fog instanceof THREE.Fog ? scene.fog : null;

          // Union ALL geometry (terrain mesh + overlay meshes + curves) for accurate framing
          const allPts: THREE.Vector3[] = [];
          if (built.meshVisual) {
            for (const v of built.meshVisual.vertices) {
              allPts.push(new THREE.Vector3(v.x, v.y, v.z));
            }
          }
          for (const m of built.overlayMeshes) {
            for (const v of m.vertices) {
              allPts.push(new THREE.Vector3(v.x, v.y, v.z));
            }
          }
          for (const c of built.overlayCurves) {
            for (const v of c.points) {
              allPts.push(new THREE.Vector3(v.x, v.y, v.z));
            }
          }
          if (allPts.length > 0) {
            const box = new THREE.Box3();
            for (const p of allPts) box.expandByPoint(p);
            zoomToFit(camera, controls, box, fog);
          }
        });
      } catch (err) {
        if (err instanceof DOMException && err.name === "AbortError") return;
        const msg = err instanceof Error ? err.message : String(err);
        setErrorMessage(msg);
        setKernelStatus("error");
        appendLog("error", `LandXML load failed: ${msg}`);
      } finally {
        if (pendingAsyncExampleRef.current === controller) {
          pendingAsyncExampleRef.current = null;
        }
      }
    },
    [appendLog, releaseOwnedCurveHandles],
  );

  useEffect(() => {
    const data = landXmlCurveDataRef.current;
    if (!data || activeExample !== "landxmlViewer") return;
    setOverlayCurves(applyDatumAndExaggeration(data, landXmlDatumOffset, landXmlVertExag, landXmlScaleFactor));
  }, [landXmlDatumOffset, landXmlVertExag, landXmlScaleFactor, activeExample]);

  useEffect(() => {
    const raw = landXmlRawMeshRef.current;
    if (!raw || activeExample !== "landxmlViewer") return;
    const s = landXmlScaleFactor;
    if (s === 1) {
      setMeshVisual(raw.meshVisual);
      setOverlayMeshes(raw.overlayMeshes);
      return;
    }
    if (raw.meshVisual) {
      setMeshVisual({
        ...raw.meshVisual,
        vertices: raw.meshVisual.vertices.map((v) => ({ x: v.x * s, y: v.y * s, z: v.z * s })),
      });
    }
    setOverlayMeshes(
      raw.overlayMeshes.map((m) => ({
        ...m,
        vertices: m.vertices.map((v) => ({ x: v.x * s, y: v.y * s, z: v.z * s })),
      })),
    );
  }, [landXmlScaleFactor, activeExample]);

  useEffect(() => {
    if (activeExample !== "landxmlViewer") return;
    const ctx = landXmlContextRef.current;
    if (!ctx || ctx.alignments.length === 0) return;
    if (ctx.alignments[landXmlProbeAlignIdx]?.profileCount === 0) return;
    updateLandXmlProbe(
      landXmlProbeUiState.stationNorm,
      landXmlProbeAlignIdx,
      landXmlProbeProfileIdx,
      false,
    );
  }, [landXmlDatumOffset, landXmlVertExag, landXmlAlignments]); // eslint-disable-line react-hooks/exhaustive-deps

  const loadDefaultPreset = useCallback(async (): Promise<CurvePreset> => {
    const response = await fetch("/showcases/default.json");
    if (!response.ok) {
      throw new Error(`Failed to load default preset (${response.status})`);
    }

    const data = await response.json();
    return parseCurvePreset(data);
  }, []);

  const cameraSnapshot = useCallback((): CameraSnapshot | null => {
    const camera = cameraRef.current;
    const controls = controlsRef.current;
    if (!camera || !controls) {
      return null;
    }

    return {
      position: toPoint3(camera.position),
      target: toPoint3(controls.target),
      up: toPoint3(camera.up),
      fov: camera instanceof THREE.PerspectiveCamera ? camera.fov : 46,
      mode: cameraModeRef.current,
    };
  }, []);

  const zoomExtents = useCallback((): void => {
    const camera = cameraRef.current;
    const controls = controlsRef.current;
    const session = sessionRef.current;
    const scene = sceneRef.current;
    if (!camera || !controls) return;

    const fog = scene?.fog as THREE.Fog | null;

    if (session && activeHandlesRef.current.length > 0) {
      const box = computeSceneBounds(session, activeHandlesRef.current);
      if (!box.isEmpty()) {
        zoomToFit(camera, controls, box, fog);
        return;
      }
    }

    // For LandXML, union ALL geometry (terrain mesh + overlay meshes + curves)
    const curvePoints = overlayCurves.flatMap((c) => c.points);
    if (activeExample === "landxmlViewer") {
      const allPts = [
        ...(meshVisual?.vertices ?? []),
        ...overlayMeshes.flatMap((visual) => visual.vertices),
        ...curvePoints,
      ];
      if (allPts.length > 0) {
        const box = new THREE.Box3();
        for (const p of allPts) box.expandByPoint(new THREE.Vector3(p.x, p.y, p.z));
        zoomToFit(camera, controls, box, fog);
        return;
      }
    }

    const allPoints = sampledPoints.length > 0
      ? sampledPoints
      : [
          ...(meshVisual?.vertices ?? []),
          ...overlayMeshes.flatMap((visual) => visual.vertices),
          ...curvePoints,
        ];
    if (allPoints.length === 0) return;
    const box = new THREE.Box3();
    for (const p of allPoints) box.expandByPoint(new THREE.Vector3(p.x, p.y, p.z));
    zoomToFit(camera, controls, box, fog);
  }, [activeExample, meshVisual, overlayMeshes, overlayCurves, sampledPoints]);

  const resetCamera = useCallback((): void => {
    const camera = cameraRef.current;
    const controls = controlsRef.current;
    if (!camera || !controls) {
      return;
    }

    camera.position.copy(DEFAULT_CAMERA_POSITION);
    controls.target.copy(DEFAULT_CAMERA_TARGET);
    camera.up.set(0, 0, 1);
    syncControlsUpAxis(controls, camera);
    if (gridRef.current) gridRef.current.rotation.x = Math.PI / 2;
    controls.update();
  }, []);

  const toggleCameraMode = useCallback((): void => {
    const persp = perspCameraRef.current;
    const ortho = orthoCameraRef.current;
    const controls = controlsRef.current;
    if (!persp || !ortho || !controls) return;

    const nextMode: CameraMode = cameraModeRef.current === "perspective" ? "orthographic" : "perspective";
    cameraModeRef.current = nextMode;
    setCameraMode(nextMode);

    if (nextMode === "orthographic") {
      const dist = persp.position.distanceTo(controls.target);
      const halfH = dist * Math.tan(THREE.MathUtils.degToRad(persp.fov / 2));
      const halfW = halfH * persp.aspect;
      ortho.left = -halfW;
      ortho.right = halfW;
      ortho.top = halfH;
      ortho.bottom = -halfH;
      ortho.near = 0.001;
      ortho.far = dist * 10;
      ortho.position.copy(persp.position);
      ortho.quaternion.copy(persp.quaternion);
      ortho.up.copy(persp.up);
      ortho.updateProjectionMatrix();
      cameraRef.current = ortho;
      (controls as unknown as { object: THREE.Camera }).object = ortho;
      syncControlsUpAxis(controls, ortho);
    } else {
      persp.position.copy(ortho.position);
      persp.quaternion.copy(ortho.quaternion);
      persp.up.copy(ortho.up);

      // Sync near/far from the current ortho setup so the scene doesn't clip
      const dist = ortho.position.distanceTo(controls.target);
      persp.near = Math.max(0.001, dist * 0.01);
      persp.far = Math.max(1200, dist * 10);
      persp.updateProjectionMatrix();
      cameraRef.current = persp;
      (controls as unknown as { object: THREE.Camera }).object = persp;
      syncControlsUpAxis(controls, persp);
    }
    controls.update();
  }, []);

  const applyViewPreset = useCallback((presetName: ViewPresetName): void => {
    const controls = controlsRef.current;
    const session = sessionRef.current;
    const scene = sceneRef.current;
    if (!controls) return;

    if (cameraModeRef.current === "perspective") {
      toggleCameraMode();
    }

    // Re-read camera AFTER potential toggle so we operate on the active (ortho) camera
    const camera = cameraRef.current;
    if (!camera) return;

    const presets = getViewPresets(sceneUpAxis);
    const preset = presets[presetName];

    let box: THREE.Box3 | null = null;
    if (session && activeHandlesRef.current.length > 0) {
      box = computeSceneBounds(session, activeHandlesRef.current);
      if (box.isEmpty()) box = null;
    }

    const center = box ? box.getCenter(new THREE.Vector3()) : controls.target.clone();
    const radius = box ? box.getBoundingSphere(new THREE.Sphere()).radius : 10;
    const distance = Math.max(radius * 2, 4);

    camera.position.copy(center).addScaledVector(preset.dir, -distance);
    camera.up.copy(preset.up);
    syncControlsUpAxis(controls, camera);
    controls.target.copy(center);

    if (box) {
      zoomToFit(camera, controls, box, scene?.fog as THREE.Fog | null);
    } else {
      camera.lookAt(center);
      controls.update();
    }
  }, [sceneUpAxis, toggleCameraMode]);

  const updateSurfaceProbeForUv = useCallback(
    (nextU: number, nextV: number, logCommit: boolean): void => {
      const u = Math.min(1, Math.max(0, nextU));
      const v = Math.min(1, Math.max(0, nextV));
      surfaceProbeUvRef.current = { u, v };

      const liveSession = sessionRef.current;
      const liveSurfaceHandle = surfaceProbeHandleRef.current;
      if (!liveSession || liveSurfaceHandle === null) {
        setSurfaceProbeUiState((previous) => ({ ...previous, u, v }));
        return;
      }

      try {
        const _ptArr = liveSession.surface_point_at(liveSurfaceHandle, u, v);
        const point: RgmPoint3 = { x: _ptArr[0], y: _ptArr[1], z: _ptArr[2] };
        const frame = liveSession.surface_frame_at(liveSurfaceHandle, u, v);
        const du: RgmVec3 = { x: frame.du_x, y: frame.du_y, z: frame.du_z };
        const dv: RgmVec3 = { x: frame.dv_x, y: frame.dv_y, z: frame.dv_z };
        const normal: RgmVec3 = { x: frame.nx, y: frame.ny, z: frame.nz };

        let hasD2 = false;
        let duu: RgmVec3 = { x: 0, y: 0, z: 0 };
        let duv: RgmVec3 = { x: 0, y: 0, z: 0 };
        let dvv: RgmVec3 = { x: 0, y: 0, z: 0 };
        try {
          const d2Arr = liveSession.surface_d2_at(liveSurfaceHandle, u, v);
          duu = { x: d2Arr[0], y: d2Arr[1], z: d2Arr[2] };
          duv = { x: d2Arr[3], y: d2Arr[4], z: d2Arr[5] };
          dvv = { x: d2Arr[6], y: d2Arr[7], z: d2Arr[8] };
          hasD2 = true;
        } catch {
          hasD2 = false;
        }

        const d1Scale = surfaceProbeD1ScaleRef.current;
        const d2Scale = surfaceProbeD2ScaleRef.current;
        const arrows: SegmentOverlayVisual[] = [];
        const pushArrow = (
          name: string,
          color: string,
          vector: RgmVec3,
          scale: number,
          opacity: number,
          width: number,
        ): void => {
          const points = buildArrowSegments(point, vector, scale);
          if (points.length >= 2) {
            arrows.push({ name, color, opacity, width, points });
          }
        };

        pushArrow("D1 du", "#ffb36c", du, d1Scale, 0.98, 2.5);
        pushArrow("D1 dv", "#74dfff", dv, d1Scale, 0.98, 2.5);
        if (hasD2) {
          pushArrow("D2 duu", "#ffd8a0", duu, d2Scale, 0.92, 2.0);
          pushArrow("D2 duv", "#da9fff", duv, d2Scale, 0.92, 2.0);
          pushArrow("D2 dvv", "#9eb9ff", dvv, d2Scale, 0.92, 2.0);
        }
        setSegmentOverlays(arrows);
        setIntersectionPoints([]);

        probePointRef.current = point;
        if (probeRef.current) {
          probeRef.current.position.set(point.x, point.y, point.z);
          probeRef.current.visible = true;
        }

        setSurfaceProbeUiState({
          u,
          v,
          point,
          du,
          dv,
          normal,
          hasD2,
          duu,
          duv,
          dvv,
        });
        setErrorMessage(null);

        if (logCommit) {
          appendLog(
            "debug",
            `Surface probe uv=(${u.toFixed(4)}, ${v.toFixed(4)}) D0=${formatPoint(point)} |du|=${magnitude(du).toFixed(4)} |dv|=${magnitude(dv).toFixed(4)}${
              hasD2 ? ` |duu|=${magnitude(duu).toFixed(4)} |duv|=${magnitude(duv).toFixed(4)} |dvv|=${magnitude(dvv).toFixed(4)}` : " D2=n/a"
            }`,
          );
        }
      } catch (error) {
        setErrorMessage(error instanceof Error ? error.message : String(error));
      }
    },
    [appendLog],
  );

  useEffect(() => {
    updateSurfaceProbeForUvRef.current = updateSurfaceProbeForUv;
  }, [updateSurfaceProbeForUv]);

  const updateProbeForT = useCallback(
    (nextValue: number, logCommit: boolean): void => {
      const next = Math.min(1, Math.max(0, nextValue));
      probeTNormRef.current = next;

      const liveSession = sessionRef.current;
      const liveCurveHandle = curveHandleRef.current;
      if (!liveSession || liveCurveHandle === null) {
        setProbeUiState((previous) => ({ ...previous, tNorm: next }));
        return;
      }

      try {
        const _ptArr = liveSession.curve_point_at(liveCurveHandle, next);
        const point = { x: _ptArr[0], y: _ptArr[1], z: _ptArr[2] };
        const probeLength = liveSession.curve_length_at(liveCurveHandle, next);
        const totalLength = totalLengthRef.current;

        let tangent: { x: number; y: number; z: number } | undefined;
        let normal: { x: number; y: number; z: number } | undefined;
        let binormal: { x: number; y: number; z: number } | undefined;
        try {
          const frame = liveSession.curve_plane_at(liveCurveHandle, next);
          tangent = { x: frame[3], y: frame[4], z: frame[5] };
          binormal = { x: frame[6], y: frame[7], z: frame[8] };
          normal = { x: frame[9], y: frame[10], z: frame[11] };
        } catch {
          // Frenet frame may not exist for degenerate curves
        }

        probePointRef.current = point;
        if (probeRef.current) {
          probeRef.current.position.set(point.x, point.y, point.z);
          probeRef.current.visible = shouldShowProbeForExample(activeExample);
        }

        setProbeUiState({
          tNorm: next,
          x: point.x,
          y: point.y,
          z: point.z,
          probeLength,
          totalLength,
          tangent,
          normal,
          binormal,
        });
        setErrorMessage(null);

        if (followCameraRef.current && tangent) {
          const cam = cameraRef.current;
          const ctrl = controlsRef.current;
          if (cam && ctrl) {
            const probePos = new THREE.Vector3(point.x, point.y, point.z);
            const t = new THREE.Vector3(tangent.x, tangent.y, tangent.z).normalize();
            const up = normal
              ? new THREE.Vector3(normal.x, normal.y, normal.z).normalize()
              : new THREE.Vector3(0, 1, 0);
            const scale = Math.max(1, totalLength * 0.08);
            cam.position.copy(probePos)
              .addScaledVector(t, -scale)
              .addScaledVector(up, scale * 0.4);
            ctrl.target.copy(probePos);
            cam.up.copy(up);
            syncControlsUpAxis(ctrl, cam);
            cam.lookAt(probePos);
            ctrl.update();
          }
        }

        if (logCommit) {
          appendLog(
            "debug",
            `Probe t=${next.toFixed(5)} point=(${point.x.toFixed(5)}, ${point.y.toFixed(5)}, ${point.z.toFixed(5)}) len=${probeLength.toFixed(5)}/${totalLength.toFixed(5)}`,
          );
        }
      } catch (error) {
        setErrorMessage(error instanceof Error ? error.message : String(error));
      }
    },
    [activeExample, appendLog],
  );

  const updateLandXmlProbe = useCallback(
    (stationNorm: number, alignIdx: number, profIdx: number, logCommit: boolean): void => {
      const session = sessionRef.current;
      const ctx = landXmlContextRef.current;
      if (!session || !ctx) return;

      const info = ctx.alignments[alignIdx];
      if (!info || info.profileCount === 0) return;
      const pIdx = Math.min(profIdx, info.profileCount - 1);

      const t = Math.min(1, Math.max(0, stationNorm));
      const station = info.staStart + t * (info.staEnd - info.staStart);

      try {
        const packed = session.landxml_probe_alignment(ctx.docHandle, alignIdx, pIdx, station);
        const px = packed[0] - ctx.centroidX;
        const py = packed[1] - ctx.centroidY;
        const pz = packed[2] - ctx.centroidZ;
        const tx = packed[3];
        const ty = packed[4];
        const tz = packed[5];
        const grade = packed[6];

        const displayZ = (pz - landXmlDatumOffset) * landXmlVertExag;
        const profilePt = { x: px, y: py, z: displayZ };

        probePointRef.current = profilePt;
        if (probeRef.current) {
          probeRef.current.position.set(profilePt.x, profilePt.y, profilePt.z);
          probeRef.current.scale.setScalar(8);
          probeRef.current.visible = true;
        }

        // Vertical plane perpendicular to the horizontal alignment tangent.
        // Use only the plan component (tx, ty) so the plane is perfectly vertical.
        const planLen = Math.sqrt(tx * tx + ty * ty);
        const ntx = planLen > 1e-12 ? tx / planLen : 1;
        const nty = planLen > 1e-12 ? ty / planLen : 0;

        setIntersectionPlane({
          origin: profilePt,
          x_axis: { x: -nty, y: ntx, z: 0 },
          y_axis: { x: 0, y: 0, z: 1 },
          z_axis: { x: ntx, y: nty, z: 0 },
        });
        setIntersectionPoints([profilePt]);

        setLandXmlProbeUiState({
          station,
          stationNorm: t,
          alignmentIndex: alignIdx,
          profileIndex: pIdx,
          alignmentPoint: { x: packed[0], y: packed[1], z: 0 },
          profilePoint: { x: packed[0], y: packed[1], z: packed[2] },
          tangent: { x: tx, y: ty, z: tz },
          grade,
        });
        setErrorMessage(null);

        if (followCameraRef.current) {
          const cam = cameraRef.current;
          const ctrl = controlsRef.current;
          if (cam && ctrl) {
            const probePos = new THREE.Vector3(profilePt.x, profilePt.y, profilePt.z);
            const tangentVec = new THREE.Vector3(ntx, nty, 0).normalize();
            const up = new THREE.Vector3(0, 0, 1);
            const alignLen = Math.max(1, info.staEnd - info.staStart);
            const scale = Math.max(10, alignLen * 0.05);
            cam.position.copy(probePos)
              .addScaledVector(tangentVec, -scale)
              .addScaledVector(up, scale * 0.35);
            ctrl.target.copy(probePos);
            cam.up.copy(up);
            syncControlsUpAxis(ctrl, cam);
            cam.lookAt(probePos);
            ctrl.update();
          }
        }

        if (logCommit) {
          appendLog(
            "debug",
            `LandXML probe sta=${station.toFixed(2)} pt=(${packed[0].toFixed(2)},${packed[1].toFixed(2)},${packed[2].toFixed(2)}) grade=${grade.toFixed(4)}`,
          );
        }
      } catch (error) {
        setErrorMessage(error instanceof Error ? error.message : String(error));
      }
    },
    [appendLog, landXmlDatumOffset, landXmlVertExag],
  );

  const applyTransformTargetSelection = useCallback(
    (nextKey: string, logSelection: boolean): void => {
      if (activeExample !== "meshTransform" && activeExample !== "meshBoolean") {
        return;
      }
      const session = sessionRef.current;
      if (!session) {
        return;
      }
      const options = transformTargetsRef.current;
      const selected = options.find((target) => target.key === nextKey);
      if (!selected) {
        return;
      }

      if (activeExample === "meshBoolean") {
        const resultHandle = booleanResultMeshHandleRef.current;
        if (resultHandle === null) {
          return;
        }
        const primaryBuffers = meshToBuffers(session, selected.handle);
        const resultBuffers = meshToBuffers(session, resultHandle);
        const overlays: MeshVisual[] = options
          .filter((target) => target.key !== nextKey)
          .map((target) => {
            const buffers = meshToBuffers(session, target.handle);
            return {
              vertices: buffers.vertices,
              indices: buffers.indices,
              color: target.color,
              opacity: Math.max(0.08, target.opacity * 0.45),
              wireframe: false,
              name: target.label,
            } satisfies MeshVisual;
          });
        overlays.push({
          vertices: resultBuffers.vertices,
          indices: resultBuffers.indices,
          color: "#8ac6ff",
          opacity: 0.95,
          wireframe: false,
          name: "boolean result (A - B)",
        } satisfies MeshVisual);

        interactiveMeshHandleRef.current = selected.handle;
        setTransformTargetKey(nextKey);
        setMeshVisual({
          vertices: primaryBuffers.vertices,
          indices: primaryBuffers.indices,
          color: selected.color,
          opacity: selected.opacity,
          wireframe: false,
          name: `${selected.label} (active target)`,
        });
        setOverlayMeshes(overlays);
        if (logSelection) {
          appendLog(
            "info",
            `CSG target selected: ${selected.label} (${selected.handle.toString()})`,
          );
        }
        return;
      }

      const primaryBuffers = meshToBuffers(session, selected.handle);
      const overlays = options
        .filter((target) => target.key !== nextKey)
        .map((target) => {
          const buffers = meshToBuffers(session, target.handle);
          return {
            vertices: buffers.vertices,
            indices: buffers.indices,
            color: target.color,
            opacity: Math.max(0.28, target.opacity * 0.46),
            wireframe: target.wireframe,
            name: target.label,
          } satisfies MeshVisual;
        });

      interactiveMeshHandleRef.current = selected.handle;
      setTransformTargetKey(nextKey);
      setMeshVisual({
        vertices: primaryBuffers.vertices,
        indices: primaryBuffers.indices,
        color: selected.color,
        opacity: selected.opacity,
        wireframe: selected.wireframe,
        name: selected.label,
      });
      setOverlayMeshes(overlays);
      if (logSelection) {
        appendLog(
          "info",
          `Transform target selected: ${selected.label} (${selected.handle.toString()})`,
        );
      }
    },
    [activeExample, appendLog],
  );

  const recomputeMeshPlaneIntersection = useCallback(
    (reason: string): void => {
      if (activeExample !== "meshIntersectMeshPlane") {
        return;
      }
      const session = sessionRef.current;
      const meshHandle = meshPlaneMeshHandleRef.current;
      const plane = meshPlanePlaneRef.current;
      if (!session || meshHandle === null || !plane) {
        return;
      }

      const start = performance.now();
      const hits = flatToPoints(session.intersect_mesh_plane(meshHandle, flattenPlane(plane)));
      const intersectionMs = performance.now() - start;
      const triangleCount = session.mesh_triangle_count(meshHandle);

      setSegmentOverlays([
        {
          points: hits,
          color: "#ffef7f",
          opacity: 0.99,
          name: "mesh-plane-hit",
        },
      ]);
      setPerfStats((previous) => ({ ...previous, intersectionMs }));
      setStatusMessage(
        `Mesh-plane intersection updated (${reason}) • segments ${Math.floor(hits.length / 2)} • intersection ${intersectionMs.toFixed(2)}ms • triangles ${triangleCount}`,
      );
      appendLog(
        "debug",
        `mesh-plane intersection recomputed reason=${reason} segments=${Math.floor(hits.length / 2)} time=${intersectionMs.toFixed(2)}ms`,
      );
    },
    [activeExample, appendLog],
  );

  const planeFromGroup = useCallback((group: THREE.Object3D): RgmPlane => {
    const quaternion = group.quaternion;
    const xAxis = new THREE.Vector3(1, 0, 0).applyQuaternion(quaternion).normalize();
    const yAxis = new THREE.Vector3(0, 1, 0).applyQuaternion(quaternion).normalize();
    const zAxis = new THREE.Vector3(0, 0, 1).applyQuaternion(quaternion).normalize();
    return {
      origin: toPoint3(group.position),
      x_axis: toPoint3(xAxis),
      y_axis: toPoint3(yAxis),
      z_axis: toPoint3(zAxis),
    };
  }, []);

  const computeGizmoDelta = useCallback(
    (
      object: THREE.Object3D,
      dragStart: { position: THREE.Vector3; quaternion: THREE.Quaternion; scale: THREE.Vector3 },
    ):
      | { kind: "none" }
      | { kind: "translate"; delta: RgmPoint3 }
      | { kind: "rotate"; axis: RgmPoint3; angle: number; pivot: RgmPoint3 }
      | { kind: "scale"; scale: RgmPoint3; pivot: RgmPoint3 } => {
      const pivot = toPoint3(dragStart.position);
      if (gizmoMode === "translate") {
        const delta = object.position.clone().sub(dragStart.position);
        if (delta.lengthSq() <= 1e-12) {
          return { kind: "none" };
        }
        return { kind: "translate", delta: toPoint3(delta) };
      }

      if (gizmoMode === "rotate") {
        const deltaQuaternion = object.quaternion
          .clone()
          .multiply(dragStart.quaternion.clone().invert());
        const clampedW = Math.min(1, Math.max(-1, deltaQuaternion.w));
        let angle = 2 * Math.acos(clampedW);
        const sinHalf = Math.sqrt(Math.max(0, 1 - clampedW * clampedW));
        const axis =
          sinHalf > 1e-8
            ? new THREE.Vector3(
                deltaQuaternion.x / sinHalf,
                deltaQuaternion.y / sinHalf,
                deltaQuaternion.z / sinHalf,
              )
            : new THREE.Vector3(1, 0, 0);
        if (angle > Math.PI) {
          angle = 2 * Math.PI - angle;
          axis.multiplyScalar(-1);
        }
        if (!Number.isFinite(angle) || angle <= 1e-7) {
          return { kind: "none" };
        }
        return { kind: "rotate", axis: toPoint3(axis.normalize()), angle, pivot };
      }

      const scale = {
        x: object.scale.x / dragStart.scale.x,
        y: object.scale.y / dragStart.scale.y,
        z: object.scale.z / dragStart.scale.z,
      };
      const delta = Math.max(
        Math.abs(scale.x - 1),
        Math.abs(scale.y - 1),
        Math.abs(scale.z - 1),
      );
      if (delta <= 1e-6) {
        return { kind: "none" };
      }
      return { kind: "scale", scale, pivot };
    },
    [gizmoMode],
  );

  const scheduleLiveMeshPlanePreview = useCallback((): void => {
    if (activeExample !== "meshIntersectMeshPlane") {
      return;
    }
    if (liveIntersectionTimerRef.current !== null) {
      return;
    }

    liveIntersectionTimerRef.current = window.setTimeout(() => {
      liveIntersectionTimerRef.current = null;
      const session = sessionRef.current;
      const dragStart = dragStartTransformRef.current;
      if (!session || !dragStart) {
        return;
      }

      try {
        if (meshPlaneTarget === "plane") {
          const planeGroup = planeGroupRef.current;
          const meshHandle = meshPlaneMeshHandleRef.current;
          if (!planeGroup || meshHandle === null) {
            return;
          }
          const livePlane = planeFromGroup(planeGroup);
          const start = performance.now();
          const hits = flatToPoints(session.intersect_mesh_plane(meshHandle, flattenPlane(livePlane)));
          const intersectionMs = performance.now() - start;
          setSegmentOverlays([
            {
              points: hits,
              color: "#ffef7f",
              opacity: 0.99,
              name: "mesh-plane-hit",
            },
          ]);
          setPerfStats((previous) => ({ ...previous, intersectionMs }));
          return;
        }

        const baseMeshHandle = meshPlaneMeshHandleRef.current;
        const mesh = meshRef.current;
        const plane = meshPlanePlaneRef.current;
        if (baseMeshHandle === null || !mesh || !plane) {
          return;
        }
        const delta = computeGizmoDelta(mesh, dragStart);
        if (delta.kind === "none") {
          return;
        }

        if (previewMeshHandleRef.current !== null) {
          previewMeshHandleRef.current = null;
        }
        let previewHandle: MeshHandle;
        if (delta.kind === "translate") {
          previewHandle = session.mesh_translate(baseMeshHandle, delta.delta.x, delta.delta.y, delta.delta.z);
        } else if (delta.kind === "rotate") {
          previewHandle = session.mesh_rotate(baseMeshHandle, delta.axis.x, delta.axis.y, delta.axis.z, delta.angle, delta.pivot.x, delta.pivot.y, delta.pivot.z);
        } else {
          previewHandle = session.mesh_scale(baseMeshHandle, delta.scale.x, delta.scale.y, delta.scale.z, delta.pivot.x, delta.pivot.y, delta.pivot.z);
        }
        previewMeshHandleRef.current = previewHandle;

        const start = performance.now();
        const hits = flatToPoints(session.intersect_mesh_plane(previewHandle, flattenPlane(plane)));
        const intersectionMs = performance.now() - start;
        setSegmentOverlays([
          {
            points: hits,
            color: "#ffef7f",
            opacity: 0.99,
            name: "mesh-plane-hit",
          },
        ]);
        setPerfStats((previous) => ({ ...previous, intersectionMs }));
      } catch {
        // Keep gizmo interaction smooth even if a preview solve fails transiently.
      }
    }, 40);
  }, [activeExample, computeGizmoDelta, meshPlaneTarget, planeFromGroup]);

  const commitInteractiveMeshTransform = useCallback((): void => {
    const session = sessionRef.current;
    const dragStart = dragStartTransformRef.current;
    if (!session || !dragStart) {
      return;
    }

    const mode = gizmoMode;
    if (liveIntersectionTimerRef.current !== null) {
      window.clearTimeout(liveIntersectionTimerRef.current);
      liveIntersectionTimerRef.current = null;
    }
    if (previewMeshHandleRef.current !== null) {
      previewMeshHandleRef.current = null;
    }

    try {
      if (activeExample === "meshIntersectMeshPlane" && meshPlaneTarget === "plane") {
        const planeGroup = planeGroupRef.current;
        if (!planeGroup) {
          return;
        }
        const moved =
          planeGroup.position.distanceToSquared(dragStart.position) > 1e-12 ||
          planeGroup.quaternion.angleTo(dragStart.quaternion) > 1e-7;
        if (!moved) {
          return;
        }
        const nextPlane = planeFromGroup(planeGroup);
        meshPlanePlaneRef.current = nextPlane;
        setIntersectionPlane(nextPlane);
        recomputeMeshPlaneIntersection("plane gizmo");
        setErrorMessage(null);
        return;
      }

      const mesh = meshRef.current;
      const meshHandle = interactiveMeshHandleRef.current;
      if (!mesh || meshHandle === null) {
        return;
      }
      let nextHandle = meshHandle;
      const delta = computeGizmoDelta(mesh, dragStart);
      if (delta.kind === "none") {
        return;
      }
      if (delta.kind === "translate") {
        nextHandle = session.mesh_translate(meshHandle, delta.delta.x, delta.delta.y, delta.delta.z);
      } else if (delta.kind === "rotate") {
        nextHandle = session.mesh_rotate(meshHandle, delta.axis.x, delta.axis.y, delta.axis.z, delta.angle, delta.pivot.x, delta.pivot.y, delta.pivot.z);
      } else {
        nextHandle = session.mesh_scale(meshHandle, delta.scale.x, delta.scale.y, delta.scale.z, delta.pivot.x, delta.pivot.y, delta.pivot.z);
      }

      const triangleCount = session.mesh_triangle_count(nextHandle);
      interactiveMeshHandleRef.current = nextHandle;
      ownedCurveHandlesRef.current = ownedCurveHandlesRef.current.map((handle) =>
        handle === meshHandle ? nextHandle : handle,
      );

      if (activeExample === "meshTransform") {
        transformTargetsRef.current = transformTargetsRef.current.map((target) =>
          target.key === transformTargetKey ? { ...target, handle: nextHandle } : target,
        );
        applyTransformTargetSelection(transformTargetKey, false);
      } else if (activeExample === "meshBoolean") {
        const activeTargetKey =
          transformTargetKey === "base" || transformTargetKey === "tool"
            ? transformTargetKey
            : (transformTargetsRef.current.find((target) => target.handle === meshHandle)?.key ?? "tool");
        transformTargetsRef.current = transformTargetsRef.current.map((target) =>
          target.key === activeTargetKey ? { ...target, handle: nextHandle } : target,
        );

        if (activeTargetKey === "base") {
          booleanBaseMeshHandleRef.current = nextHandle;
        } else if (activeTargetKey === "tool") {
          booleanToolMeshHandleRef.current = nextHandle;
        }

        const baseHandle = booleanBaseMeshHandleRef.current;
        const toolHandle = booleanToolMeshHandleRef.current;
        const previousResult = booleanResultMeshHandleRef.current;
        if (baseHandle === null || toolHandle === null || previousResult === null) {
          throw new Error("Boolean state is not initialized");
        }

        const csgStart = performance.now();
        const nextResult = session.mesh_boolean(baseHandle, toolHandle, 2);
        const csgMs = performance.now() - csgStart;
        const resultTriangles = session.mesh_triangle_count(nextResult);
        booleanResultMeshHandleRef.current = nextResult;
        ownedCurveHandlesRef.current = ownedCurveHandlesRef.current.map((handle) =>
          handle === previousResult ? nextResult : handle,
        );

        setTransformTargetKey(activeTargetKey);
        applyTransformTargetSelection(activeTargetKey, false);
        setPerfStats((previous) => ({ ...previous, intersectionMs: csgMs }));
        setStatusMessage(
          `CSG updated • mode ${mode} • target ${activeTargetKey} • result triangles ${resultTriangles} • solve ${csgMs.toFixed(2)}ms`,
        );
        setErrorMessage(null);
        appendLog(
          "debug",
          `csg recompute mode=${mode} target=${activeTargetKey} result=${resultTriangles} time=${csgMs.toFixed(2)}ms`,
        );
        return;
      } else if (activeExample === "meshIntersectMeshPlane") {
        meshPlaneMeshHandleRef.current = nextHandle;
        const buffers = meshToBuffers(session, nextHandle);
        setMeshVisual((previous) =>
          previous
            ? {
                ...previous,
                vertices: buffers.vertices,
                indices: buffers.indices,
              }
            : previous,
        );
        recomputeMeshPlaneIntersection("mesh gizmo");
      }

      setStatusMessage(
        `Mesh transform committed • mode ${mode} • triangles ${triangleCount} • target ${transformTargetKey || meshPlaneTarget}`,
      );
      setErrorMessage(null);
      appendLog(
        "debug",
        `mesh gizmo commit mode=${mode} handle=${nextHandle.toString()} triangles=${triangleCount}`,
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setErrorMessage(message);
      appendLog("error", `Mesh transform commit failed: ${message}`);
      const liveMesh = meshRef.current;
      if (liveMesh) {
        liveMesh.position.copy(dragStart.position);
        liveMesh.quaternion.copy(dragStart.quaternion);
        liveMesh.scale.copy(dragStart.scale);
      }
      const livePlane = planeGroupRef.current;
      if (livePlane && meshPlanePlaneRef.current) {
        const frame = buildPlaneFrame(meshPlanePlaneRef.current);
        livePlane.position.copy(frame.origin);
        livePlane.quaternion.setFromRotationMatrix(
          new THREE.Matrix4().makeBasis(frame.xAxis, frame.yAxis, frame.normal),
        );
      }
    } finally {
      dragStartTransformRef.current = null;
    }
  }, [
    activeExample,
    appendLog,
    applyTransformTargetSelection,
    computeGizmoDelta,
    gizmoMode,
    meshPlaneTarget,
    planeFromGroup,
    recomputeMeshPlaneIntersection,
    transformTargetKey,
  ]);

  useEffect(() => {
    if (activeExample !== "landxmlViewer") return;
    if (!sessionRef.current) return;
    void updateLandXmlFile(activeLandXmlFile);
  }, [activeLandXmlFile]); // eslint-disable-line react-hooks/exhaustive-deps

  const toggleInspector = useCallback((): void => {
    setIsInspectorOpen((current) => {
      const next = !current;
      if (next && isMobileLayout) {
        setIsConsoleOpen(false);
      }
      return next;
    });
  }, [isMobileLayout]);

  const toggleConsole = useCallback((): void => {
    setIsConsoleOpen((current) => {
      const next = !current;
      if (next && isMobileLayout) {
        setIsInspectorOpen(false);
      }
      return next;
    });
  }, [isMobileLayout]);

  const applySession = useCallback(
    (sessionFile: ViewerSessionFile): void => {
      suppressAutoFitRef.current = true;
      updateCurveForExample("nurbs", "Session loaded", sessionFile.preset);
      setShowGrid(sessionFile.view.showGrid);
      setShowAxes(sessionFile.view.showAxes);
      setOrbitEnabled(sessionFile.view.orbitEnabled);

      const savedMode = sessionFile.view.camera.mode ?? "perspective";
      if (savedMode !== cameraModeRef.current) {
        toggleCameraMode();
      }

      const camera = cameraRef.current;
      const controls = controlsRef.current;
      if (camera && controls) {
        camera.position.copy(fromPoint3(sessionFile.view.camera.position));
        camera.up.copy(fromPoint3(sessionFile.view.camera.up));
        syncControlsUpAxis(controls, camera);
        if (camera instanceof THREE.PerspectiveCamera) {
          camera.fov = sessionFile.view.camera.fov;
        }
        camera.updateProjectionMatrix();
        controls.target.copy(fromPoint3(sessionFile.view.camera.target));
        controls.update();
      }
      suppressAutoFitRef.current = false;
    },
    [updateCurveForExample, toggleCameraMode],
  );

  useEffect(() => {
    let disposed = false;

    (async () => {
      try {
        appendLog("info", "Loading kernel WASM runtime");
        await loadKernel("/wasm/rusted_geom.wasm");
        const session = new KernelSession();
        appendLog("info", "Kernel session created");
        const loadedPreset = await loadDefaultPreset();
        if (disposed) {
          session.free();
          return;
        }

        sessionRef.current = session;
        setCapabilities({ igesImport: false, igesExport: true });
        nurbsPresetRef.current = loadedPreset;
        setPreset(loadedPreset);
        updateCurveForExample("nurbs", "Default example loaded", loadedPreset);
        setKernelStatus("ready");
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        setErrorMessage(message);
        setKernelStatus("error");
        appendLog("error", `Startup failed: ${message}`);
      }
    })();

    return () => {
      disposed = true;
      releaseOwnedCurveHandles();
      sessionRef.current?.free();
      appendLog("info", "Kernel session destroyed");
      sessionRef.current = null;
      curveHandleRef.current = null;
    };
  }, [appendLog, loadDefaultPreset, releaseOwnedCurveHandles, updateCurveForExample]);

  useEffect(() => {
    const media = window.matchMedia(MOBILE_MEDIA_QUERY);
    const syncLayout = (matches: boolean): void => {
      setIsMobileLayout(matches);
      if (matches) {
        setIsInspectorOpen(false);
      } else {
        setIsInspectorOpen(true);
      }
    };

    syncLayout(media.matches);
    const listener = (event: MediaQueryListEvent): void => {
      syncLayout(event.matches);
    };
    media.addEventListener("change", listener);

    return () => {
      media.removeEventListener("change", listener);
    };
  }, []);

  // Sync isDarkModeRef for use inside Three.js loop
  useEffect(() => {
    isDarkModeRef.current = isDarkMode;
  }, [isDarkMode]);

  // Update Three.js scene colors when dark mode changes
  useEffect(() => {
    const scene = sceneRef.current;
    if (!scene) return;
    const bg = getComputedStyle(document.documentElement).getPropertyValue("--viewport-bg").trim() || "#edf1f7";
    scene.background = new THREE.Color(bg);
    if (scene.fog) {
      (scene.fog as THREE.Fog).color = new THREE.Color(bg);
    }
  }, [isDarkMode]);

  useKeyboardShortcuts([
    {
      key: "k",
      meta: true,
      allowInInput: false,
      handler: () => setIsExampleBrowserOpen((v) => !v),
    },
    {
      key: "Escape",
      allowInInput: false,
      handler: () => setIsExampleBrowserOpen(false),
    },
    {
      key: "g",
      allowInInput: false,
      handler: () => setShowGrid((v) => !v),
    },
    {
      key: "a",
      allowInInput: false,
      handler: () => setShowAxes((v) => !v),
    },
    {
      key: "i",
      allowInInput: false,
      handler: () => toggleInspector(),
    },
    {
      key: "c",
      allowInInput: false,
      handler: () => toggleConsole(),
    },
  ]);

  useEffect(() => {
    const viewport = viewportRef.current;
    if (!viewport) {
      return;
    }

    const scene = new THREE.Scene();
    const initialBg = getComputedStyle(document.documentElement).getPropertyValue("--viewport-bg").trim() || "#edf1f7";
    scene.background = new THREE.Color(initialBg);
    scene.fog = new THREE.Fog(initialBg, 34, 138);

    const aspect = viewport.clientWidth / Math.max(1, viewport.clientHeight);
    const perspCamera = new THREE.PerspectiveCamera(46, aspect, 0.01, 1200);
    perspCamera.position.copy(DEFAULT_CAMERA_POSITION);
    perspCamera.up.set(0, 0, 1);

    const orthoHalf = 10;
    const orthoCamera = new THREE.OrthographicCamera(
      -orthoHalf * aspect, orthoHalf * aspect,
      orthoHalf, -orthoHalf,
      0.001, 2400,
    );
    orthoCamera.position.copy(DEFAULT_CAMERA_POSITION);
    orthoCamera.up.set(0, 0, 1);

    // cameraRef (React ref) is the source of truth for the active camera.
    // The render loop reads cameraRef.current so toggleCameraMode takes effect immediately.

    let renderer: THREE.WebGLRenderer | null = null;
    let renderCanvas: HTMLCanvasElement | null = null;
    let fallbackContext: CanvasRenderingContext2D | null = null;
    const forceFallback = /HeadlessChrome/i.test(window.navigator.userAgent);
    if (!forceFallback) {
      try {
        renderer = new THREE.WebGLRenderer({
          antialias: true,
          alpha: true,
          preserveDrawingBuffer: true,
        });
        renderer.setPixelRatio(window.devicePixelRatio);
        renderer.setSize(viewport.clientWidth, Math.max(1, viewport.clientHeight));
        renderer.outputColorSpace = THREE.SRGBColorSpace;
        renderCanvas = renderer.domElement;
      } catch {
        renderer = null;
      }
    }

    if (!renderCanvas) {
      // Headless CI can lack a usable WebGL context. Keep the UI operational.
      renderCanvas = document.createElement("canvas");
      renderCanvas.className = "viewport-fallback-canvas";
      fallbackContext = renderCanvas.getContext("2d");
    }

    viewport.appendChild(renderCanvas);

    cameraRef.current = perspCamera;
    const controls = new OrbitControls(perspCamera, renderCanvas);
    controls.enableDamping = true;
    controls.target.copy(DEFAULT_CAMERA_TARGET);
    syncControlsUpAxis(controls, perspCamera);
    controls.update();

    const grid = new THREE.GridHelper(30, 30, "#8596b6", "#b9c4d8");
    grid.rotation.x = Math.PI / 2;
    grid.material.opacity = 0.5;
    grid.material.transparent = true;
    scene.add(grid);

    const axes = new THREE.AxesHelper(3.5);
    axes.visible = false;
    scene.add(axes);

    const probe = new THREE.Mesh(
      new THREE.SphereGeometry(0.22, 28, 28),
      new THREE.MeshStandardMaterial({
        color: "#9fc0ff",
        emissive: "#335fc2",
        emissiveIntensity: 0.58,
        roughness: 0.2,
        metalness: 0.1,
      }),
    );
    probe.visible = false;
    if (probePointRef.current) {
      probe.position.set(
        probePointRef.current.x,
        probePointRef.current.y,
        probePointRef.current.z,
      );
      probe.visible = true;
    }
    scene.add(probe);

    const key = new THREE.DirectionalLight("#f6fbff", 0.65);
    key.position.set(3, 10, 7);
    scene.add(key);
    scene.add(new THREE.AmbientLight("#9eb0d2", 0.52));

    const onResize = (): void => {
      const width = viewport.clientWidth;
      const height = Math.max(1, viewport.clientHeight);
      const a = width / height;

      perspCamera.aspect = a;
      perspCamera.updateProjectionMatrix();

      const orthoH = (orthoCamera.top - orthoCamera.bottom) / 2;
      orthoCamera.left = -orthoH * a;
      orthoCamera.right = orthoH * a;
      orthoCamera.updateProjectionMatrix();

      if (renderer) {
        renderer.setSize(width, height);
      } else {
        renderCanvas.width = Math.floor(width * window.devicePixelRatio);
        renderCanvas.height = Math.floor(height * window.devicePixelRatio);
        renderCanvas.style.width = `${width}px`;
        renderCanvas.style.height = `${height}px`;
        if (fallbackContext) {
          fallbackContext.save();
          fallbackContext.scale(window.devicePixelRatio, window.devicePixelRatio);
          fallbackContext.clearRect(0, 0, width, height);
          fallbackContext.fillStyle = getComputedStyle(document.documentElement).getPropertyValue("--viewport-bg").trim() || "#edf1f7";
          fallbackContext.fillRect(0, 0, width, height);
          fallbackContext.fillStyle = "#576b89";
          fallbackContext.font = "600 13px sans-serif";
          fallbackContext.fillText("WebGL unavailable in this environment", 14, 28);
          fallbackContext.restore();
        }
      }
      if (lineRef.current) {
        const material = lineRef.current.material as LineMaterial;
        material.resolution.set(width, height);
      }
      for (const overlay of overlayLineRefs.current) {
        const material = overlay.material as LineMaterial;
        material.resolution.set(width, height);
      }
      for (const overlay of segmentOverlayRefs.current) {
        const material = overlay.material as LineMaterial;
        material.resolution.set(width, height);
      }
    };

    const resizeObserver = new ResizeObserver(onResize);
    resizeObserver.observe(viewport);

    let frame = 0;
    const animate = (): void => {
      frame = window.requestAnimationFrame(animate);
      controls.update();
      if (renderer) {
        renderer.render(scene, cameraRef.current!);
      }
    };
    animate();
    onResize();

    sceneRef.current = scene;
    perspCameraRef.current = perspCamera;
    orthoCameraRef.current = orthoCamera;
    // cameraRef.current is already set above (to perspCamera)
    controlsRef.current = controls;
    rendererRef.current = renderer;
    gridRef.current = grid;
    axesRef.current = axes;
    probeRef.current = probe;

    return () => {
      window.cancelAnimationFrame(frame);
      resizeObserver.disconnect();
      controls.dispose();
      renderer?.dispose();
      if (lineRef.current) {
        lineRef.current.geometry.dispose();
        lineRef.current.material.dispose();
      }
      if (meshRef.current) {
        meshRef.current.geometry.dispose();
        meshRef.current.material.dispose();
      }
      if (meshWireRef.current) {
        meshWireRef.current.geometry.dispose();
        meshWireRef.current.material.dispose();
      }
      if (transformControlsRef.current) {
        if (transformControlsHelperRef.current) {
          scene.remove(transformControlsHelperRef.current);
        }
        transformControlsRef.current.detach();
        transformControlsRef.current.dispose();
      }
      for (const overlay of overlayLineRefs.current) {
        overlay.geometry.dispose();
        overlay.material.dispose();
      }
      overlayLineRefs.current = [];
      for (const overlay of segmentOverlayRefs.current) {
        overlay.geometry.dispose();
        overlay.material.dispose();
      }
      segmentOverlayRefs.current = [];
      for (const overlay of overlayMeshRefs.current) {
        overlay.mesh.geometry.dispose();
        overlay.mesh.material.dispose();
        if (overlay.wire) {
          overlay.wire.geometry.dispose();
          overlay.wire.material.dispose();
        }
      }
      overlayMeshRefs.current = [];
      for (const marker of intersectionMarkerRefs.current) {
        marker.geometry.dispose();
        marker.material.dispose();
      }
      intersectionMarkerRefs.current = [];
      if (planeGroupRef.current) {
        scene.remove(planeGroupRef.current);
      }
      if (planeMeshRef.current) {
        planeMeshRef.current.geometry.dispose();
        planeMeshRef.current.material.dispose();
      }
      if (planeWireRef.current) {
        planeWireRef.current.geometry.dispose();
        planeWireRef.current.material.dispose();
      }
      if (planeNormalRef.current) {
        scene.remove(planeNormalRef.current);
      }
      if (probeRef.current) {
        probeRef.current.geometry.dispose();
        probeRef.current.material.dispose();
      }
      if (renderCanvas.parentElement === viewport) {
        viewport.removeChild(renderCanvas);
      }
      scene.clear();
      sceneRef.current = null;
      controlsRef.current = null;
      cameraRef.current = null;
      rendererRef.current = null;
      gridRef.current = null;
      axesRef.current = null;
      probeRef.current = null;
      lineRef.current = null;
      planeMeshRef.current = null;
      planeWireRef.current = null;
      planeNormalRef.current = null;
      planeGroupRef.current = null;
      meshRef.current = null;
      meshWireRef.current = null;
      transformControlsRef.current = null;
      transformControlsHelperRef.current = null;
    };
  }, []);

  useEffect(() => {
    if (!sceneRef.current) {
      return;
    }

    if (lineRef.current) {
      lineRef.current.geometry.dispose();
      lineRef.current.material.dispose();
      sceneRef.current.remove(lineRef.current);
      lineRef.current = null;
    }

    if (!sampledPoints.length) {
      return;
    }

    const curveDegree = activeRenderDegree;
    const line = createWideLine(
      sampledPoints,
      curveColorForDegree(curveDegree),
      curveWidthForDegree(curveDegree),
      1,
      viewportRef.current,
    );
    line.renderOrder = 30;
    lineRef.current = line;
    sceneRef.current.add(line);
  }, [activeRenderDegree, sampledPoints]);

  useEffect(() => {
    const scene = sceneRef.current;
    if (!scene) {
      return;
    }

    if (meshRef.current) {
      scene.remove(meshRef.current);
      meshRef.current.geometry.dispose();
      meshRef.current.material.dispose();
      meshRef.current = null;
    }
    if (meshWireRef.current) {
      meshWireRef.current.geometry.dispose();
      meshWireRef.current.material.dispose();
      meshWireRef.current = null;
    }

    if (!meshVisual) {
      return;
    }

    const interactive =
      (activeExample === "meshTransform" || activeExample === "meshBoolean") &&
      interactiveMeshHandleRef.current !== null;
    const origin = interactive ? centroidOfPoints(meshVisual.vertices) : undefined;
    const geometry = createMeshGeometry(meshVisual.vertices, meshVisual.indices, origin);
    const brepVisual = isBrepExample(activeExample);
    const booleanVisual = activeExample === "meshBoolean";
    const booleanResultVisual = booleanVisual && meshVisual.name === "boolean result (A - B)";
    const visualOpacity = brepVisual ? Math.min(meshVisual.opacity, 0.58) : meshVisual.opacity;
    const material = new THREE.MeshStandardMaterial({
      color: meshVisual.color,
      transparent: visualOpacity < 1,
      opacity: visualOpacity,
      roughness: brepVisual ? 0.32 : 0.5,
      metalness: brepVisual ? 0.04 : 0.08,
      side: THREE.DoubleSide,
      depthWrite: brepVisual ? false : booleanResultVisual,
      depthTest: booleanVisual ? booleanResultVisual : true,
      polygonOffset: booleanVisual && !booleanResultVisual,
      polygonOffsetFactor: booleanVisual && !booleanResultVisual ? 2 : 0,
      polygonOffsetUnits: booleanVisual && !booleanResultVisual ? 2 : 0,
    });
    const mesh = new THREE.Mesh(geometry, material);
    if (origin) {
      mesh.position.copy(origin);
    }
    mesh.renderOrder = booleanVisual ? (booleanResultVisual ? 20 : 10) : 18;
    scene.add(mesh);
    meshRef.current = mesh;

    if (meshVisual.wireframe && !brepVisual && !booleanVisual) {
      const wire = new THREE.LineSegments(
        new THREE.WireframeGeometry(geometry),
        new THREE.LineBasicMaterial({
          color: "#f4fbff",
          transparent: true,
          opacity: 0.55,
        }),
      );
      wire.renderOrder = 19;
      mesh.add(wire);
      meshWireRef.current = wire;
    }
  }, [activeExample, meshVisual]);

  useEffect(() => {
    const scene = sceneRef.current;
    const camera = cameraRef.current;
    const orbit = controlsRef.current;
    const domElement = rendererRef.current?.domElement;
    if (!scene || !camera || !orbit || !domElement) {
      return;
    }

    if (transformControlsRef.current) {
      if (transformControlsHelperRef.current) {
        scene.remove(transformControlsHelperRef.current);
        transformControlsHelperRef.current = null;
      }
      transformControlsRef.current.detach();
      transformControlsRef.current.dispose();
      transformControlsRef.current = null;
    }

    const mesh = meshRef.current;
    let targetObject: THREE.Object3D | null = null;
    if (activeExample === "meshTransform" || activeExample === "meshBoolean") {
      if (mesh && interactiveMeshHandleRef.current !== null) {
        targetObject = mesh;
      }
    } else if (activeExample === "meshIntersectMeshPlane") {
      if (meshPlaneTarget === "plane") {
        targetObject = planeGroupRef.current;
      } else if (mesh && interactiveMeshHandleRef.current !== null) {
        targetObject = mesh;
      }
    }

    if (!targetObject) {
      return;
    }

    const transform = new TransformControls(camera, domElement);
    const effectiveMode =
      activeExample === "meshIntersectMeshPlane" &&
      meshPlaneTarget === "plane" &&
      gizmoMode === "scale"
        ? "translate"
        : gizmoMode;
    transform.setMode(effectiveMode);
    transform.size = isMobileLayout ? 0.88 : 1.0;
    transform.attach(targetObject);

    const onMouseDown = (): void => {
      if (liveIntersectionTimerRef.current !== null) {
        window.clearTimeout(liveIntersectionTimerRef.current);
        liveIntersectionTimerRef.current = null;
      }
      if (previewMeshHandleRef.current !== null) {
        const liveSession = sessionRef.current;
        if (liveSession) {
        }
        previewMeshHandleRef.current = null;
      }
      dragStartTransformRef.current = {
        position: targetObject.position.clone(),
        quaternion: targetObject.quaternion.clone(),
        scale: targetObject.scale.clone(),
      };
    };
    const onMouseUp = (): void => {
      isTransformDraggingRef.current = false;
      orbit.enabled = orbitEnabled;
      commitInteractiveMeshTransform();
    };
    const onDraggingChanged = (event: { value: unknown }): void => {
      isTransformDraggingRef.current = Boolean(event.value);
      orbit.enabled = orbitEnabled && !isTransformDraggingRef.current;
    };
    const onObjectChange = (): void => {
      if (
        activeExample === "meshIntersectMeshPlane" &&
        isTransformDraggingRef.current &&
        dragStartTransformRef.current
      ) {
        scheduleLiveMeshPlanePreview();
      }
    };

    transform.addEventListener("mouseDown", onMouseDown);
    transform.addEventListener("mouseUp", onMouseUp);
    transform.addEventListener("dragging-changed", onDraggingChanged);
    transform.addEventListener("objectChange", onObjectChange);
    const helper = transform.getHelper();
    scene.add(helper);
    transformControlsRef.current = transform;
    transformControlsHelperRef.current = helper;

    return () => {
      transform.removeEventListener("mouseDown", onMouseDown);
      transform.removeEventListener("mouseUp", onMouseUp);
      transform.removeEventListener("dragging-changed", onDraggingChanged);
      transform.removeEventListener("objectChange", onObjectChange);
      scene.remove(helper);
      transform.detach();
      transform.dispose();
      if (transformControlsRef.current === transform) {
        transformControlsRef.current = null;
      }
      if (transformControlsHelperRef.current === helper) {
        transformControlsHelperRef.current = null;
      }
      isTransformDraggingRef.current = false;
      orbit.enabled = orbitEnabled;
    };
  }, [
    activeExample,
    commitInteractiveMeshTransform,
    gizmoMode,
    intersectionPlane,
    isMobileLayout,
    meshPlaneTarget,
    meshVisual,
    orbitEnabled,
    scheduleLiveMeshPlanePreview,
  ]);

  useEffect(() => {
    const scene = sceneRef.current;
    if (!scene) {
      return;
    }

    for (const overlay of overlayMeshRefs.current) {
      scene.remove(overlay.mesh);
      overlay.mesh.geometry.dispose();
      overlay.mesh.material.dispose();
      if (overlay.wire) {
        scene.remove(overlay.wire);
        overlay.wire.geometry.dispose();
        overlay.wire.material.dispose();
      }
    }
    overlayMeshRefs.current = [];

    const brepVisual = isBrepExample(activeExample);
    const booleanVisual = activeExample === "meshBoolean";
    for (const visual of overlayMeshes) {
      const geometry = createMeshGeometry(visual.vertices, visual.indices);
      const booleanResultVisual = booleanVisual && visual.name === "boolean result (A - B)";
      const visualOpacity = brepVisual ? Math.min(visual.opacity, 0.18) : visual.opacity;
      const material = new THREE.MeshStandardMaterial({
        color: visual.color,
        transparent: visualOpacity < 1,
        opacity: visualOpacity,
        roughness: brepVisual ? 0.36 : 0.55,
        metalness: brepVisual ? 0.02 : 0.05,
        side: THREE.DoubleSide,
        depthWrite: brepVisual ? false : booleanResultVisual,
        depthTest: booleanVisual ? booleanResultVisual : true,
        polygonOffset: booleanVisual && !booleanResultVisual,
        polygonOffsetFactor: booleanVisual && !booleanResultVisual ? 2 : 0,
        polygonOffsetUnits: booleanVisual && !booleanResultVisual ? 2 : 0,
      });
      const mesh = new THREE.Mesh(geometry, material);
      mesh.renderOrder = booleanVisual ? (booleanResultVisual ? 20 : 11) : 14;
      scene.add(mesh);

      let wire: THREE.LineSegments<THREE.WireframeGeometry, THREE.LineBasicMaterial> | null = null;
      if (visual.wireframe && !brepVisual && !booleanVisual) {
        wire = new THREE.LineSegments(
          new THREE.WireframeGeometry(geometry),
          new THREE.LineBasicMaterial({
            color: "#eaf5ff",
            transparent: true,
            opacity: Math.min(1, visual.opacity + 0.2),
          }),
        );
        wire.renderOrder = 15;
        scene.add(wire);
      }
      overlayMeshRefs.current.push({ mesh, wire });
    }
  }, [activeExample, overlayMeshes]);

  useEffect(() => {
    const scene = sceneRef.current;
    if (!scene) {
      return;
    }

    for (const overlay of overlayLineRefs.current) {
      scene.remove(overlay);
      overlay.geometry.dispose();
      overlay.material.dispose();
    }
    overlayLineRefs.current = [];

    for (const overlayCurve of overlayCurves) {
      if (overlayCurve.points.length < 2) {
        continue;
      }
      const overlay = createWideLine(
        overlayCurve.points,
        overlayCurve.color,
        overlayCurve.width,
        overlayCurve.opacity,
        viewportRef.current,
      );
      overlay.renderOrder = 26;
      scene.add(overlay);
      overlayLineRefs.current.push(overlay);
    }
  }, [overlayCurves]);

  useEffect(() => {
    const scene = sceneRef.current;
    if (!scene) {
      return;
    }

    for (const overlay of segmentOverlayRefs.current) {
      scene.remove(overlay);
      overlay.geometry.dispose();
      overlay.material.dispose();
    }
    segmentOverlayRefs.current = [];

    for (const overlay of segmentOverlays) {
      const lineSegments = createSegmentLines(
        overlay.points,
        overlay.color,
        overlay.opacity,
        overlay.width ?? 3.2,
        viewportRef.current,
      );
      if (!lineSegments) {
        continue;
      }
      lineSegments.renderOrder = 27;
      scene.add(lineSegments);
      segmentOverlayRefs.current.push(lineSegments);
    }
  }, [segmentOverlays]);

  useEffect(() => {
    const scene = sceneRef.current;
    if (!scene) {
      return;
    }

    for (const marker of intersectionMarkerRefs.current) {
      scene.remove(marker);
      marker.geometry.dispose();
      marker.material.dispose();
    }
    intersectionMarkerRefs.current = [];

    const markerRadius = activeExample === "landxmlViewer" ? 2.0 : 0.25;
    for (const hit of intersectionPoints) {
      const marker = new THREE.Mesh(
        new THREE.SphereGeometry(markerRadius, 20, 20),
        new THREE.MeshStandardMaterial({
          color: "#ff8fd9",
          emissive: "#7e2f67",
          emissiveIntensity: 0.64,
          roughness: 0.18,
          metalness: 0.2,
          depthTest: false,
          depthWrite: false,
        }),
      );
      marker.position.set(hit.x, hit.y, hit.z);
      marker.renderOrder = 40;
      scene.add(marker);
      intersectionMarkerRefs.current.push(marker);
    }
  }, [activeExample, intersectionPoints]);

  useEffect(() => {
    const scene = sceneRef.current;
    if (!scene) {
      return;
    }

    if (planeGroupRef.current) {
      scene.remove(planeGroupRef.current);
      planeGroupRef.current = null;
    }
    if (planeMeshRef.current) {
      planeMeshRef.current.geometry.dispose();
      planeMeshRef.current.material.dispose();
      planeMeshRef.current = null;
    }
    if (planeWireRef.current) {
      planeWireRef.current.geometry.dispose();
      planeWireRef.current.material.dispose();
      planeWireRef.current = null;
    }
    if (planeNormalRef.current) {
      planeNormalRef.current = null;
    }

    if (!intersectionPlane) {
      return;
    }

    const isLandXml = activeExample === "landxmlViewer";

    if (isLandXml) {
      // For LandXML (Z-up world), build the vertical plane directly from
      // world-space vertices to avoid any Y-up / Z-up coordinate confusion.
      const o = new THREE.Vector3(
        intersectionPlane.origin.x,
        intersectionPlane.origin.y,
        intersectionPlane.origin.z,
      );
      const xDir = new THREE.Vector3(
        intersectionPlane.x_axis.x,
        intersectionPlane.x_axis.y,
        intersectionPlane.x_axis.z,
      ).normalize();
      const zUp = new THREE.Vector3(0, 0, 1);
      const nDir = new THREE.Vector3(
        intersectionPlane.z_axis.x,
        intersectionPlane.z_axis.y,
        intersectionPlane.z_axis.z,
      ).normalize();

      const refPts = overlayCurves.flatMap((c) => c.points);
      const half = Math.max(10, planeVisualSize(refPts) * 0.125);

      // Four corners: origin ± half*xDir ± half*zUp
      const positions = new Float32Array(4 * 3);
      const c0 = o.clone().addScaledVector(xDir, -half).addScaledVector(zUp, -half);
      const c1 = o.clone().addScaledVector(xDir, half).addScaledVector(zUp, -half);
      const c2 = o.clone().addScaledVector(xDir, half).addScaledVector(zUp, half);
      const c3 = o.clone().addScaledVector(xDir, -half).addScaledVector(zUp, half);
      positions.set([c0.x, c0.y, c0.z, c1.x, c1.y, c1.z, c2.x, c2.y, c2.z, c3.x, c3.y, c3.z]);

      const quadGeom = new THREE.BufferGeometry();
      quadGeom.setAttribute("position", new THREE.BufferAttribute(positions, 3));
      quadGeom.setIndex([0, 1, 2, 0, 2, 3]);
      quadGeom.computeVertexNormals();

      const planeGroup = new THREE.Group();
      scene.add(planeGroup);
      planeGroupRef.current = planeGroup;

      const planeMesh = new THREE.Mesh(
        quadGeom,
        new THREE.MeshStandardMaterial({
          color: "#66c5f6",
          transparent: true,
          opacity: 0.24,
          side: THREE.DoubleSide,
          roughness: 0.62,
          metalness: 0.06,
          depthWrite: false,
        }),
      );
      planeMesh.renderOrder = 8;
      planeGroup.add(planeMesh);
      planeMeshRef.current = planeMesh;

      const edgePositions = new Float32Array([
        c0.x, c0.y, c0.z, c1.x, c1.y, c1.z,
        c1.x, c1.y, c1.z, c2.x, c2.y, c2.z,
        c2.x, c2.y, c2.z, c3.x, c3.y, c3.z,
        c3.x, c3.y, c3.z, c0.x, c0.y, c0.z,
      ]);
      const wireGeom = new THREE.BufferGeometry();
      wireGeom.setAttribute("position", new THREE.BufferAttribute(edgePositions, 3));
      const planeWire = new THREE.LineSegments(
        wireGeom,
        new THREE.LineBasicMaterial({ color: "#8fdbff", transparent: true, opacity: 0.7 }),
      );
      planeWire.renderOrder = 9;
      planeGroup.add(planeWire);
      planeWireRef.current = planeWire;

      const arrowLen = Math.max(3, half * 0.5);
      const normalArrow = new THREE.ArrowHelper(nDir, o, arrowLen, 0x95e3ff, arrowLen * 0.16, arrowLen * 0.08);
      planeGroup.add(normalArrow);
      planeNormalRef.current = normalArrow;
    } else {
      // Non-LandXML: use the existing basis-matrix approach (Y-up world)
      const frame = buildPlaneFrame(intersectionPlane);
      const referencePoints = sampledPoints.length > 0
        ? sampledPoints
        : (meshVisual?.vertices ?? []);
      const center = projectedPointOnPlane(
        centroidOfPoints(referencePoints),
        frame.origin,
        frame.normal,
      );
      const size = planeVisualSize(referencePoints);
      const basis = new THREE.Matrix4().makeBasis(frame.xAxis, frame.yAxis, frame.normal);
      const planeGroup = new THREE.Group();
      planeGroup.position.copy(center);
      planeGroup.setRotationFromMatrix(basis);
      scene.add(planeGroup);
      planeGroupRef.current = planeGroup;

      const planeMesh = new THREE.Mesh(
        new THREE.PlaneGeometry(size, size, 1, 1),
        new THREE.MeshStandardMaterial({
          color: "#66c5f6",
          transparent: true,
          opacity: 0.24,
          side: THREE.DoubleSide,
          roughness: 0.62,
          metalness: 0.06,
          depthWrite: false,
        }),
      );
      planeMesh.renderOrder = 8;
      planeGroup.add(planeMesh);
      planeMeshRef.current = planeMesh;

      const planeWire = new THREE.LineSegments(
        new THREE.EdgesGeometry(new THREE.PlaneGeometry(size, size, 1, 1)),
        new THREE.LineBasicMaterial({
          color: "#8fdbff",
          transparent: true,
          opacity: 0.7,
        }),
      );
      planeWire.renderOrder = 9;
      planeGroup.add(planeWire);
      planeWireRef.current = planeWire;

      const arrowLength = Math.max(3, size * 0.34);
      const normalArrow = new THREE.ArrowHelper(
        new THREE.Vector3(0, 0, 1),
        new THREE.Vector3(0, 0, 0),
        arrowLength,
        0x95e3ff,
        arrowLength * 0.16,
        arrowLength * 0.08,
      );
      planeGroup.add(normalArrow);
      planeNormalRef.current = normalArrow;
    }
  }, [activeExample, intersectionPlane, meshVisual, overlayCurves, sampledPoints]);

  useEffect(() => {
    if (gridRef.current) {
      gridRef.current.visible = showGrid;
    }
  }, [showGrid]);

  useEffect(() => {
    if (axesRef.current) {
      axesRef.current.visible = showAxes;
    }
  }, [showAxes]);

  useEffect(() => {
    if (controlsRef.current) {
      controlsRef.current.enabled = orbitEnabled && !isTransformDraggingRef.current;
    }
  }, [orbitEnabled]);

  const canExportIges = useMemo(() => capabilities.igesExport, [capabilities.igesExport]);
  const canImportIges = useMemo(() => capabilities.igesImport, [capabilities.igesImport]);

  const collectActiveObjectIds = useCallback((): number[] => {
    const idSet = new Set<number>();

    for (const h of exportHandlesRef.current) {
      idSet.add(h.objectId);
    }

    const ctx = landXmlContextRef.current;
    if (ctx) {
      try {
        idSet.add(ctx.docHandle.object_id());
      } catch { /* freed handle */ }
    }
    return Array.from(idSet);
  }, []);

  const onExportIges = useCallback(() => {
    const session = sessionRef.current;
    if (!session) return;
    try {
      const ids = collectActiveObjectIds();
      const igesText = (session as any).export_iges(new Float64Array(ids));
      downloadTextFile(igesText, "export.igs", "application/iges");
    } catch (err) {
      appendLog("error", `IGES export failed: ${err}`);
    }
  }, [collectActiveObjectIds, appendLog]);

  const onExportSat = useCallback(() => {
    const session = sessionRef.current;
    if (!session) return;
    try {
      const ids = collectActiveObjectIds();
      const satText = (session as any).export_sat(new Float64Array(ids));
      downloadTextFile(satText, "export.sat", "text/plain");
    } catch (err) {
      appendLog("error", `SAT export failed: ${err}`);
    }
  }, [collectActiveObjectIds, appendLog]);

  const onExportStl = useCallback(() => {
    const session = sessionRef.current;
    if (!session) return;
    try {
      const ids = collectActiveObjectIds();
      const stlText = (session as any).export_stl(new Float64Array(ids));
      downloadTextFile(stlText, "export.stl", "application/sla");
    } catch (err) {
      appendLog("error", `STL export failed: ${err}`);
    }
  }, [collectActiveObjectIds, appendLog]);

  const onExportGltf = useCallback(() => {
    const session = sessionRef.current;
    if (!session) return;
    try {
      const ids = collectActiveObjectIds();
      const gltfText = (session as any).export_gltf(new Float64Array(ids));
      downloadTextFile(gltfText, "export.gltf", "model/gltf+json");
    } catch (err) {
      appendLog("error", `glTF export failed: ${err}`);
    }
  }, [collectActiveObjectIds, appendLog]);

  const exportMode = useMemo((): "cad" | "mesh" => {
    return isMeshOnlyExample(activeExample) ? "mesh" : "cad";
  }, [activeExample]);

  const showSurfaceProbeControls = useMemo(() => activeExample === "surfaceUvEval", [activeExample]);
  const showProbeControls = useMemo(() => shouldShowProbeForExample(activeExample), [activeExample]);
  const showGizmoControls = useMemo(
    () =>
      activeExample === "meshTransform" ||
      activeExample === "meshBoolean" ||
      activeExample === "meshIntersectMeshPlane",
    [activeExample],
  );
  const showTransformTargetControls = useMemo(
    () =>
      (activeExample === "meshTransform" || activeExample === "meshBoolean") &&
      transformTargetsUi.length > 1,
    [activeExample, transformTargetsUi.length],
  );
  const showMeshPlaneTargetControls = useMemo(
    () => activeExample === "meshIntersectMeshPlane",
    [activeExample],
  );

  const onLandXmlAlignmentChange = useCallback(
    (idx: number) => {
      setLandXmlProbeAlignIdx(idx);
      setLandXmlProbeProfileIdx(0);
      updateLandXmlProbe(landXmlProbeUiState.stationNorm, idx, 0, true);
    },
    [landXmlProbeUiState.stationNorm, updateLandXmlProbe],
  );

  const onLandXmlProfileChange = useCallback(
    (idx: number) => {
      setLandXmlProbeProfileIdx(idx);
      updateLandXmlProbe(landXmlProbeUiState.stationNorm, landXmlProbeAlignIdx, idx, true);
    },
    [landXmlProbeAlignIdx, landXmlProbeUiState.stationNorm, updateLandXmlProbe],
  );

  const onLandXmlStationChange = useCallback(
    (stationNorm: number, commit: boolean) => {
      updateLandXmlProbe(stationNorm, landXmlProbeAlignIdx, landXmlProbeProfileIdx, commit);
    },
    [landXmlProbeAlignIdx, landXmlProbeProfileIdx, updateLandXmlProbe],
  );

  const onSaveSession = useCallback(() => {
    if (!preset) {
      return;
    }

    const snapshot = cameraSnapshot();
    if (!snapshot) {
      return;
    }

    const payload: ViewerSessionFile = {
      version: 1,
      preset,
      view: {
        camera: snapshot,
        showGrid,
        showAxes,
        orbitEnabled,
      },
    };

    downloadJson("rusted-geom-session.json", payload);
    setStatusMessage("Session saved");
  }, [cameraSnapshot, orbitEnabled, preset, showAxes, showGrid]);

  const onSaveScreenshot = useCallback(() => {
    const renderer = rendererRef.current;
    if (!renderer) {
      return;
    }

    downloadDataUrl("rusted-geom-view.png", renderer.domElement.toDataURL("image/png"));
    setStatusMessage("PNG snapshot saved");
  }, []);

  const onLoadSessionFile = useCallback(
    async (file: File): Promise<void> => {
      try {
        const text = await file.text();
        const parsed = parseViewerSession(JSON.parse(text));
        applySession(parsed);
      } catch (error) {
        setErrorMessage(error instanceof Error ? error.message : String(error));
      }
    },
    [applySession],
  );

  const onLoadSessionClick = useCallback(() => {
    sessionFileInputRef.current?.click();
  }, []);

  const onClearLogs = useCallback(() => {
    clearLogs();
    setStatusMessage("Console cleared");
  }, [clearLogs]);

  const onExportLogs = useCallback(() => {
    const filename = `rusted-geom-console-${fileSafeStamp()}.log`;
    downloadText(filename, formatLogsAsText(logs));
    setStatusMessage(`Console exported (${logs.length} entries)`);
  }, [logs]);

  const onExampleSelectionChange = useCallback(
    (value: string): void => {
      const next = parseExampleSelection(value);
      if (!next || next === activeExample) {
        return;
      }
      if (isAsyncExample(next)) {
        void updateLandXmlFile(activeLandXmlFile);
        return;
      }
      try {
        setLandXmlStats(null);
        updateCurveForExample(next, "Example switched");
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        setErrorMessage(message);
        appendLog("error", `Example switch failed: ${message}`);
      }
    },
    [activeExample, activeLandXmlFile, appendLog, updateCurveForExample, updateLandXmlFile],
  );

  const onExampleBrowserSelect = useCallback(
    (key: ExampleKey): void => {
      const next = key;
      if (!next || next === activeExample) return;
      if (isAsyncExample(next)) {
        void updateLandXmlFile(activeLandXmlFile);
        return;
      }
      try {
        setLandXmlStats(null);
        updateCurveForExample(next, "Example switched");
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        setErrorMessage(message);
        appendLog("error", `Example switch failed: ${message}`);
      }
    },
    [activeExample, activeLandXmlFile, appendLog, updateCurveForExample, updateLandXmlFile],
  );

  return (
    <div className="viewer-shell">
      <input
        ref={sessionFileInputRef}
        type="file"
        accept="application/json"
        className="hidden-input"
        onChange={(event) => {
          const file = event.target.files?.[0];
          if (file) {
            void onLoadSessionFile(file);
          }
          event.currentTarget.value = "";
        }}
      />

      {/* Boot overlay */}
      <div className={`kernel-boot-overlay ${kernelStatus !== "booting" ? "is-done" : ""}`}>
        <div className="welcome-card">
          <div className="welcome-card-mark">◈</div>
          <h1 className="welcome-card-title">rusted-geom</h1>
          <p className="welcome-card-caption">
            <span className="kernel-boot-spinner" />
            Loading geometry kernel…
          </p>
        </div>
      </div>

      <ViewerToolbar
        canImportIges={canImportIges}
        canExportIges={canExportIges}
        canExportSat={canExportIges}
        onLoadSession={onLoadSessionClick}
        onSaveSession={onSaveSession}
        onExportIges={onExportIges}
        onExportSat={onExportSat}
        onExportStl={onExportStl}
        onExportGltf={onExportGltf}
        exportMode={exportMode}
        landXmlScaleFactor={landXmlScaleFactor}
        onLandXmlScaleFactorChange={setLandXmlScaleFactor}
        showLandXmlScale={activeExample === "landxmlViewer"}
        orbitEnabled={orbitEnabled}
        showGrid={showGrid}
        showAxes={showAxes}
        cameraMode={cameraMode}
        onToggleCameraMode={toggleCameraMode}
        onApplyViewPreset={applyViewPreset}
        onZoomExtents={zoomExtents}
        onResetCamera={resetCamera}
        onToggleOrbit={() => setOrbitEnabled((v) => !v)}
        onToggleGrid={() => setShowGrid((v) => !v)}
        onToggleAxes={() => setShowAxes((v) => !v)}
        onSaveScreenshot={onSaveScreenshot}
        isInspectorOpen={isInspectorOpen}
        isConsoleOpen={isConsoleOpen}
        onToggleInspector={toggleInspector}
        onToggleConsole={toggleConsole}
        onClearLogs={onClearLogs}
        onExportLogs={onExportLogs}
        isDarkMode={isDarkMode}
        onToggleDarkMode={toggleDarkMode}
        onOpenExampleBrowser={() => setIsExampleBrowserOpen(true)}
      />

      <main className="viewer-main">
        <section className="viewport-wrap">
          <div ref={viewportRef} className="viewport" aria-label="Three.js viewport" />
        </section>
      </main>

      <InspectorPanel
        isOpen={isInspectorOpen}
        activeExample={activeExample}
        activeCurveName={activeCurveName}
        activeDegreeLabel={activeDegreeLabel}
        perfStats={perfStats}
        showGizmoControls={showGizmoControls}
        showTransformTargetControls={showTransformTargetControls}
        showMeshPlaneTargetControls={showMeshPlaneTargetControls}
        showSurfaceProbeControls={showSurfaceProbeControls}
        showProbeControls={showProbeControls}
        gizmoMode={gizmoMode}
        onSetGizmoMode={setGizmoMode}
        transformTargetsUi={transformTargetsUi}
        transformTargetKey={transformTargetKey}
        onTransformTargetChange={(key) => applyTransformTargetSelection(key, true)}
        meshPlaneTarget={meshPlaneTarget}
        onMeshPlaneTargetChange={setMeshPlaneTarget}
        probeUiState={probeUiState}
        onUpdateProbe={updateProbeForT}
        surfaceProbeUiState={surfaceProbeUiState}
        onUpdateSurfaceProbe={updateSurfaceProbeForUv}
        onOpenExampleBrowser={() => setIsExampleBrowserOpen(true)}
        activeLandXmlFile={activeLandXmlFile}
        onLandXmlFileChange={setActiveLandXmlFile}
        landXmlStats={landXmlStats}
        landXmlDatumOffset={landXmlDatumOffset}
        onLandXmlDatumOffsetChange={setLandXmlDatumOffset}
        landXmlVertExag={landXmlVertExag}
        onLandXmlVertExagChange={setLandXmlVertExag}
        landXmlZRange={landXmlZRange}
        landXmlAlignments={landXmlAlignments}
        landXmlProbeState={landXmlProbeUiState}
        landXmlProbeAlignIdx={landXmlProbeAlignIdx}
        landXmlProbeProfileIdx={landXmlProbeProfileIdx}
        onLandXmlAlignmentChange={onLandXmlAlignmentChange}
        onLandXmlProfileChange={onLandXmlProfileChange}
        onLandXmlStationChange={onLandXmlStationChange}
      />

      <KernelConsole
        isOpen={isConsoleOpen}
        logs={logs}
        onExportLogs={onExportLogs}
        onClearLogs={onClearLogs}
        activeFilter={consoleFilter}
        onFilterChange={setConsoleFilter}
      />

      <ExampleBrowser
        isOpen={isExampleBrowserOpen}
        activeExample={activeExample}
        onSelect={onExampleBrowserSelect}
        onClose={() => setIsExampleBrowserOpen(false)}
      />
    </div>
  );
}
