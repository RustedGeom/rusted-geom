import type { NativeExports } from "../generated/native";
import type {
  RgmAabb3,
  RgmBounds3,
  RgmBoundsOptions,
  RgmBrepValidationReport,
  RgmIntersectionBranchSummary,
  RgmObb3,
  RgmArc3,
  RgmCircle3,
  RgmLine3,
  RgmNurbsSurfaceDesc,
  RgmPlane,
  RgmPoint3,
  RgmPolycurveSegment,
  RgmTrimEdgeInput,
  RgmTrimLoopInput,
  RgmSurfaceEvalFrame,
  RgmSurfaceTessellationOptions,
  RgmToleranceContext,
  RgmUv2,
  RgmValidationIssue,
  RgmValidationSeverity,
  RgmVec3,
} from "../generated/types";
import { RgmStatus } from "../generated/types";
import { KernelRuntimeError } from "./errors";

const F64_BYTES = 8;
const I32_BYTES = 4;
const U64_BYTES = 8;
const U8_BYTES = 1;
const POINT3_BYTES = F64_BYTES * 3;
const VEC3_BYTES = F64_BYTES * 3;
const PLANE_BYTES = POINT3_BYTES + VEC3_BYTES * 3;
const LINE3_BYTES = POINT3_BYTES * 2;
const CIRCLE3_BYTES = PLANE_BYTES + F64_BYTES;
const ARC3_BYTES = PLANE_BYTES + F64_BYTES * 3;
const POLYCURVE_SEGMENT_BYTES = 16;
const TOLERANCE_BYTES = F64_BYTES * 3;
const UV2_BYTES = F64_BYTES * 2;
const NURBS_SURFACE_DESC_BYTES = 20;
const TRIM_LOOP_INPUT_BYTES = 8;
const TRIM_EDGE_INPUT_BYTES = UV2_BYTES * 2 + U64_BYTES + 8;
const SURFACE_EVAL_FRAME_BYTES = POINT3_BYTES + VEC3_BYTES * 3;
const SURFACE_TESSELLATION_OPTIONS_BYTES = I32_BYTES * 4 + F64_BYTES * 2;
const INTERSECTION_BRANCH_SUMMARY_BYTES = 24;
const VALIDATION_ISSUE_BYTES = I32_BYTES * 4 + F64_BYTES * 2;
const BREP_VALIDATION_REPORT_BYTES = 16 + VALIDATION_ISSUE_BYTES * 16;
const BOUNDS_OPTIONS_BYTES = 16;
const AABB3_BYTES = POINT3_BYTES * 2;
const OBB3_BYTES = POINT3_BYTES + VEC3_BYTES * 4;
const BOUNDS3_BYTES = AABB3_BYTES + OBB3_BYTES + AABB3_BYTES;

export class KernelMemory {
  constructor(
    private readonly api: NativeExports,
    private readonly wasmMemory: WebAssembly.Memory,
  ) {}

  alloc(byteLen: number, align = 8): number {
    const ptr = Number(this.api.rgm_alloc_addr(byteLen, align));
    if (!Number.isInteger(ptr) || ptr <= 0) {
      throw new KernelRuntimeError("Kernel allocation failed", RgmStatus.InternalError);
    }

    return ptr;
  }

  free(ptr: number, byteLen: number, align = 8): void {
    if (ptr <= 0) {
      return;
    }

    const status = this.api.rgm_dealloc(ptr, byteLen, align) as RgmStatus;
    if (status !== RgmStatus.Ok) {
      throw new KernelRuntimeError("Kernel deallocation failed", status);
    }
  }

  writePointArray(ptr: number, points: RgmPoint3[]): void {
    for (let idx = 0; idx < points.length; idx += 1) {
      this.writePoint(ptr + idx * POINT3_BYTES, points[idx]);
    }
  }

  writeF64Array(ptr: number, values: number[]): void {
    const view = this.dataView();
    for (let idx = 0; idx < values.length; idx += 1) {
      view.setFloat64(ptr + idx * F64_BYTES, values[idx], true);
    }
  }

