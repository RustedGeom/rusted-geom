import type { NativeExports } from "../generated/native";
import type {
  RgmArc3,
  RgmCircle3,
  RgmLine3,
  RgmPlane,
  RgmPoint3,
  RgmPolycurveSegment,
  RgmToleranceContext,
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

  readPoint(ptr: number): RgmPoint3 {
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

  readBytes(ptr: number, length: number): Uint8Array {
    return new Uint8Array(this.wasmMemory.buffer, ptr, length);
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
} as const;
