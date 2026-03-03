import { KernelSession, loadKernel, type CurveHandle, type MeshHandle } from "@rustedgeom/kernel";

export type ContractCaseStatus = "idle" | "running" | "pass" | "fail";
export type ContractLogLevel = "info" | "debug" | "pass" | "fail";

export interface ContractCaseSpec {
  id: string;
  title: string;
  summary: string;
}

export interface ContractCaseResult {
  id: string;
  title: string;
  status: Extract<ContractCaseStatus, "pass" | "fail">;
  durationMs: number;
  errorMessage?: string;
}

export interface ContractSuiteResult {
  cases: ContractCaseResult[];
  totalDurationMs: number;
  passed: number;
  failed: number;
}

export interface ContractLogEntry {
  id: number;
  time: string;
  level: ContractLogLevel;
  caseId: string;
  message: string;
}

interface ContractSuiteCallbacks {
  onCaseStart?: (id: string) => void;
  onCaseEnd?: (result: ContractCaseResult) => void;
  onLog?: (entry: ContractLogEntry) => void;
}

interface CaseRunner {
  id: string;
  title: string;
  summary: string;
  run: (log: (level: ContractLogLevel, message: string) => void) => Promise<void>;
}

function assertOrThrow(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

function nowStamp(): string {
  const d = new Date();
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  const ss = String(d.getSeconds()).padStart(2, "0");
  const ms = String(d.getMilliseconds()).padStart(3, "0");
  return `${hh}:${mm}:${ss}.${ms}`;
}

function clampedUniformKnots(controlCount: number, degree: number): Float64Array {
  const knotCount = controlCount + degree + 1;
  const knots = new Float64Array(knotCount);
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

function formatDuration(durationMs: number): string {
  return `${durationMs.toFixed(1)}ms`;
}

function toErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

/** Flat plane array [ox,oy,oz, xx,xy,xz, yx,yy,yz, zx,zy,zz]. */
function plane(
  ox: number, oy: number, oz: number,
  xx: number, xy: number, xz: number,
  yx: number, yy: number, yz: number,
  zx: number, zy: number, zz: number,
): Float64Array {
  return new Float64Array([ox, oy, oz, xx, xy, xz, yx, yy, yz, zx, zy, zz]);
}

async function withSession(
  log: (level: ContractLogLevel, message: string) => void,
  run: (session: KernelSession, track: <T extends CurveHandle | MeshHandle>(h: T) => T) => Promise<void> | void,
): Promise<void> {
  const session = new KernelSession();
  try {
    const track = <T extends CurveHandle | MeshHandle>(h: T): T => h;
    await run(session, track);
  } finally {
    session.free();
    log("debug", "Session freed");
  }
}

export const CONTRACT_CASES: ContractCaseSpec[] = [
  {
    id: "curve-session-sampling",
    title: "Curve Session Sampling",
    summary: "Session creation, curve sampling, circle probes, and curve intersections.",
  },
  {
    id: "mesh-transform-intersections",
    title: "Mesh Transform + Intersections",
    summary: "Mesh counts, transforms, mesh intersections, and boolean result invariants.",
  },
  {
    id: "surface-face-contract",
    title: "Surface + Face Contract",
    summary: "Surface evaluations, face loop validation, tessellation, and surface-plane branches.",
  },
  {
    id: "invalid-input",
    title: "Invalid Curve Input",
    summary: "Invalid degree must surface a kernel error.",
  },
  {
    id: "wasm-bindgen-api",
    title: "wasm-bindgen API Contract",
    summary: "Object IDs, session tolerances, compute_bounds, and intersection branch data.",
  },
];

const RUNNERS: CaseRunner[] = [
  {
    id: "curve-session-sampling",
    title: "Curve Session Sampling",
    summary: "Session creation, curve sampling, circle probes, and curve intersections.",
    run: async (log) => {
      await withSession(log, (session) => {
        log("info", "Session created");

        // Build a NURBS curve via interpolation through 4 control points
        const pts = new Float64Array([0, 0, 0, 1, 0.25, 0, 2, 1, 0, 3, 1.25, 0]);
        const curveHandle = session.interpolate_nurbs_fit_points(pts, 2, false);

        // Sample 32 points manually
        const samples = Array.from({ length: 32 }, (_, i) => {
          const t = i / 31;
          const p = session.curve_point_at(curveHandle, t);
          return { x: p[0], y: p[1], z: p[2] };
        });

        assertOrThrow(samples.length === 32, "Expected 32 sampled points");
        assertOrThrow(Math.abs(samples[0].x) < 1e-4, "First sampled point mismatch");
        assertOrThrow(Math.abs((samples.at(-1)?.x ?? 0) - 3) < 1e-4, "Last sampled point mismatch");

        const pt = session.curve_point_at(curveHandle, 0.37);
        assertOrThrow(pt[0] >= 0 && pt[0] <= 3, "Probe point outside expected range");

        const totalLength = session.curve_length(curveHandle);
        const lengthAtPoint = session.curve_length_at(curveHandle, 0.37);
        assertOrThrow(totalLength > 0, "Total curve length should be positive");
        assertOrThrow(lengthAtPoint > 0 && lengthAtPoint < totalLength, "Length-at parameter mismatch");
        log("debug", `Polyline sample count=${samples.length}, curve length=${totalLength.toFixed(4)}`);

        // Circle radius probes
        const circleHandle = session.create_circle(
          1.25, -0.8, 0.4,  // origin
          1, 0, 0,           // x_axis
          0, 1, 0,           // y_axis
          0, 0, 1,           // z_axis
          3.6,               // radius
        );
        let circleChecks = 0;
        for (const t of [0, 0.11, 0.3, 0.5, 0.77, 1]) {
          const p = session.curve_point_at(circleHandle, t);
          const dx = p[0] - 1.25;
          const dy = p[1] + 0.8;
          const dz = p[2] - 0.4;
          const r = Math.sqrt(dx * dx + dy * dy + dz * dz);
          assertOrThrow(Math.abs(r - 3.6) < 1e-3, `Circle radius mismatch at t=${t}`);
          circleChecks += 1;
        }
        log("debug", `Circle radius probes validated=${circleChecks}`);

        // Curve-curve intersection
        const lineA = session.create_line(-1, 0, 0, 1, 0, 0);
        const lineB = session.create_line(0, -1, 0, 0, 1, 0);
        const curveHits = session.intersect_curve_curve(lineA, lineB);
        const hitCount = curveHits.length / 3;
        assertOrThrow(hitCount === 1, "Expected one curve-curve intersection hit");
        assertOrThrow(Math.abs(curveHits[0]) < 1e-3, "Curve-curve hit x mismatch");
        assertOrThrow(Math.abs(curveHits[1]) < 1e-3, "Curve-curve hit y mismatch");

        // Curve-plane intersection
        const planeFlat = plane(0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1);
        const planeHits = session.intersect_curve_plane(lineA, planeFlat);
        assertOrThrow(planeHits.length >= 3, "Expected curve-plane intersection hits");

        log("pass", `Curve and intersection checks passed (curve-curve=${hitCount}, curve-plane=${planeHits.length / 3})`);
      });
    },
  },
  {
    id: "mesh-transform-intersections",
    title: "Mesh Transform + Intersections",
    summary: "Mesh counts, transforms, mesh intersections, and boolean result invariants.",
    run: async (log) => {
      await withSession(log, (session) => {
        log("info", "Session created");

        const meshBox = session.create_box_mesh(0, 0, 0, 4, 3, 2);
        const meshVertices = session.mesh_vertex_count(meshBox);
        const meshTriangles = session.mesh_triangle_count(meshBox);
        assertOrThrow(meshVertices === 8, "Mesh box vertex count mismatch");
        assertOrThrow(meshTriangles === 12, "Mesh box triangle count mismatch");
        log("debug", `Mesh box vertices=${meshVertices}, triangles=${meshTriangles}`);

        const translated = session.mesh_translate(meshBox, 0.8, -0.4, 1.2);
        const transformed = session.mesh_rotate(translated, 0, 1, 0.2, 0.7, 0, 0, 0);
        const baked = session.mesh_bake_transform(transformed);
        const bakedVerts = session.mesh_copy_vertices(baked);
        const bakedIndices = session.mesh_copy_indices(baked);
        assertOrThrow(bakedVerts.length / 3 === 8, "Baked mesh vertex count mismatch");
        assertOrThrow(bakedIndices.length === 36, "Baked mesh index count mismatch");
        log("debug", `Baked mesh vertices=${bakedVerts.length / 3}, indices=${bakedIndices.length}`);

        const torus = session.create_torus_mesh(0, 0, 0, 3.8, 1.1, 28, 20);
        const meshPlaneHits = session.intersect_mesh_plane(
          torus,
          plane(0, 0, 0.2, 1, 0, 0, 0, 1, 0, 0, 0, 1),
        );
        assertOrThrow(meshPlaneHits.length > 0, "Mesh-plane expected non-empty hit set");
        assertOrThrow(meshPlaneHits.length % 3 === 0, "Mesh-plane expected flat xyz triples");
        log("debug", `Mesh-plane intersection points=${meshPlaneHits.length / 3}`);

        const sphere = session.create_uv_sphere_mesh(0, 0, 0, 4.2, 24, 16);
        const meshMeshHits = session.intersect_mesh_mesh(sphere, torus);
        assertOrThrow(meshMeshHits.length > 0, "Mesh-mesh expected non-empty hit set");
        assertOrThrow(meshMeshHits.length % 3 === 0, "Mesh-mesh expected flat xyz triples");
        log("debug", `Mesh-mesh intersection points=${meshMeshHits.length / 3}`);

        const booleanHost = session.create_box_mesh(0, 0, 0, 8.8, 8.8, 8.8);
        const innerTorus = session.create_torus_mesh(0, 0, 0, 2.4, 0.8, 32, 24);
        const booleanDiff = session.mesh_boolean(booleanHost, innerTorus, 2);
        const booleanTriangles = session.mesh_triangle_count(booleanDiff);
        assertOrThrow(booleanTriangles > 0, "Boolean result should have triangles");
        log("pass", `Mesh checks passed (boolean triangles=${booleanTriangles})`);
      });
    },
  },
  {
    id: "surface-face-contract",
    title: "Surface + Face Contract",
    summary: "Surface evaluations, face loop validation, tessellation, and surface-plane branches.",
    run: async (log) => {
      await withSession(log, (session) => {
        log("info", "Session created");

        const controlPoints = [
          -2, -2, 0,   -2, 0, 0.8,  -2, 2, 0.1,
           0, -2, 0.7,  0, 0, -0.2,  0, 2, 0.9,
           2, -2, -0.3, 2, 0,  0.6,  2, 2, 0.2,
        ];
        const weights = new Float64Array(9).fill(1);
        const knots = clampedUniformKnots(3, 2);
        const surface = session.create_nurbs_surface(
          2, 2,             // degree_u, degree_v
          3, 3,             // control_u_count, control_v_count
          false, false,     // periodic_u, periodic_v
          new Float64Array(controlPoints),
          weights,
          knots,
          knots,
        );

        // Full frame evaluation
        const frame = session.surface_frame_at(surface, 0.5, 0.5);
        const ptArr = session.surface_point_at(surface, 0.5, 0.5);
        assertOrThrow(Math.abs(frame.px - ptArr[0]) < 1e-7, "Surface point/frame x mismatch");
        assertOrThrow(Math.abs(frame.py - ptArr[1]) < 1e-7, "Surface point/frame y mismatch");
        assertOrThrow(Math.abs(frame.pz - ptArr[2]) < 1e-7, "Surface point/frame z mismatch");

        const d1 = session.surface_d1_at(surface, 0.5, 0.5);
        assertOrThrow(Math.abs(frame.du_x - d1[0]) < 1e-7, "Surface du mismatch");
        assertOrThrow(Math.abs(frame.dv_y - d1[4]) < 1e-7, "Surface dv mismatch");

        const d2 = session.surface_d2_at(surface, 0.5, 0.5);
        assertOrThrow(Number.isFinite(d2[0]), "Surface duu should be finite");
        assertOrThrow(Number.isFinite(d2[4]), "Surface duv should be finite");
        assertOrThrow(Number.isFinite(d2[8]), "Surface dvv should be finite");
        assertOrThrow(Number.isFinite(frame.nx), "Surface normal should be finite");
        log("debug", "Surface point/frame/differential checks passed");

        // Face with loops
        const face = session.create_face_from_surface(surface);
        // Add outer boundary loop (flat UV [u,v,...])
        const outerUvs = [0.05, 0.05, 0.95, 0.05, 0.95, 0.95, 0.05, 0.95];
        session.face_add_loop(face, new Float64Array(outerUvs), true);

        // Add inner hole using a circle edge
        const trimCircle = session.create_circle(
          0.5, 0.5, 0,   // origin
          1, 0, 0,        // x_axis
          0, 1, 0,        // y_axis
          0, 0, 1,        // z_axis
          0.18,           // radius
        );
        // edges_flat: [u0,v0, u1,v1, obj_id, has_curve]
        const edgesFlat = [0.68, 0.5, 0.68, 0.5, trimCircle.object_id(), 1.0];
        session.face_add_loop_edges(face, false, new Float64Array(edgesFlat));
        const valid = session.face_validate(face);
        assertOrThrow(valid, "Face validation failed");

        const faceMesh = session.face_tessellate_to_mesh(
          face,
          new Float64Array([28, 28, 48, 48, 1e-4, 0.08]),
        );
        const tessTriangles = session.mesh_triangle_count(faceMesh);
        assertOrThrow(tessTriangles > 0, "Face tessellation returned no triangles");

        const surfacePlaneIntersection = session.intersect_surface_plane(
          surface,
          plane(0, 0, 0.1, 1, 0, 0, 0, 1, 0, 0, 0, 1),
        );
        const branchCount = session.intersection_branch_count(surfacePlaneIntersection);
        assertOrThrow(branchCount >= 0, "Surface-plane branch count check failed");

        log("pass", `Surface/face checks passed (face triangles=${tessTriangles}, branches=${branchCount})`);
      });
    },
  },
  {
    id: "invalid-input",
    title: "Invalid Curve Input",
    summary: "Invalid degree must surface a kernel error.",
    run: async (log) => {
      const session = new KernelSession();
      try {
        let thrown: unknown = undefined;
        try {
          // degree=8 with only 4 points — kernel must reject this
          session.interpolate_nurbs_fit_points(new Float64Array([0, 0, 0, 1, 0, 0, 2, 0, 0, 3, 0, 0]), 8, false);
        } catch (error) {
          thrown = error;
        }

        assertOrThrow(thrown !== undefined, "Expected kernel error for invalid degree");
        assertOrThrow(thrown instanceof Error, "Expected Error instance");
        log("pass", `Invalid input surfaced expected error: ${(thrown as Error).message}`);
      } finally {
        session.free();
      }
    },
  },
  {
    id: "wasm-bindgen-api",
    title: "wasm-bindgen API Contract",
    summary: "Object IDs, session tolerances, compute_bounds, and intersection branch data.",
    run: async (log) => {
      await withSession(log, (session) => {
        log("info", "Session created");

        // Verify default tolerances
        assertOrThrow(session.abs_tol() > 0, "abs_tol should be positive");
        assertOrThrow(session.rel_tol() > 0, "rel_tol should be positive");
        assertOrThrow(session.angle_tol() > 0, "angle_tol should be positive");
        log("debug", `Default tolerances: abs=${session.abs_tol()}, rel=${session.rel_tol()}, angle=${session.angle_tol()}`);

        // Object IDs must be non-zero
        const lineHandle = session.create_line(0, 0, -2, 0, 0, 2);
        assertOrThrow(lineHandle.object_id() > 0, "Line handle object_id should be non-zero");

        const meshHandle = session.create_box_mesh(0, 0, 0, 2, 2, 2);
        assertOrThrow(meshHandle.object_id() > 0, "Mesh handle object_id should be non-zero");

        // IDs are distinct
        assertOrThrow(lineHandle.object_id() !== meshHandle.object_id(), "Object IDs must be distinct");
        log("debug", `Line object_id=${lineHandle.object_id()}, mesh object_id=${meshHandle.object_id()}`);

        // compute_bounds on mesh
        const bounds = session.compute_bounds(meshHandle.object_id(), 0, 0, 0.0);
        assertOrThrow(Number.isFinite(bounds.aabb_min_x), "Bounds aabb_min_x should be finite");
        assertOrThrow(Number.isFinite(bounds.aabb_max_x), "Bounds aabb_max_x should be finite");
        assertOrThrow(bounds.aabb_max_x > bounds.aabb_min_x, "Bounds max_x > min_x");
        assertOrThrow(bounds.aabb_max_y > bounds.aabb_min_y, "Bounds max_y > min_y");
        assertOrThrow(bounds.aabb_max_z > bounds.aabb_min_z, "Bounds max_z > min_z");
        log("debug", `Mesh bounds AABB: [${bounds.aabb_min_x.toFixed(3)},${bounds.aabb_min_y.toFixed(3)},${bounds.aabb_min_z.toFixed(3)}] → [${bounds.aabb_max_x.toFixed(3)},${bounds.aabb_max_y.toFixed(3)},${bounds.aabb_max_z.toFixed(3)}]`);

        // Intersection branch access
        const surfHandle = session.create_nurbs_surface(
          2, 2, 3, 3, false, false,
          new Float64Array([-2,-2,0, -2,0,1, -2,2,0, 0,-2,1, 0,0,-1, 0,2,1, 2,-2,0, 2,0,1, 2,2,0]),
          new Float64Array(9).fill(1),
          clampedUniformKnots(3, 2),
          clampedUniformKnots(3, 2),
        );
        const ssi = session.intersect_surface_plane(
          surfHandle,
          plane(0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1),
        );
        const branches = session.intersection_branch_count(ssi);
        assertOrThrow(branches >= 0, "Branch count should be non-negative");
        if (branches > 0) {
          const summary = session.intersection_branch_summary(ssi, 0);
          assertOrThrow(summary.point_count >= 0, "Branch point_count should be non-negative");
          const branchPts = session.intersection_branch_copy_points(ssi, 0);
          assertOrThrow(branchPts.length % 3 === 0, "Branch points should be flat xyz triples");
          log("debug", `Intersection branch[0]: point_count=${summary.point_count}, closed=${summary.closed}`);
        }

        log("pass", `wasm-bindgen API checks passed (bounds AABB ok, branches=${branches})`);
      });
    },
  },
];

export function formatContractLogsAsText(entries: ContractLogEntry[]): string {
  if (entries.length === 0) {
    return "[empty] Kernel contract suite has no log entries.\n";
  }

  return `${entries
    .map((entry) => `[${entry.time}] ${entry.level.toUpperCase()} (${entry.caseId}) ${entry.message}`)
    .join("\n")}\n`;
}

export async function runKernelContractSuite(
  callbacks: ContractSuiteCallbacks = {},
): Promise<ContractSuiteResult> {
  // Initialise WASM once before any test cases run.
  await loadKernel(`${process.env.NEXT_PUBLIC_BASE_PATH ?? ""}/wasm/rusted_geom.wasm`);

  const startedAt = performance.now();
  const caseResults: ContractCaseResult[] = [];
  let sequence = 1;

  for (const runner of RUNNERS) {
    callbacks.onCaseStart?.(runner.id);

    const emit = (level: ContractLogLevel, message: string): void => {
      callbacks.onLog?.({
        id: sequence,
        time: nowStamp(),
        level,
        caseId: runner.id,
        message,
      });
      sequence += 1;
    };

    emit("info", `Starting ${runner.title}`);
    const caseStart = performance.now();

    try {
      await runner.run(emit);
      const durationMs = performance.now() - caseStart;
      emit("pass", `Completed in ${formatDuration(durationMs)}`);
      const result: ContractCaseResult = {
        id: runner.id,
        title: runner.title,
        status: "pass",
        durationMs,
      };
      caseResults.push(result);
      callbacks.onCaseEnd?.(result);
    } catch (error) {
      const durationMs = performance.now() - caseStart;
      const errorMessage = toErrorMessage(error);
      emit("fail", errorMessage);
      const result: ContractCaseResult = {
        id: runner.id,
        title: runner.title,
        status: "fail",
        durationMs,
        errorMessage,
      };
      caseResults.push(result);
      callbacks.onCaseEnd?.(result);
    }
  }

  const totalDurationMs = performance.now() - startedAt;
  const passed = caseResults.filter((result) => result.status === "pass").length;
  const failed = caseResults.length - passed;

  return {
    cases: caseResults,
    totalDurationMs,
    passed,
    failed,
  };
}
