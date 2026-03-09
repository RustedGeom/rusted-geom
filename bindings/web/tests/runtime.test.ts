import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { beforeAll, describe, expect, it } from "vitest";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "../../..");
const pkgDir = path.join(repoRoot, "crates/kernel/pkg");
const wasmPath = path.join(pkgDir, "rusted_geom_bg.wasm");

const pkgAvailable = existsSync(wasmPath);

// Helper: build a clamped-uniform knot vector.
function clampedUniformKnots(controlCount: number, degree: number): number[] {
  const knotCount = controlCount + degree + 1;
  const knots = new Array<number>(knotCount).fill(0);
  const interior = controlCount - degree - 1;
  for (let i = 0; i <= degree; i += 1) {
    knots[i] = 0;
    knots[knotCount - 1 - i] = 1;
  }
  for (let i = 1; i <= interior; i += 1) {
    knots[degree + i] = i / (interior + 1);
  }
  return knots;
}

// Identity plane axes.
const I_XX = 1, I_XY = 0, I_XZ = 0;
const I_YX = 0, I_YY = 1, I_YZ = 0;
const I_ZX = 0, I_ZY = 0, I_ZZ = 1;

describe.skipIf(!pkgAvailable)("kernel wasm-bindgen API", () => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let KernelSession: any;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let initFn: any;

  beforeAll(async () => {
    const pkg = await import(`${pkgDir}/rusted_geom.js`);
    initFn = pkg.default;
    KernelSession = pkg.KernelSession;
    const wasmBytes = readFileSync(wasmPath);
    await initFn(wasmBytes.buffer);
  }, 60_000);

  it("creates a session, builds curves, evaluates them", () => {
    const session = new KernelSession();

    // Line
    const line = session.create_line(-1, 0, 0, 1, 0, 0);
    const ptMid = session.curve_point_at(line, 0.5);
    expect(ptMid[0]).toBeCloseTo(0, 6);
    expect(ptMid[1]).toBeCloseTo(0, 6);

    // Circle
    const circle = session.create_circle(
      0, 0, 0,                     // origin
      I_XX, I_XY, I_XZ,           // x_axis
      I_YX, I_YY, I_YZ,           // y_axis
      I_ZX, I_ZY, I_ZZ,           // z_axis
      3.5,                         // radius
    );
    for (const t of [0, 0.25, 0.5, 0.75]) {
      const p = session.curve_point_at(circle, t);
      const r = Math.sqrt(p[0] ** 2 + p[1] ** 2 + p[2] ** 2);
      expect(r).toBeCloseTo(3.5, 3);
    }

    // Polyline curve from fit points
    const pts = [0, 0, 0, 1, 0.5, 0, 2, 1, 0, 3, 1.25, 0];
    const nurbs = session.interpolate_nurbs_fit_points(pts, 2, false);
    const totalLen = session.curve_length(nurbs);
    const halfLen = session.curve_length_at(nurbs, 0.5);
    expect(totalLen).toBeGreaterThan(0);
    expect(halfLen).toBeGreaterThan(0);
    expect(halfLen).toBeLessThan(totalLen);

    // Tangent is a unit vector
    const tan = session.curve_tangent_at(nurbs, 0.5);
    const tanLen = Math.sqrt(tan[0] ** 2 + tan[1] ** 2 + tan[2] ** 2);
    expect(tanLen).toBeCloseTo(1, 4);

    line.free();
    circle.free();
    nurbs.free();
    session.free();
  });

  it("creates a NURBS surface and evaluates it", () => {
    const session = new KernelSession();

    const uCount = 3, vCount = 3;
    const rawPts = [
      -2, -2, 0,   -2, 0, 0.8,   -2, 2, 0.1,
       0, -2, 0.7,   0, 0, -0.2,   0, 2, 0.9,
       2, -2, -0.3,  2, 0, 0.6,   2, 2, 0.2,
    ];
    const weights = new Array(uCount * vCount).fill(1.0);
    const ku = clampedUniformKnots(uCount, 2);
    const kv = clampedUniformKnots(vCount, 2);

    const surface = session.create_nurbs_surface(
      2, 2,           // degree_u, degree_v
      uCount, vCount, // control_u_count, control_v_count
      false, false,   // periodic_u, periodic_v
      rawPts, weights, ku, kv,
    );

    // Position at centre
    const pt = session.surface_point_at(surface, 0.5, 0.5);
    expect(Number.isFinite(pt[0])).toBe(true);

    // Full evaluation frame
    const frame = session.surface_frame_at(surface, 0.5, 0.5);
    // frame: SurfaceEvalResult with px,py,pz,du_x,…,nx,ny,nz
    expect(frame.px).toBeCloseTo(pt[0], 6);
    expect(frame.py).toBeCloseTo(pt[1], 6);
    // Normal is unit
    const nLen = Math.sqrt(frame.nx ** 2 + frame.ny ** 2 + frame.nz ** 2);
    expect(nLen).toBeCloseTo(1, 4);

    surface.free();
    session.free();
  });

  it("creates mesh primitives and queries them", () => {
    const session = new KernelSession();

    const box_ = session.create_box_mesh(0, 0, 0, 4, 3, 2);
    expect(session.mesh_vertex_count(box_)).toBe(8);
    expect(session.mesh_triangle_count(box_)).toBe(12);

    const verts = session.mesh_copy_vertices(box_);
    expect(verts.length).toBe(8 * 3);

    const indices = session.mesh_copy_indices(box_);
    expect(indices.length).toBe(12 * 3);

    // Translate + bake
    const translated = session.mesh_translate(box_, 1, 0, 0);
    const baked = session.mesh_bake_transform(translated);
    const bakedVerts = session.mesh_copy_vertices(baked);
    expect(bakedVerts[0]).toBeCloseTo(verts[0] + 1, 6);

    box_.free();
    translated.free();
    baked.free();
    session.free();
  });

  it("computes bounds for a line and surface", () => {
    const session = new KernelSession();

    const line = session.create_line(-3, 1.25, 4.5, 6, -2.75, -1.5);
    const b = session.compute_bounds(line.object_id(), 0, 0, 0);
    expect(Number.isFinite(b.aabb_min_x)).toBe(true);
    expect(b.aabb_min_x).toBeLessThanOrEqual(-3 + 1e-9);
    expect(b.aabb_max_x).toBeGreaterThanOrEqual(6 - 1e-9);

    // Surface bounds
    const uCount = 4, vCount = 4;
    const pts: number[] = [];
    for (let iu = 0; iu < uCount; iu++) {
      for (let iv = 0; iv < vCount; iv++) {
        pts.push(iu, iv, Math.sin(iu + iv));
      }
    }
    const surface = session.create_nurbs_surface(
      2, 2, uCount, vCount, false, false,
      pts, new Array(uCount * vCount).fill(1),
      clampedUniformKnots(uCount, 2), clampedUniformKnots(vCount, 2),
    );
    const sb = session.compute_bounds(surface.object_id(), 0, 0, 0);
    expect(Number.isFinite(sb.aabb_min_x)).toBe(true);
    expect(sb.obb_half_x).toBeGreaterThan(0);

    b.free();
    sb.free();
    line.free();
    surface.free();
    session.free();
  });

  it("intersects curves and surfaces", () => {
    const session = new KernelSession();

    const lineA = session.create_line(-1, 0, 0, 1, 0, 0);
    const lineB = session.create_line(0, -1, 0, 0, 1, 0);
    const hits = session.intersect_curve_curve(lineA, lineB);
    // hits is Float64Array [x,y,z, ...]
    expect(hits.length).toBeGreaterThanOrEqual(3);
    expect(hits[0]).toBeCloseTo(0, 3);
    expect(hits[1]).toBeCloseTo(0, 3);

    // Surface-plane intersection
    const uCount = 4, vCount = 4;
    const pts: number[] = [];
    for (let iu = 0; iu < uCount; iu++) {
      for (let iv = 0; iv < vCount; iv++) {
        pts.push(-3 + iu * 2, -3 + iv * 2, Math.sin(iu * 0.9 + iv * 0.7) * 0.5);
      }
    }
    const surface = session.create_nurbs_surface(
      2, 2, uCount, vCount, false, false,
      pts, new Array(uCount * vCount).fill(1),
      clampedUniformKnots(uCount, 2), clampedUniformKnots(vCount, 2),
    );
    // Plane at z=0: [ox,oy,oz, xx,xy,xz, yx,yy,yz, zx,zy,zz]
    const intersect = session.intersect_surface_plane(
      surface,
      [0, 0, 0,  1, 0, 0,  0, 1, 0,  0, 0, 1],
    );
    const branchCount = session.intersection_branch_count(intersect);
    expect(branchCount).toBeGreaterThanOrEqual(0);

    lineA.free();
    lineB.free();
    surface.free();
    intersect.free();
    session.free();
  });

});
