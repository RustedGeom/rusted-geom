import type { KernelSession as LegacyKernelSession } from "./core";

export interface KernelClient {
  readonly handle: LegacyKernelSession["handle"];
  releaseObject: LegacyKernelSession["releaseObject"];
  lastError: LegacyKernelSession["lastError"];
}

export class KernelClientImpl implements KernelClient {
  constructor(private readonly session: LegacyKernelSession) {}

  get handle(): LegacyKernelSession["handle"] {
    return this.session.handle;
  }

  releaseObject: KernelClient["releaseObject"] = (objectHandle) =>
    this.session.releaseObject(objectHandle);

  lastError: KernelClient["lastError"] = () => this.session.lastError();
}
