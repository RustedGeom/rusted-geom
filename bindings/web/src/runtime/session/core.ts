import type { NativeExports } from "../../generated/native";
import type {
  RgmBrepValidationReport,
  RgmIntersectionBranchSummary,
  RgmArc3,
  RgmCircle3,
  RgmLine3,
  RgmNurbsSurfaceDesc,
  RgmPlane,
  RgmPoint3,
  RgmPolycurveSegment,
  RgmTrimEdgeInput,
  RgmTrimLoopInput,
  RgmSurfaceEvalFrame,
  RgmSurfaceTessellationOptions,
  RgmToleranceContext,
  RgmUv2,
  RgmVec3,
} from "../../generated/types";
import { RgmStatus } from "../../generated/types";
import { KernelRuntimeError, statusToName } from "../errors";
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
import { KERNEL_LAYOUT, KernelMemory } from "../memory";
import { sampleCurvePolyline } from "../scene-sampler";
import { loadKernelWasm, type WasmSource } from "../wasm-loader";
import type {
  CurvePresetInput,
  KernelCapabilities,
  KernelRuntime,
  KernelSession,
  RgmSurfaceFirstDerivatives,
  RgmSurfaceSecondDerivatives,
} from "./core-types";
export type {
  CurvePresetInput,
  KernelCapabilities,
  KernelRuntime,
  KernelSession,
  RgmSurfaceFirstDerivatives,
  RgmSurfaceSecondDerivatives,
} from "./core-types";

import { KernelSessionBase } from "./core-base";

