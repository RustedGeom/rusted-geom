import type { NativeExports } from "../generated/native";

export type WasmSource = string | URL | ArrayBuffer | Uint8Array;

export interface KernelWasmExports extends NativeExports {
  memory: WebAssembly.Memory;
}

export interface KernelWasmModule {
  instance: WebAssembly.Instance;
  exports: KernelWasmExports;
}

async function instantiateFromBuffer(buffer: ArrayBuffer): Promise<KernelWasmModule> {
  const { instance } = await WebAssembly.instantiate(buffer, {});
  const exports = instance.exports as unknown as KernelWasmExports;
  if (!(exports.memory instanceof WebAssembly.Memory)) {
    throw new Error("WASM module does not export memory");
  }

  return { instance, exports };
}

export async function loadKernelWasm(source: WasmSource): Promise<KernelWasmModule> {
  if (source instanceof ArrayBuffer) {
    return instantiateFromBuffer(source);
  }

  if (source instanceof Uint8Array) {
    const copy = new Uint8Array(source.byteLength);
    copy.set(source);
    return instantiateFromBuffer(copy.buffer);
  }

  const url = source instanceof URL ? source.toString() : source;
  if (typeof fetch !== "function") {
    throw new Error("fetch is required when loading wasm from URL");
  }

  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch wasm: ${response.status} ${response.statusText}`);
  }

  const bytes = await response.arrayBuffer();
  return instantiateFromBuffer(bytes);
}
