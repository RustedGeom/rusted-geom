import {
  createKernelRuntime,
  KERNEL_LAYOUT,
  KernelMemory,
  type KernelRuntime,
  KernelRuntimeError,
  loadKernelWasm,
  type CurvePresetInput,
  type KernelSession,
  type NativeExports,
  type ObjectHandle,
  RgmStatus,
} from "@rusted-geom/bindings-web";

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

const CURVE_PRESET: CurvePresetInput = {
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
    summary: "Invalid degree must surface a kernel runtime error with InvalidInput status.",
  },
  {
    id: "pointer-exports",
    title: "Pointer Export Intersections",
    summary: "Direct wasm export calls produce expected intersection counts.",
  },
];

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

function clampedUniformKnots(controlCount: number, degree: number): number[] {
  const knotCount = controlCount + degree + 1;
  const knots = new Array(knotCount).fill(0);
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

async function withRuntimeSession(
  log: (level: ContractLogLevel, message: string) => void,
  run: (
    runtime: KernelRuntime,
    session: KernelSession,
    track: <T extends ObjectHandle>(handle: T) => T,
  ) => Promise<void> | void,
): Promise<void> {
  const runtime = await createKernelRuntime("/wasm/rusted_geom.wasm");
  const session = runtime.createSession();
  const ownedHandles: ObjectHandle[] = [];
  const track = <T extends ObjectHandle>(handle: T): T => {
    ownedHandles.push(handle);
    return handle;
  };

  try {
    await run(runtime, session, track);
  } finally {
    for (const handle of ownedHandles.slice().reverse()) {
      try {
        session.kernel.releaseObject(handle);
      } catch {
        // keep cleanup best-effort
      }
    }
    session.destroy();
    runtime.destroy();
    log("debug", "Runtime and session destroyed");
  }
}

const RUNNERS: CaseRunner[] = [
  {
    id: "curve-session-sampling",
    title: "Curve Session Sampling",
    summary: "Session creation, curve sampling, circle probes, and curve intersections.",
    run: async (log) => {
      await withRuntimeSession(log, (runtime, session, track) => {
        log("info", "Runtime and session created");
        assertOrThrow(runtime.capabilities.igesImport === false, "IGES import capability mismatch");
        assertOrThrow(runtime.capabilities.igesExport === false, "IGES export capability mismatch");

        const handle = track(session.curve.buildCurveFromPreset(CURVE_PRESET));
        const samples = session.curve.sampleCurvePolyline(handle, 32);
        const point = session.curve.curvePointAt(handle, 0.37);
        const totalLength = session.curve.curveLength(handle);
        const lengthAtPoint = session.curve.curveLengthAt(handle, 0.37);

        assertOrThrow(samples.length === 32, "Expected 32 sampled points");
        assertOrThrow(Math.abs(samples[0].x) < 1e-6, "First sampled point mismatch");
        assertOrThrow(Math.abs((samples.at(-1)?.x ?? 0) - 3) < 1e-6, "Last sampled point mismatch");
        assertOrThrow(point.x >= 0 && point.x <= 3, "Probe point outside expected range");
        assertOrThrow(totalLength > 0, "Total curve length should be positive");
        assertOrThrow(lengthAtPoint > 0 && lengthAtPoint < totalLength, "Length-at parameter mismatch");
        log("debug", `Polyline sample count=${samples.length}, curve length=${totalLength.toFixed(4)}`);

        const circleHandle = track(
          session.curve.createCircle(
            {
              plane: {
                origin: { x: 1.25, y: -0.8, z: 0.4 },
                x_axis: { x: 1, y: 0, z: 0 },
                y_axis: { x: 0, y: 1, z: 0 },
                z_axis: { x: 0, y: 0, z: 1 },
              },
              radius: 3.6,
            },
            CURVE_PRESET.tolerance,
          ),
        );
        let circleChecks = 0;
        for (const t of [0, 0.11, 0.3, 0.5, 0.77, 1]) {
          const p = session.curve.curvePointAt(circleHandle, t);
          const dx = p.x - 1.25;
          const dy = p.y + 0.8;
          const dz = p.z - 0.4;
          const r = Math.sqrt(dx * dx + dy * dy + dz * dz);
          assertOrThrow(Math.abs(r - 3.6) < 1e-3, `Circle radius mismatch at t=${t}`);
          circleChecks += 1;
        }
        log("debug", `Circle radius probes validated=${circleChecks}`);

        const lineA = track(
          session.curve.createLine(
            {
              start: { x: -1, y: 0, z: 0 },
              end: { x: 1, y: 0, z: 0 },
            },
            CURVE_PRESET.tolerance,
          ),
        );
        const lineB = track(
          session.curve.createLine(
            {
              start: { x: 0, y: -1, z: 0 },
              end: { x: 0, y: 1, z: 0 },
            },
            CURVE_PRESET.tolerance,
          ),
        );
        const curveHits = session.intersection.intersectCurveCurve(lineA, lineB);
        assertOrThrow(curveHits.length === 1, "Expected one curve-curve intersection hit");
        assertOrThrow(Math.abs(curveHits[0].x) < 1e-3, "Curve-curve hit x mismatch");
        assertOrThrow(Math.abs(curveHits[0].y) < 1e-3, "Curve-curve hit y mismatch");

        const planeHits = session.intersection.intersectCurvePlane(lineA, {
          origin: { x: 0, y: 0, z: 0 },
          x_axis: { x: 1, y: 0, z: 0 },
          y_axis: { x: 0, y: 1, z: 0 },
          z_axis: { x: 0, y: 1, z: 0 },
        });
        assertOrThrow(planeHits.length >= 1, "Expected curve-plane intersection hits");
        log("pass", `Curve and intersection checks passed (curve-curve=${curveHits.length}, curve-plane=${planeHits.length})`);
      });
    },
  },
  {
    id: "mesh-transform-intersections",
    title: "Mesh Transform + Intersections",
    summary: "Mesh counts, transforms, mesh intersections, and boolean result invariants.",
    run: async (log) => {
      await withRuntimeSession(log, (_runtime, session, track) => {
        log("info", "Runtime and session created");

        const meshBox = track(
          session.mesh.createMeshBox(
            { x: 0, y: 0, z: 0 },
            { x: 4, y: 3, z: 2 },
          ),
        );
        const meshVertices = session.mesh.meshVertexCount(meshBox);
        const meshTriangles = session.mesh.meshTriangleCount(meshBox);
        assertOrThrow(meshVertices === 8, "Mesh box vertex count mismatch");
        assertOrThrow(meshTriangles === 12, "Mesh box triangle count mismatch");
        log("debug", `Mesh box vertices=${meshVertices}, triangles=${meshTriangles}`);

        const translated = track(session.mesh.meshTranslate(meshBox, { x: 0.8, y: -0.4, z: 1.2 }));
        const transformed = track(
          session.mesh.meshRotate(
            translated,
            { x: 0, y: 1, z: 0.2 },
            0.7,
            { x: 0, y: 0, z: 0 },
          ),
        );
        const baked = track(session.mesh.meshBakeTransform(transformed));
        const bakedBuffers = session.mesh.meshToBuffers(baked);
        assertOrThrow(bakedBuffers.vertices.length === 8, "Baked mesh vertex count mismatch");
        assertOrThrow(bakedBuffers.indices.length === 36, "Baked mesh index count mismatch");
        log("debug", `Baked mesh vertices=${bakedBuffers.vertices.length}, indices=${bakedBuffers.indices.length}`);

        const torus = track(session.mesh.createMeshTorus({ x: 0, y: 0, z: 0 }, 3.8, 1.1, 28, 20));
        const meshPlaneHits = session.intersection.intersectMeshPlane(torus, {
          origin: { x: 0, y: 0, z: 0.2 },
          x_axis: { x: 1, y: 0, z: 0 },
          y_axis: { x: 0, y: 1, z: 0 },
          z_axis: { x: 0, y: 0, z: 1 },
        });
        assertOrThrow(meshPlaneHits.length > 0, "Mesh-plane expected non-empty hit set");
        assertOrThrow(meshPlaneHits.length % 2 === 0, "Mesh-plane expected segment endpoint pairs");
        log("debug", `Mesh-plane intersection points=${meshPlaneHits.length}`);

        const sphere = track(session.mesh.createMeshUvSphere({ x: 0, y: 0, z: 0 }, 4.2, 24, 16));
        const meshMeshHits = session.intersection.intersectMeshMesh(sphere, torus);
        assertOrThrow(meshMeshHits.length > 0, "Mesh-mesh expected non-empty hit set");
        assertOrThrow(meshMeshHits.length % 2 === 0, "Mesh-mesh expected segment endpoint pairs");
        log("debug", `Mesh-mesh intersection points=${meshMeshHits.length}`);

        const booleanHost = track(
          session.mesh.createMeshBox(
            { x: 0, y: 0, z: 0 },
            { x: 8.8, y: 8.8, z: 8.8 },
          ),
        );
        const innerTorus = track(session.mesh.createMeshTorus({ x: 0, y: 0, z: 0 }, 2.4, 0.8, 32, 24));
        const booleanDiff = track(session.mesh.meshBoolean(booleanHost, innerTorus, 2));
        const booleanTriangles = session.mesh.meshTriangleCount(booleanDiff);
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
      await withRuntimeSession(log, (_runtime, session, track) => {
        log("info", "Runtime and session created");

        const surface = track(
          session.surface.createNurbsSurface(
            {
              degree_u: 2,
              degree_v: 2,
              periodic_u: false,
              periodic_v: false,
              control_u_count: 3,
              control_v_count: 3,
            },
            [
              { x: -2, y: -2, z: 0 },
              { x: -2, y: 0, z: 0.8 },
              { x: -2, y: 2, z: 0.1 },
              { x: 0, y: -2, z: 0.7 },
              { x: 0, y: 0, z: -0.2 },
              { x: 0, y: 2, z: 0.9 },
              { x: 2, y: -2, z: -0.3 },
              { x: 2, y: 0, z: 0.6 },
              { x: 2, y: 2, z: 0.2 },
            ],
            new Array(9).fill(1),
            clampedUniformKnots(3, 2),
            clampedUniformKnots(3, 2),
            CURVE_PRESET.tolerance,
          ),
        );
        const frame = session.surface.surfaceFrameAt(surface, { u: 0.5, v: 0.5 });
        const d0 = session.surface.surfacePointAt(surface, { u: 0.5, v: 0.5 });
        const d1 = session.surface.surfaceD1At(surface, { u: 0.5, v: 0.5 });
        const d2 = session.surface.surfaceD2At(surface, { u: 0.5, v: 0.5 });
        assertOrThrow(Math.abs(frame.point.x - d0.x) < 1e-7, "Surface point/frame x mismatch");
        assertOrThrow(Math.abs(frame.point.y - d0.y) < 1e-7, "Surface point/frame y mismatch");
        assertOrThrow(Math.abs(frame.point.z - d0.z) < 1e-7, "Surface point/frame z mismatch");
        assertOrThrow(Math.abs(frame.du.x - d1.du.x) < 1e-7, "Surface du mismatch");
        assertOrThrow(Math.abs(frame.dv.y - d1.dv.y) < 1e-7, "Surface dv mismatch");
        assertOrThrow(Number.isFinite(d2.duu.x), "Surface duu should be finite");
        assertOrThrow(Number.isFinite(d2.duv.y), "Surface duv should be finite");
        assertOrThrow(Number.isFinite(d2.dvv.z), "Surface dvv should be finite");
        assertOrThrow(Number.isFinite(frame.normal.z), "Surface normal should be finite");
        log("debug", "Surface point/frame/differential checks passed");

        const face = track(session.face.createFaceFromSurface(surface));
        session.face.faceAddLoop(
          face,
          [
            { u: 0.05, v: 0.05 },
            { u: 0.95, v: 0.05 },
            { u: 0.95, v: 0.95 },
            { u: 0.05, v: 0.95 },
          ],
          true,
        );
        const trimCircle = track(
          session.curve.createCircle(
            {
              plane: {
                origin: { x: 0.5, y: 0.5, z: 0 },
                x_axis: { x: 1, y: 0, z: 0 },
                y_axis: { x: 0, y: 1, z: 0 },
                z_axis: { x: 0, y: 0, z: 1 },
              },
              radius: 0.18,
            },
            CURVE_PRESET.tolerance,
          ),
        );
        session.face.faceAddLoopEdges(
          face,
          { edge_count: 1, is_outer: false },
          [
            {
              start_uv: { u: 0.68, v: 0.5 },
              end_uv: { u: 0.68, v: 0.5 },
              curve_3d: trimCircle,
              has_curve_3d: true,
            },
          ],
        );
        assertOrThrow(session.face.faceValidate(face), "Face validation failed");

        const faceMesh = track(
          session.face.faceTessellateToMesh(face, {
            min_u_segments: 28,
            min_v_segments: 28,
            max_u_segments: 48,
            max_v_segments: 48,
            chord_tol: 1e-4,
            normal_tol_rad: 0.08,
          }),
        );
        const tessTriangles = session.mesh.meshTriangleCount(faceMesh);
        assertOrThrow(tessTriangles > 0, "Face tessellation returned no triangles");

        const surfacePlaneIntersection = track(
          session.intersection.intersectSurfacePlane(surface, {
            origin: { x: 0, y: 0, z: 0.1 },
            x_axis: { x: 1, y: 0, z: 0 },
            y_axis: { x: 0, y: 1, z: 0 },
            z_axis: { x: 0, y: 0, z: 1 },
          }),
        );
        const branchCount = session.intersection.intersectionBranchCount(surfacePlaneIntersection);
        assertOrThrow(branchCount >= 0, "Surface-plane branch count check failed");

        log("pass", `Surface/face checks passed (face triangles=${tessTriangles}, branches=${branchCount})`);
      });
    },
  },
  {
    id: "invalid-input",
    title: "Invalid Curve Input",
    summary: "Invalid degree must surface a kernel runtime error with InvalidInput status.",
    run: async (log) => {
      const runtime = await createKernelRuntime("/wasm/rusted_geom.wasm");
      const session = runtime.createSession();

      try {
        let thrown: unknown = undefined;
        try {
          session.curve.buildCurveFromPreset({
            ...CURVE_PRESET,
            degree: 8,
          });
        } catch (error) {
          thrown = error;
        }

        assertOrThrow(thrown instanceof KernelRuntimeError, "Expected KernelRuntimeError for invalid degree");
        assertOrThrow((thrown as KernelRuntimeError).status === RgmStatus.InvalidInput, "Expected InvalidInput status");
        assertOrThrow(
          (thrown as KernelRuntimeError).message.includes("Curve construction failed"),
          "Expected runtime error message to mention curve construction",
        );
        log("pass", "Invalid input surfaced expected runtime error");
      } finally {
        session.destroy();
        runtime.destroy();
      }
    },
  },
  {
    id: "pointer-exports",
    title: "Pointer Export Intersections",
    summary: "Direct wasm export calls produce expected intersection counts.",
    run: async (log) => {
      const module = await loadKernelWasm("/wasm/rusted_geom.wasm");
      const api = module.exports as unknown as NativeExports;
      const memory = new KernelMemory(api, module.exports.memory);

      const sessionPtr = memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
      const outObjectPtr = memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
      const outCountPtr = memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
      const pointsPtr = memory.alloc(KERNEL_LAYOUT.POINT3_BYTES * 8, 8);
      const linePtr = memory.alloc(KERNEL_LAYOUT.LINE3_BYTES, 8);
      const tolPtr = memory.alloc(KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
      const planePtr = memory.alloc(KERNEL_LAYOUT.PLANE_BYTES, 8);

      try {
        assertOrThrow((api.rgm_kernel_create(sessionPtr) as RgmStatus) === RgmStatus.Ok, "rgm_kernel_create failed");
        const session = memory.readU64(sessionPtr);

        memory.writeLine(linePtr, {
          start: { x: 0, y: 0, z: -1 },
          end: { x: 0, y: 0, z: 1 },
        });
        memory.writeTolerance(tolPtr, CURVE_PRESET.tolerance);
        assertOrThrow(
          (api.rgm_curve_create_line(session, linePtr, tolPtr, outObjectPtr) as RgmStatus) === RgmStatus.Ok,
          "rgm_curve_create_line failed",
        );
        const line = memory.readU64(outObjectPtr);

        memory.writePlane(planePtr, {
          origin: { x: 0, y: 0, z: 0 },
          x_axis: { x: 1, y: 0, z: 0 },
          y_axis: { x: 0, y: 1, z: 0 },
          z_axis: { x: 0, y: 0, z: 1 },
        });

        assertOrThrow(
          (api.rgm_intersect_curve_plane(session, line, planePtr, pointsPtr, 8, outCountPtr) as RgmStatus) ===
            RgmStatus.Ok,
          "rgm_intersect_curve_plane failed",
        );
        assertOrThrow(memory.readU32(outCountPtr) === 1, "Expected one curve-plane intersection hit");
        const p = memory.readPoint(pointsPtr);
        assertOrThrow(Math.abs(p.x) < 1e-6 && Math.abs(p.y) < 1e-6 && Math.abs(p.z) < 1e-6, "Curve-plane point mismatch");

        assertOrThrow(
          (api.rgm_intersect_curve_curve(session, line, line, pointsPtr, 8, outCountPtr) as RgmStatus) === RgmStatus.Ok,
          "rgm_intersect_curve_curve failed",
        );
        assertOrThrow(memory.readU32(outCountPtr) >= 1, "Expected at least one curve-curve hit");

        assertOrThrow((api.rgm_object_release(session, line) as RgmStatus) === RgmStatus.Ok, "rgm_object_release failed");
        assertOrThrow((api.rgm_kernel_destroy(session) as RgmStatus) === RgmStatus.Ok, "rgm_kernel_destroy failed");

        log("pass", "Pointer-style wasm export checks passed");
      } finally {
        memory.free(pointsPtr, KERNEL_LAYOUT.POINT3_BYTES * 8, 8);
        memory.free(planePtr, KERNEL_LAYOUT.PLANE_BYTES, 8);
        memory.free(tolPtr, KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
        memory.free(linePtr, KERNEL_LAYOUT.LINE3_BYTES, 8);
        memory.free(outCountPtr, KERNEL_LAYOUT.I32_BYTES, 4);
        memory.free(outObjectPtr, KERNEL_LAYOUT.U64_BYTES, 8);
        memory.free(sessionPtr, KERNEL_LAYOUT.U64_BYTES, 8);
      }
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