  writeU64Array(ptr: number, values: bigint[]): void {
    const view = this.dataView();
    for (let idx = 0; idx < values.length; idx += 1) {
      view.setBigUint64(ptr + idx * U64_BYTES, values[idx], true);
    }
  }

  writePolycurveSegmentArray(ptr: number, segments: RgmPolycurveSegment[]): void {
    const view = this.dataView();
    for (let idx = 0; idx < segments.length; idx += 1) {
      const base = ptr + idx * POLYCURVE_SEGMENT_BYTES;
      view.setBigUint64(base, segments[idx].curve, true);
      view.setUint8(base + U64_BYTES, segments[idx].reversed ? 1 : 0);
      for (let pad = base + U64_BYTES + U8_BYTES; pad < base + POLYCURVE_SEGMENT_BYTES; pad += 1) {
        view.setUint8(pad, 0);
      }
    }
  }

  writeTolerance(ptr: number, tolerance: RgmToleranceContext): void {
    const view = this.dataView();
    view.setFloat64(ptr, tolerance.abs_tol, true);
    view.setFloat64(ptr + F64_BYTES, tolerance.rel_tol, true);
    view.setFloat64(ptr + F64_BYTES * 2, tolerance.angle_tol, true);
  }

  writeUv(ptr: number, uv: RgmUv2): void {
    const view = this.dataView();
    view.setFloat64(ptr, uv.u, true);
    view.setFloat64(ptr + F64_BYTES, uv.v, true);
  }

  readUv(ptr: number): RgmUv2 {
    const view = this.dataView();
    return {
      u: view.getFloat64(ptr, true),
      v: view.getFloat64(ptr + F64_BYTES, true),
    };
  }

  readPoint(ptr: number): RgmPoint3 {
    const view = this.dataView();
    return {
      x: view.getFloat64(ptr, true),
      y: view.getFloat64(ptr + F64_BYTES, true),
      z: view.getFloat64(ptr + F64_BYTES * 2, true),
    };
  }

  readVec(ptr: number): RgmVec3 {
    const view = this.dataView();
    return {
      x: view.getFloat64(ptr, true),
      y: view.getFloat64(ptr + F64_BYTES, true),
      z: view.getFloat64(ptr + F64_BYTES * 2, true),
    };
  }

  readI32(ptr: number): number {
    return this.dataView().getInt32(ptr, true);
  }

  readF64(ptr: number): number {
    return this.dataView().getFloat64(ptr, true);
  }

  readU32(ptr: number): number {
    return this.dataView().getUint32(ptr, true);
  }

  readU64(ptr: number): bigint {
    return this.dataView().getBigUint64(ptr, true);
  }

  readBool(ptr: number): boolean {
    return this.dataView().getUint8(ptr) !== 0;
  }

  writeBool(ptr: number, value: boolean): void {
    this.dataView().setUint8(ptr, value ? 1 : 0);
  }

  readBytes(ptr: number, length: number): Uint8Array {
    return new Uint8Array(this.wasmMemory.buffer, ptr, length);
  }

  writeBytes(ptr: number, bytes: Uint8Array): void {
    this.readBytes(ptr, bytes.length).set(bytes);
  }

  writeU64(ptr: number, value: bigint): void {
    this.dataView().setBigUint64(ptr, value, true);
  }

  writePoint(ptr: number, point: RgmPoint3): void {
    const view = this.dataView();
    view.setFloat64(ptr, point.x, true);
    view.setFloat64(ptr + F64_BYTES, point.y, true);
    view.setFloat64(ptr + F64_BYTES * 2, point.z, true);
  }

  writeVec(ptr: number, vec: RgmVec3): void {
    const view = this.dataView();
    view.setFloat64(ptr, vec.x, true);
    view.setFloat64(ptr + F64_BYTES, vec.y, true);
    view.setFloat64(ptr + F64_BYTES * 2, vec.z, true);
  }

  writePlane(ptr: number, plane: RgmPlane): void {
    this.writePoint(ptr, plane.origin);
    this.writeVec(ptr + POINT3_BYTES, plane.x_axis);
    this.writeVec(ptr + POINT3_BYTES + VEC3_BYTES, plane.y_axis);
    this.writeVec(ptr + POINT3_BYTES + VEC3_BYTES * 2, plane.z_axis);
  }

