import type {
  KernelCapabilities,
  KernelRuntime as LegacyKernelRuntime,
  KernelSession as LegacyKernelSession,
} from "./core";

export type { KernelCapabilities };
export type SessionBridge = LegacyKernelSession;
export type RuntimeBridge = LegacyKernelRuntime;