class KernelSessionImpl extends KernelSessionBase implements KernelSession {
  createNurbsSurface(
    desc: RgmNurbsSurfaceDesc,
    controlPoints: RgmPoint3[],
    weights: number[],
    knotsU: number[],
    knotsV: number[],
    tolerance: RgmToleranceContext,
  ): SurfaceHandle {
    this.ensureAlive();
    if (
      controlPoints.length === 0 ||
      weights.length !== controlPoints.length ||
      desc.control_u_count * desc.control_v_count !== controlPoints.length
    ) {
      throw new Error("Invalid surface control net/weights");
    }
    const descPtr = this.memory.alloc(KERNEL_LAYOUT.NURBS_SURFACE_DESC_BYTES, 8);
    const ctrlPtr = this.memory.alloc(controlPoints.length * KERNEL_LAYOUT.POINT3_BYTES, 8);
    const weightsPtr = this.memory.alloc(weights.length * KERNEL_LAYOUT.F64_BYTES, 8);
    const knotsUPtr = this.memory.alloc(knotsU.length * KERNEL_LAYOUT.F64_BYTES, 8);
    const knotsVPtr = this.memory.alloc(knotsV.length * KERNEL_LAYOUT.F64_BYTES, 8);
    const tolPtr = this.memory.alloc(KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);

    try {
      this.memory.writeNurbsSurfaceDesc(descPtr, desc);
      this.memory.writePointArray(ctrlPtr, controlPoints);
      this.memory.writeF64Array(weightsPtr, weights);
      this.memory.writeF64Array(knotsUPtr, knotsU);
      this.memory.writeF64Array(knotsVPtr, knotsV);
      this.memory.writeTolerance(tolPtr, tolerance);

      const status = this.api.rgm_surface_create_nurbs(
        this.handle,
        descPtr,
        ctrlPtr,
        controlPoints.length,
        weightsPtr,
        weights.length,
        knotsUPtr,
        knotsU.length,
        knotsVPtr,
        knotsV.length,
        tolPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface construction failed");
      return this.memory.readU64(outPtr) as SurfaceHandle;
    } finally {
      this.memory.free(descPtr, KERNEL_LAYOUT.NURBS_SURFACE_DESC_BYTES, 8);
      this.memory.free(ctrlPtr, controlPoints.length * KERNEL_LAYOUT.POINT3_BYTES, 8);
      this.memory.free(weightsPtr, weights.length * KERNEL_LAYOUT.F64_BYTES, 8);
      this.memory.free(knotsUPtr, knotsU.length * KERNEL_LAYOUT.F64_BYTES, 8);
      this.memory.free(knotsVPtr, knotsV.length * KERNEL_LAYOUT.F64_BYTES, 8);
      this.memory.free(tolPtr, KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  surfacePointAt(surfaceHandle: SurfaceHandle, uvNorm: RgmUv2): RgmPoint3 {
    this.ensureAlive();
    const uvPtr = this.memory.alloc(KERNEL_LAYOUT.UV2_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.POINT3_BYTES, 8);
    try {
      this.memory.writeUv(uvPtr, uvNorm);
      const status = this.api.rgm_surface_point_at(this.handle, surfaceHandle, uvPtr, outPtr) as RgmStatus;
      this.assertOk(status, "Surface point evaluation failed");
      return this.memory.readPoint(outPtr);
    } finally {
      this.memory.free(uvPtr, KERNEL_LAYOUT.UV2_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.POINT3_BYTES, 8);
    }
  }

  surfaceD1At(surfaceHandle: SurfaceHandle, uvNorm: RgmUv2): RgmSurfaceFirstDerivatives {
    this.ensureAlive();
    const uvPtr = this.memory.alloc(KERNEL_LAYOUT.UV2_BYTES, 8);
    const outDuPtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    const outDvPtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    try {
      this.memory.writeUv(uvPtr, uvNorm);
      const status = this.api.rgm_surface_d1_at(
        this.handle,
        surfaceHandle,
        uvPtr,
        outDuPtr,
        outDvPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface first derivatives failed");
      return {
        du: this.memory.readVec(outDuPtr),
        dv: this.memory.readVec(outDvPtr),
      };
    } finally {
      this.memory.free(uvPtr, KERNEL_LAYOUT.UV2_BYTES, 8);
      this.memory.free(outDuPtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
      this.memory.free(outDvPtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
    }
  }

  surfaceD2At(surfaceHandle: SurfaceHandle, uvNorm: RgmUv2): RgmSurfaceSecondDerivatives {
    this.ensureAlive();
    const uvPtr = this.memory.alloc(KERNEL_LAYOUT.UV2_BYTES, 8);
    const outDuuPtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    const outDuvPtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    const outDvvPtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    try {
      this.memory.writeUv(uvPtr, uvNorm);
      const status = this.api.rgm_surface_d2_at(
        this.handle,
        surfaceHandle,
        uvPtr,
        outDuuPtr,
        outDuvPtr,
        outDvvPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface second derivatives failed");
      return {
        duu: this.memory.readVec(outDuuPtr),
        duv: this.memory.readVec(outDuvPtr),
        dvv: this.memory.readVec(outDvvPtr),
      };
    } finally {
      this.memory.free(uvPtr, KERNEL_LAYOUT.UV2_BYTES, 8);
      this.memory.free(outDuuPtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
      this.memory.free(outDuvPtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
      this.memory.free(outDvvPtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
    }
  }

  surfaceFrameAt(surfaceHandle: SurfaceHandle, uvNorm: RgmUv2): RgmSurfaceEvalFrame {
    this.ensureAlive();
    const uvPtr = this.memory.alloc(KERNEL_LAYOUT.UV2_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.SURFACE_EVAL_FRAME_BYTES, 8);
    try {
      this.memory.writeUv(uvPtr, uvNorm);
      const status = this.api.rgm_surface_frame_at(this.handle, surfaceHandle, uvPtr, outPtr) as RgmStatus;
      this.assertOk(status, "Surface frame evaluation failed");
      return this.memory.readSurfaceEvalFrame(outPtr);
    } finally {
      this.memory.free(uvPtr, KERNEL_LAYOUT.UV2_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.SURFACE_EVAL_FRAME_BYTES, 8);
    }
  }

  surfaceTranslate(surfaceHandle: SurfaceHandle, delta: RgmVec3): SurfaceHandle {
    this.ensureAlive();
    const deltaPtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writeVec(deltaPtr, delta);
      const status = this.api.rgm_surface_translate(
        this.handle,
        surfaceHandle,
        deltaPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface translation failed");
      return this.memory.readU64(outPtr) as SurfaceHandle;
    } finally {
      this.memory.free(deltaPtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  surfaceRotate(surfaceHandle: SurfaceHandle, axis: RgmVec3, angleRad: number, pivot: RgmPoint3): SurfaceHandle {
    this.ensureAlive();
    const axisPtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    const pivotPtr = this.memory.alloc(KERNEL_LAYOUT.POINT3_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writeVec(axisPtr, axis);
      this.memory.writePoint(pivotPtr, pivot);
      const status = this.api.rgm_surface_rotate(
        this.handle,
        surfaceHandle,
        axisPtr,
        angleRad,
        pivotPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface rotation failed");
      return this.memory.readU64(outPtr) as SurfaceHandle;
    } finally {
      this.memory.free(axisPtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
      this.memory.free(pivotPtr, KERNEL_LAYOUT.POINT3_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  surfaceScale(surfaceHandle: SurfaceHandle, scale: RgmVec3, pivot: RgmPoint3): SurfaceHandle {
    this.ensureAlive();
    const scalePtr = this.memory.alloc(KERNEL_LAYOUT.VEC3_BYTES, 8);
    const pivotPtr = this.memory.alloc(KERNEL_LAYOUT.POINT3_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writeVec(scalePtr, scale);
      this.memory.writePoint(pivotPtr, pivot);
      const status = this.api.rgm_surface_scale(
        this.handle,
        surfaceHandle,
        scalePtr,
        pivotPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface scale failed");
      return this.memory.readU64(outPtr) as SurfaceHandle;
    } finally {
      this.memory.free(scalePtr, KERNEL_LAYOUT.VEC3_BYTES, 8);
      this.memory.free(pivotPtr, KERNEL_LAYOUT.POINT3_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  surfaceBakeTransform(surfaceHandle: SurfaceHandle): SurfaceHandle {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_surface_bake_transform(this.handle, surfaceHandle, outPtr) as RgmStatus;
      this.assertOk(status, "Surface bake failed");
      return this.memory.readU64(outPtr) as SurfaceHandle;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  surfaceTessellateToMesh(surfaceHandle: SurfaceHandle, options?: RgmSurfaceTessellationOptions): MeshHandle {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    const optionsPtr = options
      ? this.memory.alloc(KERNEL_LAYOUT.SURFACE_TESSELLATION_OPTIONS_BYTES, 8)
      : 0;
    try {
      if (options && optionsPtr !== 0) {
        this.memory.writeSurfaceTessellationOptions(optionsPtr, options);
      }
      const status = this.api.rgm_surface_tessellate_to_mesh(
        this.handle,
        surfaceHandle,
        optionsPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface tessellation failed");
      return this.memory.readU64(outPtr) as MeshHandle;
    } finally {
      if (optionsPtr !== 0) {
        this.memory.free(optionsPtr, KERNEL_LAYOUT.SURFACE_TESSELLATION_OPTIONS_BYTES, 8);
      }
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  createFaceFromSurface(surfaceHandle: SurfaceHandle): FaceHandle {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_face_create_from_surface(this.handle, surfaceHandle, outPtr) as RgmStatus;
      this.assertOk(status, "Face creation failed");
      return this.memory.readU64(outPtr) as FaceHandle;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  faceAddLoop(faceHandle: FaceHandle, points: RgmUv2[], isOuter: boolean): void {
    this.ensureAlive();
    const pointsPtr = this.memory.alloc(points.length * KERNEL_LAYOUT.UV2_BYTES, 8);
    try {
      for (let idx = 0; idx < points.length; idx += 1) {
        this.memory.writeUv(pointsPtr + idx * KERNEL_LAYOUT.UV2_BYTES, points[idx]);
      }
      const status = this.api.rgm_face_add_loop(
        this.handle,
        faceHandle,
        pointsPtr,
        points.length,
        isOuter,
      ) as RgmStatus;
      this.assertOk(status, "Face add loop failed");
    } finally {
      this.memory.free(pointsPtr, points.length * KERNEL_LAYOUT.UV2_BYTES, 8);
    }
  }

  faceAddLoopEdges(faceHandle: FaceHandle, loopInput: RgmTrimLoopInput, edges: RgmTrimEdgeInput[]): void {
    this.ensureAlive();
    if (edges.length === 0) {
      throw new Error("Face edge loop requires at least one edge");
    }
    const loopInputPtr = this.memory.alloc(KERNEL_LAYOUT.TRIM_LOOP_INPUT_BYTES, 8);
    const edgesBytes = edges.length * KERNEL_LAYOUT.TRIM_EDGE_INPUT_BYTES;
    const edgesPtr = this.memory.alloc(edgesBytes, 8);
    try {
      this.memory.writeTrimLoopInput(loopInputPtr, loopInput);
      this.memory.writeTrimEdgeInputArray(edgesPtr, edges);
      const status = this.api.rgm_face_add_loop_edges(
        this.handle,
        faceHandle,
        loopInputPtr,
        edgesPtr,
        edges.length,
      ) as RgmStatus;
      this.assertOk(status, "Face add edge loop failed");
    } finally {
      this.memory.free(loopInputPtr, KERNEL_LAYOUT.TRIM_LOOP_INPUT_BYTES, 8);
      this.memory.free(edgesPtr, edgesBytes, 8);
    }
  }

  faceRemoveLoop(faceHandle: FaceHandle, loopIndex: number): void {
    this.ensureAlive();
    const status = this.api.rgm_face_remove_loop(this.handle, faceHandle, loopIndex) as RgmStatus;
    this.assertOk(status, "Face remove loop failed");
  }

  faceSplitTrimEdge(faceHandle: FaceHandle, loopIndex: number, edgeIndex: number, splitT: number): void {
    this.ensureAlive();
    const status = this.api.rgm_face_split_trim_edge(
      this.handle,
      faceHandle,
      loopIndex,
      edgeIndex,
      splitT,
    ) as RgmStatus;
    this.assertOk(status, "Face split trim edge failed");
  }

  faceReverseLoop(faceHandle: FaceHandle, loopIndex: number): void {
    this.ensureAlive();
    const status = this.api.rgm_face_reverse_loop(this.handle, faceHandle, loopIndex) as RgmStatus;
    this.assertOk(status, "Face reverse loop failed");
  }

  faceValidate(faceHandle: FaceHandle): boolean {
    this.ensureAlive();
    const validPtr = this.memory.alloc(1, 1);
    try {
      const status = this.api.rgm_face_validate(this.handle, faceHandle, validPtr) as RgmStatus;
      this.assertOk(status, "Face validation failed");
      return this.memory.readBool(validPtr);
    } finally {
      this.memory.free(validPtr, 1, 1);
    }
  }

  faceHeal(faceHandle: FaceHandle): void {
    this.ensureAlive();
    const status = this.api.rgm_face_heal(this.handle, faceHandle) as RgmStatus;
    this.assertOk(status, "Face heal failed");
  }

  faceTessellateToMesh(faceHandle: FaceHandle, options?: RgmSurfaceTessellationOptions): MeshHandle {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    const optionsPtr = options
      ? this.memory.alloc(KERNEL_LAYOUT.SURFACE_TESSELLATION_OPTIONS_BYTES, 8)
      : 0;
    try {
      if (options && optionsPtr !== 0) {
        this.memory.writeSurfaceTessellationOptions(optionsPtr, options);
      }
      const status = this.api.rgm_face_tessellate_to_mesh(
        this.handle,
        faceHandle,
        optionsPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Face tessellation failed");
      return this.memory.readU64(outPtr) as MeshHandle;
    } finally {
      if (optionsPtr !== 0) {
        this.memory.free(optionsPtr, KERNEL_LAYOUT.SURFACE_TESSELLATION_OPTIONS_BYTES, 8);
      }
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  brepCreateEmpty(): BrepHandle {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_brep_create_empty(this.handle, outPtr) as RgmStatus;
      this.assertOk(status, "BREP create empty failed");
      return this.memory.readU64(outPtr) as BrepHandle;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  brepCreateFromFaces(faces: FaceHandle[]): BrepHandle {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    const facesPtr =
      faces.length > 0 ? this.memory.alloc(faces.length * KERNEL_LAYOUT.U64_BYTES, 8) : 0;
    try {
      if (facesPtr !== 0) {
        this.memory.writeU64Array(facesPtr, faces);
      }
      const status = this.api.rgm_brep_create_from_faces(
        this.handle,
        facesPtr,
        faces.length,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "BREP create from faces failed");
      return this.memory.readU64(outPtr) as BrepHandle;
    } finally {
      if (facesPtr !== 0) {
        this.memory.free(facesPtr, faces.length * KERNEL_LAYOUT.U64_BYTES, 8);
      }
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  brepCreateFromSurface(surfaceHandle: SurfaceHandle): BrepHandle {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_brep_create_from_surface(
        this.handle,
        surfaceHandle,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "BREP create from surface failed");
      return this.memory.readU64(outPtr) as BrepHandle;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  brepAddFace(brepHandle: BrepHandle, faceHandle: FaceHandle): BrepFaceId {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const status = this.api.rgm_brep_add_face(
        this.handle,
        brepHandle,
        faceHandle,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "BREP add face failed");
      return this.memory.readU32(outPtr) as BrepFaceId;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  brepAddFaceFromSurface(brepHandle: BrepHandle, surfaceHandle: SurfaceHandle): BrepFaceId {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const status = this.api.rgm_brep_add_face_from_surface(
        this.handle,
        brepHandle,
        surfaceHandle,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "BREP add face from surface failed");
      return this.memory.readU32(outPtr) as BrepFaceId;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  brepAddLoopUv(
    brepHandle: BrepHandle,
    faceId: BrepFaceId,
    points: RgmUv2[],
    isOuter: boolean,
  ): BrepLoopId {
    this.ensureAlive();
    if (points.length === 0) {
      throw new Error("BREP loop requires at least one UV point");
    }
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    const pointsPtr = this.memory.alloc(points.length * KERNEL_LAYOUT.UV2_BYTES, 8);
    try {
      for (let idx = 0; idx < points.length; idx += 1) {
        this.memory.writeUv(pointsPtr + idx * KERNEL_LAYOUT.UV2_BYTES, points[idx]);
      }
      const status = this.api.rgm_brep_add_loop_uv(
        this.handle,
        brepHandle,
        faceId,
        pointsPtr,
        points.length,
        isOuter,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "BREP add loop failed");
      return this.memory.readU32(outPtr) as BrepLoopId;
    } finally {
      this.memory.free(pointsPtr, points.length * KERNEL_LAYOUT.UV2_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  brepFinalizeShell(brepHandle: BrepHandle): BrepShellId {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const status = this.api.rgm_brep_finalize_shell(this.handle, brepHandle, outPtr) as RgmStatus;
      this.assertOk(status, "BREP finalize shell failed");
      return this.memory.readU32(outPtr) as BrepShellId;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  brepFinalizeSolid(brepHandle: BrepHandle): BrepSolidId {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const status = this.api.rgm_brep_finalize_solid(this.handle, brepHandle, outPtr) as RgmStatus;
      this.assertOk(status, "BREP finalize solid failed");
      return this.memory.readU32(outPtr) as BrepSolidId;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  brepValidate(brepHandle: BrepHandle): RgmBrepValidationReport {
    this.ensureAlive();
    const reportPtr = this.memory.alloc(KERNEL_LAYOUT.BREP_VALIDATION_REPORT_BYTES, 8);
    try {
      const status = this.api.rgm_brep_validate(this.handle, brepHandle, reportPtr) as RgmStatus;
      this.assertOk(status, "BREP validate failed");
      return this.memory.readBrepValidationReport(reportPtr);
    } finally {
      this.memory.free(reportPtr, KERNEL_LAYOUT.BREP_VALIDATION_REPORT_BYTES, 8);
    }
  }

  brepHeal(brepHandle: BrepHandle): number {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const status = this.api.rgm_brep_heal(this.handle, brepHandle, outPtr) as RgmStatus;
      this.assertOk(status, "BREP heal failed");
      return this.memory.readU32(outPtr);
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  brepClone(brepHandle: BrepHandle): BrepHandle {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_brep_clone(this.handle, brepHandle, outPtr) as RgmStatus;
      this.assertOk(status, "BREP clone failed");
      return this.memory.readU64(outPtr) as BrepHandle;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  brepFaceCount(brepHandle: BrepHandle): number {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const status = this.api.rgm_brep_face_count(this.handle, brepHandle, outPtr) as RgmStatus;
      this.assertOk(status, "BREP face count failed");
      return this.memory.readU32(outPtr);
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  brepShellCount(brepHandle: BrepHandle): number {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const status = this.api.rgm_brep_shell_count(this.handle, brepHandle, outPtr) as RgmStatus;
      this.assertOk(status, "BREP shell count failed");
      return this.memory.readU32(outPtr);
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  brepSolidCount(brepHandle: BrepHandle): number {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const status = this.api.rgm_brep_solid_count(this.handle, brepHandle, outPtr) as RgmStatus;
      this.assertOk(status, "BREP solid count failed");
      return this.memory.readU32(outPtr);
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  brepIsSolid(brepHandle: BrepHandle): boolean {
    this.ensureAlive();
    const outPtr = this.memory.alloc(1, 1);
    try {
      const status = this.api.rgm_brep_is_solid(this.handle, brepHandle, outPtr) as RgmStatus;
      this.assertOk(status, "BREP solid query failed");
      return this.memory.readBool(outPtr);
    } finally {
      this.memory.free(outPtr, 1, 1);
    }
  }

  brepFaceAdjacency(brepHandle: BrepHandle, faceId: BrepFaceId): BrepFaceId[] {
    this.ensureAlive();
    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      let status = this.api.rgm_brep_face_adjacency(
        this.handle,
        brepHandle,
        faceId,
        0,
        0,
        countPtr,
      ) as RgmStatus;
      this.assertOk(status, "BREP face adjacency failed");
      const count = this.memory.readU32(countPtr);
      if (count === 0) return [];
      const idsPtr = this.memory.alloc(count * KERNEL_LAYOUT.I32_BYTES, 4);
      try {
        status = this.api.rgm_brep_face_adjacency(
          this.handle,
          brepHandle,
          faceId,
          idsPtr,
          count,
          countPtr,
        ) as RgmStatus;
        this.assertOk(status, "BREP face adjacency failed");
        const actual = this.memory.readU32(countPtr);
        const out: BrepFaceId[] = [];
        for (let idx = 0; idx < actual; idx += 1) {
          out.push(this.memory.readU32(idsPtr + idx * KERNEL_LAYOUT.I32_BYTES) as BrepFaceId);
        }
        return out;
      } finally {
        this.memory.free(idsPtr, count * KERNEL_LAYOUT.I32_BYTES, 4);
      }
    } finally {
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  brepTessellateToMesh(brepHandle: BrepHandle, options?: RgmSurfaceTessellationOptions): MeshHandle {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    const optionsPtr = options
      ? this.memory.alloc(KERNEL_LAYOUT.SURFACE_TESSELLATION_OPTIONS_BYTES, 8)
      : 0;
    try {
      if (options && optionsPtr !== 0) {
        this.memory.writeSurfaceTessellationOptions(optionsPtr, options);
      }
      const status = this.api.rgm_brep_tessellate_to_mesh(
        this.handle,
        brepHandle,
        optionsPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "BREP tessellation failed");
      return this.memory.readU64(outPtr) as MeshHandle;
    } finally {
      if (optionsPtr !== 0) {
        this.memory.free(optionsPtr, KERNEL_LAYOUT.SURFACE_TESSELLATION_OPTIONS_BYTES, 8);
      }
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  brepFromFaceObject(faceHandle: FaceHandle): BrepHandle {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_brep_from_face_object(this.handle, faceHandle, outPtr) as RgmStatus;
      this.assertOk(status, "BREP from face object failed");
      return this.memory.readU64(outPtr) as BrepHandle;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  brepExtractFaceObject(brepHandle: BrepHandle, faceId: BrepFaceId): FaceHandle {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_brep_extract_face_object(
        this.handle,
        brepHandle,
        faceId,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "BREP extract face object failed");
      return this.memory.readU64(outPtr) as FaceHandle;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  brepState(brepHandle: BrepHandle): 0 | 1 {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const status = this.api.rgm_brep_state(this.handle, brepHandle, outPtr) as RgmStatus;
      this.assertOk(status, "BREP state query failed");
      return this.memory.readU32(outPtr) === 0 ? 0 : 1;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  brepEstimateArea(brepHandle: BrepHandle): number {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.F64_BYTES, 8);
    try {
      const status = this.api.rgm_brep_estimate_area(this.handle, brepHandle, outPtr) as RgmStatus;
      this.assertOk(status, "BREP area estimate failed");
      return this.memory.readF64(outPtr);
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.F64_BYTES, 8);
    }
  }

  brepSaveNative(brepHandle: BrepHandle): Uint8Array {
    this.ensureAlive();
    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      let status = this.api.rgm_brep_save_native(
        this.handle,
        brepHandle,
        0,
        0,
        countPtr,
      ) as RgmStatus;
      this.assertOk(status, "BREP native save failed");
      const count = this.memory.readU32(countPtr);
      if (count === 0) {
        return new Uint8Array(0);
      }
      const bytesPtr = this.memory.alloc(count, 1);
      try {
        status = this.api.rgm_brep_save_native(
          this.handle,
          brepHandle,
          bytesPtr,
          count,
          countPtr,
        ) as RgmStatus;
        this.assertOk(status, "BREP native save failed");
        const actual = this.memory.readU32(countPtr);
        return new Uint8Array(this.memory.readBytes(bytesPtr, actual));
      } finally {
        this.memory.free(bytesPtr, count, 1);
      }
    } finally {
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  brepLoadNative(bytes: Uint8Array | ArrayBuffer | ArrayBufferView): BrepHandle {
    this.ensureAlive();
    const payload = this.normalizeBytes(bytes);
    const bytesPtr = payload.length > 0 ? this.memory.alloc(payload.length, 1) : 0;
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      if (payload.length > 0 && bytesPtr !== 0) {
        this.memory.writeBytes(bytesPtr, payload);
      }
      const status = this.api.rgm_brep_load_native(
        this.handle,
        bytesPtr,
        payload.length,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "BREP native load failed");
      return this.memory.readU64(outPtr) as BrepHandle;
    } finally {
      if (bytesPtr !== 0) {
        this.memory.free(bytesPtr, payload.length, 1);
      }
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  intersectSurfaceSurface(surfaceA: SurfaceHandle, surfaceB: SurfaceHandle): IntersectionHandle {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_intersect_surface_surface(
        this.handle,
        surfaceA,
        surfaceB,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Surface-surface intersection failed");
      return this.memory.readU64(outPtr) as IntersectionHandle;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  intersectSurfacePlane(surface: SurfaceHandle, plane: RgmPlane): IntersectionHandle {
    this.ensureAlive();
    const planePtr = this.memory.alloc(KERNEL_LAYOUT.PLANE_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writePlane(planePtr, plane);
      const status = this.api.rgm_intersect_surface_plane(this.handle, surface, planePtr, outPtr) as RgmStatus;
      this.assertOk(status, "Surface-plane intersection failed");
      return this.memory.readU64(outPtr) as IntersectionHandle;
    } finally {
      this.memory.free(planePtr, KERNEL_LAYOUT.PLANE_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  intersectSurfaceCurve(surface: SurfaceHandle, curve: CurveHandle): IntersectionHandle {
    this.ensureAlive();
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_intersect_surface_curve(this.handle, surface, curve, outPtr) as RgmStatus;
      this.assertOk(status, "Surface-curve intersection failed");
      return this.memory.readU64(outPtr) as IntersectionHandle;
    } finally {
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  intersectionBranchCount(intersection: IntersectionHandle): number {
    this.ensureAlive();
    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const status = this.api.rgm_intersection_branch_count(this.handle, intersection, countPtr) as RgmStatus;
      this.assertOk(status, "Intersection branch count failed");
      return this.memory.readU32(countPtr);
    } finally {
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  intersectionBranchSummary(intersection: IntersectionHandle, branchIndex: number): RgmIntersectionBranchSummary {
    this.ensureAlive();
    const ptr = this.memory.alloc(KERNEL_LAYOUT.INTERSECTION_BRANCH_SUMMARY_BYTES, 8);
    try {
      const status = this.api.rgm_intersection_branch_summary(
        this.handle,
        intersection,
        branchIndex,
        ptr,
      ) as RgmStatus;
      this.assertOk(status, "Intersection branch summary failed");
      return this.memory.readIntersectionBranchSummary(ptr);
    } finally {
      this.memory.free(ptr, KERNEL_LAYOUT.INTERSECTION_BRANCH_SUMMARY_BYTES, 8);
    }
  }

  intersectionBranchPoints(intersection: IntersectionHandle, branchIndex: number): RgmPoint3[] {
    this.ensureAlive();
    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      let status = this.api.rgm_intersection_copy_branch_points(
        this.handle,
        intersection,
        branchIndex,
        0,
        0,
        countPtr,
      ) as RgmStatus;
      this.assertOk(status, "Intersection points copy failed");
      const count = this.memory.readU32(countPtr);
      if (count === 0) return [];
      const pointsPtr = this.memory.alloc(count * KERNEL_LAYOUT.POINT3_BYTES, 8);
      try {
        status = this.api.rgm_intersection_copy_branch_points(
          this.handle,
          intersection,
          branchIndex,
          pointsPtr,
          count,
          countPtr,
        ) as RgmStatus;
        this.assertOk(status, "Intersection points copy failed");
        const actual = this.memory.readU32(countPtr);
        const points: RgmPoint3[] = [];
        for (let idx = 0; idx < actual; idx += 1) {
          points.push(this.memory.readPoint(pointsPtr + idx * KERNEL_LAYOUT.POINT3_BYTES));
        }
        return points;
      } finally {
        this.memory.free(pointsPtr, count * KERNEL_LAYOUT.POINT3_BYTES, 8);
      }
    } finally {
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  intersectionBranchUvA(intersection: IntersectionHandle, branchIndex: number): RgmUv2[] {
    return this.copyIntersectionUv(intersection, branchIndex, "a");
  }

  intersectionBranchUvB(intersection: bigint, branchIndex: number): RgmUv2[] {
    return this.copyIntersectionUv(intersection, branchIndex, "b");
  }

  intersectionBranchCurveT(intersection: IntersectionHandle, branchIndex: number): number[] {
    this.ensureAlive();
    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      let status = this.api.rgm_intersection_copy_branch_curve_t(
        this.handle,
        intersection,
        branchIndex,
        0,
        0,
        countPtr,
      ) as RgmStatus;
      this.assertOk(status, "Intersection curve-t copy failed");
      const count = this.memory.readU32(countPtr);
      if (count === 0) return [];
      const valuesPtr = this.memory.alloc(count * KERNEL_LAYOUT.F64_BYTES, 8);
      try {
        status = this.api.rgm_intersection_copy_branch_curve_t(
          this.handle,
          intersection,
          branchIndex,
          valuesPtr,
          count,
          countPtr,
        ) as RgmStatus;
        this.assertOk(status, "Intersection curve-t copy failed");
        const actual = this.memory.readU32(countPtr);
        const values: number[] = [];
        for (let idx = 0; idx < actual; idx += 1) {
          values.push(this.memory.readF64(valuesPtr + idx * KERNEL_LAYOUT.F64_BYTES));
        }
        return values;
      } finally {
        this.memory.free(valuesPtr, count * KERNEL_LAYOUT.F64_BYTES, 8);
      }
    } finally {
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  intersectionBranchToNurbs(
    intersection: IntersectionHandle,
    branchIndex: number,
    tolerance: RgmToleranceContext,
  ): CurveHandle {
    this.ensureAlive();
    const tolPtr = this.memory.alloc(KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
    const outPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      this.memory.writeTolerance(tolPtr, tolerance);
      const status = this.api.rgm_intersection_branch_to_nurbs(
        this.handle,
        intersection,
        branchIndex,
        tolPtr,
        outPtr,
      ) as RgmStatus;
      this.assertOk(status, "Intersection branch to nurbs failed");
      return this.memory.readU64(outPtr) as CurveHandle;
    } finally {
      this.memory.free(tolPtr, KERNEL_LAYOUT.TOLERANCE_BYTES, 8);
      this.memory.free(outPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  releaseObject(objectHandle: ObjectHandle): void {
    this.ensureAlive();
    const status = this.api.rgm_object_release(this.handle, objectHandle) as RgmStatus;
    if (status !== RgmStatus.Ok && status !== RgmStatus.NotFound) {
      this.assertOk(status, "Object release failed");
    }
  }

  lastError(): { code: number; message: string } {
    this.ensureAlive();

    const codePtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    const bufferLen = 2048;
    const messagePtr = this.memory.alloc(bufferLen, 1);
    const writtenPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);

    try {
      const statusCode = this.api.rgm_last_error_code(this.handle, codePtr) as RgmStatus;
      const statusMessage = this.api.rgm_last_error_message(
        this.handle,
        messagePtr,
        bufferLen,
        writtenPtr,
      ) as RgmStatus;

      if (statusCode !== RgmStatus.Ok || statusMessage !== RgmStatus.Ok) {
        return {
          code: -1,
          message: "Unable to retrieve kernel error",
        };
      }

      const code = this.memory.readI32(codePtr);
      const written = this.memory.readU32(writtenPtr);
      const bytes = this.memory.readBytes(messagePtr, written);
      return {
        code,
        message: this.decoder.decode(bytes),
      };
    } finally {
      this.memory.free(codePtr, KERNEL_LAYOUT.I32_BYTES, 4);
      this.memory.free(messagePtr, bufferLen, 1);
      this.memory.free(writtenPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  destroy(): void {
    if (this.destroyed) {
      return;
    }

    const status = this.api.rgm_kernel_destroy(this.handle) as RgmStatus;
    if (status !== RgmStatus.Ok && status !== RgmStatus.NotFound) {
      this.assertOk(status, "Kernel session destroy failed");
    }

    this.destroyed = true;
    this.onDestroy();
  }

  private copyIntersectionUv(
    intersection: bigint,
    branchIndex: number,
    kind: "a" | "b",
  ): RgmUv2[] {
    this.ensureAlive();
    const countPtr = this.memory.alloc(KERNEL_LAYOUT.I32_BYTES, 4);
    try {
      const fn =
        kind === "a"
          ? this.api.rgm_intersection_copy_branch_uv_on_surface_a
          : this.api.rgm_intersection_copy_branch_uv_on_surface_b;
      let status = fn(this.handle, intersection, branchIndex, 0, 0, countPtr) as RgmStatus;
      this.assertOk(status, "Intersection uv copy failed");
      const count = this.memory.readU32(countPtr);
      if (count === 0) return [];
      const valuesPtr = this.memory.alloc(count * KERNEL_LAYOUT.UV2_BYTES, 8);
      try {
        status = fn(this.handle, intersection, branchIndex, valuesPtr, count, countPtr) as RgmStatus;
        this.assertOk(status, "Intersection uv copy failed");
        const actual = this.memory.readU32(countPtr);
        const points: RgmUv2[] = [];
        for (let idx = 0; idx < actual; idx += 1) {
          points.push(this.memory.readUv(valuesPtr + idx * KERNEL_LAYOUT.UV2_BYTES));
        }
        return points;
      } finally {
        this.memory.free(valuesPtr, count * KERNEL_LAYOUT.UV2_BYTES, 8);
      }
    } finally {
      this.memory.free(countPtr, KERNEL_LAYOUT.I32_BYTES, 4);
    }
  }

  private normalizeBytes(bytes: Uint8Array | ArrayBuffer | ArrayBufferView): Uint8Array {
    if (bytes instanceof Uint8Array) {
      return bytes;
    }
    if (bytes instanceof ArrayBuffer) {
      return new Uint8Array(bytes);
    }
    return new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  }

}

class KernelRuntimeImpl implements KernelRuntime {
  readonly capabilities: KernelCapabilities = {
    igesImport: false,
    igesExport: false,
  };

  private readonly sessions = new Set<KernelSessionImpl>();

  constructor(
    private readonly api: NativeExports,
    private readonly memory: KernelMemory,
  ) {}

  createSession(): KernelSession {
    const outSessionPtr = this.memory.alloc(KERNEL_LAYOUT.U64_BYTES, 8);
    try {
      const status = this.api.rgm_kernel_create(outSessionPtr) as RgmStatus;
      if (status !== RgmStatus.Ok) {
        throw new KernelRuntimeError(
          `Kernel session create failed (${statusToName(status)})`,
          status,
        );
      }

      const handle = this.memory.readU64(outSessionPtr);
      const session = new KernelSessionImpl(this.api, this.memory, handle, () => {
        this.sessions.delete(session);
      });
      this.sessions.add(session);
      return session;
    } finally {
      this.memory.free(outSessionPtr, KERNEL_LAYOUT.U64_BYTES, 8);
    }
  }

  destroy(): void {
    for (const session of [...this.sessions]) {
      session.destroy();
    }
  }
}

export async function createKernelRuntime(wasmSource: WasmSource): Promise<KernelRuntime> {
  const wasm = await loadKernelWasm(wasmSource);
  const exports = wasm.exports as unknown as NativeExports;
  const api = exports;
  const memory = new KernelMemory(api, wasm.exports.memory);
  return new KernelRuntimeImpl(api, memory);
}