  writeLine(ptr: number, line: RgmLine3): void {
    this.writePoint(ptr, line.start);
    this.writePoint(ptr + POINT3_BYTES, line.end);
  }

  writeCircle(ptr: number, circle: RgmCircle3): void {
    this.writePlane(ptr, circle.plane);
    this.dataView().setFloat64(ptr + PLANE_BYTES, circle.radius, true);
  }

  writeArc(ptr: number, arc: RgmArc3): void {
    this.writePlane(ptr, arc.plane);
    const view = this.dataView();
    view.setFloat64(ptr + PLANE_BYTES, arc.radius, true);
    view.setFloat64(ptr + PLANE_BYTES + F64_BYTES, arc.start_angle, true);
    view.setFloat64(ptr + PLANE_BYTES + F64_BYTES * 2, arc.sweep_angle, true);
  }

  writeNurbsSurfaceDesc(ptr: number, desc: RgmNurbsSurfaceDesc): void {
    const view = this.dataView();
    view.setUint32(ptr, desc.degree_u, true);
    view.setUint32(ptr + I32_BYTES, desc.degree_v, true);
    view.setUint8(ptr + I32_BYTES * 2, desc.periodic_u ? 1 : 0);
    view.setUint8(ptr + I32_BYTES * 2 + 1, desc.periodic_v ? 1 : 0);
    view.setUint16(ptr + I32_BYTES * 2 + 2, 0, true);
    view.setUint32(ptr + 12, desc.control_u_count, true);
    view.setUint32(ptr + 16, desc.control_v_count, true);
  }

  writeTrimLoopInput(ptr: number, input: RgmTrimLoopInput): void {
    const view = this.dataView();
    view.setUint32(ptr, input.edge_count, true);
    view.setUint8(ptr + I32_BYTES, input.is_outer ? 1 : 0);
    view.setUint8(ptr + I32_BYTES + 1, 0);
    view.setUint8(ptr + I32_BYTES + 2, 0);
    view.setUint8(ptr + I32_BYTES + 3, 0);
  }

  writeTrimEdgeInput(ptr: number, input: RgmTrimEdgeInput): void {
    this.writeUv(ptr, input.start_uv);
    this.writeUv(ptr + UV2_BYTES, input.end_uv);
    const view = this.dataView();
    view.setBigUint64(ptr + UV2_BYTES * 2, input.curve_3d, true);
    view.setUint8(ptr + UV2_BYTES * 2 + U64_BYTES, input.has_curve_3d ? 1 : 0);
    for (let off = ptr + UV2_BYTES * 2 + U64_BYTES + 1; off < ptr + TRIM_EDGE_INPUT_BYTES; off += 1) {
      view.setUint8(off, 0);
    }
  }

  writeTrimEdgeInputArray(ptr: number, edges: RgmTrimEdgeInput[]): void {
    for (let idx = 0; idx < edges.length; idx += 1) {
      this.writeTrimEdgeInput(ptr + idx * TRIM_EDGE_INPUT_BYTES, edges[idx]);
    }
  }

  writeSurfaceTessellationOptions(
    ptr: number,
    options: RgmSurfaceTessellationOptions,
  ): void {
    const view = this.dataView();
    view.setUint32(ptr, options.min_u_segments, true);
    view.setUint32(ptr + I32_BYTES, options.min_v_segments, true);
    view.setUint32(ptr + I32_BYTES * 2, options.max_u_segments, true);
    view.setUint32(ptr + I32_BYTES * 3, options.max_v_segments, true);
    view.setFloat64(ptr + I32_BYTES * 4, options.chord_tol, true);
    view.setFloat64(ptr + I32_BYTES * 4 + F64_BYTES, options.normal_tol_rad, true);
  }

  writeBoundsOptions(ptr: number, options: RgmBoundsOptions): void {
    const view = this.dataView();
    view.setInt32(ptr, options.mode, true);
    view.setUint32(ptr + I32_BYTES, options.sample_budget, true);
    view.setFloat64(ptr + I32_BYTES * 2, options.padding, true);
  }

