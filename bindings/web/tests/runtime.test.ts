import { execSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
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
  if (!existsSync(wasmPath)) {
    execSync("cargo build -p kernel-ffi --target wasm32-unknown-unknown", {
      cwd: repoRoot,
      stdio: "inherit",
    });
  }
}, 120_000);

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

describe("kernel runtime", () => {
  it("creates a session, builds a curve, and samples points", async () => {
    const wasmBytes = readFileSync(wasmPath);
    const runtime = await createKernelRuntime(wasmBytes);
    const session = runtime.createSession();

    const handle = session.curve.buildCurveFromPreset(preset);
    const samples = session.curve.sampleCurvePolyline(handle, 32);
    const point = session.curve.curvePointAt(handle, 0.37);
    const totalLength = session.curve.curveLength(handle);
    const lengthAtPoint = session.curve.curveLengthAt(handle, 0.37);

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

    const circleHandle = session.curve.createCircle(
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
      const p = session.curve.curvePointAt(circleHandle, t);
      const dx = p.x - 1.25;
      const dy = p.y + 0.8;
      const dz = p.z - 0.4;
      const r = Math.sqrt(dx * dx + dy * dy + dz * dz);
      expect(r).toBeCloseTo(3.6, 3);
    }

    const lineA = session.curve.createLine(
      {
        start: { x: -1, y: 0, z: 0 },
        end: { x: 1, y: 0, z: 0 },
      },
      preset.tolerance,
    );
    const lineB = session.curve.createLine(
      {
        start: { x: 0, y: -1, z: 0 },
        end: { x: 0, y: 1, z: 0 },
      },
      preset.tolerance,
    );

    const curveHits = session.intersection.intersectCurveCurve(lineA, lineB);
    expect(curveHits.length).toBe(1);
    expect(curveHits[0].x).toBeCloseTo(0, 3);
    expect(curveHits[0].y).toBeCloseTo(0, 3);

    const planeHits = session.intersection.intersectCurvePlane(lineA, {
      origin: { x: 0, y: 0, z: 0 },
      x_axis: { x: 1, y: 0, z: 0 },
      y_axis: { x: 0, y: 1, z: 0 },
      z_axis: { x: 0, y: 1, z: 0 },
    });
    expect(planeHits.length).toBeGreaterThanOrEqual(1);

    const meshBox = session.mesh.createMeshBox(
      { x: 0, y: 0, z: 0 },
      { x: 4, y: 3, z: 2 },
    );
    expect(session.mesh.meshVertexCount(meshBox)).toBe(8);
    expect(session.mesh.meshTriangleCount(meshBox)).toBe(12);

    const translated = session.mesh.meshTranslate(meshBox, { x: 0.8, y: -0.4, z: 1.2 });
    const transformed = session.mesh.meshRotate(
      translated,
      { x: 0, y: 1, z: 0.2 },
      0.7,
      { x: 0, y: 0, z: 0 },
    );
    const baked = session.mesh.meshBakeTransform(transformed);
    const bakedBuffers = session.mesh.meshToBuffers(baked);
    expect(bakedBuffers.vertices.length).toBe(8);
    expect(bakedBuffers.indices.length).toBe(36);

    const torus = session.mesh.createMeshTorus(
      { x: 0, y: 0, z: 0 },
      3.8,
      1.1,
      28,
      20,
    );
    const meshPlaneHits = session.intersection.intersectMeshPlane(torus, {
      origin: { x: 0, y: 0, z: 0.2 },
      x_axis: { x: 1, y: 0, z: 0 },
      y_axis: { x: 0, y: 1, z: 0 },
      z_axis: { x: 0, y: 0, z: 1 },
    });
    expect(meshPlaneHits.length).toBeGreaterThan(0);
    expect(meshPlaneHits.length % 2).toBe(0);

    const sphere = session.mesh.createMeshUvSphere(
      { x: 0, y: 0, z: 0 },
      4.2,
      24,
      16,
    );
    const meshMeshHits = session.intersection.intersectMeshMesh(sphere, torus);
    expect(meshMeshHits.length).toBeGreaterThan(0);
    expect(meshMeshHits.length % 2).toBe(0);

    const booleanHost = session.mesh.createMeshBox(
      { x: 0, y: 0, z: 0 },
      { x: 8.8, y: 8.8, z: 8.8 },
    );
    const innerTorus = session.mesh.createMeshTorus(
      { x: 0, y: 0, z: 0 },
      2.4,
      0.8,
      32,
      24,
    );
    const booleanDiff = session.mesh.meshBoolean(booleanHost, innerTorus, 2);
    expect(session.mesh.meshTriangleCount(booleanDiff)).toBeGreaterThan(0);

    const surfacePoints = [
      { x: -2, y: -2, z: 0 },
      { x: -2, y: 0, z: 0.8 },
      { x: -2, y: 2, z: 0.1 },
      { x: 0, y: -2, z: 0.7 },
      { x: 0, y: 0, z: -0.2 },
      { x: 0, y: 2, z: 0.9 },
      { x: 2, y: -2, z: -0.3 },
      { x: 2, y: 0, z: 0.6 },
      { x: 2, y: 2, z: 0.2 },
    ];
    const surface = session.surface.createNurbsSurface(
      {
        degree_u: 2,
        degree_v: 2,
        periodic_u: false,
        periodic_v: false,
        control_u_count: 3,
        control_v_count: 3,
      },
      surfacePoints,
      new Array(9).fill(1),
      clampedUniformKnots(3, 2),
      clampedUniformKnots(3, 2),
      preset.tolerance,
    );
    const frame = session.surface.surfaceFrameAt(surface, { u: 0.5, v: 0.5 });
    const d0 = session.surface.surfacePointAt(surface, { u: 0.5, v: 0.5 });
    const d1 = session.surface.surfaceD1At(surface, { u: 0.5, v: 0.5 });
    const d2 = session.surface.surfaceD2At(surface, { u: 0.5, v: 0.5 });
    expect(frame.point.x).toBeCloseTo(d0.x, 7);
    expect(frame.point.y).toBeCloseTo(d0.y, 7);
    expect(frame.point.z).toBeCloseTo(d0.z, 7);
    expect(frame.du.x).toBeCloseTo(d1.du.x, 7);
    expect(frame.dv.y).toBeCloseTo(d1.dv.y, 7);
    expect(Number.isFinite(d2.duu.x)).toBe(true);
    expect(Number.isFinite(d2.duv.y)).toBe(true);
    expect(Number.isFinite(d2.dvv.z)).toBe(true);
    expect(Number.isFinite(frame.point.x)).toBe(true);
    expect(Number.isFinite(frame.normal.z)).toBe(true);

    const face = session.face.createFaceFromSurface(surface);
    session.face.faceAddLoop(
      face,
      [
        { u: 0.05, v: 0.05 },
        { u: 0.95, v: 0.05 },
        { u: 0.95, v: 0.95 },
        { u: 0.05, v: 0.95 },
      ],
      true,
    );
    const trimCircle = session.curve.createCircle(
      {
        plane: {
          origin: { x: 0.5, y: 0.5, z: 0 },
          x_axis: { x: 1, y: 0, z: 0 },
          y_axis: { x: 0, y: 1, z: 0 },
          z_axis: { x: 0, y: 0, z: 1 },
        },
        radius: 0.18,
      },
      preset.tolerance,
    );
    session.face.faceAddLoopEdges(
      face,
      { edge_count: 1, is_outer: false },
      [
        {
          start_uv: { u: 0.68, v: 0.5 },
          end_uv: { u: 0.68, v: 0.5 },
          curve_3d: trimCircle,
          has_curve_3d: true,
        },
      ],
    );
    expect(session.face.faceValidate(face)).toBe(true);
    const faceMesh = session.face.faceTessellateToMesh(face, {
      min_u_segments: 28,
      min_v_segments: 28,
      max_u_segments: 48,
      max_v_segments: 48,
      chord_tol: 1e-4,
      normal_tol_rad: 0.08,
    });
    expect(session.mesh.meshTriangleCount(faceMesh)).toBeGreaterThan(0);

    const surfacePlaneIntersection = session.intersection.intersectSurfacePlane(surface, {
      origin: { x: 0, y: 0, z: 0.1 },
      x_axis: { x: 1, y: 0, z: 0 },
      y_axis: { x: 0, y: 1, z: 0 },
      z_axis: { x: 0, y: 0, z: 1 },
    });
    const branchCount = session.intersection.intersectionBranchCount(surfacePlaneIntersection);
    expect(branchCount).toBeGreaterThanOrEqual(0);

    session.kernel.releaseObject(lineB);
    session.kernel.releaseObject(lineA);
    session.kernel.releaseObject(circleHandle);
    session.kernel.releaseObject(booleanDiff);
    session.kernel.releaseObject(booleanHost);
    session.kernel.releaseObject(innerTorus);
    session.kernel.releaseObject(surfacePlaneIntersection);
    session.kernel.releaseObject(faceMesh);
    session.kernel.releaseObject(trimCircle);
    session.kernel.releaseObject(face);
    session.kernel.releaseObject(surface);
    session.kernel.releaseObject(sphere);
    session.kernel.releaseObject(torus);
    session.kernel.releaseObject(baked);
    session.kernel.releaseObject(transformed);
    session.kernel.releaseObject(translated);
    session.kernel.releaseObject(meshBox);
    session.kernel.releaseObject(handle);
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
      session.curve.buildCurveFromPreset(invalidPreset);
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
      expect(api.rgm_curve_create_line(session, linePtr, tolPtr, outObjectPtr)).toBe(
        RgmStatus.Ok,
      );
      const line = memory.readU64(outObjectPtr);

      memory.writePlane(planePtr, {
        origin: { x: 0, y: 0, z: 0 },
        x_axis: { x: 1, y: 0, z: 0 },
        y_axis: { x: 0, y: 1, z: 0 },
        z_axis: { x: 0, y: 0, z: 1 },
      });

      expect(api.rgm_intersect_curve_plane(session, line, planePtr, pointsPtr, 8, outCountPtr)).toBe(
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
