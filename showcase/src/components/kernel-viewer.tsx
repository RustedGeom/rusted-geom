"use client";

import {
  createKernelRuntime,
  type CurveHandle,
  type CurvePresetInput,
  type KernelRuntime,
  type KernelSession,
  type MeshHandle,
  type ObjectHandle,
  type SurfaceHandle,
} from "@rusted-geom/bindings-web";
import type {
  RgmArc3,
  RgmCircle3,
  RgmLine3,
  RgmNurbsSurfaceDesc,
  RgmPlane,
  RgmPoint3,
  RgmPolycurveSegment,
  RgmSurfaceTessellationOptions,
  RgmToleranceContext,
  RgmUv2,
  RgmVec3,
} from "@rusted-geom/bindings-web";
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
  CameraSnapshot,
  ExampleKey,
  GizmoMode,
  KernelStatus,
  LogEntry,
  LogLevel,
  MeshVisual,
  OverlayCurveVisual,
  ProbeUiState,
  SegmentOverlayVisual,
  SurfaceProbeUiState,
  TransformTarget,
  ViewerPerformance,
} from "@/lib/viewer-types";
import { ViewerToolbar } from "./viewer/toolbar/ViewerToolbar";
import { InspectorPanel } from "./viewer/inspector/InspectorPanel";
import { KernelConsole } from "./viewer/console/KernelConsole";
import { ExampleBrowser } from "./viewer/ExampleBrowser";

const DEFAULT_CAMERA_POSITION = new THREE.Vector3(10, 8, 11);
const DEFAULT_CAMERA_TARGET = new THREE.Vector3(0, 0, 0);
const MIN_RENDER_SAMPLES = 2048;
const MAX_RENDER_SAMPLES = 12000;
const MOBILE_MEDIA_QUERY = "(max-width: 880px)";


