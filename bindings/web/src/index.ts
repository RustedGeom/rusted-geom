import { KernelSession } from "../../../crates/kernel/pkg/rusted_geom.js";
import "./kernel-augment.js";
// wasm-bindgen generated classes (KernelSession, CurveHandle, SurfaceHandle, …).
export * from "../../../crates/kernel/pkg/rusted_geom.js";

if (!(KernelSession.prototype as { export_usdc?: unknown }).export_usdc) {
  Object.defineProperties(KernelSession.prototype, {
    export_usdc: {
      value(this: KernelSession, objectIds?: Float64Array, endpoint?: string) {
        return import("./export-runtime.js").then((mod) => mod.exportUsdcBinary(this, objectIds, endpoint));
      },
    },
    export_glb: {
      value(this: KernelSession, objectIds: Float64Array) {
        return import("./export-runtime.js").then((mod) => mod.exportGlbBinary(this, objectIds));
      },
    },
  });
}

// USD value/schema interfaces and compatibility aliases.
export * from "./types";

// Convenience WASM loader.
export { loadKernel } from "./loader";
