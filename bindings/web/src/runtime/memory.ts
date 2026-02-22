import type { RustedGeomApi } from "../generated/native";
import type { RgmPoint3, RgmToleranceContext } from "../generated/types";
import { RgmStatus } from "../generated/types";
import { KernelRuntimeError } from "./errors";

const F64_BYTES = 8;
const I32_BYTES = 4;
const U64_BYTES = 8;
const POINT3_BYTES = F64_BYTES * 3;
const TOLERANCE_BYTES = F64_BYTES * 3;

export class KernelMemory {
  constructor(
    private readonly api: RustedGeomApi,
    private readonly wasmMemory: WebAssembly.Memory,
  ) {}

  alloc(byteLen: number, align = 8): number {
    const ptr = Number(this.api.allocAddr(byteLen, align));
    if (!Number.isInteger(ptr) || ptr <= 0) {
      throw new KernelRuntimeError("Kernel allocation failed", RgmStatus.InternalError);
    }

    return ptr;
  }

  free(ptr: number, byteLen: number, align = 8): void {
    if (ptr <= 0) {
      return;
    }

    const status = this.api.dealloc(ptr, byteLen, align) as RgmStatus;
    if (status !== RgmStatus.Ok) {
      throw new KernelRuntimeError("Kernel deallocation failed", status);
    }
  }

  writePointArray(ptr: number, points: RgmPoint3[]): void {
    const view = this.dataView();
    for (let idx = 0; idx < points.length; idx += 1) {
      const base = ptr + idx * POINT3_BYTES;
      const point = points[idx];
      view.setFloat64(base, point.x, true);
      view.setFloat64(base + F64_BYTES, point.y, true);
      view.setFloat64(base + F64_BYTES * 2, point.z, true);
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

  private dataView(): DataView {
    return new DataView(this.wasmMemory.buffer);
  }
}

export const KERNEL_LAYOUT = {
  I32_BYTES,
  U64_BYTES,
  POINT3_BYTES,
  TOLERANCE_BYTES,
} as const;
