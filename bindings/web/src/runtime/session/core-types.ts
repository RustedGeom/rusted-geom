import type {
  RgmArc3,
  RgmBrepValidationReport,
  RgmCircle3,
  RgmIntersectionBranchSummary,
  RgmLine3,
  RgmNurbsSurfaceDesc,
  RgmPlane,
  RgmPoint3,
  RgmPolycurveSegment,
  RgmSurfaceEvalFrame,
  RgmSurfaceTessellationOptions,
  RgmToleranceContext,
  RgmTrimEdgeInput,
  RgmTrimLoopInput,
  RgmUv2,
  RgmVec3,
} from "../../generated/types";
import type {
  BrepFaceId,
  BrepHandle,
  BrepLoopId,
  BrepShellId,
  BrepSolidId,
  CurveHandle,
  FaceHandle,
  IntersectionHandle,
  MeshHandle,
  ObjectHandle,
  SurfaceHandle,
} from "./handles";

export interface CurvePresetInput {
  name?: string;
  degree: number;
  closed: boolean;
  points: RgmPoint3[];
  tolerance: RgmToleranceContext;
}

export interface KernelCapabilities {
  igesImport: boolean;
  igesExport: boolean;
}

export interface RgmSurfaceFirstDerivatives {
  du: RgmVec3;
  dv: RgmVec3;
}

export interface RgmSurfaceSecondDerivatives {
  duu: RgmVec3;
  duv: RgmVec3;
  dvv: RgmVec3;
}

