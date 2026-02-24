import { execSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { beforeAll, describe, expect, it } from "vitest";

import {
  createKernelRuntime,
  KernelRuntimeError,
  type CurvePresetInput,
  type RgmBounds3,
  RgmBoundsMode,
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

function buildWarpedSurfaceNet(
  uCount: number,
  vCount: number,
  spanU: number,
  spanV: number,
  warpScale: number,
): {
  desc: {
    degree_u: number;
    degree_v: number;
    periodic_u: boolean;
    periodic_v: boolean;
    control_u_count: number;
    control_v_count: number;
  };
  points: Array<{ x: number; y: number; z: number }>;
  weights: number[];
  knotsU: number[];
  knotsV: number[];
} {
  const points: Array<{ x: number; y: number; z: number }> = [];
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

function pointInsideAabb(
  aabb: { min: { x: number; y: number; z: number }; max: { x: number; y: number; z: number } },
  point: { x: number; y: number; z: number },
  eps = 1e-7,
): boolean {
  return (
    point.x >= aabb.min.x - eps &&
    point.x <= aabb.max.x + eps &&
    point.y >= aabb.min.y - eps &&
    point.y <= aabb.max.y + eps &&
    point.z >= aabb.min.z - eps &&
    point.z <= aabb.max.z + eps
  );
}

function obbLocalPoint(
  bounds: RgmBounds3,
  point: { x: number; y: number; z: number },
): { x: number; y: number; z: number } {
  const relX = point.x - bounds.world_obb.center.x;
  const relY = point.y - bounds.world_obb.center.y;
  const relZ = point.z - bounds.world_obb.center.z;
  const dot = (axis: { x: number; y: number; z: number }) =>
    relX * axis.x + relY * axis.y + relZ * axis.z;
  return {
    x: dot(bounds.world_obb.x_axis),
    y: dot(bounds.world_obb.y_axis),
    z: dot(bounds.world_obb.z_axis),
  };
}

function expectFiniteBounds(bounds: RgmBounds3): void {
  const values = [
    bounds.world_aabb.min.x,
    bounds.world_aabb.min.y,
    bounds.world_aabb.min.z,
    bounds.world_aabb.max.x,
    bounds.world_aabb.max.y,
    bounds.world_aabb.max.z,
    bounds.world_obb.center.x,
    bounds.world_obb.center.y,
    bounds.world_obb.center.z,
    bounds.world_obb.half_extents.x,
    bounds.world_obb.half_extents.y,
    bounds.world_obb.half_extents.z,
    bounds.local_aabb.min.x,
    bounds.local_aabb.min.y,
    bounds.local_aabb.min.z,
    bounds.local_aabb.max.x,
    bounds.local_aabb.max.y,
    bounds.local_aabb.max.z,
  ];
  for (const value of values) {
    expect(Number.isFinite(value)).toBe(true);
  }
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

  it("computes bounds for curve/surface/mesh/brep and preserves containment", async () => {
    const wasmBytes = readFileSync(wasmPath);
    const runtime = await createKernelRuntime(wasmBytes);
    const session = runtime.createSession();

    const line = session.curve.createLine(
      {
        start: { x: -3, y: 1.25, z: 4.5 },
        end: { x: 6, y: -2.75, z: -1.5 },
      },
      preset.tolerance,
    );
    const curveBoundsFast = session.curve.bounds(line, {
      mode: RgmBoundsMode.Fast,
      sample_budget: 0,
      padding: 0,
    });
    const curveBoundsOptimal = session.curve.bounds(line, {
      mode: RgmBoundsMode.Optimal,
      sample_budget: 128,
      padding: 0,
    });
    expectFiniteBounds(curveBoundsFast);
    expectFiniteBounds(curveBoundsOptimal);
    expect(pointInsideAabb(curveBoundsFast.world_aabb, { x: -3, y: 1.25, z: 4.5 })).toBe(true);
    expect(pointInsideAabb(curveBoundsFast.world_aabb, { x: 6, y: -2.75, z: -1.5 })).toBe(true);

    const net = buildWarpedSurfaceNet(8, 7, 9, 7, 0.9);
    const surface = session.surface.createNurbsSurface(
      net.desc,
      net.points,
      net.weights,
      net.knotsU,
      net.knotsV,
      preset.tolerance,
    );
    const surfaceBounds = session.surface.bounds(surface, {
      mode: RgmBoundsMode.Fast,
      sample_budget: 0,
      padding: 0,
    });
    expectFiniteBounds(surfaceBounds);
    for (let iu = 0; iu <= 16; iu += 1) {
      const u = iu / 16;
      for (let iv = 0; iv <= 16; iv += 1) {
        const v = iv / 16;
        const point = session.surface.surfacePointAt(surface, { u, v });
        expect(pointInsideAabb(surfaceBounds.world_aabb, point, 1e-6)).toBe(true);
      }
    }

    const meshBox = session.mesh.createMeshBox(
      { x: 0, y: 0, z: 0 },
      { x: 4, y: 2, z: 3 },
    );
    const meshMoved = session.mesh.meshTranslate(meshBox, { x: 1.5, y: -0.75, z: 2.25 });
    const meshRot = session.mesh.meshRotate(
      meshMoved,
      { x: 0.3, y: 1.0, z: 0.4 },
      0.62,
      { x: 0.2, y: -0.3, z: 0.0 },
    );
    const meshScaled = session.mesh.meshScale(
      meshRot,
      { x: 1.25, y: 0.85, z: 1.4 },
      { x: 0, y: 0, z: 0 },
    );
    const meshBoundsA = session.mesh.bounds(meshScaled, {
      mode: RgmBoundsMode.Fast,
      sample_budget: 256,
      padding: 0,
    });
    const meshBoundsB = session.mesh.bounds(meshScaled, {
      mode: RgmBoundsMode.Fast,
      sample_budget: 256,
      padding: 0,
    });
    expectFiniteBounds(meshBoundsA);
    expect(meshBoundsB.world_aabb.min.x).toBeCloseTo(meshBoundsA.world_aabb.min.x, 9);
    expect(meshBoundsB.world_aabb.max.z).toBeCloseTo(meshBoundsA.world_aabb.max.z, 9);
    const meshBuffers = session.mesh.meshToBuffers(meshScaled);
    for (const vertex of meshBuffers.vertices) {
      expect(pointInsideAabb(meshBoundsA.world_aabb, vertex, 1e-6)).toBe(true);
      const local = obbLocalPoint(meshBoundsA, vertex);
      expect(pointInsideAabb(meshBoundsA.local_aabb, local, 1e-5)).toBe(true);
    }

    const brep = session.brep.brepCreateEmpty();
    const faceId = session.brep.brepAddFaceFromSurface(brep, surface);
    session.brep.brepAddLoopUv(
      brep,
      faceId,
      [
        { u: 0.05, v: 0.05 },
        { u: 0.95, v: 0.07 },
        { u: 0.94, v: 0.94 },
        { u: 0.06, v: 0.92 },
      ],
      true,
    );
    const brepBounds = session.brep.bounds(brep, {
      mode: RgmBoundsMode.Fast,
      sample_budget: 0,
      padding: 0,
    });
    expectFiniteBounds(brepBounds);
    const brepMesh = session.brep.brepTessellateToMesh(brep, {
      min_u_segments: 14,
      min_v_segments: 14,
      max_u_segments: 32,
      max_v_segments: 32,
      chord_tol: 2e-4,
      normal_tol_rad: 0.1,
    });
    const brepMeshBuffers = session.mesh.meshToBuffers(brepMesh);
    for (const vertex of brepMeshBuffers.vertices) {
      expect(pointInsideAabb(brepBounds.world_aabb, vertex, 1e-5)).toBe(true);
    }

    session.kernel.releaseObject(brepMesh);
    session.kernel.releaseObject(brep);
    session.kernel.releaseObject(meshScaled);
    session.kernel.releaseObject(meshRot);
    session.kernel.releaseObject(meshMoved);
    session.kernel.releaseObject(meshBox);
    session.kernel.releaseObject(surface);
    session.kernel.releaseObject(line);
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

  it("matches showcase surface-curve intersection hit count", async () => {
    const wasmBytes = readFileSync(wasmPath);
    const runtime = await createKernelRuntime(wasmBytes);
    const session = runtime.createSession();

    const tol = {
      abs_tol: 1e-9,
      rel_tol: 1e-9,
      angle_tol: 1e-9,
    };

    const net = buildWarpedSurfaceNet(16, 16, 12, 12, 1.2);
    const surface = session.surface.createNurbsSurface(
      net.desc,
      net.points,
      net.weights,
      net.knotsU,
      net.knotsV,
      tol,
    );

    const curve = session.curve.buildCurveFromPreset({
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

    const inter = session.intersection.intersectSurfaceCurve(surface, curve);
    const branchCount = session.intersection.intersectionBranchCount(inter);
    let totalHits = 0;
    for (let i = 0; i < branchCount; i += 1) {
      totalHits += session.intersection.intersectionBranchPoints(inter, i).length;
    }

    expect(totalHits).toBeGreaterThanOrEqual(3);

    session.kernel.releaseObject(inter);
    session.kernel.releaseObject(curve);
    session.kernel.releaseObject(surface);
    session.destroy();
    runtime.destroy();
  });
});
