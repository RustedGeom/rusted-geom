import type { KernelSession } from "../../../crates/kernel/pkg/rusted_geom.js";

function normalizeObjectIds(objectIds?: Float64Array): Float64Array {
  return objectIds && objectIds.length > 0 ? objectIds : new Float64Array(0);
}

function encodeUtf8(text: string): Uint8Array {
  return new TextEncoder().encode(text);
}

function decodeDataUri(uri: string): Uint8Array {
  const prefix = "data:application/octet-stream;base64,";
  if (!uri.startsWith(prefix)) {
    throw new Error("Expected embedded glTF buffer data URI");
  }
  const b64 = uri.slice(prefix.length);
  if (typeof window === "undefined") {
    return Uint8Array.from(Buffer.from(b64, "base64"));
  }
  const binary = atob(b64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

function pad4(bytes: Uint8Array, fill = 0x20): Uint8Array {
  const padded = (bytes.length + 3) & ~3;
  if (padded === bytes.length) {
    return bytes;
  }
  const next = new Uint8Array(padded);
  next.set(bytes);
  next.fill(fill, bytes.length);
  return next;
}

export async function exportUsdcBinary(
  session: KernelSession,
  objectIds?: Float64Array,
  endpoint = "/api/usd/convert",
): Promise<Uint8Array> {
  const ids = normalizeObjectIds(objectIds);
  const usda = ids.length > 0 ? session.export_usda_prims(ids) : session.export_usda();

  if (typeof window !== "undefined") {
    const response = await fetch(endpoint, {
      method: "POST",
      headers: {
        "content-type": "text/plain; charset=utf-8",
      },
      body: usda,
    });
    if (!response.ok) {
      throw new Error(`USDC conversion failed: ${response.status}`);
    }
    return new Uint8Array(await response.arrayBuffer());
  }

  const fs = await import("node:fs/promises");
  const os = await import("node:os");
  const path = await import("node:path");
  const { execFile } = await import("node:child_process");
  const { promisify } = await import("node:util");
  const execFileAsync = promisify(execFile);

  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "rusted-geom-usdc-"));
  const inputPath = path.join(tempDir, "stage.usda");
  const outputPath = path.join(tempDir, "stage.usdc");

  try {
    await fs.writeFile(inputPath, usda, "utf8");
    await execFileAsync("usdcat", [inputPath, "-o", outputPath, "--usdFormat", "usdc"]);
    return new Uint8Array(await fs.readFile(outputPath));
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

export async function exportGlbBinary(
  session: KernelSession,
  objectIds: Float64Array,
): Promise<Uint8Array> {
  const gltfText = session.export_gltf(normalizeObjectIds(objectIds));
  const gltf = JSON.parse(gltfText) as {
    buffers?: Array<{ uri?: string; byteLength: number }>;
  };

  const uri = gltf.buffers?.[0]?.uri;
  if (!uri) {
    throw new Error("glTF export did not include an embedded buffer");
  }

  const binChunk = pad4(decodeDataUri(uri), 0x00);
  delete gltf.buffers?.[0]?.uri;
  if (gltf.buffers?.[0]) {
    gltf.buffers[0].byteLength = binChunk.length;
  }

  const jsonChunk = pad4(encodeUtf8(JSON.stringify(gltf)), 0x20);
  const totalLength = 12 + 8 + jsonChunk.length + 8 + binChunk.length;
  const glb = new Uint8Array(totalLength);
  const view = new DataView(glb.buffer);

  view.setUint32(0, 0x46546c67, true);
  view.setUint32(4, 2, true);
  view.setUint32(8, totalLength, true);

  view.setUint32(12, jsonChunk.length, true);
  view.setUint32(16, 0x4e4f534a, true);
  glb.set(jsonChunk, 20);

  const binOffset = 20 + jsonChunk.length;
  view.setUint32(binOffset, binChunk.length, true);
  view.setUint32(binOffset + 4, 0x004e4942, true);
  glb.set(binChunk, binOffset + 8);

  return glb;
}