export interface KernelSession {
  readonly handle: bigint;
  buildCurveFromPreset(preset: CurvePresetInput): CurveHandle;
  createLine(line: RgmLine3, tolerance: RgmToleranceContext): CurveHandle;
  createArc(arc: RgmArc3, tolerance: RgmToleranceContext): CurveHandle;
  createCircle(circle: RgmCircle3, tolerance: RgmToleranceContext): CurveHandle;
  createPolyline(points: RgmPoint3[], closed: boolean, tolerance: RgmToleranceContext): CurveHandle;
  createPolycurve(segments: RgmPolycurveSegment[], tolerance: RgmToleranceContext): CurveHandle;
  sampleCurvePolyline(curveHandle: CurveHandle, sampleCount: number): RgmPoint3[];
  curvePointAt(curveHandle: CurveHandle, tNorm: number): RgmPoint3;
  curveLength(curveHandle: CurveHandle): number;
  curveLengthAt(curveHandle: CurveHandle, tNorm: number): number;
  intersectCurvePlane(curveHandle: CurveHandle, plane: RgmPlane): RgmPoint3[];
  intersectCurveCurve(curveA: CurveHandle, curveB: CurveHandle): RgmPoint3[];
  createMeshBox(center: RgmPoint3, size: RgmVec3): MeshHandle;
  createMeshUvSphere(center: RgmPoint3, radius: number, uSteps: number, vSteps: number): MeshHandle;
  createMeshTorus(
    center: RgmPoint3,
    majorRadius: number,
    minorRadius: number,
    majorSteps: number,
    minorSteps: number,
  ): MeshHandle;
  meshTranslate(meshHandle: MeshHandle, delta: RgmVec3): MeshHandle;
  meshRotate(meshHandle: MeshHandle, axis: RgmVec3, angleRad: number, pivot: RgmPoint3): MeshHandle;
  meshScale(meshHandle: MeshHandle, scale: RgmVec3, pivot: RgmPoint3): MeshHandle;
  meshBakeTransform(meshHandle: MeshHandle): MeshHandle;
  meshBoolean(meshA: MeshHandle, meshB: MeshHandle, op: 0 | 1 | 2): MeshHandle;
  intersectMeshPlane(meshHandle: MeshHandle, plane: RgmPlane): RgmPoint3[];
  intersectMeshMesh(meshA: MeshHandle, meshB: MeshHandle): RgmPoint3[];
  meshVertexCount(meshHandle: MeshHandle): number;
  meshTriangleCount(meshHandle: MeshHandle): number;
  meshToBuffers(meshHandle: MeshHandle): { vertices: RgmPoint3[]; indices: number[] };
  createNurbsSurface(
    desc: RgmNurbsSurfaceDesc,
    controlPoints: RgmPoint3[],
    weights: number[],
    knotsU: number[],
    knotsV: number[],
    tolerance: RgmToleranceContext,
  ): SurfaceHandle;
  surfacePointAt(surfaceHandle: SurfaceHandle, uvNorm: RgmUv2): RgmPoint3;
  surfaceD1At(surfaceHandle: SurfaceHandle, uvNorm: RgmUv2): RgmSurfaceFirstDerivatives;
  surfaceD2At(surfaceHandle: SurfaceHandle, uvNorm: RgmUv2): RgmSurfaceSecondDerivatives;
  surfaceFrameAt(surfaceHandle: SurfaceHandle, uvNorm: RgmUv2): RgmSurfaceEvalFrame;
  surfaceTranslate(surfaceHandle: SurfaceHandle, delta: RgmVec3): SurfaceHandle;
  surfaceRotate(surfaceHandle: SurfaceHandle, axis: RgmVec3, angleRad: number, pivot: RgmPoint3): SurfaceHandle;
  surfaceScale(surfaceHandle: SurfaceHandle, scale: RgmVec3, pivot: RgmPoint3): SurfaceHandle;
  surfaceBakeTransform(surfaceHandle: SurfaceHandle): SurfaceHandle;
  surfaceTessellateToMesh(
    surfaceHandle: SurfaceHandle,
    options?: RgmSurfaceTessellationOptions,
  ): MeshHandle;
  createFaceFromSurface(surfaceHandle: SurfaceHandle): FaceHandle;
  faceAddLoop(faceHandle: FaceHandle, points: RgmUv2[], isOuter: boolean): void;
  faceAddLoopEdges(faceHandle: FaceHandle, loopInput: RgmTrimLoopInput, edges: RgmTrimEdgeInput[]): void;
  faceRemoveLoop(faceHandle: FaceHandle, loopIndex: number): void;
  faceSplitTrimEdge(
    faceHandle: FaceHandle,
    loopIndex: number,
    edgeIndex: number,
    splitT: number,
  ): void;
  faceReverseLoop(faceHandle: FaceHandle, loopIndex: number): void;
  faceValidate(faceHandle: FaceHandle): boolean;
  faceHeal(faceHandle: FaceHandle): void;
  faceTessellateToMesh(faceHandle: FaceHandle, options?: RgmSurfaceTessellationOptions): MeshHandle;
  brepCreateEmpty(): BrepHandle;
  brepCreateFromFaces(faces: FaceHandle[]): BrepHandle;
  brepCreateFromSurface(surfaceHandle: SurfaceHandle): BrepHandle;
  brepAddFace(brepHandle: BrepHandle, faceHandle: FaceHandle): BrepFaceId;
  brepAddFaceFromSurface(brepHandle: BrepHandle, surfaceHandle: SurfaceHandle): BrepFaceId;
  brepAddLoopUv(
    brepHandle: BrepHandle,
    faceId: BrepFaceId,
    points: RgmUv2[],
    isOuter: boolean,
  ): BrepLoopId;
  brepFinalizeShell(brepHandle: BrepHandle): BrepShellId;
  brepFinalizeSolid(brepHandle: BrepHandle): BrepSolidId;
  brepValidate(brepHandle: BrepHandle): RgmBrepValidationReport;
  brepHeal(brepHandle: BrepHandle): number;
  brepClone(brepHandle: BrepHandle): BrepHandle;
  brepFaceCount(brepHandle: BrepHandle): number;
  brepShellCount(brepHandle: BrepHandle): number;
  brepSolidCount(brepHandle: BrepHandle): number;
  brepIsSolid(brepHandle: BrepHandle): boolean;
  brepFaceAdjacency(brepHandle: BrepHandle, faceId: BrepFaceId): BrepFaceId[];
  brepTessellateToMesh(
    brepHandle: BrepHandle,
    options?: RgmSurfaceTessellationOptions,
  ): MeshHandle;
  brepFromFaceObject(faceHandle: FaceHandle): BrepHandle;
  brepExtractFaceObject(brepHandle: BrepHandle, faceId: BrepFaceId): FaceHandle;
  brepState(brepHandle: BrepHandle): 0 | 1;
  brepEstimateArea(brepHandle: BrepHandle): number;
  brepSaveNative(brepHandle: BrepHandle): Uint8Array;
  brepLoadNative(bytes: Uint8Array | ArrayBuffer | ArrayBufferView): BrepHandle;
  intersectSurfaceSurface(surfaceA: SurfaceHandle, surfaceB: SurfaceHandle): IntersectionHandle;
  intersectSurfacePlane(surface: SurfaceHandle, plane: RgmPlane): IntersectionHandle;
  intersectSurfaceCurve(surface: SurfaceHandle, curve: CurveHandle): IntersectionHandle;
  intersectionBranchCount(intersection: IntersectionHandle): number;
  intersectionBranchSummary(intersection: IntersectionHandle, branchIndex: number): RgmIntersectionBranchSummary;
  intersectionBranchPoints(intersection: IntersectionHandle, branchIndex: number): RgmPoint3[];
  intersectionBranchUvA(intersection: IntersectionHandle, branchIndex: number): RgmUv2[];
  intersectionBranchUvB(intersection: IntersectionHandle, branchIndex: number): RgmUv2[];
  intersectionBranchCurveT(intersection: IntersectionHandle, branchIndex: number): number[];
  intersectionBranchToNurbs(
    intersection: IntersectionHandle,
    branchIndex: number,
    tolerance: RgmToleranceContext,
  ): CurveHandle;
  releaseObject(objectHandle: ObjectHandle): void;
  lastError(): { code: number; message: string };
  destroy(): void;
}

export interface KernelRuntime {
  readonly capabilities: KernelCapabilities;
  createSession(): KernelSession;
  destroy(): void;
}
