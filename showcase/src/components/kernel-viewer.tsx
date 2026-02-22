"use client";

import {
  createKernelRuntime,
  type CurvePresetInput,
  type KernelRuntime,
  type KernelSession,
} from "@rusted-geom/bindings-web";
import type {
  RgmArc3,
  RgmCircle3,
  RgmLine3,
  RgmPlane,
  RgmPoint3,
  RgmPolycurveSegment,
  RgmToleranceContext,
} from "@rusted-geom/bindings-web";
import { type ReactNode, useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { Line2 } from "three/examples/jsm/lines/Line2.js";
import { LineGeometry } from "three/examples/jsm/lines/LineGeometry.js";
import { LineMaterial } from "three/examples/jsm/lines/LineMaterial.js";

import {
  parseCurvePreset,
  parseViewerSession,
  type CurvePreset,
  type ViewerSessionFile,
} from "@/lib/preset-schema";

const DEFAULT_CAMERA_POSITION = new THREE.Vector3(10, 8, 11);
const DEFAULT_CAMERA_TARGET = new THREE.Vector3(0, 0, 0);
const MIN_RENDER_SAMPLES = 2048;
const MAX_RENDER_SAMPLES = 12000;
const MOBILE_MEDIA_QUERY = "(max-width: 880px)";

interface CameraSnapshot {
  position: RgmPoint3;
  target: RgmPoint3;
  up: RgmPoint3;
  fov: number;
}

type LogLevel = "info" | "debug" | "error";

interface LogEntry {
  id: number;
  level: LogLevel;
  time: string;
  message: string;
}

interface ProbeUiState {
  tNorm: number;
  x: number;
  y: number;
  z: number;
  probeLength: number;
  totalLength: number;
}

type ExampleKey =
  | "nurbs"
  | "line"
  | "polyline"
  | "polycurve"
  | "arc"
  | "circle"
  | "intersectCurveCurve"
  | "intersectCurvePlane";

const EXAMPLE_OPTIONS: Record<string, ExampleKey> = {
  "NURBS (fit points)": "nurbs",
  "Line (3D skew)": "line",
  "Polyline (spatial)": "polyline",
  "Polycurve (mixed)": "polycurve",
  "Arc (tilted)": "arc",
  "Circle (tilted)": "circle",
  "Intersection (curve-curve)": "intersectCurveCurve",
  "Intersection (curve-plane)": "intersectCurvePlane",
};

interface OverlayCurveVisual {
  points: RgmPoint3[];
  color: string;
  width: number;
  opacity: number;
  name: string;
}

interface BuiltCurveExample {
  curveHandle: bigint;
  ownedHandles: bigint[];
  name: string;
  degreeLabel: string;
  renderDegree: number;
  renderSamples: number;
  overlayCurves: OverlayCurveVisual[];
  intersectionPoints: RgmPoint3[];
  planeVisual: RgmPlane | null;
  logs: string[];
}

function parseExampleSelection(value: unknown): ExampleKey | null {
  const raw = String(value);
  if (
    raw === "nurbs" ||
    raw === "line" ||
    raw === "polyline" ||
    raw === "polycurve" ||
    raw === "arc" ||
    raw === "circle" ||
    raw === "intersectCurveCurve" ||
    raw === "intersectCurvePlane"
  ) {
    return raw;
  }

  const mapped = EXAMPLE_OPTIONS[raw];
  return mapped ?? null;
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

function ToolIcon({ children }: { children: ReactNode }) {
  return (
    <svg
      className="tool-icon"
      viewBox="0 0 16 16"
      width="14"
      height="14"
      aria-hidden="true"
      focusable="false"
    >
      {children}
    </svg>
  );
}

function dist(a: RgmPoint3, b: RgmPoint3): number {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  const dz = a.z - b.z;
  return Math.sqrt(dx * dx + dy * dy + dz * dz);
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
  return example !== "intersectCurveCurve" && example !== "intersectCurvePlane";
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

export function KernelViewer() {
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const logBodyRef = useRef<HTMLDivElement | null>(null);
  const sessionFileInputRef = useRef<HTMLInputElement | null>(null);

  const runtimeRef = useRef<KernelRuntime | null>(null);
  const sessionRef = useRef<KernelSession | null>(null);
  const curveHandleRef = useRef<bigint | null>(null);
  const ownedCurveHandlesRef = useRef<bigint[]>([]);
  const nurbsPresetRef = useRef<CurvePreset | null>(null);

  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const controlsRef = useRef<OrbitControls | null>(null);
  const lineRef = useRef<Line2 | null>(null);
  const overlayLineRefs = useRef<Line2[]>([]);
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
  const logSequenceRef = useRef(1);

  const [preset, setPreset] = useState<CurvePreset | null>(null);
  const [activeExample, setActiveExample] = useState<ExampleKey>("nurbs");
  const [activeCurveName, setActiveCurveName] = useState("NURBS");
  const [activeDegreeLabel, setActiveDegreeLabel] = useState("");
  const [activeRenderDegree, setActiveRenderDegree] = useState(3);
  const [sampledPoints, setSampledPoints] = useState<RgmPoint3[]>([]);
  const [overlayCurves, setOverlayCurves] = useState<OverlayCurveVisual[]>([]);
  const [intersectionPoints, setIntersectionPoints] = useState<RgmPoint3[]>([]);
  const [intersectionPlane, setIntersectionPlane] = useState<RgmPlane | null>(null);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [statusMessage, setStatusMessage] = useState("Booting kernel runtime...");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
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
  const [isInspectorOpen, setIsInspectorOpen] = useState(true);
  const [isConsoleOpen, setIsConsoleOpen] = useState(true);
  const [isMobileLayout, setIsMobileLayout] = useState(false);

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
      session.releaseObject(handle);
    }
    ownedCurveHandlesRef.current = [];
    curveHandleRef.current = null;
  }, []);

  const buildExampleCurve = useCallback(
    (
      session: KernelSession,
      example: ExampleKey,
      nurbsPresetOverride?: CurvePreset,
    ): BuiltCurveExample => {
      const tol = nurbsPresetOverride?.tolerance ?? nurbsPresetRef.current?.tolerance ?? fallbackTolerance();

      if (example === "nurbs") {
        const presetToUse = nurbsPresetOverride ?? nurbsPresetRef.current;
        if (!presetToUse) {
          throw new Error("NURBS preset is not loaded");
        }
        const handle = session.buildCurveFromPreset(presetToUse as CurvePresetInput);
        return {
          curveHandle: handle,
          ownedHandles: [handle],
          name: presetToUse.name,
          degreeLabel: `NURBS p=${presetToUse.degree}`,
          renderDegree: presetToUse.degree,
          renderSamples: renderSampleCountForPreset(presetToUse),
          overlayCurves: [],
          intersectionPoints: [],
          planeVisual: null,
          logs: constructionLogLines(presetToUse),
        };
      }

      if (example === "line") {
        const line: RgmLine3 = {
          start: { x: -7.8, y: -2.9, z: 1.6 },
          end: { x: 8.1, y: 3.4, z: -2.3 },
        };
        const handle = session.createLine(line, tol);
        return {
          curveHandle: handle,
          ownedHandles: [handle],
          name: "Skew 3D Line Span",
          degreeLabel: "Line (p=1)",
          renderDegree: 1,
          renderSamples: 320,
          overlayCurves: [],
          intersectionPoints: [],
          planeVisual: null,
          logs: [
            `Line start=(${line.start.x}, ${line.start.y}, ${line.start.z})`,
            `Line end=(${line.end.x}, ${line.end.y}, ${line.end.z})`,
          ],
        };
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
        const handle = session.createPolyline(points, false, tol);
        return {
          curveHandle: handle,
          ownedHandles: [handle],
          name: "Spatial Polyline Traverse",
          degreeLabel: "Polyline (p=1)",
          renderDegree: 1,
          renderSamples: 1200,
          overlayCurves: [],
          intersectionPoints: [],
          planeVisual: null,
          logs: [`Polyline vertices=${points.length} closed=false`],
        };
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
        const handle = session.createArc(arc, tol);
        return {
          curveHandle: handle,
          ownedHandles: [handle],
          name: "Tilted Rational Arc",
          degreeLabel: "Arc (rational p=2)",
          renderDegree: 2,
          renderSamples: 1800,
          overlayCurves: [],
          intersectionPoints: [],
          planeVisual: null,
          logs: [`Arc radius=${arc.radius} start=${arc.start_angle} sweep=${arc.sweep_angle}`],
        };
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
        const handle = session.createCircle(circle, tol);
        return {
          curveHandle: handle,
          ownedHandles: [handle],
          name: "Tilted Rational Circle",
          degreeLabel: "Circle (rational p=2 periodic)",
          renderDegree: 2,
          renderSamples: 2400,
          overlayCurves: [],
          intersectionPoints: [],
          planeVisual: null,
          logs: [`Circle radius=${circle.radius}`],
        };
      }

      if (example === "intersectCurveCurve") {
        const builtHandles: bigint[] = [];
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

          const primaryHandle = session.createCircle(circlePrimary, tol);
          builtHandles.push(primaryHandle);
          const secondaryHandle = session.createCircle(circleSecondary, tol);
          builtHandles.push(secondaryHandle);

          const secondarySamples = session.sampleCurvePolyline(secondaryHandle, 2400);
          const hits = session.intersectCurveCurve(primaryHandle, secondaryHandle);
          const hitLogs = hits.map(
            (point, idx) =>
              `Curve-curve hit ${idx + 1}: (${point.x.toFixed(4)}, ${point.y.toFixed(4)}, ${point.z.toFixed(4)})`,
          );

          return {
            curveHandle: primaryHandle,
            ownedHandles: builtHandles,
            name: "Dual Tilted Circle Intersection",
            degreeLabel: "Intersection (curve-curve)",
            renderDegree: 2,
            renderSamples: 2400,
            overlayCurves: [
              {
                points: secondarySamples,
                color: "#f8ae63",
                width: 2.4,
                opacity: 0.95,
                name: "secondary curve",
              },
            ],
            intersectionPoints: hits,
            planeVisual: null,
            logs: [
              "Primary: rational circle in tilted plane",
              "Secondary: orthogonal tilted circle transformed in world space",
              `Intersection count=${hits.length}`,
              ...hitLogs,
            ],
          };
        } catch (error) {
          for (const handle of builtHandles) {
            session.releaseObject(handle);
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
        const curveHandle = session.buildCurveFromPreset(curvePreset);
        const hits = session.intersectCurvePlane(curveHandle, plane);
        const hitLogs = hits.map(
          (point, idx) =>
            `Curve-plane hit ${idx + 1}: (${point.x.toFixed(4)}, ${point.y.toFixed(4)}, ${point.z.toFixed(4)})`,
        );

        return {
          curveHandle,
          ownedHandles: [curveHandle],
          name: "NURBS vs Tilted Plane",
          degreeLabel: "Intersection (curve-plane)",
          renderDegree: 3,
          renderSamples: 3600,
          overlayCurves: [],
          intersectionPoints: hits,
          planeVisual: plane,
          logs: [
            `Curve control points=${fitPoints.length}`,
            "Plane is intentionally oblique to world axes",
            `Intersection count=${hits.length}`,
            ...hitLogs,
          ],
        };
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

      const builtHandles: bigint[] = [];
      try {
        const hLineA = session.createLine(lineA, tol);
        builtHandles.push(hLineA);
        const hArcA = session.createArc(arcA, tol);
        builtHandles.push(hArcA);
        const hLineB = session.createLine(lineB, tol);
        builtHandles.push(hLineB);
        const hArcB = session.createArc(arcB, tol);
        builtHandles.push(hArcB);

        const segments: RgmPolycurveSegment[] = [
          { curve: hLineA, reversed: false },
          { curve: hArcA, reversed: false },
          { curve: hLineB, reversed: false },
          { curve: hArcB, reversed: false },
        ];
        const poly = session.createPolycurve(segments, tol);
        builtHandles.unshift(poly);

        return {
          curveHandle: poly,
          ownedHandles: builtHandles,
          name: "Mixed Polycurve Ribbon",
          degreeLabel: "Polycurve (line+arc+line+arc)",
          renderDegree: 3,
          renderSamples: 2800,
          overlayCurves: [],
          intersectionPoints: [],
          planeVisual: null,
          logs: [`Polycurve segments=${segments.length}`],
        };
      } catch (error) {
        for (const handle of builtHandles) {
          session.releaseObject(handle);
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

      const built = buildExampleCurve(session, example, nurbsPresetOverride);
      curveHandleRef.current = built.curveHandle;
      ownedCurveHandlesRef.current = built.ownedHandles;

      for (const line of built.logs) {
        appendLog("debug", line);
      }

      const curveSamples = session.sampleCurvePolyline(built.curveHandle, built.renderSamples);
      const totalLength = session.curveLength(built.curveHandle);
      totalLengthRef.current = totalLength;
      const evaluatedProbe = session.pointAt(built.curveHandle, probeTNormRef.current);
      const probeLength = session.curveLengthAt(built.curveHandle, probeTNormRef.current);

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

      if (nurbsPresetOverride) {
        nurbsPresetRef.current = nurbsPresetOverride;
        setPreset(nurbsPresetOverride);
      }

      setActiveExample(example);
      setActiveCurveName(built.name);
      setActiveDegreeLabel(built.degreeLabel);
      setActiveRenderDegree(built.renderDegree);
      setSampledPoints(curveSamples);
      setOverlayCurves(built.overlayCurves);
      setIntersectionPoints(built.intersectionPoints);
      setIntersectionPlane(built.planeVisual);
      const intersectionSummary =
        built.intersectionPoints.length > 0
          ? ` • intersections ${built.intersectionPoints.length}`
          : "";
      setStatusMessage(
        `${successMessage} • ${built.name} • ${built.degreeLabel}${intersectionSummary} • exact length ${totalLength.toFixed(6)} • render samples ${curveSamples.length}`,
      );
      setErrorMessage(null);
      appendLog(
        "info",
        `Built handle ${built.curveHandle.toString()} with exact length=${totalLength.toFixed(6)}, samples=${curveSamples.length}, intersections=${built.intersectionPoints.length}`,
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
    if (!camera || !controls || sampledPoints.length === 0) {
      return;
    }

    const bounds = new THREE.Box3();
    sampledPoints.forEach((point) => {
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
  }, [sampledPoints]);

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
        const point = liveSession.pointAt(liveCurveHandle, next);
        const probeLength = liveSession.curveLengthAt(liveCurveHandle, next);
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
        appendLog("info", `Kernel session created: ${session.handle.toString()}`);
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
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        setErrorMessage(message);
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

  useEffect(() => {
    const container = logBodyRef.current;
    if (!container) {
      return;
    }
    container.scrollTop = container.scrollHeight;
  }, [logs]);

  useEffect(() => {
    const viewport = viewportRef.current;
    if (!viewport) {
      return;
    }

    const scene = new THREE.Scene();
    scene.background = new THREE.Color("#edf1f7");
    scene.fog = new THREE.Fog("#edf1f7", 34, 138);

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
          fallbackContext.fillStyle = "#edf1f7";
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
      for (const overlay of overlayLineRefs.current) {
        overlay.geometry.dispose();
        overlay.material.dispose();
      }
      overlayLineRefs.current = [];
      for (const marker of intersectionMarkerRefs.current) {
        marker.geometry.dispose();
        marker.material.dispose();
      }
      intersectionMarkerRefs.current = [];
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

    if (planeMeshRef.current) {
      scene.remove(planeMeshRef.current);
      planeMeshRef.current.geometry.dispose();
      planeMeshRef.current.material.dispose();
      planeMeshRef.current = null;
    }
    if (planeWireRef.current) {
      scene.remove(planeWireRef.current);
      planeWireRef.current.geometry.dispose();
      planeWireRef.current.material.dispose();
      planeWireRef.current = null;
    }
    if (planeNormalRef.current) {
      scene.remove(planeNormalRef.current);
      planeNormalRef.current = null;
    }

    if (!intersectionPlane) {
      return;
    }

    const frame = buildPlaneFrame(intersectionPlane);
    const center = projectedPointOnPlane(
      centroidOfPoints(sampledPoints),
      frame.origin,
      frame.normal,
    );
    const size = planeVisualSize(sampledPoints);
    const basis = new THREE.Matrix4().makeBasis(frame.xAxis, frame.yAxis, frame.normal);

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
    planeMesh.position.copy(center);
    planeMesh.setRotationFromMatrix(basis);
    planeMesh.renderOrder = 8;
    scene.add(planeMesh);
    planeMeshRef.current = planeMesh;

    const planeWire = new THREE.LineSegments(
      new THREE.EdgesGeometry(new THREE.PlaneGeometry(size, size, 1, 1)),
      new THREE.LineBasicMaterial({
        color: "#8fdbff",
        transparent: true,
        opacity: 0.7,
      }),
    );
    planeWire.position.copy(center);
    planeWire.setRotationFromMatrix(basis);
    planeWire.renderOrder = 9;
    scene.add(planeWire);
    planeWireRef.current = planeWire;

    const arrowLength = Math.max(3, size * 0.34);
    const normalArrow = new THREE.ArrowHelper(
      frame.normal.clone(),
      center.clone(),
      arrowLength,
      0x95e3ff,
      arrowLength * 0.16,
      arrowLength * 0.08,
    );
    scene.add(normalArrow);
    planeNormalRef.current = normalArrow;
  }, [intersectionPlane, sampledPoints]);

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
      controlsRef.current.enabled = orbitEnabled;
    }
  }, [orbitEnabled]);

  useEffect(() => {
    if (sampledPoints.length > 0) {
      zoomExtents();
    }
  }, [sampledPoints, zoomExtents]);

  const canExportIges = useMemo(() => capabilities.igesExport, [capabilities.igesExport]);
  const canImportIges = useMemo(() => capabilities.igesImport, [capabilities.igesImport]);
  const showProbeControls = useMemo(() => shouldShowProbeForExample(activeExample), [activeExample]);

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

      <header className="toolbar" role="toolbar" aria-label="Viewer actions">
        <div className="toolbar-group">
          <button
            type="button"
            className="tool-btn"
            onClick={onLoadSessionClick}
            aria-label="Load Session"
            title="Load Session"
          >
            <ToolIcon>
              <path d="M2.5 5.5h4.2l1.2 1.2h5.8v5.8H2.5z" />
              <path d="M8 9.1v3.4" />
              <path d="m6.6 11.1 1.4 1.4 1.4-1.4" />
            </ToolIcon>
            <span className="tool-label">Load Session</span>
          </button>
          <button
            type="button"
            className="tool-btn"
            onClick={onSaveSession}
            aria-label="Save Session"
            title="Save Session"
          >
            <ToolIcon>
              <path d="M3 2.7h8.4l1.6 1.6V13.3H3z" />
              <path d="M5 2.7v3.2h5.5V2.7" />
              <path d="M5.2 10.6h5.5" />
            </ToolIcon>
            <span className="tool-label">Save Session</span>
          </button>
          <button
            type="button"
            className="tool-btn"
            disabled={!canImportIges}
            title="Kernel IGES import API pending"
            aria-label="Load IGES"
          >
            <ToolIcon>
              <path d="M2.8 3.2h10.4v9.6H2.8z" />
              <path d="M4.9 6.2h6.2" />
              <path d="M4.9 8h4.2" />
              <path d="M4.9 9.8h6.2" />
            </ToolIcon>
            <span className="tool-label">Load IGES</span>
          </button>
          <button
            type="button"
            className="tool-btn"
            disabled={!canExportIges}
            title="Kernel IGES export API pending"
            aria-label="Save IGES"
          >
            <ToolIcon>
              <path d="M2.8 3.2h10.4v9.6H2.8z" />
              <path d="M8 6v4.2" />
              <path d="m6.4 8.7 1.6 1.6 1.6-1.6" />
            </ToolIcon>
            <span className="tool-label">Save IGES</span>
          </button>
        </div>

        <div className="toolbar-group">
          <button
            type="button"
            className="tool-btn"
            onClick={zoomExtents}
            aria-label="Zoom Extents"
            title="Zoom Extents"
          >
            <ToolIcon>
              <rect x="3.2" y="3.2" width="9.6" height="9.6" />
              <path d="M1.6 1.6 4 4" />
              <path d="m14.4 1.6-2.4 2.4" />
              <path d="m1.6 14.4 2.4-2.4" />
              <path d="m14.4 14.4-2.4-2.4" />
            </ToolIcon>
            <span className="tool-label">Zoom Extents</span>
          </button>
          <button
            type="button"
            className="tool-btn"
            onClick={resetCamera}
            aria-label="Reset View"
            title="Reset View"
          >
            <ToolIcon>
              <path d="M8 2.6a5.4 5.4 0 1 0 5.1 7.2" />
              <path d="M10.6 2.8h2.8v2.8" />
              <path d="m10.8 5.6 2.6-2.8" />
            </ToolIcon>
            <span className="tool-label">Reset View</span>
          </button>
          <button
            type="button"
            className={`tool-btn ${orbitEnabled ? "is-active" : ""}`}
            onClick={() => setOrbitEnabled((value) => !value)}
            aria-label="Orbit"
            title="Orbit"
          >
            <ToolIcon>
              <circle cx="8" cy="8" r="1.8" />
              <ellipse cx="8" cy="8" rx="5.2" ry="2.8" />
              <path d="M8 2.8a5.2 5.2 0 0 1 0 10.4" />
            </ToolIcon>
            <span className="tool-label">Orbit</span>
          </button>
          <button
            type="button"
            className={`tool-btn ${showGrid ? "is-active" : ""}`}
            onClick={() => setShowGrid((value) => !value)}
            aria-label="Grid"
            title="Grid"
          >
            <ToolIcon>
              <path d="M3 3h10v10H3z" />
              <path d="M6.3 3v10" />
              <path d="M9.7 3v10" />
              <path d="M3 6.3h10" />
              <path d="M3 9.7h10" />
            </ToolIcon>
            <span className="tool-label">Grid</span>
          </button>
          <button
            type="button"
            className={`tool-btn ${showAxes ? "is-active" : ""}`}
            onClick={() => setShowAxes((value) => !value)}
            aria-label="Axes"
            title="Axes"
          >
            <ToolIcon>
              <path d="M2.8 12.8 8 8" />
              <path d="M8 8 13.2 3.2" />
              <path d="M8 8v5.2" />
              <circle cx="8" cy="8" r="1.1" />
            </ToolIcon>
            <span className="tool-label">Axes</span>
          </button>
          <button
            type="button"
            className="tool-btn"
            onClick={onSaveScreenshot}
            aria-label="Save PNG"
            title="Save PNG"
          >
            <ToolIcon>
              <rect x="2.8" y="4.1" width="10.4" height="8.2" rx="1.2" ry="1.2" />
              <path d="m5.2 9.8 1.8-1.8 1.8 1.8 1.3-1.3 1.9 1.9" />
              <circle cx="6.1" cy="6.6" r="0.8" />
            </ToolIcon>
            <span className="tool-label">Save PNG</span>
          </button>
        </div>

        <div className="toolbar-group">
          <button
            type="button"
            className={`tool-btn ${isInspectorOpen ? "is-active" : ""}`}
            onClick={toggleInspector}
            aria-label="Toggle Controls"
            title="Toggle Controls"
          >
            <ToolIcon>
              <path d="M3 4.2h10" />
              <path d="M3 8h10" />
              <path d="M3 11.8h10" />
              <circle cx="5.4" cy="4.2" r="0.9" />
              <circle cx="10.6" cy="8" r="0.9" />
              <circle cx="7.2" cy="11.8" r="0.9" />
            </ToolIcon>
            <span className="tool-label">Controls</span>
          </button>
          <button
            type="button"
            className={`tool-btn ${isConsoleOpen ? "is-active" : ""}`}
            onClick={toggleConsole}
            aria-label="Toggle Console"
            title="Toggle Console"
          >
            <ToolIcon>
              <rect x="2.6" y="3.1" width="10.8" height="9.8" rx="1.4" ry="1.4" />
              <path d="M4.7 10.2h6.6" />
              <path d="M4.7 7.8h3.8" />
            </ToolIcon>
            <span className="tool-label">Console</span>
          </button>
          <button
            type="button"
            className="tool-btn"
            onClick={onExportLogs}
            aria-label="Export Logs"
            title="Export Logs"
          >
            <ToolIcon>
              <path d="M8 2.5v7.2" />
              <path d="m5.6 7.3 2.4 2.4 2.4-2.4" />
              <path d="M3 11.1h10v2.2H3z" />
            </ToolIcon>
            <span className="tool-label">Export Logs</span>
          </button>
          <button
            type="button"
            className="tool-btn"
            onClick={onClearLogs}
            aria-label="Clear Logs"
            title="Clear Logs"
          >
            <ToolIcon>
              <path d="M3.9 4.2h8.2" />
              <path d="M5 4.2v8.1h6v-8.1" />
              <path d="M6.3 6.1v4.2" />
              <path d="M9.7 6.1v4.2" />
              <path d="M6.1 2.8h3.8" />
            </ToolIcon>
            <span className="tool-label">Clear Logs</span>
          </button>
        </div>

        <div className="status-pill" aria-live="polite">
          {errorMessage ? `Error: ${errorMessage}` : statusMessage}
        </div>
      </header>

      <main className="viewer-main">
        <section className="viewport-wrap">
          <div ref={viewportRef} className="viewport" aria-label="Three.js viewport" />
        </section>
      </main>

      <aside
        className={`inspector-panel ${isInspectorOpen ? "is-open" : "is-collapsed"}`}
        aria-label="Viewer controls"
        aria-hidden={!isInspectorOpen}
      >
        <div className="inspector-header">
          <strong>Controls</strong>
        </div>
        <div className="inspector-body">
          <section className="inspector-section" aria-label="Example selection">
            <h2>Example</h2>
            <label className="inspector-field">
              <span>Curve</span>
              <select
                value={activeExample}
                onChange={(event) => {
                  onExampleSelectionChange(event.currentTarget.value);
                }}
              >
                {Object.entries(EXAMPLE_OPTIONS).map(([label, value]) => (
                  <option key={value} value={value}>
                    {label}
                  </option>
                ))}
              </select>
            </label>
            <div className="inspector-readout">
              <span>Name</span>
              <output>{activeCurveName}</output>
            </div>
            <div className="inspector-readout">
              <span>Type</span>
              <output>{activeDegreeLabel}</output>
            </div>
          </section>

          {showProbeControls ? (
            <section className="inspector-section" aria-label="Probe controls">
              <h2>Probe</h2>
              <label className="inspector-field">
                <span>t</span>
                <input
                  type="range"
                  min={0}
                  max={1}
                  step={0.0005}
                  value={probeUiState.tNorm}
                  onChange={(event) => {
                    updateProbeForT(Number(event.currentTarget.value), false);
                  }}
                  onPointerUp={(event) => {
                    updateProbeForT(Number(event.currentTarget.value), true);
                  }}
                  onTouchEnd={(event) => {
                    updateProbeForT(Number(event.currentTarget.value), true);
                  }}
                />
              </label>
              <div className="inspector-readout">
                <span>t value</span>
                <output>{probeUiState.tNorm.toFixed(5)}</output>
              </div>
              <div className="inspector-readout">
                <span>x</span>
                <output>{probeUiState.x.toFixed(5)}</output>
              </div>
              <div className="inspector-readout">
                <span>y</span>
                <output>{probeUiState.y.toFixed(5)}</output>
              </div>
              <div className="inspector-readout">
                <span>z</span>
                <output>{probeUiState.z.toFixed(5)}</output>
              </div>
              <div className="inspector-readout">
                <span>s(t)</span>
                <output>{probeUiState.probeLength.toFixed(5)}</output>
              </div>
              <div className="inspector-readout">
                <span>s(total)</span>
                <output>{probeUiState.totalLength.toFixed(5)}</output>
              </div>
            </section>
          ) : (
            <section className="inspector-section" aria-label="Probe controls unavailable">
              <h2>Probe</h2>
              <p className="inspector-note">
                Probe readout is hidden for intersection-focused examples.
              </p>
            </section>
          )}
        </div>
      </aside>

      <aside
        className={`kernel-console ${isConsoleOpen ? "is-open" : "is-collapsed"}`}
        aria-label="Kernel console"
        aria-hidden={!isConsoleOpen}
      >
        <div className="kernel-console-header">
          <strong>Kernel Console</strong>
          <div className="kernel-console-actions">
            <button type="button" className="tool-btn" onClick={onExportLogs}>
              Export
            </button>
            <button type="button" className="tool-btn" onClick={onClearLogs}>
              Clear
            </button>
          </div>
        </div>
        <div ref={logBodyRef} className="kernel-console-body">
          {logs.length > 0 ? (
            logs.map((entry) => (
              <div key={entry.id} className={`kernel-log kernel-log-${entry.level}`}>
                <span className="kernel-log-time">{entry.time}</span>
                <span className="kernel-log-level">{entry.level.toUpperCase()}</span>
                <span className="kernel-log-message">{entry.message}</span>
              </div>
            ))
          ) : (
            <div className="kernel-console-empty">No logs yet.</div>
          )}
        </div>
      </aside>
    </div>
  );
}
