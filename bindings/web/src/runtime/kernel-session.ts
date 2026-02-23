import type { NativeExports } from "../generated/native";
import type {
  RgmIntersectionBranchSummary,
  RgmArc3,
  RgmCircle3,
  RgmLine3,
  RgmNurbsSurfaceDesc,
  RgmPlane,
  RgmPoint3,
  RgmPolycurveSegment,
  RgmSurfaceEvalFrame,
  RgmSurfaceTessellationOptions,
  RgmToleranceContext,
  RgmUv2,
  RgmVec3,
} from "../generated/types";
import { RgmStatus } from "../generated/types";
import { KernelRuntimeError, statusToName } from "./errors";
import { KERNEL_LAYOUT, KernelMemory } from "./memory";
import { sampleCurvePolyline } from "./scene-sampler";
import { loadKernelWasm, type WasmSource } from "./wasm-loader";

export interface CurvePresetInput {
  name?: string;
  degree: number;
  closed: boolean;
  points: RgmPoint3[];
  tolerance: RgmToleranceContext;
}

export interface KernelCapabilities {
  igesImport: boolean;
  igesExport: boolean;
}

export interface KernelSession {
  readonly handle: bigint;
  buildCurveFromPreset(preset: CurvePresetInput): bigint;
  createLine(line: RgmLine3, tolerance: RgmToleranceContext): bigint;
  createArc(arc: RgmArc3, tolerance: RgmToleranceContext): bigint;
  createCircle(circle: RgmCircle3, tolerance: RgmToleranceContext): bigint;
  createPolyline(points: RgmPoint3[], closed: boolean, tolerance: RgmToleranceContext): bigint;
  createPolycurve(segments: RgmPolycurveSegment[], tolerance: RgmToleranceContext): bigint;
  sampleCurvePolyline(curveHandle: bigint, sampleCount: number): RgmPoint3[];
  pointAt(curveHandle: bigint, tNorm: number): RgmPoint3;
  curveLength(curveHandle: bigint): number;
  curveLengthAt(curveHandle: bigint, tNorm: number): number;
  intersectCurvePlane(curveHandle: bigint, plane: RgmPlane): RgmPoint3[];
  intersectCurveCurve(curveA: bigint, curveB: bigint): RgmPoint3[];
  createMeshBox(center: RgmPoint3, size: RgmVec3): bigint;
  createMeshUvSphere(center: RgmPoint3, radius: number, uSteps: number, vSteps: number): bigint;
  createMeshTorus(
    center: RgmPoint3,
    majorRadius: number,
    minorRadius: number,
    majorSteps: number,
    minorSteps: number,
  ): bigint;
  meshTranslate(meshHandle: bigint, delta: RgmVec3): bigint;
  meshRotate(meshHandle: bigint, axis: RgmVec3, angleRad: number, pivot: RgmPoint3): bigint;
  meshScale(meshHandle: bigint, scale: RgmVec3, pivot: RgmPoint3): bigint;
  meshBakeTransform(meshHandle: bigint): bigint;
  meshBoolean(meshA: bigint, meshB: bigint, op: 0 | 1 | 2): bigint;
  intersectMeshPlane(meshHandle: bigint, plane: RgmPlane): RgmPoint3[];
  intersectMeshMesh(meshA: bigint, meshB: bigint): RgmPoint3[];
  meshVertexCount(meshHandle: bigint): number;
  meshTriangleCount(meshHandle: bigint): number;
  meshToBuffers(meshHandle: bigint): { vertices: RgmPoint3[]; indices: number[] };
  createNurbsSurface(
    desc: RgmNurbsSurfaceDesc,
    controlPoints: RgmPoint3[],
    weights: number[],
    knotsU: number[],
    knotsV: number[],
    tolerance: RgmToleranceContext,
  ): bigint;
  surfacePointAt(surfaceHandle: bigint, uvNorm: RgmUv2): RgmPoint3;
  surfaceFrameAt(surfaceHandle: bigint, uvNorm: RgmUv2): RgmSurfaceEvalFrame;
  surfaceTranslate(surfaceHandle: bigint, delta: RgmVec3): bigint;
  surfaceRotate(surfaceHandle: bigint, axis: RgmVec3, angleRad: number, pivot: RgmPoint3): bigint;
  surfaceScale(surfaceHandle: bigint, scale: RgmVec3, pivot: RgmPoint3): bigint;
  surfaceBakeTransform(surfaceHandle: bigint): bigint;
  surfaceTessellateToMesh(
    surfaceHandle: bigint,
    options?: RgmSurfaceTessellationOptions,
  ): bigint;
  createFaceFromSurface(surfaceHandle: bigint): bigint;
  faceAddLoop(faceHandle: bigint, points: RgmUv2[], isOuter: boolean): void;
  faceRemoveLoop(faceHandle: bigint, loopIndex: number): void;
  faceSplitTrimEdge(
    faceHandle: bigint,
    loopIndex: number,
    edgeIndex: number,
    splitT: number,
  ): void;
  faceReverseLoop(faceHandle: bigint, loopIndex: number): void;
  faceValidate(faceHandle: bigint): boolean;
  faceHeal(faceHandle: bigint): void;
  faceTessellateToMesh(faceHandle: bigint, options?: RgmSurfaceTessellationOptions): bigint;
  intersectSurfaceSurface(surfaceA: bigint, surfaceB: bigint): bigint;
  intersectSurfacePlane(surface: bigint, plane: RgmPlane): bigint;
  intersectSurfaceCurve(surface: bigint, curve: bigint): bigint;
  intersectionBranchCount(intersection: bigint): number;
  intersectionBranchSummary(intersection: bigint, branchIndex: number): RgmIntersectionBranchSummary;
  intersectionBranchPoints(intersection: bigint, branchIndex: number): RgmPoint3[];
  intersectionBranchUvA(intersection: bigint, branchIndex: number): RgmUv2[];
  intersectionBranchUvB(intersection: bigint, branchIndex: number): RgmUv2[];
  intersectionBranchCurveT(intersection: bigint, branchIndex: number): number[];
  intersectionBranchToNurbs(
    intersection: bigint,
    branchIndex: number,
    tolerance: RgmToleranceContext,
  ): bigint;
  releaseObject(objectHandle: bigint): void;
  lastError(): { code: number; message: string };
  destroy(): void;
}

