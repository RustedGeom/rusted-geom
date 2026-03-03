import type { RgmPoint3, RgmToleranceContext } from "@rustedgeom/kernel";
import type { CameraMode } from "./viewer-types";

export interface CurvePreset {
  name: string;
  degree: number;
  closed: boolean;
  sampleCount: number;
  tolerance: RgmToleranceContext;
  points: RgmPoint3[];
}

export interface SavedCamera {
  position: RgmPoint3;
  target: RgmPoint3;
  up: RgmPoint3;
  fov: number;
  mode?: CameraMode;
}

export interface ViewerSessionFile {
  version: 1;
  preset: CurvePreset;
  view: {
    camera: SavedCamera;
    showGrid: boolean;
    showAxes: boolean;
    orbitEnabled: boolean;
  };
}

function asNumber(value: unknown, field: string): number {
  if (typeof value !== "number" || Number.isNaN(value)) {
    throw new Error(`Expected numeric field: ${field}`);
  }

  return value;
}

function asBoolean(value: unknown, field: string): boolean {
  if (typeof value !== "boolean") {
    throw new Error(`Expected boolean field: ${field}`);
  }

  return value;
}

function asString(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`Expected non-empty string field: ${field}`);
  }

  return value;
}

function asObject(value: unknown, field: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`Expected object field: ${field}`);
  }

  return value as Record<string, unknown>;
}

function parsePoint(value: unknown, field: string): RgmPoint3 {
  const data = asObject(value, field);
  return {
    x: asNumber(data.x, `${field}.x`),
    y: asNumber(data.y, `${field}.y`),
    z: asNumber(data.z, `${field}.z`),
  };
}

function parseTolerance(value: unknown, field: string): RgmToleranceContext {
  const data = asObject(value, field);
  return {
    abs_tol: asNumber(data.abs_tol, `${field}.abs_tol`),
    rel_tol: asNumber(data.rel_tol, `${field}.rel_tol`),
    angle_tol: asNumber(data.angle_tol, `${field}.angle_tol`),
  };
}

export function parseCurvePreset(value: unknown): CurvePreset {
  const data = asObject(value, "preset");
  const pointsRaw = data.points;
  if (!Array.isArray(pointsRaw) || pointsRaw.length < 2) {
    throw new Error("Preset must include at least two points");
  }

  const points = pointsRaw.map((point, idx) => parsePoint(point, `points[${idx}]`));

  const degree = Math.floor(asNumber(data.degree, "degree"));
  if (degree < 1) {
    throw new Error("Preset degree must be >= 1");
  }

  const sampleCount = Math.floor(asNumber(data.sampleCount, "sampleCount"));
  if (sampleCount < 2) {
    throw new Error("Preset sampleCount must be >= 2");
  }

  return {
    name: asString(data.name, "name"),
    degree,
    closed: asBoolean(data.closed, "closed"),
    sampleCount,
    tolerance: parseTolerance(data.tolerance, "tolerance"),
    points,
  };
}

export function parseViewerSession(value: unknown): ViewerSessionFile {
  const data = asObject(value, "session");
  const version = asNumber(data.version, "version");
  if (version !== 1) {
    throw new Error(`Unsupported session version: ${version}`);
  }

  const view = asObject(data.view, "view");
  const camera = asObject(view.camera, "view.camera");

  return {
    version: 1,
    preset: parseCurvePreset(data.preset),
    view: {
      camera: {
        position: parsePoint(camera.position, "view.camera.position"),
        target: parsePoint(camera.target, "view.camera.target"),
        up: parsePoint(camera.up, "view.camera.up"),
        fov: asNumber(camera.fov, "view.camera.fov"),
        mode: typeof camera.mode === "string" && (camera.mode === "perspective" || camera.mode === "orthographic")
          ? (camera.mode as CameraMode)
          : undefined,
      },
      showGrid: asBoolean(view.showGrid, "view.showGrid"),
      showAxes: asBoolean(view.showAxes, "view.showAxes"),
      orbitEnabled: asBoolean(view.orbitEnabled, "view.orbitEnabled"),
    },
  };
}
