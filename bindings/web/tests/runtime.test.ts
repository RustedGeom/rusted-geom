import { execSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { beforeAll, describe, expect, it } from "vitest";

import {
  createKernelRuntime,
  KernelRuntimeError,
  type CurvePresetInput,
  RgmStatus,
} from "../src/index";
import type { NativeExports } from "../src/generated/native";
import { KERNEL_LAYOUT, KernelMemory } from "../src/runtime/memory";
import { loadKernelWasm } from "../src/runtime/wasm-loader";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "../../..");
const wasmPath = path.join(
  repoRoot,
  "target/wasm32-unknown-unknown/debug/rusted_geom.wasm",
);

beforeAll(() => {
  execSync("cargo build -p kernel-ffi --target wasm32-unknown-unknown", {
    cwd: repoRoot,
    stdio: "inherit",
  });
});

const preset: CurvePresetInput = {
  degree: 2,
  closed: false,
  points: [
    { x: 0, y: 0, z: 0 },
    { x: 1, y: 0.25, z: 0 },
    { x: 2, y: 1, z: 0 },
    { x: 3, y: 1.25, z: 0 },
  ],
  tolerance: {
    abs_tol: 1e-9,
    rel_tol: 1e-9,
    angle_tol: 1e-9,
  },
};

describe("kernel runtime", () => {
  it("creates a session, builds a curve, and samples points", async () => {
    const wasmBytes = readFileSync(wasmPath);
    const runtime = await createKernelRuntime(wasmBytes);
    const session = runtime.createSession();

    const handle = session.buildCurveFromPreset(preset);
    const samples = session.sampleCurvePolyline(handle, 32);
    const point = session.pointAt(handle, 0.37);
    const totalLength = session.curveLength(handle);
    const lengthAtPoint = session.curveLengthAt(handle, 0.37);

    expect(runtime.capabilities.igesImport).toBe(false);
    expect(runtime.capabilities.igesExport).toBe(false);
    expect(samples).toHaveLength(32);
    expect(samples[0].x).toBeCloseTo(0, 6);
    expect(samples.at(-1)?.x).toBeCloseTo(3, 6);
    expect(point.x).toBeGreaterThanOrEqual(0);
    expect(point.x).toBeLessThanOrEqual(3);
    expect(totalLength).toBeGreaterThan(0);
    expect(lengthAtPoint).toBeGreaterThan(0);
    expect(lengthAtPoint).toBeLessThan(totalLength);

    const circleHandle = session.createCircle(
      {
        plane: {
          origin: { x: 1.25, y: -0.8, z: 0.4 },
          x_axis: { x: 1, y: 0, z: 0 },
          y_axis: { x: 0, y: 1, z: 0 },
          z_axis: { x: 0, y: 0, z: 1 },
        },
        radius: 3.6,
      },
      preset.tolerance,
    );
    for (const t of [0, 0.11, 0.3, 0.5, 0.77, 1]) {
      const p = session.pointAt(circleHandle, t);
      const dx = p.x - 1.25;
      const dy = p.y + 0.8;
      const dz = p.z - 0.4;
      const r = Math.sqrt(dx * dx + dy * dy + dz * dz);
      expect(r).toBeCloseTo(3.6, 3);
    }

    const lineA = session.createLine(
      {
        start: { x: -1, y: 0, z: 0 },
        end: { x: 1, y: 0, z: 0 },
      },
      preset.tolerance,
    );
    const lineB = session.createLine(
      {
        start: { x: 0, y: -1, z: 0 },
        end: { x: 0, y: 1, z: 0 },
      },
      preset.tolerance,
    );

    const curveHits = session.intersectCurveCurve(lineA, lineB);
    expect(curveHits.length).toBe(1);
    expect(curveHits[0].x).toBeCloseTo(0, 3);
    expect(curveHits[0].y).toBeCloseTo(0, 3);

    const planeHits = session.intersectCurvePlane(lineA, {
      origin: { x: 0, y: 0, z: 0 },
      x_axis: { x: 1, y: 0, z: 0 },
      y_axis: { x: 0, y: 1, z: 0 },
      z_axis: { x: 0, y: 1, z: 0 },
    });
    expect(planeHits.length).toBeGreaterThanOrEqual(1);

    session.releaseObject(lineB);
    session.releaseObject(lineA);
    session.releaseObject(circleHandle);
    session.releaseObject(handle);
    session.destroy();
    runtime.destroy();
  });

  it("surfaces kernel errors for invalid curve construction", async () => {
    const wasmBytes = readFileSync(wasmPath);
    const runtime = await createKernelRuntime(wasmBytes);
    const session = runtime.createSession();

    const invalidPreset: CurvePresetInput = {
      ...preset,
      degree: 8,
    };

    let thrown: unknown = undefined;
    try {
      session.buildCurveFromPreset(invalidPreset);
    } catch (error) {
      thrown = error;
    }

    expect(thrown).toBeInstanceOf(KernelRuntimeError);
    const kernelError = thrown as KernelRuntimeError;
    expect(kernelError.status).toBe(RgmStatus.InvalidInput);
    expect(kernelError.message).toContain("Curve construction failed");

    session.destroy();
    runtime.destroy();
  });

  it("executes intersection exports in wasm with expected counts", async () => {
    const wasmBytes = readFileSync(wasmPath);
    const module = await loadKernelWasm(wasmBytes);
    const api = module.exports as unknown as NativeExports;
    const memory = new KernelMemory(api, module.exports.memory);

    const sessionPtr = memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    const outObjectPtr = memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    const outCountPtr = memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    const pointsPtr = memory.alloc(KERNEL_LAYOUT.POINT3_BYTES * 8, 8);
    const linePtr = memory.alloc(KERNEL_LAYOUT.LINE3_BYTES, 8);
    const tolPtr = memory.alloc(KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
    const planePtr = memory.alloc(KERNEL_LAYOUT.PLANE_BYTES, 8);

    try {
      expect(api.rgm_kernel_create(sessionPtr)).toBe(RgmStatus.Ok);
      const session = memory.readU64(sessionPtr);

      memory.writeLine(linePtr, {
        start: { x: 0, y: 0, z: -1 },
        end: { x: 0, y: 0, z: 1 },
      });
      memory.writeTolerance(tolPtr, preset.tolerance);
      expect(api.rgm_curve_create_line_ptr_tol(session, linePtr, tolPtr, outObjectPtr)).toBe(
        RgmStatus.Ok,
      );
      const line = memory.readU64(outObjectPtr);

      memory.writePlane(planePtr, {
        origin: { x: 0, y: 0, z: 0 },
        x_axis: { x: 1, y: 0, z: 0 },
        y_axis: { x: 0, y: 1, z: 0 },
        z_axis: { x: 0, y: 0, z: 1 },
      });

      expect(api.rgm_intersect_curve_plane_ptr(session, line, planePtr, pointsPtr, 8, outCountPtr)).toBe(
        RgmStatus.Ok,
      );
      expect(memory.readU32(outCountPtr)).toBe(1);
      const p = memory.readPoint(pointsPtr);
      expect(Math.abs(p.x)).toBeLessThan(1e-6);
      expect(Math.abs(p.y)).toBeLessThan(1e-6);
      expect(Math.abs(p.z)).toBeLessThan(1e-6);

      expect(api.rgm_intersect_curve_curve(session, line, line, pointsPtr, 8, outCountPtr)).toBe(
        RgmStatus.Ok,
      );
      expect(memory.readU32(outCountPtr)).toBeGreaterThanOrEqual(1);

      expect(api.rgm_object_release(session, line)).toBe(RgmStatus.Ok);
      expect(api.rgm_kernel_destroy(session)).toBe(RgmStatus.Ok);
    } finally {
      memory.free(pointsPtr, KERNEL_LAYOUT.POINT3_BYTES * 8, 8);
      memory.free(planePtr, KERNEL_LAYOUT.PLANE_BYTES, 8);
      memory.free(tolPtr, KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
      memory.free(linePtr, KERNEL_LAYOUT.LINE3_BYTES, 8);
      memory.free(outCountPtr, KERNEL_LAYOUT.I32_BYTES, 4);
      memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
      memory.free(sessionPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  });
});
