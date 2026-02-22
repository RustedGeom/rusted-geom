import { describe, expect, it } from "vitest";

import { parseCurvePreset, parseViewerSession } from "../../src/lib/preset-schema";

const preset = {
  name: "Spec",
  degree: 3,
  closed: false,
  sampleCount: 120,
  tolerance: {
    abs_tol: 1e-9,
    rel_tol: 1e-9,
    angle_tol: 1e-9,
  },
  points: [
    { x: 0, y: 0, z: 0 },
    { x: 1, y: 1, z: 0 },
    { x: 2, y: 0.5, z: 0 },
    { x: 3, y: 0, z: 0 },
  ],
};

describe("preset schema", () => {
  it("parses a valid curve preset", () => {
    const parsed = parseCurvePreset(preset);
    expect(parsed.degree).toBe(3);
    expect(parsed.points).toHaveLength(4);
  });

  it("parses a valid session file", () => {
    const parsed = parseViewerSession({
      version: 1,
      preset,
      view: {
        camera: {
          position: { x: 4, y: 5, z: 6 },
          target: { x: 0, y: 0, z: 0 },
          up: { x: 0, y: 1, z: 0 },
          fov: 46,
        },
        showGrid: true,
        showAxes: false,
        orbitEnabled: true,
      },
    });

    expect(parsed.version).toBe(1);
    expect(parsed.view.camera.position.x).toBe(4);
  });

  it("rejects invalid preset", () => {
    expect(() =>
      parseCurvePreset({
        ...preset,
        points: [{ x: 0, y: 0, z: 0 }],
      }),
    ).toThrowError(/at least two points/i);
  });
});
