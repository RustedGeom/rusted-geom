import type { KernelSession as LegacyKernelSession } from "./core";

export interface FaceClient {
  createFaceFromSurface: LegacyKernelSession["createFaceFromSurface"];
  faceAddLoop: LegacyKernelSession["faceAddLoop"];
  faceAddLoopEdges: LegacyKernelSession["faceAddLoopEdges"];
  faceRemoveLoop: LegacyKernelSession["faceRemoveLoop"];
  faceSplitTrimEdge: LegacyKernelSession["faceSplitTrimEdge"];
  faceReverseLoop: LegacyKernelSession["faceReverseLoop"];
  faceValidate: LegacyKernelSession["faceValidate"];
  faceHeal: LegacyKernelSession["faceHeal"];
  faceTessellateToMesh: LegacyKernelSession["faceTessellateToMesh"];
}

export class FaceClientImpl implements FaceClient {
  constructor(private readonly session: LegacyKernelSession) {}

  createFaceFromSurface: FaceClient["createFaceFromSurface"] = (surfaceHandle) =>
    this.session.createFaceFromSurface(surfaceHandle);

  faceAddLoop: FaceClient["faceAddLoop"] = (faceHandle, points, isOuter) =>
    this.session.faceAddLoop(faceHandle, points, isOuter);

  faceAddLoopEdges: FaceClient["faceAddLoopEdges"] = (faceHandle, loopInput, edges) =>
    this.session.faceAddLoopEdges(faceHandle, loopInput, edges);

  faceRemoveLoop: FaceClient["faceRemoveLoop"] = (faceHandle, loopIndex) =>
    this.session.faceRemoveLoop(faceHandle, loopIndex);

  faceSplitTrimEdge: FaceClient["faceSplitTrimEdge"] = (faceHandle, loopIndex, edgeIndex, splitT) =>
    this.session.faceSplitTrimEdge(faceHandle, loopIndex, edgeIndex, splitT);

  faceReverseLoop: FaceClient["faceReverseLoop"] = (faceHandle, loopIndex) =>
    this.session.faceReverseLoop(faceHandle, loopIndex);

  faceValidate: FaceClient["faceValidate"] = (faceHandle) => this.session.faceValidate(faceHandle);

  faceHeal: FaceClient["faceHeal"] = (faceHandle) => this.session.faceHeal(faceHandle);

  faceTessellateToMesh: FaceClient["faceTessellateToMesh"] = (faceHandle, options) =>
    this.session.faceTessellateToMesh(faceHandle, options);
}