interface BuiltExample {
  kind: "curve" | "mesh";
  curveHandle: CurveHandle | null;
  ownedHandles: ObjectHandle[];
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

function clampedUniformKnots(controlCount: number, degree: number): number[] {
  const knotCount = controlCount + degree + 1;
  const knots = new Array(knotCount).fill(0);
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
): { desc: RgmNurbsSurfaceDesc; points: RgmPoint3[]; weights: number[]; knotsU: number[]; knotsV: number[] } {
  const points: RgmPoint3[] = [];
  const weights: number[] = [];
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
      points.push({ x, y, z });
      weights.push(1.0 + 0.08 * Math.sin((u + v) * Math.PI));
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
    points,
    weights,
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

function preview(values: number[], max = 12): string {
  if (values.length <= max) {
    return `[${values.map((v) => Number(v.toFixed(6))).join(", ")}]`;
  }
  const head = values.slice(0, Math.floor(max / 2)).map((v) => Number(v.toFixed(6)));
  const tail = values.slice(values.length - Math.floor(max / 2)).map((v) => Number(v.toFixed(6)));
  return `[${head.join(", ")}, ..., ${tail.join(", ")}]`;
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

function fallbackTolerance(): RgmToleranceContext {
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
    example !== "surfaceLarge" &&
    example !== "surfaceTransform" &&
    example !== "surfaceUvEval" &&
    example !== "surfaceIntersectSurface" &&
    example !== "surfaceIntersectPlane" &&
    example !== "surfaceIntersectCurve" &&
    example !== "trimEditWorkflow" &&
    example !== "trimValidationFailures"
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

function fitViewToPoints(
  camera: THREE.PerspectiveCamera,
  controls: OrbitControls,
  points: RgmPoint3[],
): void {
  if (points.length === 0) {
    return;
  }

  const bounds = new THREE.Box3();
  points.forEach((point) => {
    bounds.expandByPoint(new THREE.Vector3(point.x, point.y, point.z));
  });

  const sphere = bounds.getBoundingSphere(new THREE.Sphere());
  const distance = Math.max(4, sphere.radius * 2.8);
  camera.position.set(
    sphere.center.x + distance,
    sphere.center.y + distance * 0.55,
    sphere.center.z + distance,
  );
  controls.target.copy(sphere.center);
  controls.update();
}

export function KernelViewer() {
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const sessionFileInputRef = useRef<HTMLInputElement | null>(null);

  const runtimeRef = useRef<KernelRuntime | null>(null);
  const sessionRef = useRef<KernelSession | null>(null);
  const curveHandleRef = useRef<CurveHandle | null>(null);
  const ownedCurveHandlesRef = useRef<ObjectHandle[]>([]);
  const nurbsPresetRef = useRef<CurvePreset | null>(null);

  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const controlsRef = useRef<OrbitControls | null>(null);
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
  const planeMeshRef = useRef<THREE.Mesh<THREE.PlaneGeometry, THREE.MeshStandardMaterial> | null>(
    null,
  );
  const planeWireRef = useRef<THREE.LineSegments<THREE.EdgesGeometry, THREE.LineBasicMaterial> | null>(
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
  const [perfStats, setPerfStats] = useState<ViewerPerformance>({ loadMs: 0, intersectionMs: 0 });
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [, setStatusMessage] = useState("Booting kernel runtime...");
  const [, setErrorMessage] = useState<string | null>(null);
  const [capabilities, setCapabilities] = useState({ igesImport: false, igesExport: false });
  const [showGrid, setShowGrid] = useState(true);
  const [showAxes, setShowAxes] = useState(false);
  const [orbitEnabled, setOrbitEnabled] = useState(true);
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
    const session = sessionRef.current;
    if (!session) {
      return;
    }
    for (const handle of ownedCurveHandlesRef.current) {
      session.kernel.releaseObject(handle);
    }
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
      session.kernel.releaseObject(previewMeshHandleRef.current);
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
        ownedHandles: ObjectHandle[],
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
        logs,
      });

      if (example === "nurbs") {
        const presetToUse = nurbsPresetOverride ?? nurbsPresetRef.current;
        if (!presetToUse) {
          throw new Error("NURBS preset is not loaded");
        }
        const handle = session.curve.buildCurveFromPreset(presetToUse as CurvePresetInput);
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
        const handle = session.curve.createLine(line, tol);
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
        const handle = session.curve.createPolyline(points, false, tol);
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
        const handle = session.curve.createArc(arc, tol);
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
        const handle = session.curve.createCircle(circle, tol);
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

      if (example === "intersectCurveCurve") {
        const builtHandles: ObjectHandle[] = [];
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

          const primaryHandle = session.curve.createCircle(circlePrimary, tol);
          builtHandles.push(primaryHandle);
          const secondaryHandle = session.curve.createCircle(circleSecondary, tol);
          builtHandles.push(secondaryHandle);

          const secondarySamples = session.curve.sampleCurvePolyline(secondaryHandle, 2400);
          const hits = session.intersection.intersectCurveCurve(primaryHandle, secondaryHandle);
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
          for (const handle of builtHandles) {
            session.kernel.releaseObject(handle);
          }
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

        const curvePreset: CurvePresetInput = {
          name: "Woven Plane-Crossing NURBS",
          degree: 3,
          closed: false,
          points: fitPoints,
          tolerance: tol,
        };
        const curveHandle = session.curve.buildCurveFromPreset(curvePreset);
        const hits = session.intersection.intersectCurvePlane(curveHandle, plane);
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
        const mesh = session.mesh.createMeshTorus(
          { x: 0, y: 0, z: 0 },
          6.0,
          1.35,
          240,
          160,
        );
        const buffers = session.mesh.meshToBuffers(mesh);
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
            `mesh vertices=${session.mesh.meshVertexCount(mesh)}`,
            `mesh triangles=${session.mesh.meshTriangleCount(mesh)}`,
          ],
        };
      }

      if (example === "meshTransform") {
        const built: ObjectHandle[] = [];
        try {
          const base = session.mesh.createMeshBox({ x: 0.0, y: 0.0, z: -1.0 }, { x: 7.2, y: 2.6, z: 1.2 });
          built.push(base);
          const rotor = session.mesh.createMeshTorus({ x: 0, y: 0, z: 0 }, 2.0, 0.52, 108, 72);
          built.push(rotor);

          const baseBuffers = session.mesh.meshToBuffers(base);
          const rotorBuffers = session.mesh.meshToBuffers(rotor);
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
              `base triangles=${session.mesh.meshTriangleCount(base)}`,
              `rotor triangles=${session.mesh.meshTriangleCount(rotor)}`,
              "Use target selector + gizmo mode to transform either fixture or rotor.",
              "Each drag commit updates the kernel mesh and refreshes geometry from kernel buffers.",
            ],
          };
        } catch (error) {
          for (const handle of built) {
            session.kernel.releaseObject(handle);
          }
          throw error;
        }
      }

      if (example === "meshIntersectMeshMesh") {
        const built: ObjectHandle[] = [];
        try {
          const sphere = session.mesh.createMeshUvSphere({ x: 0, y: 0, z: 0 }, 4.6, 56, 40);
          built.push(sphere);
          const torus = session.mesh.createMeshTorus({ x: 0.5, y: 0.2, z: 0.1 }, 4.2, 1.15, 92, 64);
          built.push(torus);
          const intersectionStart = performance.now();
          const hits = session.intersection.intersectMeshMesh(sphere, torus);
          const intersectionMs = performance.now() - intersectionStart;
          const sphereBuffers = session.mesh.meshToBuffers(sphere);
          const torusBuffers = session.mesh.meshToBuffers(torus);
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
          for (const handle of built) {
            session.kernel.releaseObject(handle);
          }
          throw error;
        }
      }

