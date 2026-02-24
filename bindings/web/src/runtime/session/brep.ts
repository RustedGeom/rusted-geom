import type { KernelSession as LegacyKernelSession } from "./core";
import type { RgmBounds3, RgmBoundsOptions } from "../../generated/types";
import type { BrepHandle } from "./handles";

export interface BrepClient {
  brepCreateEmpty: LegacyKernelSession["brepCreateEmpty"];
  brepCreateFromFaces: LegacyKernelSession["brepCreateFromFaces"];
  brepCreateFromSurface: LegacyKernelSession["brepCreateFromSurface"];
  brepAddFace: LegacyKernelSession["brepAddFace"];
  brepAddFaceFromSurface: LegacyKernelSession["brepAddFaceFromSurface"];
  brepAddLoopUv: LegacyKernelSession["brepAddLoopUv"];
  brepFinalizeShell: LegacyKernelSession["brepFinalizeShell"];
  brepFinalizeSolid: LegacyKernelSession["brepFinalizeSolid"];
  brepValidate: LegacyKernelSession["brepValidate"];
  brepHeal: LegacyKernelSession["brepHeal"];
  brepClone: LegacyKernelSession["brepClone"];
  brepFaceCount: LegacyKernelSession["brepFaceCount"];
  brepShellCount: LegacyKernelSession["brepShellCount"];
  brepSolidCount: LegacyKernelSession["brepSolidCount"];
  brepIsSolid: LegacyKernelSession["brepIsSolid"];
  brepFaceAdjacency: LegacyKernelSession["brepFaceAdjacency"];
  brepTessellateToMesh: LegacyKernelSession["brepTessellateToMesh"];
  brepFromFaceObject: LegacyKernelSession["brepFromFaceObject"];
  brepExtractFaceObject: LegacyKernelSession["brepExtractFaceObject"];
  brepState: LegacyKernelSession["brepState"];
  brepEstimateArea: LegacyKernelSession["brepEstimateArea"];
  brepSaveNative: LegacyKernelSession["brepSaveNative"];
  brepLoadNative: LegacyKernelSession["brepLoadNative"];
  bounds: (brepHandle: BrepHandle, options?: RgmBoundsOptions) => RgmBounds3;
}

export class BrepClientImpl implements BrepClient {
  constructor(private readonly session: LegacyKernelSession) {}

  brepCreateEmpty: BrepClient["brepCreateEmpty"] = () => this.session.brepCreateEmpty();

  brepCreateFromFaces: BrepClient["brepCreateFromFaces"] = (faces) =>
    this.session.brepCreateFromFaces(faces);

  brepCreateFromSurface: BrepClient["brepCreateFromSurface"] = (surfaceHandle) =>
    this.session.brepCreateFromSurface(surfaceHandle);

  brepAddFace: BrepClient["brepAddFace"] = (brepHandle, faceHandle) =>
    this.session.brepAddFace(brepHandle, faceHandle);

  brepAddFaceFromSurface: BrepClient["brepAddFaceFromSurface"] = (brepHandle, surfaceHandle) =>
    this.session.brepAddFaceFromSurface(brepHandle, surfaceHandle);

  brepAddLoopUv: BrepClient["brepAddLoopUv"] = (brepHandle, faceId, points, isOuter) =>
    this.session.brepAddLoopUv(brepHandle, faceId, points, isOuter);

  brepFinalizeShell: BrepClient["brepFinalizeShell"] = (brepHandle) =>
    this.session.brepFinalizeShell(brepHandle);

  brepFinalizeSolid: BrepClient["brepFinalizeSolid"] = (brepHandle) =>
    this.session.brepFinalizeSolid(brepHandle);

  brepValidate: BrepClient["brepValidate"] = (brepHandle) => this.session.brepValidate(brepHandle);

  brepHeal: BrepClient["brepHeal"] = (brepHandle) => this.session.brepHeal(brepHandle);

  brepClone: BrepClient["brepClone"] = (brepHandle) => this.session.brepClone(brepHandle);

  brepFaceCount: BrepClient["brepFaceCount"] = (brepHandle) => this.session.brepFaceCount(brepHandle);

  brepShellCount: BrepClient["brepShellCount"] = (brepHandle) =>
    this.session.brepShellCount(brepHandle);

  brepSolidCount: BrepClient["brepSolidCount"] = (brepHandle) =>
    this.session.brepSolidCount(brepHandle);

  brepIsSolid: BrepClient["brepIsSolid"] = (brepHandle) => this.session.brepIsSolid(brepHandle);

  brepFaceAdjacency: BrepClient["brepFaceAdjacency"] = (brepHandle, faceId) =>
    this.session.brepFaceAdjacency(brepHandle, faceId);

  brepTessellateToMesh: BrepClient["brepTessellateToMesh"] = (brepHandle, options) =>
    this.session.brepTessellateToMesh(brepHandle, options);

  brepFromFaceObject: BrepClient["brepFromFaceObject"] = (faceHandle) =>
    this.session.brepFromFaceObject(faceHandle);

  brepExtractFaceObject: BrepClient["brepExtractFaceObject"] = (brepHandle, faceId) =>
    this.session.brepExtractFaceObject(brepHandle, faceId);

  brepState: BrepClient["brepState"] = (brepHandle) => this.session.brepState(brepHandle);

  brepEstimateArea: BrepClient["brepEstimateArea"] = (brepHandle) =>
    this.session.brepEstimateArea(brepHandle);

  brepSaveNative: BrepClient["brepSaveNative"] = (brepHandle) => this.session.brepSaveNative(brepHandle);

  brepLoadNative: BrepClient["brepLoadNative"] = (bytes) => this.session.brepLoadNative(bytes);

  bounds: BrepClient["bounds"] = (brepHandle, options) =>
    this.session.objectComputeBounds(brepHandle, options);
}
