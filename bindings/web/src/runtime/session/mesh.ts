import type { KernelSession as LegacyKernelSession } from "./core";
import type { RgmBounds3, RgmBoundsOptions } from "../../generated/types";
import type { MeshHandle } from "./handles";

export interface MeshClient {
  createMeshBox: LegacyKernelSession["createMeshBox"];
  createMeshUvSphere: LegacyKernelSession["createMeshUvSphere"];
  createMeshTorus: LegacyKernelSession["createMeshTorus"];
  meshTranslate: LegacyKernelSession["meshTranslate"];
  meshRotate: LegacyKernelSession["meshRotate"];
  meshScale: LegacyKernelSession["meshScale"];
  meshBakeTransform: LegacyKernelSession["meshBakeTransform"];
  meshBoolean: LegacyKernelSession["meshBoolean"];
  meshVertexCount: LegacyKernelSession["meshVertexCount"];
  meshTriangleCount: LegacyKernelSession["meshTriangleCount"];
  meshToBuffers: LegacyKernelSession["meshToBuffers"];
  bounds: (meshHandle: MeshHandle, options?: RgmBoundsOptions) => RgmBounds3;
}

export class MeshClientImpl implements MeshClient {
  constructor(private readonly session: LegacyKernelSession) {}

  createMeshBox: MeshClient["createMeshBox"] = (center, size) =>
    this.session.createMeshBox(center, size);

  createMeshUvSphere: MeshClient["createMeshUvSphere"] = (center, radius, uSteps, vSteps) =>
    this.session.createMeshUvSphere(center, radius, uSteps, vSteps);

  createMeshTorus: MeshClient["createMeshTorus"] = (
    center,
    majorRadius,
    minorRadius,
    majorSteps,
    minorSteps,
  ) => this.session.createMeshTorus(center, majorRadius, minorRadius, majorSteps, minorSteps);

  meshTranslate: MeshClient["meshTranslate"] = (meshHandle, delta) =>
    this.session.meshTranslate(meshHandle, delta);

  meshRotate: MeshClient["meshRotate"] = (meshHandle, axis, angleRad, pivot) =>
    this.session.meshRotate(meshHandle, axis, angleRad, pivot);

  meshScale: MeshClient["meshScale"] = (meshHandle, scale, pivot) =>
    this.session.meshScale(meshHandle, scale, pivot);

  meshBakeTransform: MeshClient["meshBakeTransform"] = (meshHandle) =>
    this.session.meshBakeTransform(meshHandle);

  meshBoolean: MeshClient["meshBoolean"] = (meshA, meshB, op) =>
    this.session.meshBoolean(meshA, meshB, op);

  meshVertexCount: MeshClient["meshVertexCount"] = (meshHandle) =>
    this.session.meshVertexCount(meshHandle);

  meshTriangleCount: MeshClient["meshTriangleCount"] = (meshHandle) =>
    this.session.meshTriangleCount(meshHandle);

  meshToBuffers: MeshClient["meshToBuffers"] = (meshHandle) => this.session.meshToBuffers(meshHandle);

  bounds: MeshClient["bounds"] = (meshHandle, options) =>
    this.session.objectComputeBounds(meshHandle, options);
}
