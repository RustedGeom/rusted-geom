import { execSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { beforeAll, describe, expect, it } from "vitest";

import {
  createKernelRuntime,
  KernelRuntimeError,
  type CurvePresetInput,
  RgmStatus,
} from "../src/index";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "../../..");
const wasmPath = path.join(
  repoRoot,
  "target/wasm32-unknown-unknown/debug/kernel_ffi.wasm",
);

beforeAll(() => {
  execSync("cargo build -p kernel-ffi --target wasm32-unknown-unknown", {
    cwd: repoRoot,
    stdio: "inherit",
  });
});

const preset: CurvePresetInput = {
  degree: 2,
  closed: false,
  points: [
    { x: 0, y: 0, z: 0 },
    { x: 1, y: 0.25, z: 0 },
    { x: 2, y: 1, z: 0 },
    { x: 3, y: 1.25, z: 0 },
  ],
  tolerance: {
    abs_tol: 1e-9,
    rel_tol: 1e-9,
    angle_tol: 1e-9,
  },
};

describe("kernel runtime", () => {
  it("creates a session, builds a curve, and samples points", async () => {
    const wasmBytes = readFileSync(wasmPath);
    const runtime = await createKernelRuntime(wasmBytes);
    const session = runtime.createSession();

    const handle = session.buildCurveFromPreset(preset);
    const samples = session.sampleCurvePolyline(handle, 32);

    expect(runtime.capabilities.igesImport).toBe(false);
    expect(runtime.capabilities.igesExport).toBe(false);
    expect(samples).toHaveLength(32);
    expect(samples[0].x).toBeCloseTo(0, 6);
    expect(samples.at(-1)?.x).toBeCloseTo(3, 6);

    session.releaseObject(handle);
    session.destroy();
    runtime.destroy();
  });

  it("surfaces kernel errors for invalid curve construction", async () => {
    const wasmBytes = readFileSync(wasmPath);
    const runtime = await createKernelRuntime(wasmBytes);
    const session = runtime.createSession();

    const invalidPreset: CurvePresetInput = {
      ...preset,
      degree: 8,
    };

    let thrown: unknown = undefined;
    try {
      session.buildCurveFromPreset(invalidPreset);
    } catch (error) {
      thrown = error;
    }

    expect(thrown).toBeInstanceOf(KernelRuntimeError);
    const kernelError = thrown as KernelRuntimeError;
    expect(kernelError.status).toBe(RgmStatus.InvalidInput);
    expect(kernelError.message).toContain("Curve construction failed");

    session.destroy();
    runtime.destroy();
  });
});