export interface KernelRuntime {
  readonly capabilities: KernelCapabilities;
  createSession(): KernelSession;
  destroy(): void;
}

class KernelSessionImpl implements KernelSession {
  private readonly decoder = new TextDecoder();
  private destroyed = false;

  constructor(
    private readonly api: NativeExports,
    private readonly memory: KernelMemory,
    readonly handle: bigint,
    private readonly onDestroy: () => void,
  ) {}

  buildCurveFromPreset(preset: CurvePresetInput): bigint {
    this.ensureAlive();
    if (!preset.points.length) {
      throw new Error("Curve preset must contain at least one point");
    }

    const pointsBytes = preset.points.length * KERNEL_LAYOUT.POINT3_BYTES;
    const pointsPtr = this.memory.alloc(pointsBytes, 8);
    const tolerancePtr = this.memory.alloc(KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);

    try {
      this.memory.writePointArray(pointsPtr, preset.points);
      this.memory.writeTolerance(tolerancePtr, preset.tolerance);
      const status = this.api.rgm_nurbs_interpolate_fit_points_ptr_tol(
        this.handle,
        pointsPtr,
        preset.points.length,
        preset.degree,
        preset.closed,
        tolerancePtr,
        outObjectPtr,
      ) as RgmStatus;

      this.assertOk(status, "Curve construction failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(pointsPtr, pointsBytes, 8);
      this.memory.free(tolerancePtr, KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  createLine(line: RgmLine3, tolerance: RgmToleranceContext): bigint {
    this.ensureAlive();

    const linePtr = this.memory.alloc(KERNEL_LAYOUT.LINE3_BYTES, 8);
    const tolerancePtr = this.memory.alloc(KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);

    try {
      this.memory.writeLine(linePtr, line);
      this.memory.writeTolerance(tolerancePtr, tolerance);
      const status = this.api.rgm_curve_create_line_ptr_tol(
        this.handle,
        linePtr,
        tolerancePtr,
        outObjectPtr,
      ) as RgmStatus;
      this.assertOk(status, "Line construction failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(linePtr, KERNEL_LAYOUT.LINE3_BYTES, 8);
      this.memory.free(tolerancePtr, KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  createArc(arc: RgmArc3, tolerance: RgmToleranceContext): bigint {
    this.ensureAlive();

    const arcPtr = this.memory.alloc(KERNEL_LAYOUT.ARC3_BYTES, 8);
    const tolerancePtr = this.memory.alloc(KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);

    try {
      this.memory.writeArc(arcPtr, arc);
      this.memory.writeTolerance(tolerancePtr, tolerance);
      const status = this.api.rgm_curve_create_arc_ptr_tol(
        this.handle,
        arcPtr,
        tolerancePtr,
        outObjectPtr,
      ) as RgmStatus;
      this.assertOk(status, "Arc construction failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(arcPtr, KERNEL_LAYOUT.ARC3_BYTES, 8);
      this.memory.free(tolerancePtr, KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  createCircle(circle: RgmCircle3, tolerance: RgmToleranceContext): bigint {
    this.ensureAlive();

    const circlePtr = this.memory.alloc(KERNEL_LAYOUT.CIRCLE3_BYTES, 8);
    const tolerancePtr = this.memory.alloc(KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);

    try {
      this.memory.writeCircle(circlePtr, circle);
      this.memory.writeTolerance(tolerancePtr, tolerance);
      const status = this.api.rgm_curve_create_circle_ptr_tol(
        this.handle,
        circlePtr,
        tolerancePtr,
        outObjectPtr,
      ) as RgmStatus;
      this.assertOk(status, "Circle construction failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(circlePtr, KERNEL_LAYOUT.CIRCLE3_BYTES, 8);
      this.memory.free(tolerancePtr, KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  createPolyline(points: RgmPoint3[], closed: boolean, tolerance: RgmToleranceContext): bigint {
    this.ensureAlive();
    if (points.length < 2) {
      throw new Error("Polyline requires at least two points");
    }

    const pointsBytes = points.length * KERNEL_LAYOUT.POINT3_BYTES;
    const pointsPtr = this.memory.alloc(pointsBytes, 8);
    const tolerancePtr = this.memory.alloc(KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);

    try {
      this.memory.writePointArray(pointsPtr, points);
      this.memory.writeTolerance(tolerancePtr, tolerance);
      const status = this.api.rgm_curve_create_polyline_ptr_tol(
        this.handle,
        pointsPtr,
        points.length,
        closed,
        tolerancePtr,
        outObjectPtr,
      ) as RgmStatus;
      this.assertOk(status, "Polyline construction failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(pointsPtr, pointsBytes, 8);
      this.memory.free(tolerancePtr, KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  createPolycurve(segments: RgmPolycurveSegment[], tolerance: RgmToleranceContext): bigint {
    this.ensureAlive();
    if (segments.length === 0) {
      throw new Error("Polycurve requires at least one segment");
    }

    const segmentsBytes = segments.length * KERNEL_LAYOUT.POLYCURVE_SEGMENT_BYTES;
    const segmentsPtr = this.memory.alloc(segmentsBytes, 8);
    const tolerancePtr = this.memory.alloc(KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);

    try {
      this.memory.writePolycurveSegmentArray(segmentsPtr, segments);
      this.memory.writeTolerance(tolerancePtr, tolerance);
      const status = this.api.rgm_curve_create_polycurve_ptr_tol(
        this.handle,
        segmentsPtr,
        segments.length,
        tolerancePtr,
        outObjectPtr,
      ) as RgmStatus;
      this.assertOk(status, "Polycurve construction failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(segmentsPtr, segmentsBytes, 8);
      this.memory.free(tolerancePtr, KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  createMeshBox(center: RgmPoint3, size: RgmVec3): bigint {
    this.ensureAlive();
    const centerPtr = this.memory.alloc(KERNEL_LAYOUT.POINT3_BYTES, 8);
    const sizePtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writePoint(centerPtr, center);
      this.memory.writeVec(sizePtr, size);
      const status = this.api.rgm_mesh_create_box_ptr(
        this.handle,
        centerPtr,
        sizePtr,
        outObjectPtr,
      ) as RgmStatus;
      this.assertOk(status, "Mesh box construction failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(centerPtr, KERNEL_LAYOUT.POINT3_BYTES, 8);
      this.memory.free(sizePtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  createMeshUvSphere(center: RgmPoint3, radius: number, uSteps: number, vSteps: number): bigint {
    this.ensureAlive();
    const centerPtr = this.memory.alloc(KERNEL_LAYOUT.POINT3_BYTES, 8);
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writePoint(centerPtr, center);
      const status = this.api.rgm_mesh_create_uv_sphere_ptr(
        this.handle,
        centerPtr,
        radius,
        Math.max(3, Math.floor(uSteps)),
        Math.max(2, Math.floor(vSteps)),
        outObjectPtr,
      ) as RgmStatus;
      this.assertOk(status, "Mesh UV sphere construction failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(centerPtr, KERNEL_LAYOUT.POINT3_BYTES, 8);
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  createMeshTorus(
    center: RgmPoint3,
    majorRadius: number,
    minorRadius: number,
    majorSteps: number,
    minorSteps: number,
  ): bigint {
    this.ensureAlive();
    const centerPtr = this.memory.alloc(KERNEL_LAYOUT.POINT3_BYTES, 8);
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writePoint(centerPtr, center);
      const status = this.api.rgm_mesh_create_torus_ptr(
        this.handle,
        centerPtr,
        majorRadius,
        minorRadius,
        Math.max(3, Math.floor(majorSteps)),
        Math.max(3, Math.floor(minorSteps)),
        outObjectPtr,
      ) as RgmStatus;
      this.assertOk(status, "Mesh torus construction failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(centerPtr, KERNEL_LAYOUT.POINT3_BYTES, 8);
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  meshTranslate(meshHandle: bigint, delta: RgmVec3): bigint {
    this.ensureAlive();
    const deltaPtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writeVec(deltaPtr, delta);
      const status = this.api.rgm_mesh_translate_ptr(
        this.handle,
        meshHandle,
        deltaPtr,
        outObjectPtr,
      ) as RgmStatus;
      this.assertOk(status, "Mesh translate failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(deltaPtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  meshRotate(meshHandle: bigint, axis: RgmVec3, angleRad: number, pivot: RgmPoint3): bigint {
    this.ensureAlive();
    const axisPtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    const pivotPtr = this.memory.alloc(KERNEL_LAYOUT.POINT3_BYTES, 8);
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writeVec(axisPtr, axis);
      this.memory.writePoint(pivotPtr, pivot);
      const status = this.api.rgm_mesh_rotate_ptr(
        this.handle,
        meshHandle,
        axisPtr,
        angleRad,
        pivotPtr,
        outObjectPtr,
      ) as RgmStatus;
      this.assertOk(status, "Mesh rotation failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(axisPtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
      this.memory.free(pivotPtr, KERNEL_LAYOUT.POINT3_BYTES, 8);
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  meshScale(meshHandle: bigint, scale: RgmVec3, pivot: RgmPoint3): bigint {
    this.ensureAlive();
    const scalePtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    const pivotPtr = this.memory.alloc(KERNEL_LAYOUT.POINT3_BYTES, 8);
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writeVec(scalePtr, scale);
      this.memory.writePoint(pivotPtr, pivot);
      const status = this.api.rgm_mesh_scale_ptr(
        this.handle,
        meshHandle,
        scalePtr,
        pivotPtr,
        outObjectPtr,
      ) as RgmStatus;
      this.assertOk(status, "Mesh scale failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(scalePtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
      this.memory.free(pivotPtr, KERNEL_LAYOUT.POINT3_BYTES, 8);
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  meshBakeTransform(meshHandle: bigint): bigint {
    this.ensureAlive();
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_mesh_bake_transform(
        this.handle,
        meshHandle,
        outObjectPtr,
      ) as RgmStatus;
      this.assertOk(status, "Mesh bake transform failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  meshBoolean(meshA: bigint, meshB: bigint, op: 0 | 1 | 2): bigint {
    this.ensureAlive();
    const outObjectPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_mesh_boolean(this.handle, meshA, meshB, op, outObjectPtr) as RgmStatus;
      this.assertOk(status, "Mesh boolean failed");
      return this.memory.readU64(outObjectPtr);
    } finally {
      this.memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  intersectMeshPlane(meshHandle: bigint, plane: RgmPlane): RgmPoint3[] {
    this.ensureAlive();
    const planePtr = this.memory.alloc(KERNEL_LAYOUT.PLANE_BYTES, 8);
    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      this.memory.writePlane(planePtr, plane);
      let status = this.api.rgm_intersect_mesh_plane(
        this.handle,
        meshHandle,
        planePtr,
        0,
        0,
        countPtr,
      ) as RgmStatus;
      this.assertOk(status, "Mesh-plane intersection failed");
      const count = this.memory.readU32(countPtr);
      if (count === 0) {
        return [];
      }
      const pointsPtr = this.memory.alloc(count * KERNEL_LAYOUT.POINT3_BYTES, 8);
      try {
        status = this.api.rgm_intersect_mesh_plane(
          this.handle,
          meshHandle,
          planePtr,
          pointsPtr,
          count,
          countPtr,
        ) as RgmStatus;
        this.assertOk(status, "Mesh-plane intersection failed");
        const actual = this.memory.readU32(countPtr);
        const points: RgmPoint3[] = [];
        for (let idx = 0; idx < actual; idx += 1) {
          points.push(this.memory.readPoint(pointsPtr + idx * KERNEL_LAYOUT.POINT3_BYTES));
        }
        return points;
      } finally {
        this.memory.free(pointsPtr, count * KERNEL_LAYOUT.POINT3_BYTES, 8);
      }
    } finally {
      this.memory.free(planePtr, KERNEL_LAYOUT.PLANE_BYTES, 8);
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  intersectMeshMesh(meshA: bigint, meshB: bigint): RgmPoint3[] {
    this.ensureAlive();
    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      let status = this.api.rgm_intersect_mesh_mesh(
        this.handle,
        meshA,
        meshB,
        0,
        0,
        countPtr,
      ) as RgmStatus;
      this.assertOk(status, "Mesh-mesh intersection failed");
      const count = this.memory.readU32(countPtr);
      if (count === 0) {
        return [];
      }
      const pointsPtr = this.memory.alloc(count * KERNEL_LAYOUT.POINT3_BYTES, 8);
      try {
        status = this.api.rgm_intersect_mesh_mesh(
          this.handle,
          meshA,
          meshB,
          pointsPtr,
          count,
          countPtr,
        ) as RgmStatus;
        this.assertOk(status, "Mesh-mesh intersection failed");
        const actual = this.memory.readU32(countPtr);
        const points: RgmPoint3[] = [];
        for (let idx = 0; idx < actual; idx += 1) {
          points.push(this.memory.readPoint(pointsPtr + idx * KERNEL_LAYOUT.POINT3_BYTES));
        }
        return points;
      } finally {
        this.memory.free(pointsPtr, count * KERNEL_LAYOUT.POINT3_BYTES, 8);
      }
    } finally {
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  meshVertexCount(meshHandle: bigint): number {
    this.ensureAlive();
    const outCountPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const status = this.api.rgm_mesh_vertex_count(this.handle, meshHandle, outCountPtr) as RgmStatus;
      this.assertOk(status, "Mesh vertex count failed");
      return this.memory.readU32(outCountPtr);
    } finally {
      this.memory.free(outCountPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  meshTriangleCount(meshHandle: bigint): number {
    this.ensureAlive();
    const outCountPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const status = this.api.rgm_mesh_triangle_count(this.handle, meshHandle, outCountPtr) as RgmStatus;
      this.assertOk(status, "Mesh triangle count failed");
      return this.memory.readU32(outCountPtr);
    } finally {
      this.memory.free(outCountPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  meshToBuffers(meshHandle: bigint): { vertices: RgmPoint3[]; indices: number[] } {
    this.ensureAlive();

    const vertexCountPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    const indexCountPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      let status = this.api.rgm_mesh_copy_vertices(
        this.handle,
        meshHandle,
        0,
        0,
        vertexCountPtr,
      ) as RgmStatus;
      this.assertOk(status, "Mesh copy vertices failed");
      const vertexCount = this.memory.readU32(vertexCountPtr);

      status = this.api.rgm_mesh_copy_indices(this.handle, meshHandle, 0, 0, indexCountPtr) as RgmStatus;
      this.assertOk(status, "Mesh copy indices failed");
      const indexCount = this.memory.readU32(indexCountPtr);

      const verticesPtr = this.memory.alloc(vertexCount * KERNEL_LAYOUT.POINT3_BYTES, 8);
      const indicesPtr = this.memory.alloc(indexCount * KERNEL_LAYOUT.I32_BYTES, 4);
      try {
        status = this.api.rgm_mesh_copy_vertices(
          this.handle,
          meshHandle,
          verticesPtr,
          vertexCount,
          vertexCountPtr,
        ) as RgmStatus;
        this.assertOk(status, "Mesh copy vertices failed");
        status = this.api.rgm_mesh_copy_indices(
          this.handle,
          meshHandle,
          indicesPtr,
          indexCount,
          indexCountPtr,
        ) as RgmStatus;
        this.assertOk(status, "Mesh copy indices failed");

        const actualVertexCount = this.memory.readU32(vertexCountPtr);
        const actualIndexCount = this.memory.readU32(indexCountPtr);
        const vertices: RgmPoint3[] = [];
        for (let idx = 0; idx < actualVertexCount; idx += 1) {
          vertices.push(this.memory.readPoint(verticesPtr + idx * KERNEL_LAYOUT.POINT3_BYTES));
        }
        const indices: number[] = [];
        for (let idx = 0; idx < actualIndexCount; idx += 1) {
          indices.push(this.memory.readU32(indicesPtr + idx * KERNEL_LAYOUT.I32_BYTES));
        }
        return { vertices, indices };
      } finally {
        this.memory.free(verticesPtr, vertexCount * KERNEL_LAYOUT.POINT3_BYTES, 8);
        this.memory.free(indicesPtr, indexCount * KERNEL_LAYOUT.I32_BYTES, 4);
      }
    } finally {
      this.memory.free(vertexCountPtr, KERNEL_LAYOUT.I32_BYTES, 4);
      this.memory.free(indexCountPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  createNurbsSurface(
    desc: RgmNurbsSurfaceDesc,
    controlPoints: RgmPoint3[],
    weights: number[],
    knotsU: number[],
    knotsV: number[],
    tolerance: RgmToleranceContext,
  ): bigint {
    this.ensureAlive();
    if (
      controlPoints.length === 0 ||
      weights.length !== controlPoints.length ||
      desc.control_u_count * desc.control_v_count !== controlPoints.length
    ) {
      throw new Error("Invalid surface control net/weights");
    }
    const descPtr = this.memory.alloc(KERNEL_LAYOUT.NURBS_SURFACE_DESC_BYTES, 8);
    const ctrlPtr = this.memory.alloc(controlPoints.length * KERNEL_LAYOUT.POINT3_BYTES, 8);
    const weightsPtr = this.memory.alloc(weights.length * KERNEL_LAYOUT.F64_BYTES, 8);
    const knotsUPtr = this.memory.alloc(knotsU.length * KERNEL_LAYOUT.F64_BYTES, 8);
    const knotsVPtr = this.memory.alloc(knotsV.length * KERNEL_LAYOUT.F64_BYTES, 8);
    const tolPtr = this.memory.alloc(KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);

    try {
      this.memory.writeNurbsSurfaceDesc(descPtr, desc);
      this.memory.writePointArray(ctrlPtr, controlPoints);
      this.memory.writeF64Array(weightsPtr, weights);
      this.memory.writeF64Array(knotsUPtr, knotsU);
      this.memory.writeF64Array(knotsVPtr, knotsV);
      this.memory.writeTolerance(tolPtr, tolerance);

      const status = this.api.rgm_surface_create_nurbs(
        this.handle,
        descPtr,
        ctrlPtr,
        controlPoints.length,
        weightsPtr,
        weights.length,
        knotsUPtr,
        knotsU.length,
        knotsVPtr,
        knotsV.length,
        tolPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface construction failed");
      return this.memory.readU64(outPtr);
    } finally {
      this.memory.free(descPtr, KERNEL_LAYOUT.NURBS_SURFACE_DESC_BYTES, 8);
      this.memory.free(ctrlPtr, controlPoints.length * KERNEL_LAYOUT.POINT3_BYTES, 8);
      this.memory.free(weightsPtr, weights.length * KERNEL_LAYOUT.F64_BYTES, 8);
      this.memory.free(knotsUPtr, knotsU.length * KERNEL_LAYOUT.F64_BYTES, 8);
      this.memory.free(knotsVPtr, knotsV.length * KERNEL_LAYOUT.F64_BYTES, 8);
      this.memory.free(tolPtr, KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  surfacePointAt(surfaceHandle: bigint, uvNorm: RgmUv2): RgmPoint3 {
    this.ensureAlive();
    const uvPtr = this.memory.alloc(KERNEL_LAYOUT.UV2_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.POINT3_BYTES, 8);
    try {
      this.memory.writeUv(uvPtr, uvNorm);
      const status = this.api.rgm_surface_point_at(this.handle, surfaceHandle, uvPtr, outPtr) as RgmStatus;
      this.assertOk(status, "Surface point evaluation failed");
      return this.memory.readPoint(outPtr);
    } finally {
      this.memory.free(uvPtr, KERNEL_LAYOUT.UV2_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.POINT3_BYTES, 8);
    }
  }

  surfaceFrameAt(surfaceHandle: bigint, uvNorm: RgmUv2): RgmSurfaceEvalFrame {
    this.ensureAlive();
    const uvPtr = this.memory.alloc(KERNEL_LAYOUT.UV2_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.SURFACE_EVAL_FRAME_BYTES, 8);
    try {
      this.memory.writeUv(uvPtr, uvNorm);
      const status = this.api.rgm_surface_frame_at(this.handle, surfaceHandle, uvPtr, outPtr) as RgmStatus;
      this.assertOk(status, "Surface frame evaluation failed");
      return this.memory.readSurfaceEvalFrame(outPtr);
    } finally {
      this.memory.free(uvPtr, KERNEL_LAYOUT.UV2_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.SURFACE_EVAL_FRAME_BYTES, 8);
    }
  }

  surfaceTranslate(surfaceHandle: bigint, delta: RgmVec3): bigint {
    this.ensureAlive();
    const deltaPtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writeVec(deltaPtr, delta);
      const status = this.api.rgm_surface_translate(
        this.handle,
        surfaceHandle,
        deltaPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface translation failed");
      return this.memory.readU64(outPtr);
    } finally {
      this.memory.free(deltaPtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  surfaceRotate(surfaceHandle: bigint, axis: RgmVec3, angleRad: number, pivot: RgmPoint3): bigint {
    this.ensureAlive();
    const axisPtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    const pivotPtr = this.memory.alloc(KERNEL_LAYOUT.POINT3_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writeVec(axisPtr, axis);
      this.memory.writePoint(pivotPtr, pivot);
      const status = this.api.rgm_surface_rotate(
        this.handle,
        surfaceHandle,
        axisPtr,
        angleRad,
        pivotPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface rotation failed");
      return this.memory.readU64(outPtr);
    } finally {
      this.memory.free(axisPtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
      this.memory.free(pivotPtr, KERNEL_LAYOUT.POINT3_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  surfaceScale(surfaceHandle: bigint, scale: RgmVec3, pivot: RgmPoint3): bigint {
    this.ensureAlive();
    const scalePtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    const pivotPtr = this.memory.alloc(KERNEL_LAYOUT.POINT3_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writeVec(scalePtr, scale);
      this.memory.writePoint(pivotPtr, pivot);
      const status = this.api.rgm_surface_scale(
        this.handle,
        surfaceHandle,
        scalePtr,
        pivotPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface scale failed");
      return this.memory.readU64(outPtr);
    } finally {
      this.memory.free(scalePtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
      this.memory.free(pivotPtr, KERNEL_LAYOUT.POINT3_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  surfaceBakeTransform(surfaceHandle: bigint): bigint {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_surface_bake_transform(this.handle, surfaceHandle, outPtr) as RgmStatus;
      this.assertOk(status, "Surface bake failed");
      return this.memory.readU64(outPtr);
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  surfaceTessellateToMesh(surfaceHandle: bigint, options?: RgmSurfaceTessellationOptions): bigint {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    const optionsPtr = options
      ? this.memory.alloc(KERNEL_LAYOUT.SURFACE_TESSELLATION_OPTIONS_BYTES, 8)
      : 0;
    try {
      if (options && optionsPtr !== 0) {
        this.memory.writeSurfaceTessellationOptions(optionsPtr, options);
      }
      const status = this.api.rgm_surface_tessellate_to_mesh(
        this.handle,
        surfaceHandle,
        optionsPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface tessellation failed");
      return this.memory.readU64(outPtr);
    } finally {
      if (optionsPtr !== 0) {
        this.memory.free(optionsPtr, KERNEL_LAYOUT.SURFACE_TESSELLATION_OPTIONS_BYTES, 8);
      }
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  createFaceFromSurface(surfaceHandle: bigint): bigint {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_face_create_from_surface(this.handle, surfaceHandle, outPtr) as RgmStatus;
      this.assertOk(status, "Face creation failed");
      return this.memory.readU64(outPtr);
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  faceAddLoop(faceHandle: bigint, points: RgmUv2[], isOuter: boolean): void {
    this.ensureAlive();
    const pointsPtr = this.memory.alloc(points.length * KERNEL_LAYOUT.UV2_BYTES, 8);
    try {
      for (let idx = 0; idx < points.length; idx += 1) {
        this.memory.writeUv(pointsPtr + idx * KERNEL_LAYOUT.UV2_BYTES, points[idx]);
      }
      const status = this.api.rgm_face_add_loop(
        this.handle,
        faceHandle,
        pointsPtr,
        points.length,
        isOuter,
      ) as RgmStatus;
      this.assertOk(status, "Face add loop failed");
    } finally {
      this.memory.free(pointsPtr, points.length * KERNEL_LAYOUT.UV2_BYTES, 8);
    }
  }

  faceRemoveLoop(faceHandle: bigint, loopIndex: number): void {
    this.ensureAlive();
    const status = this.api.rgm_face_remove_loop(this.handle, faceHandle, loopIndex) as RgmStatus;
    this.assertOk(status, "Face remove loop failed");
  }

  faceSplitTrimEdge(faceHandle: bigint, loopIndex: number, edgeIndex: number, splitT: number): void {
    this.ensureAlive();
    const status = this.api.rgm_face_split_trim_edge(
      this.handle,
      faceHandle,
      loopIndex,
      edgeIndex,
      splitT,
    ) as RgmStatus;
    this.assertOk(status, "Face split trim edge failed");
  }

  faceReverseLoop(faceHandle: bigint, loopIndex: number): void {
    this.ensureAlive();
    const status = this.api.rgm_face_reverse_loop(this.handle, faceHandle, loopIndex) as RgmStatus;
    this.assertOk(status, "Face reverse loop failed");
  }

  faceValidate(faceHandle: bigint): boolean {
    this.ensureAlive();
    const validPtr = this.memory.alloc(1, 1);
    try {
      const status = this.api.rgm_face_validate(this.handle, faceHandle, validPtr) as RgmStatus;
      this.assertOk(status, "Face validation failed");
      return this.memory.readBool(validPtr);
    } finally {
      this.memory.free(validPtr, 1, 1);
    }
  }

  faceHeal(faceHandle: bigint): void {
    this.ensureAlive();
    const status = this.api.rgm_face_heal(this.handle, faceHandle) as RgmStatus;
    this.assertOk(status, "Face heal failed");
  }

  faceTessellateToMesh(faceHandle: bigint, options?: RgmSurfaceTessellationOptions): bigint {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    const optionsPtr = options
      ? this.memory.alloc(KERNEL_LAYOUT.SURFACE_TESSELLATION_OPTIONS_BYTES, 8)
      : 0;
    try {
      if (options && optionsPtr !== 0) {
        this.memory.writeSurfaceTessellationOptions(optionsPtr, options);
      }
      const status = this.api.rgm_face_tessellate_to_mesh(
        this.handle,
        faceHandle,
        optionsPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Face tessellation failed");
      return this.memory.readU64(outPtr);
    } finally {
      if (optionsPtr !== 0) {
        this.memory.free(optionsPtr, KERNEL_LAYOUT.SURFACE_TESSELLATION_OPTIONS_BYTES, 8);
      }
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  intersectSurfaceSurface(surfaceA: bigint, surfaceB: bigint): bigint {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_intersect_surface_surface(
        this.handle,
        surfaceA,
        surfaceB,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface-surface intersection failed");
      return this.memory.readU64(outPtr);
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  intersectSurfacePlane(surface: bigint, plane: RgmPlane): bigint {
    this.ensureAlive();
    const planePtr = this.memory.alloc(KERNEL_LAYOUT.PLANE_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writePlane(planePtr, plane);
      const status = this.api.rgm_intersect_surface_plane(this.handle, surface, planePtr, outPtr) as RgmStatus;
      this.assertOk(status, "Surface-plane intersection failed");
      return this.memory.readU64(outPtr);
    } finally {
      this.memory.free(planePtr, KERNEL_LAYOUT.PLANE_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  intersectSurfaceCurve(surface: bigint, curve: bigint): bigint {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_intersect_surface_curve(this.handle, surface, curve, outPtr) as RgmStatus;
      this.assertOk(status, "Surface-curve intersection failed");
      return this.memory.readU64(outPtr);
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  intersectionBranchCount(intersection: bigint): number {
    this.ensureAlive();
    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const status = this.api.rgm_intersection_branch_count(this.handle, intersection, countPtr) as RgmStatus;
      this.assertOk(status, "Intersection branch count failed");
      return this.memory.readU32(countPtr);
    } finally {
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  intersectionBranchSummary(intersection: bigint, branchIndex: number): RgmIntersectionBranchSummary {
    this.ensureAlive();
    const ptr = this.memory.alloc(KERNEL_LAYOUT.INTERSECTION_BRANCH_SUMMARY_BYTES, 8);
    try {
      const status = this.api.rgm_intersection_branch_summary(
        this.handle,
        intersection,
        branchIndex,
        ptr,
      ) as RgmStatus;
      this.assertOk(status, "Intersection branch summary failed");
      return this.memory.readIntersectionBranchSummary(ptr);
    } finally {
      this.memory.free(ptr, KERNEL_LAYOUT.INTERSECTION_BRANCH_SUMMARY_BYTES, 8);
    }
  }

  intersectionBranchPoints(intersection: bigint, branchIndex: number): RgmPoint3[] {
    this.ensureAlive();
    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      let status = this.api.rgm_intersection_copy_branch_points(
        this.handle,
        intersection,
        branchIndex,
        0,
        0,
        countPtr,
      ) as RgmStatus;
      this.assertOk(status, "Intersection points copy failed");
      const count = this.memory.readU32(countPtr);
      if (count === 0) return [];
      const pointsPtr = this.memory.alloc(count * KERNEL_LAYOUT.POINT3_BYTES, 8);
      try {
        status = this.api.rgm_intersection_copy_branch_points(
          this.handle,
          intersection,
          branchIndex,
          pointsPtr,
          count,
          countPtr,
        ) as RgmStatus;
        this.assertOk(status, "Intersection points copy failed");
        const actual = this.memory.readU32(countPtr);
        const points: RgmPoint3[] = [];
        for (let idx = 0; idx < actual; idx += 1) {
          points.push(this.memory.readPoint(pointsPtr + idx * KERNEL_LAYOUT.POINT3_BYTES));
        }
        return points;
      } finally {
        this.memory.free(pointsPtr, count * KERNEL_LAYOUT.POINT3_BYTES, 8);
      }
    } finally {
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  intersectionBranchUvA(intersection: bigint, branchIndex: number): RgmUv2[] {
    return this.copyIntersectionUv(intersection, branchIndex, "a");
  }

  intersectionBranchUvB(intersection: bigint, branchIndex: number): RgmUv2[] {
    return this.copyIntersectionUv(intersection, branchIndex, "b");
  }

  intersectionBranchCurveT(intersection: bigint, branchIndex: number): number[] {
    this.ensureAlive();
    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      let status = this.api.rgm_intersection_copy_branch_curve_t(
        this.handle,
        intersection,
        branchIndex,
        0,
        0,
        countPtr,
      ) as RgmStatus;
      this.assertOk(status, "Intersection curve-t copy failed");
      const count = this.memory.readU32(countPtr);
      if (count === 0) return [];
      const valuesPtr = this.memory.alloc(count * KERNEL_LAYOUT.F64_BYTES, 8);
      try {
        status = this.api.rgm_intersection_copy_branch_curve_t(
          this.handle,
          intersection,
          branchIndex,
          valuesPtr,
          count,
          countPtr,
        ) as RgmStatus;
        this.assertOk(status, "Intersection curve-t copy failed");
        const actual = this.memory.readU32(countPtr);
        const values: number[] = [];
        for (let idx = 0; idx < actual; idx += 1) {
          values.push(this.memory.readF64(valuesPtr + idx * KERNEL_LAYOUT.F64_BYTES));
        }
        return values;
      } finally {
        this.memory.free(valuesPtr, count * KERNEL_LAYOUT.F64_BYTES, 8);
      }
    } finally {
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  intersectionBranchToNurbs(
    intersection: bigint,
    branchIndex: number,
    tolerance: RgmToleranceContext,
  ): bigint {
    this.ensureAlive();
    const tolPtr = this.memory.alloc(KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writeTolerance(tolPtr, tolerance);
      const status = this.api.rgm_intersection_branch_to_nurbs(
        this.handle,
        intersection,
        branchIndex,
        tolPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Intersection branch to nurbs failed");
      return this.memory.readU64(outPtr);
    } finally {
      this.memory.free(tolPtr, KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  sampleCurvePolyline(curveHandle: bigint, sampleCount: number): RgmPoint3[] {
    this.ensureAlive();
    return sampleCurvePolyline(
      {
        api: this.api,
        memory: this.memory,
        session: this.handle,
        getLastErrorMessage: () => this.lastError().message,
      },
      curveHandle,
      sampleCount,
    );
  }

  pointAt(curveHandle: bigint, tNorm: number): RgmPoint3 {
    this.ensureAlive();
    if (tNorm < 0 || tNorm > 1) {
      throw new Error("tNorm must be within [0, 1]");
    }

    const pointPtr = this.memory.alloc(KERNEL_LAYOUT.POINT3_BYTES, 8);
    try {
      const status = this.api.rgm_curve_point_at(
        this.handle,
        curveHandle,
        tNorm,
        pointPtr,
      ) as RgmStatus;
      this.assertOk(status, "Curve point evaluation failed");
      return this.memory.readPoint(pointPtr);
    } finally {
      this.memory.free(pointPtr, KERNEL_LAYOUT.POINT3_BYTES, 8);
    }
  }

  curveLength(curveHandle: bigint): number {
    this.ensureAlive();

    const outLengthPtr = this.memory.alloc(KERNEL_LAYOUT.F64_BYTES, 8);
    try {
      const status = this.api.rgm_curve_length(
        this.handle,
        curveHandle,
        outLengthPtr,
      ) as RgmStatus;
      this.assertOk(status, "Curve length evaluation failed");
      return this.memory.readF64(outLengthPtr);
    } finally {
      this.memory.free(outLengthPtr, KERNEL_LAYOUT.F64_BYTES, 8);
    }
  }

  curveLengthAt(curveHandle: bigint, tNorm: number): number {
    this.ensureAlive();
    if (tNorm < 0 || tNorm > 1) {
      throw new Error("tNorm must be within [0, 1]");
    }

    const outLengthPtr = this.memory.alloc(KERNEL_LAYOUT.F64_BYTES, 8);
    try {
      const status = this.api.rgm_curve_length_at(
        this.handle,
        curveHandle,
        tNorm,
        outLengthPtr,
      ) as RgmStatus;
      this.assertOk(status, "Curve length-at-parameter evaluation failed");
      return this.memory.readF64(outLengthPtr);
    } finally {
      this.memory.free(outLengthPtr, KERNEL_LAYOUT.F64_BYTES, 8);
    }
  }

  intersectCurvePlane(curveHandle: bigint, plane: RgmPlane): RgmPoint3[] {
    this.ensureAlive();

    const planePtr = this.memory.alloc(KERNEL_LAYOUT.PLANE_BYTES, 8);
    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      this.memory.writePlane(planePtr, plane);
      let status = this.api.rgm_intersect_curve_plane_ptr(
        this.handle,
        curveHandle,
        planePtr,
        0,
        0,
        countPtr,
      ) as RgmStatus;
      this.assertOk(status, "Curve-plane intersection failed");

      const count = this.memory.readU32(countPtr);
      if (count === 0) {
        return [];
      }

      const pointsPtr = this.memory.alloc(count * KERNEL_LAYOUT.POINT3_BYTES, 8);
      try {
        status = this.api.rgm_intersect_curve_plane_ptr(
          this.handle,
          curveHandle,
          planePtr,
          pointsPtr,
          count,
          countPtr,
        ) as RgmStatus;
        this.assertOk(status, "Curve-plane intersection failed");

        const actual = this.memory.readU32(countPtr);
        const points: RgmPoint3[] = [];
        for (let idx = 0; idx < actual; idx += 1) {
          points.push(this.memory.readPoint(pointsPtr + idx * KERNEL_LAYOUT.POINT3_BYTES));
        }
        return points;
      } finally {
        this.memory.free(pointsPtr, count * KERNEL_LAYOUT.POINT3_BYTES, 8);
      }
    } finally {
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
      this.memory.free(planePtr, KERNEL_LAYOUT.PLANE_BYTES, 8);
    }
  }

  intersectCurveCurve(curveA: bigint, curveB: bigint): RgmPoint3[] {
    this.ensureAlive();

    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      let status = this.api.rgm_intersect_curve_curve(
        this.handle,
        curveA,
        curveB,
        0,
        0,
        countPtr,
      ) as RgmStatus;
      this.assertOk(status, "Curve-curve intersection failed");

      const count = this.memory.readU32(countPtr);
      if (count === 0) {
        return [];
      }

      const pointsPtr = this.memory.alloc(count * KERNEL_LAYOUT.POINT3_BYTES, 8);
      try {
        status = this.api.rgm_intersect_curve_curve(
          this.handle,
          curveA,
          curveB,
          pointsPtr,
          count,
          countPtr,
        ) as RgmStatus;
        this.assertOk(status, "Curve-curve intersection failed");

        const actual = this.memory.readU32(countPtr);
        const points: RgmPoint3[] = [];
        for (let idx = 0; idx < actual; idx += 1) {
          points.push(this.memory.readPoint(pointsPtr + idx * KERNEL_LAYOUT.POINT3_BYTES));
        }
        return points;
      } finally {
        this.memory.free(pointsPtr, count * KERNEL_LAYOUT.POINT3_BYTES, 8);
      }
    } finally {
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  releaseObject(objectHandle: bigint): void {
    this.ensureAlive();
    const status = this.api.rgm_object_release(this.handle, objectHandle) as RgmStatus;
    if (status !== RgmStatus.Ok && status !== RgmStatus.NotFound) {
      this.assertOk(status, "Object release failed");
    }
  }

  lastError(): { code: number; message: string } {
    this.ensureAlive();

    const codePtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    const bufferLen = 2048;
    const messagePtr = this.memory.alloc(bufferLen, 1);
    const writtenPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);

    try {
      const statusCode = this.api.rgm_last_error_code(this.handle, codePtr) as RgmStatus;
      const statusMessage = this.api.rgm_last_error_message(
        this.handle,
        messagePtr,
        bufferLen,
        writtenPtr,
      ) as RgmStatus;

      if (statusCode !== RgmStatus.Ok || statusMessage !== RgmStatus.Ok) {
        return {
          code: -1,
          message: "Unable to retrieve kernel error",
        };
      }

      const code = this.memory.readI32(codePtr);
      const written = this.memory.readU32(writtenPtr);
      const bytes = this.memory.readBytes(messagePtr, written);
      return {
        code,
        message: this.decoder.decode(bytes),
      };
    } finally {
      this.memory.free(codePtr, KERNEL_LAYOUT.I32_BYTES, 4);
      this.memory.free(messagePtr, bufferLen, 1);
      this.memory.free(writtenPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  destroy(): void {
    if (this.destroyed) {
      return;
    }

    const status = this.api.rgm_kernel_destroy(this.handle) as RgmStatus;
    if (status !== RgmStatus.Ok && status !== RgmStatus.NotFound) {
      this.assertOk(status, "Kernel session destroy failed");
    }

    this.destroyed = true;
    this.onDestroy();
  }

  private copyIntersectionUv(
    intersection: bigint,
    branchIndex: number,
    kind: "a" | "b",
  ): RgmUv2[] {
    this.ensureAlive();
    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const fn =
        kind === "a"
          ? this.api.rgm_intersection_copy_branch_uv_on_surface_a
          : this.api.rgm_intersection_copy_branch_uv_on_surface_b;
      let status = fn(this.handle, intersection, branchIndex, 0, 0, countPtr) as RgmStatus;
      this.assertOk(status, "Intersection uv copy failed");
      const count = this.memory.readU32(countPtr);
      if (count === 0) return [];
      const valuesPtr = this.memory.alloc(count * KERNEL_LAYOUT.UV2_BYTES, 8);
      try {
        status = fn(this.handle, intersection, branchIndex, valuesPtr, count, countPtr) as RgmStatus;
        this.assertOk(status, "Intersection uv copy failed");
        const actual = this.memory.readU32(countPtr);
        const points: RgmUv2[] = [];
        for (let idx = 0; idx < actual; idx += 1) {
          points.push(this.memory.readUv(valuesPtr + idx * KERNEL_LAYOUT.UV2_BYTES));
        }
        return points;
      } finally {
        this.memory.free(valuesPtr, count * KERNEL_LAYOUT.UV2_BYTES, 8);
      }
    } finally {
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  private assertOk(status: RgmStatus, message: string): void {
    if (status === RgmStatus.Ok) {
      return;
    }

    const details = this.lastError().message;
    throw new KernelRuntimeError(
      `${message} (${statusToName(status)})`,
      status,
      details,
    );
  }

  private ensureAlive(): void {
    if (this.destroyed) {
      throw new Error("Kernel session is already destroyed");
    }
  }
}

class KernelRuntimeImpl implements KernelRuntime {
  readonly capabilities: KernelCapabilities = {
    igesImport: false,
    igesExport: false,
  };

  private readonly sessions = new Set<KernelSessionImpl>();

  constructor(
    private readonly api: NativeExports,
    private readonly memory: KernelMemory,
  ) {}

  createSession(): KernelSession {
    const outSessionPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_kernel_create(outSessionPtr) as RgmStatus;
      if (status !== RgmStatus.Ok) {
        throw new KernelRuntimeError(
          `Kernel session create failed (${statusToName(status)})`,
          status,
        );
      }

      const handle = this.memory.readU64(outSessionPtr);
      const session = new KernelSessionImpl(this.api, this.memory, handle, () => {
        this.sessions.delete(session);
      });
      this.sessions.add(session);
      return session;
    } finally {
      this.memory.free(outSessionPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  destroy(): void {
    for (const session of [...this.sessions]) {
      session.destroy();
    }
  }
}

export async function createKernelRuntime(wasmSource: WasmSource): Promise<KernelRuntime> {
  const wasm = await loadKernelWasm(wasmSource);
  const exports = wasm.exports as unknown as NativeExports;
  const api = exports;
  const memory = new KernelMemory(api, wasm.exports.memory);
  return new KernelRuntimeImpl(api, memory);
}
