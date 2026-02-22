import { RustedGeomApi, type NativeExports } from "../generated/native";
import type { RgmPoint3, RgmToleranceContext } from "../generated/types";
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
  sampleCurvePolyline(curveHandle: bigint, sampleCount: number): RgmPoint3[];
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
    private readonly api: RustedGeomApi,
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
      const status = this.api.nurbsInterpolateFitPointsPtrTol(
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

  releaseObject(objectHandle: bigint): void {
    this.ensureAlive();
    const status = this.api.objectRelease(this.handle, objectHandle) as RgmStatus;
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
      const statusCode = this.api.lastErrorCode(this.handle, codePtr) as RgmStatus;
      const statusMessage = this.api.lastErrorMessage(
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

    const status = this.api.kernelDestroy(this.handle) as RgmStatus;
    if (status !== RgmStatus.Ok && status !== RgmStatus.NotFound) {
      this.assertOk(status, "Kernel session destroy failed");
    }

    this.destroyed = true;
    this.onDestroy();
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
    private readonly api: RustedGeomApi,
    private readonly memory: KernelMemory,
  ) {}

  createSession(): KernelSession {
    const outSessionPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.kernelCreate(outSessionPtr) as RgmStatus;
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
  const api = new RustedGeomApi(exports);
  const memory = new KernelMemory(api, wasm.exports.memory);
  return new KernelRuntimeImpl(api, memory);
}
