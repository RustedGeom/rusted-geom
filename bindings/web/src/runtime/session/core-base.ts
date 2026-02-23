import type { NativeExports } from "../../generated/native";
import type {
  RgmArc3,
  RgmCircle3,
  RgmLine3,
  RgmPlane,
  RgmPoint3,
  RgmPolycurveSegment,
  RgmToleranceContext,
  RgmVec3,
} from "../../generated/types";
import { RgmStatus } from "../../generated/types";
import { KernelRuntimeError, statusToName } from "../errors";
import { KERNEL_LAYOUT, KernelMemory } from "../memory";
import { sampleCurvePolyline } from "../scene-sampler";
import type { CurvePresetInput } from "./core";

export abstract class KernelSessionBase {
  protected readonly decoder = new TextDecoder();
  protected destroyed = false;

  constructor(
    protected readonly api: NativeExports,
    protected readonly memory: KernelMemory,
    readonly handle: bigint,
    protected readonly onDestroy: () => void,
  ) {}

  protected abstract lastError(): { code: number; message: string };
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
      const status = this.api.rgm_nurbs_interpolate_fit_points(
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
      const status = this.api.rgm_curve_create_line(
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
      const status = this.api.rgm_curve_create_arc(
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
      const status = this.api.rgm_curve_create_circle(
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
      const status = this.api.rgm_curve_create_polyline(
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
      const status = this.api.rgm_curve_create_polycurve(
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
      const status = this.api.rgm_mesh_create_box(
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
      const status = this.api.rgm_mesh_create_uv_sphere(
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
      const status = this.api.rgm_mesh_create_torus(
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
      const status = this.api.rgm_mesh_translate(
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
      const status = this.api.rgm_mesh_rotate(
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
      const status = this.api.rgm_mesh_scale(
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
      let status = this.api.rgm_intersect_curve_plane(
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
        status = this.api.rgm_intersect_curve_plane(
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

  protected assertOk(status: RgmStatus, message: string): void {
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

  protected ensureAlive(): void {
    if (this.destroyed) {
      throw new Error("Kernel session is already destroyed");
    }
  }
}