      if (example === "meshIntersectMeshPlane") {
        const mesh = session.mesh.createMeshTorus({ x: 0.4, y: -0.2, z: 0.7 }, 5.1, 1.3, 128, 72);
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
        const hits = session.intersection.intersectMeshPlane(mesh, plane);
        const intersectionMs = performance.now() - intersectionStart;
        const meshBuffers = session.mesh.meshToBuffers(mesh);
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
            `mesh triangles=${session.mesh.meshTriangleCount(mesh)}`,
            `mesh-plane segment pairs=${Math.floor(hits.length / 2)}`,
            `intersection solve=${intersectionMs.toFixed(2)}ms`,
          ],
        };
      }

      if (example === "meshBoolean") {
        const built: ObjectHandle[] = [];
        try {
          const outer = session.mesh.createMeshBox({ x: 0, y: 0, z: 0 }, { x: 9.0, y: 9.0, z: 9.0 });
          built.push(outer);
          const inner = session.mesh.createMeshTorus({ x: 2.2, y: 0.0, z: 0.0 }, 2.8, 0.95, 72, 52);
          built.push(inner);
          const result = session.mesh.meshBoolean(outer, inner, 2);
          built.push(result);
          const outerBuffers = session.mesh.meshToBuffers(outer);
          const innerBuffers = session.mesh.meshToBuffers(inner);
          const resultBuffers = session.mesh.meshToBuffers(result);
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
              opacity: 0.28,
              wireframe: true,
              name: "subtracted solid (B): torus (active target)",
            },
            overlayMeshes: [
              {
                vertices: outerBuffers.vertices,
                indices: outerBuffers.indices,
                color: "#8aa2ba",
                opacity: 0.12,
                wireframe: true,
                name: "base solid (A): box",
              },
              {
                vertices: resultBuffers.vertices,
                indices: resultBuffers.indices,
                color: "#8ac6ff",
                opacity: 0.82,
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
                opacity: 0.22,
                wireframe: true,
              },
              {
                key: "tool",
                label: "Subtracted solid (B): torus",
                handle: inner,
                color: "#f7ba74",
                opacity: 0.28,
                wireframe: true,
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
              `outer triangles=${session.mesh.meshTriangleCount(outer)}`,
              `inner triangles=${session.mesh.meshTriangleCount(inner)}`,
              `result triangles=${session.mesh.meshTriangleCount(result)}`,
            ],
          };
        } catch (error) {
          for (const handle of built) {
            session.kernel.releaseObject(handle);
          }
          throw error;
        }
      }

      if (example === "surfaceLarge") {
        const built: ObjectHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(28, 24, 18, 14, 1.6);
          const surface = session.surface.createNurbsSurface(
            net.desc,
            net.points,
            net.weights,
            net.knotsU,
            net.knotsV,
            tol,
          );
          built.push(surface);
          const mesh = session.surface.surfaceTessellateToMesh(surface, {
            min_u_segments: 72,
            min_v_segments: 56,
            max_u_segments: 96,
            max_v_segments: 72,
            chord_tol: 1e-4,
            normal_tol_rad: 0.04,
          });
          built.push(mesh);
          const buffers = session.mesh.meshToBuffers(mesh);
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
              `triangles=${session.mesh.meshTriangleCount(mesh)}`,
            ],
          };
        } catch (error) {
          for (const handle of built) {
            session.kernel.releaseObject(handle);
          }
          throw error;
        }
      }

      if (example === "surfaceTransform") {
        const built: ObjectHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(16, 14, 12, 10, 1.1);
          const surface = session.surface.createNurbsSurface(
            net.desc,
            net.points,
            net.weights,
            net.knotsU,
            net.knotsV,
            tol,
          );
          built.push(surface);
          const moved = session.surface.surfaceTranslate(surface, { x: 1.4, y: -0.7, z: 0.9 });
          built.push(moved);
          const rotated = session.surface.surfaceRotate(
            moved,
            { x: 0.4, y: 1.0, z: 0.2 },
            0.68,
            { x: 0, y: 0, z: 0 },
          );
          built.push(rotated);
          const scaled = session.surface.surfaceScale(
            rotated,
            { x: 1.15, y: 0.82, z: 1.3 },
            { x: 0.5, y: -0.2, z: 0.1 },
          );
          built.push(scaled);
          const baseMesh = session.surface.surfaceTessellateToMesh(surface);
          const transformedMesh = session.surface.surfaceTessellateToMesh(scaled);
          built.push(baseMesh, transformedMesh);
          const baseBuffers = session.mesh.meshToBuffers(baseMesh);
          const transformedBuffers = session.mesh.meshToBuffers(transformedMesh);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
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
              `base triangles=${session.mesh.meshTriangleCount(baseMesh)}`,
              `transformed triangles=${session.mesh.meshTriangleCount(transformedMesh)}`,
              "Transform APIs used: surfaceTranslate, surfaceRotate, surfaceScale",
            ],
          };
        } catch (error) {
          for (const handle of built) {
            session.kernel.releaseObject(handle);
          }
          throw error;
        }
      }

      if (example === "surfaceUvEval") {
        const built: ObjectHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(22, 19, 16, 14, 1.55);
          const weights = net.weights.map((base, idx) =>
            Math.max(0.22, base * (1 + 0.2 * Math.sin(idx * 0.37) + 0.08 * Math.cos(idx * 0.19))),
          );
          const surfaceBase = session.surface.createNurbsSurface(
            net.desc,
            net.points,
            weights,
            net.knotsU,
            net.knotsV,
            tol,
          );
          built.push(surfaceBase);

          const surfaceRot = session.surface.surfaceRotate(
            surfaceBase,
            { x: 0.48, y: 1.0, z: 0.31 },
            0.62,
            { x: 0.3, y: -0.1, z: 0.2 },
          );
          built.push(surfaceRot);

          const surface = session.surface.surfaceTranslate(surfaceRot, { x: 0.9, y: -0.6, z: 0.5 });
          built.push(surface);

          const tessOptions: RgmSurfaceTessellationOptions = {
            min_u_segments: 30,
            min_v_segments: 26,
            max_u_segments: 54,
            max_v_segments: 48,
            chord_tol: 1.8e-4,
            normal_tol_rad: 0.075,
          };
          const mesh = session.surface.surfaceTessellateToMesh(surface, tessOptions);
          built.push(mesh);
          const buffers = session.mesh.meshToBuffers(mesh);

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
            `triangles=${session.mesh.meshTriangleCount(mesh)}`,
            "Use the Surface Probe sliders to move a UV probe and inspect D0/D1 (+D2 when available).",
            "Arrow colors: du=orange, dv=cyan, duu=peach, duv=violet, dvv=blue.",
          ];

          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
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
          for (const handle of built) {
            session.kernel.releaseObject(handle);
          }
          throw error;
        }
      }

      if (example === "surfaceIntersectSurface") {
        const built: ObjectHandle[] = [];
        try {
          const a = buildWarpedSurfaceNet(16, 15, 12, 10, 1.0);
          const b = buildWarpedSurfaceNet(15, 16, 11, 11, 1.25);
          const surfaceA = session.surface.createNurbsSurface(a.desc, a.points, a.weights, a.knotsU, a.knotsV, tol);
          const surfaceB0 = session.surface.createNurbsSurface(b.desc, b.points, b.weights, b.knotsU, b.knotsV, tol);
          built.push(surfaceA, surfaceB0);
          const surfaceB = session.surface.surfaceRotate(
            session.surface.surfaceTranslate(surfaceB0, { x: 0.6, y: 0.3, z: -0.1 }),
            { x: 0.3, y: 1.0, z: 0.2 },
            0.72,
            { x: 0, y: 0, z: 0 },
          );
          built.push(surfaceB);
          const meshA = session.surface.surfaceTessellateToMesh(surfaceA, {
            min_u_segments: 18,
            min_v_segments: 18,
            max_u_segments: 42,
            max_v_segments: 42,
            chord_tol: 2.5e-4,
            normal_tol_rad: 0.1,
          });
          const meshB = session.surface.surfaceTessellateToMesh(surfaceB, {
            min_u_segments: 18,
            min_v_segments: 18,
            max_u_segments: 42,
            max_v_segments: 42,
            chord_tol: 2.5e-4,
            normal_tol_rad: 0.1,
          });
          built.push(meshA, meshB);
          const intersectionStart = performance.now();
          const inter = session.intersection.intersectSurfaceSurface(surfaceA, surfaceB);
          const intersectionMs = performance.now() - intersectionStart;
          built.push(inter);

          const branchCount = session.intersection.intersectionBranchCount(inter);
          const segmentPts: RgmPoint3[] = [];
          for (let bi = 0; bi < branchCount; bi += 1) {
            const branch = session.intersection.intersectionBranchPoints(inter, bi);
            for (let i = 1; i < branch.length; i += 1) {
              segmentPts.push(branch[i - 1], branch[i]);
            }
          }
          const buffersA = session.mesh.meshToBuffers(meshA);
          const buffersB = session.mesh.meshToBuffers(meshB);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
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
          for (const handle of built) {
            session.kernel.releaseObject(handle);
          }
          throw error;
        }
      }

      if (example === "surfaceIntersectPlane") {
        const built: ObjectHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(18, 16, 13, 11, 1.35);
          const surface = session.surface.createNurbsSurface(
            net.desc,
            net.points,
            net.weights,
            net.knotsU,
            net.knotsV,
            tol,
          );
          built.push(surface);
          const mesh = session.surface.surfaceTessellateToMesh(surface, {
            min_u_segments: 18,
            min_v_segments: 18,
            max_u_segments: 42,
            max_v_segments: 42,
            chord_tol: 2.5e-4,
            normal_tol_rad: 0.1,
          });
          built.push(mesh);
          const plane: RgmPlane = {
            origin: { x: 0.2, y: -0.4, z: 0.25 },
            x_axis: { x: 1.0, y: 0.1, z: -0.1 },
            y_axis: { x: -0.1, y: 0.94, z: 0.32 },
            z_axis: { x: 0.12, y: -0.31, z: 0.94 },
          };
          const intersectionStart = performance.now();
          const inter = session.intersection.intersectSurfacePlane(surface, plane);
          const intersectionMs = performance.now() - intersectionStart;
          built.push(inter);
          const branchCount = session.intersection.intersectionBranchCount(inter);
          const segments: RgmPoint3[] = [];
          for (let bi = 0; bi < branchCount; bi += 1) {
            const branch = session.intersection.intersectionBranchPoints(inter, bi);
            for (let i = 1; i < branch.length; i += 1) {
              segments.push(branch[i - 1], branch[i]);
            }
          }
          const buffers = session.mesh.meshToBuffers(mesh);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
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
          for (const handle of built) {
            session.kernel.releaseObject(handle);
          }
          throw error;
        }
      }

      if (example === "surfaceIntersectCurve") {
        const built: ObjectHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(16, 16, 12, 12, 1.2);
          const surface = session.surface.createNurbsSurface(
            net.desc,
            net.points,
            net.weights,
            net.knotsU,
            net.knotsV,
            tol,
          );
          built.push(surface);
          const curveHandle = session.curve.buildCurveFromPreset({
            degree: 3,
            closed: false,
            points: [
              { x: -6.2, y: -3.4, z: -2.0 },
              { x: -3.1, y: -0.2, z: 2.5 },
              { x: -0.5, y: 2.8, z: -1.8 },
              { x: 2.2, y: 1.1, z: 2.2 },
              { x: 4.8, y: -1.6, z: -2.3 },
              { x: 6.1, y: 2.3, z: 1.9 },
            ],
            tolerance: tol,
          });
          built.push(curveHandle);
          const mesh = session.surface.surfaceTessellateToMesh(surface, {
            min_u_segments: 18,
            min_v_segments: 18,
            max_u_segments: 42,
            max_v_segments: 42,
            chord_tol: 2.5e-4,
            normal_tol_rad: 0.1,
          });
          built.push(mesh);
          const intersectionStart = performance.now();
          const inter = session.intersection.intersectSurfaceCurve(surface, curveHandle);
          const intersectionMs = performance.now() - intersectionStart;
          built.push(inter);
          const hits: RgmPoint3[] = [];
          const branchCount = session.intersection.intersectionBranchCount(inter);
          for (let bi = 0; bi < branchCount; bi += 1) {
            hits.push(...session.intersection.intersectionBranchPoints(inter, bi));
          }
          const curveSamples = session.curve.sampleCurvePolyline(curveHandle, 3600);
          const buffers = session.mesh.meshToBuffers(mesh);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
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
          for (const handle of built) {
            session.kernel.releaseObject(handle);
          }
          throw error;
        }
      }

      if (example === "trimEditWorkflow") {
        const built: ObjectHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(14, 12, 10, 9, 0.95);
          const surface = session.surface.createNurbsSurface(
            net.desc,
            net.points,
            net.weights,
            net.knotsU,
            net.knotsV,
            tol,
          );
          built.push(surface);
          const face = session.face.createFaceFromSurface(surface);
          built.push(face);
          session.face.faceAddLoop(face, rectangleLoopUV(0.05, 0.95, 0.08, 0.92), true);
          session.face.faceAddLoop(face, rectangleLoopUV(0.35, 0.65, 0.35, 0.65), false);
          session.face.faceSplitTrimEdge(face, 0, 1, 0.42);
          session.face.faceReverseLoop(face, 1);
          session.face.faceHeal(face);
          const valid = session.face.faceValidate(face);
          const mesh = session.face.faceTessellateToMesh(face);
          built.push(mesh);
          const buffers = session.mesh.meshToBuffers(mesh);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
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
            logs: [`face valid=${valid}`, `triangles=${session.mesh.meshTriangleCount(mesh)}`],
          };
        } catch (error) {
          for (const handle of built) {
            session.kernel.releaseObject(handle);
          }
          throw error;
        }
      }

      if (example === "trimValidationFailures") {
        const built: ObjectHandle[] = [];
        try {
          const net = buildWarpedSurfaceNet(12, 10, 9, 8, 0.7);
          const surface = session.surface.createNurbsSurface(
            net.desc,
            net.points,
            net.weights,
            net.knotsU,
            net.knotsV,
            tol,
          );
          built.push(surface);
          const face = session.face.createFaceFromSurface(surface);
          built.push(face);
          session.face.faceAddLoop(face, rectangleLoopUV(0.1, 0.92, 0.1, 0.9), true);
          session.face.faceAddLoop(face, rectangleLoopUV(0.22, 0.48, 0.22, 0.48), true);
          const before = session.face.faceValidate(face);
          session.face.faceHeal(face);
          const after = session.face.faceValidate(face);
          const mesh = session.face.faceTessellateToMesh(face);
          built.push(mesh);
          const buffers = session.mesh.meshToBuffers(mesh);
          return {
            kind: "mesh",
            curveHandle: null,
            ownedHandles: built,
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
          for (const handle of built) {
            session.kernel.releaseObject(handle);
          }
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

      const builtHandles: ObjectHandle[] = [];
      try {
        const hLineA = session.curve.createLine(lineA, tol);
        builtHandles.push(hLineA);
        const hArcA = session.curve.createArc(arcA, tol);
        builtHandles.push(hArcA);
        const hLineB = session.curve.createLine(lineB, tol);
        builtHandles.push(hLineB);
        const hArcB = session.curve.createArc(arcB, tol);
        builtHandles.push(hArcB);

        const segments: RgmPolycurveSegment[] = [
          { curve: hLineA, reversed: false },
          { curve: hArcA, reversed: false },
          { curve: hLineB, reversed: false },
          { curve: hArcB, reversed: false },
        ];
        const poly = session.curve.createPolycurve(segments, tol);
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
        for (const handle of builtHandles) {
          session.kernel.releaseObject(handle);
        }
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
      surfaceProbeD1ScaleRef.current = built.surfaceProbeD1Scale ?? 0.2;
      surfaceProbeD2ScaleRef.current = built.surfaceProbeD2Scale ?? 0.1;

      for (const line of built.logs) {
        appendLog("debug", line);
      }

      let curveSamples: RgmPoint3[] = [];
      let totalLength = 0;
      if (built.kind === "curve" && built.curveHandle !== null) {
        curveSamples = session.curve.sampleCurvePolyline(built.curveHandle, built.renderSamples);
        totalLength = session.curve.curveLength(built.curveHandle);
        totalLengthRef.current = totalLength;
        const evaluatedProbe = session.curve.curvePointAt(built.curveHandle, probeTNormRef.current);
        const probeLength = session.curve.curveLengthAt(built.curveHandle, probeTNormRef.current);

        probePointRef.current = evaluatedProbe;
        if (probeRef.current) {
          probeRef.current.position.set(evaluatedProbe.x, evaluatedProbe.y, evaluatedProbe.z);
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
      setPerfStats({ loadMs, intersectionMs: built.intersectionMs });
      const intersectionSummary =
        built.intersectionPoints.length > 0
          ? ` • intersections ${built.intersectionPoints.length}`
          : "";
      const meshSummary =
        built.kind === "mesh" && built.meshVisual
          ? ` • triangles ${Math.floor(built.meshVisual.indices.length / 3)}`
          : "";
      const perfSummary =
        built.intersectionMs > 0
          ? ` • load ${loadMs.toFixed(2)}ms • intersection ${built.intersectionMs.toFixed(2)}ms`
          : ` • load ${loadMs.toFixed(2)}ms`;
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
          const points =
            curveSamples.length > 0
              ? curveSamples
              : [
                  ...(built.meshVisual?.vertices ?? []),
                  ...built.overlayMeshes.flatMap((visual) => visual.vertices),
                ];
          fitViewToPoints(camera, controls, points);
        });
      }
      appendLog(
        "info",
        `Built handles=${built.ownedHandles.length} intersections=${built.intersectionPoints.length} kind=${built.kind} load=${loadMs.toFixed(2)}ms`,
      );
    },
    [appendLog, buildExampleCurve, releaseOwnedCurveHandles],
  );

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
      fov: camera.fov,
    };
  }, []);

  const zoomExtents = useCallback((): void => {
    const camera = cameraRef.current;
    const controls = controlsRef.current;
    if (!camera || !controls) {
      return;
    }

    const allPoints =
      sampledPoints.length > 0
        ? sampledPoints
        : [
            ...(meshVisual?.vertices ?? []),
            ...overlayMeshes.flatMap((visual) => visual.vertices),
          ];
    fitViewToPoints(camera, controls, allPoints);
  }, [meshVisual, overlayMeshes, sampledPoints]);

  const resetCamera = useCallback((): void => {
    const camera = cameraRef.current;
    const controls = controlsRef.current;
    if (!camera || !controls) {
      return;
    }

    camera.position.copy(DEFAULT_CAMERA_POSITION);
    controls.target.copy(DEFAULT_CAMERA_TARGET);
    camera.up.set(0, 1, 0);
    controls.update();
  }, []);

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
        const uv = { u, v };
        const point = liveSession.surface.surfacePointAt(liveSurfaceHandle, uv);
        const frame = liveSession.surface.surfaceFrameAt(liveSurfaceHandle, uv);
        const du = frame.du;
        const dv = frame.dv;
        const normal = frame.normal;

        let hasD2 = false;
        let duu: RgmVec3 = { x: 0, y: 0, z: 0 };
        let duv: RgmVec3 = { x: 0, y: 0, z: 0 };
        let dvv: RgmVec3 = { x: 0, y: 0, z: 0 };
        const d2At = (
          liveSession as KernelSession & {
            surfaceD2At?: (
              surfaceHandle: bigint,
              uvNorm: RgmUv2,
            ) => { duu: RgmVec3; duv: RgmVec3; dvv: RgmVec3 };
          }
        ).surfaceD2At;
        if (typeof d2At === "function") {
          try {
            const d2 = d2At.call(liveSession, liveSurfaceHandle, uv);
            duu = d2.duu;
            duv = d2.duv;
            dvv = d2.dvv;
            hasD2 = true;
          } catch {
            hasD2 = false;
          }
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
        const point = liveSession.curve.curvePointAt(liveCurveHandle, next);
        const probeLength = liveSession.curve.curveLengthAt(liveCurveHandle, next);
        const totalLength = totalLengthRef.current;

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
        });
        setErrorMessage(null);

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
        const primaryBuffers = session.mesh.meshToBuffers(selected.handle);
        const resultBuffers = session.mesh.meshToBuffers(resultHandle);
        const overlays: MeshVisual[] = options
          .filter((target) => target.key !== nextKey)
          .map((target) => {
            const buffers = session.mesh.meshToBuffers(target.handle);
            return {
              vertices: buffers.vertices,
              indices: buffers.indices,
              color: target.color,
              opacity: Math.max(0.14, target.opacity * 0.6),
              wireframe: true,
              name: target.label,
            } satisfies MeshVisual;
          });
        overlays.push({
          vertices: resultBuffers.vertices,
          indices: resultBuffers.indices,
          color: "#8ac6ff",
          opacity: 0.84,
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
          wireframe: true,
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

      const primaryBuffers = session.mesh.meshToBuffers(selected.handle);
      const overlays = options
        .filter((target) => target.key !== nextKey)
        .map((target) => {
          const buffers = session.mesh.meshToBuffers(target.handle);
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
      const hits = session.intersection.intersectMeshPlane(meshHandle, plane);
      const intersectionMs = performance.now() - start;
      const triangleCount = session.mesh.meshTriangleCount(meshHandle);

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
          const hits = session.intersection.intersectMeshPlane(meshHandle, livePlane);
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
          session.kernel.releaseObject(previewMeshHandleRef.current);
          previewMeshHandleRef.current = null;
        }
        let previewHandle: MeshHandle;
        if (delta.kind === "translate") {
          previewHandle = session.mesh.meshTranslate(baseMeshHandle, delta.delta);
        } else if (delta.kind === "rotate") {
          previewHandle = session.mesh.meshRotate(baseMeshHandle, delta.axis, delta.angle, delta.pivot);
        } else {
          previewHandle = session.mesh.meshScale(baseMeshHandle, delta.scale, delta.pivot);
        }
        previewMeshHandleRef.current = previewHandle;

        const start = performance.now();
        const hits = session.intersection.intersectMeshPlane(previewHandle, plane);
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
      session.kernel.releaseObject(previewMeshHandleRef.current);
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
        nextHandle = session.mesh.meshTranslate(meshHandle, delta.delta);
      } else if (delta.kind === "rotate") {
        nextHandle = session.mesh.meshRotate(meshHandle, delta.axis, delta.angle, delta.pivot);
      } else {
        nextHandle = session.mesh.meshScale(meshHandle, delta.scale, delta.pivot);
      }

      const triangleCount = session.mesh.meshTriangleCount(nextHandle);
      session.kernel.releaseObject(meshHandle);
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
        const nextResult = session.mesh.meshBoolean(baseHandle, toolHandle, 2);
        const csgMs = performance.now() - csgStart;
        const resultTriangles = session.mesh.meshTriangleCount(nextResult);

        session.kernel.releaseObject(previousResult);
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
        const buffers = session.mesh.meshToBuffers(nextHandle);
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

      const camera = cameraRef.current;
      const controls = controlsRef.current;
      if (camera && controls) {
        camera.position.copy(fromPoint3(sessionFile.view.camera.position));
        camera.up.copy(fromPoint3(sessionFile.view.camera.up));
        camera.fov = sessionFile.view.camera.fov;
        camera.updateProjectionMatrix();
        controls.target.copy(fromPoint3(sessionFile.view.camera.target));
        controls.update();
      }
      suppressAutoFitRef.current = false;
    },
    [updateCurveForExample],
  );

  useEffect(() => {
    let disposed = false;

    (async () => {
      try {
        appendLog("info", "Loading kernel WASM runtime");
        const runtime = await createKernelRuntime("/wasm/rusted_geom.wasm");
        const session = runtime.createSession();
        appendLog("info", `Kernel session created: ${session.kernel.handle.toString()}`);
        const loadedPreset = await loadDefaultPreset();
        if (disposed) {
          session.destroy();
          runtime.destroy();
          return;
        }

        runtimeRef.current = runtime;
        sessionRef.current = session;
        setCapabilities({
          igesImport: runtime.capabilities.igesImport,
          igesExport: runtime.capabilities.igesExport,
        });
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
      sessionRef.current?.destroy();
      runtimeRef.current?.destroy();
      appendLog("info", "Kernel runtime destroyed");
      sessionRef.current = null;
      runtimeRef.current = null;
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

    const camera = new THREE.PerspectiveCamera(
      46,
      viewport.clientWidth / Math.max(1, viewport.clientHeight),
      0.01,
      1200,
    );
    camera.position.copy(DEFAULT_CAMERA_POSITION);

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

    const controls = new OrbitControls(camera, renderCanvas);
    controls.enableDamping = true;
    controls.target.copy(DEFAULT_CAMERA_TARGET);
    controls.update();

    const grid = new THREE.GridHelper(30, 30, "#8596b6", "#b9c4d8");
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
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
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
        renderer.render(scene, camera);
      }
    };
    animate();
    onResize();

    sceneRef.current = scene;
    cameraRef.current = camera;
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
    const material = new THREE.MeshStandardMaterial({
      color: meshVisual.color,
      transparent: meshVisual.opacity < 1,
      opacity: meshVisual.opacity,
      roughness: 0.5,
      metalness: 0.08,
      side: THREE.DoubleSide,
    });
    const mesh = new THREE.Mesh(geometry, material);
    if (origin) {
      mesh.position.copy(origin);
    }
    mesh.renderOrder = 18;
    scene.add(mesh);
    meshRef.current = mesh;

    if (meshVisual.wireframe) {
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
          liveSession.kernel.releaseObject(previewMeshHandleRef.current);
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

    for (const visual of overlayMeshes) {
      const geometry = createMeshGeometry(visual.vertices, visual.indices);
      const material = new THREE.MeshStandardMaterial({
        color: visual.color,
        transparent: visual.opacity < 1,
        opacity: visual.opacity,
        roughness: 0.55,
        metalness: 0.05,
        side: THREE.DoubleSide,
        depthWrite: false,
      });
      const mesh = new THREE.Mesh(geometry, material);
      mesh.renderOrder = 14;
      scene.add(mesh);

      let wire: THREE.LineSegments<THREE.WireframeGeometry, THREE.LineBasicMaterial> | null = null;
      if (visual.wireframe) {
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
  }, [overlayMeshes]);

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

    for (const hit of intersectionPoints) {
      const marker = new THREE.Mesh(
        new THREE.SphereGeometry(0.25, 20, 20),
        new THREE.MeshStandardMaterial({
          color: "#ff8fd9",
          emissive: "#7e2f67",
          emissiveIntensity: 0.64,
          roughness: 0.18,
          metalness: 0.2,
          depthWrite: false,
        }),
      );
      marker.position.set(hit.x, hit.y, hit.z);
      marker.renderOrder = 40;
      scene.add(marker);
      intersectionMarkerRefs.current.push(marker);
    }
  }, [intersectionPoints]);

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

    const frame = buildPlaneFrame(intersectionPlane);
    const referencePoints =
      sampledPoints.length > 0 ? sampledPoints : (meshVisual?.vertices ?? []);
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
  }, [intersectionPlane, meshVisual, sampledPoints]);

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
      try {
        updateCurveForExample(next, "Example switched");
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        setErrorMessage(message);
        appendLog("error", `Example switch failed: ${message}`);
      }
    },
    [activeExample, appendLog, updateCurveForExample],
  );

  const onExampleBrowserSelect = useCallback(
    (key: ExampleKey): void => {
      const next = key;
      if (!next || next === activeExample) return;
      try {
        updateCurveForExample(next, "Example switched");
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        setErrorMessage(message);
        appendLog("error", `Example switch failed: ${message}`);
      }
    },
    [activeExample, appendLog, updateCurveForExample],
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
        onLoadSession={onLoadSessionClick}
        onSaveSession={onSaveSession}
        orbitEnabled={orbitEnabled}
        showGrid={showGrid}
        showAxes={showAxes}
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
