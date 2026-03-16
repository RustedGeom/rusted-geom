import { KernelSession } from "../../../crates/kernel/pkg/rusted_geom.js";
import "./kernel-augment.js";

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
