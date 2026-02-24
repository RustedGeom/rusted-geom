// Thin wrapper around the wasm-pack init() function.
// Call `loadKernel(url)` once before constructing a `KernelSession`.
import init from "../../../crates/kernel-ffi/pkg/rusted_geom.js";

export async function loadKernel(
  urlOrBuffer: string | URL | ArrayBuffer | Uint8Array,
): Promise<void> {
  if (urlOrBuffer instanceof Uint8Array) {
    await init(urlOrBuffer.buffer as ArrayBuffer);
  } else {
    await init(urlOrBuffer as string | URL | ArrayBuffer);
  }
}
