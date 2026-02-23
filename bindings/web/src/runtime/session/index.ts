import type {
  CurvePresetInput,
  RgmSurfaceFirstDerivatives,
  RgmSurfaceSecondDerivatives,
} from "./core";
import {
  createKernelRuntime as createLegacyKernelRuntime,
  type KernelRuntime as LegacyKernelRuntime,
  type KernelSession as LegacyKernelSession,
} from "./core";
import { type WasmSource } from "../wasm-loader";
import type { KernelRuntimeError } from "../errors";
import { CurveClientImpl, type CurveClient } from "./curve";
import { FaceClientImpl, type FaceClient } from "./face";
import { IntersectionClientImpl, type IntersectionClient } from "./intersection";
import { KernelClientImpl, type KernelClient } from "./kernel";
import { MeshClientImpl, type MeshClient } from "./mesh";
import { SurfaceClientImpl, type SurfaceClient } from "./surface";
import type { KernelCapabilities } from "./common";

export type {
  CurvePresetInput,
  KernelRuntimeError,
  RgmSurfaceFirstDerivatives,
  RgmSurfaceSecondDerivatives,
};

export interface KernelSession {
  readonly kernel: KernelClient;
  readonly curve: CurveClient;
  readonly mesh: MeshClient;
  readonly surface: SurfaceClient;
  readonly face: FaceClient;
  readonly intersection: IntersectionClient;
  destroy(): void;
}

class KernelSessionImpl implements KernelSession {
  readonly kernel: KernelClient;
  readonly curve: CurveClient;
  readonly mesh: MeshClient;
  readonly surface: SurfaceClient;
  readonly face: FaceClient;
  readonly intersection: IntersectionClient;

  constructor(private readonly session: LegacyKernelSession) {
    this.kernel = new KernelClientImpl(session);
    this.curve = new CurveClientImpl(session);
    this.mesh = new MeshClientImpl(session);
    this.surface = new SurfaceClientImpl(session);
    this.face = new FaceClientImpl(session);
    this.intersection = new IntersectionClientImpl(session);
  }

  destroy(): void {
    this.session.destroy();
  }
}

export interface KernelRuntime {
  readonly capabilities: KernelCapabilities;
  createSession(): KernelSession;
  destroy(): void;
}

class KernelRuntimeImpl implements KernelRuntime {
  readonly capabilities: KernelCapabilities;

  constructor(private readonly runtime: LegacyKernelRuntime) {
    this.capabilities = runtime.capabilities;
  }

  createSession(): KernelSession {
    return new KernelSessionImpl(this.runtime.createSession());
  }

  destroy(): void {
    this.runtime.destroy();
  }
}

export async function createKernelRuntime(wasmSource: WasmSource): Promise<KernelRuntime> {
  const runtime = await createLegacyKernelRuntime(wasmSource);
  return new KernelRuntimeImpl(runtime);
}