  readSurfaceEvalFrame(ptr: number): RgmSurfaceEvalFrame {
    return {
      point: this.readPoint(ptr),
      du: this.readVec(ptr + POINT3_BYTES),
      dv: this.readVec(ptr + POINT3_BYTES + VEC3_BYTES),
      normal: this.readVec(ptr + POINT3_BYTES + VEC3_BYTES * 2),
    };
  }

  readAabb3(ptr: number): RgmAabb3 {
    return {
      min: this.readPoint(ptr),
      max: this.readPoint(ptr + POINT3_BYTES),
    };
  }

  readObb3(ptr: number): RgmObb3 {
    return {
      center: this.readPoint(ptr),
      x_axis: this.readVec(ptr + POINT3_BYTES),
      y_axis: this.readVec(ptr + POINT3_BYTES + VEC3_BYTES),
      z_axis: this.readVec(ptr + POINT3_BYTES + VEC3_BYTES * 2),
      half_extents: this.readVec(ptr + POINT3_BYTES + VEC3_BYTES * 3),
    };
  }

  readBounds3(ptr: number): RgmBounds3 {
    return {
      world_aabb: this.readAabb3(ptr),
      world_obb: this.readObb3(ptr + AABB3_BYTES),
      local_aabb: this.readAabb3(ptr + AABB3_BYTES + OBB3_BYTES),
    };
  }

  readIntersectionBranchSummary(ptr: number): RgmIntersectionBranchSummary {
    const view = this.dataView();
    return {
      point_count: view.getUint32(ptr, true),
      uv_a_count: view.getUint32(ptr + 4, true),
      uv_b_count: view.getUint32(ptr + 8, true),
      curve_t_count: view.getUint32(ptr + 12, true),
      closed: view.getUint8(ptr + 16) !== 0,
      flags: view.getUint32(ptr + 20, true),
    };
  }

  readValidationIssue(ptr: number): RgmValidationIssue {
    const view = this.dataView();
    return {
      severity: view.getInt32(ptr, true) as RgmValidationSeverity,
      code: view.getUint32(ptr + 4, true),
      entity_kind: view.getUint32(ptr + 8, true),
      entity_id: view.getUint32(ptr + 12, true),
      param_u: view.getFloat64(ptr + 16, true),
      param_v: view.getFloat64(ptr + 24, true),
    };
  }

  readBrepValidationReport(ptr: number): RgmBrepValidationReport {
    const view = this.dataView();
    const issueCount = view.getUint32(ptr, true);
    const issues: RgmValidationIssue[] = [];
    const capped = Math.min(16, issueCount);
    const issueBase = ptr + 16;
    for (let idx = 0; idx < capped; idx += 1) {
      issues.push(this.readValidationIssue(issueBase + idx * VALIDATION_ISSUE_BYTES));
    }
    return {
      issue_count: issueCount,
      max_severity: view.getInt32(ptr + 4, true) as RgmValidationSeverity,
      overflow: view.getUint8(ptr + 8) !== 0,
      issues,
    };
  }

  private dataView(): DataView {
    return new DataView(this.wasmMemory.buffer);
  }
}

export const KERNEL_LAYOUT = {
  F64_BYTES,
  I32_BYTES,
  U64_BYTES,
  POINT3_BYTES,
  VEC3_BYTES,
  PLANE_BYTES,
  LINE3_BYTES,
  CIRCLE3_BYTES,
  ARC3_BYTES,
  POLYCURVE_SEGMENT_BYTES,
  TOLERANCE_BYTES,
  UV2_BYTES,
  NURBS_SURFACE_DESC_BYTES,
  TRIM_LOOP_INPUT_BYTES,
  TRIM_EDGE_INPUT_BYTES,
  SURFACE_EVAL_FRAME_BYTES,
  SURFACE_TESSELLATION_OPTIONS_BYTES,
  INTERSECTION_BRANCH_SUMMARY_BYTES,
  VALIDATION_ISSUE_BYTES,
  BREP_VALIDATION_REPORT_BYTES,
  BOUNDS_OPTIONS_BYTES,
  AABB3_BYTES,
  OBB3_BYTES,
  BOUNDS3_BYTES,
} as const;
