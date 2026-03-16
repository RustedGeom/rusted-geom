declare module "../../../crates/kernel/pkg/rusted_geom.js" {
  interface KernelSession {
    export_usdc(object_ids?: Float64Array, endpoint?: string): Promise<Uint8Array>;
    export_glb(object_ids: Float64Array): Promise<Uint8Array>;
  }
}

export {};
