import type { KernelSession as LegacyKernelSession } from "./core";
import type { RgmBounds3, RgmBoundsOptions } from "../../generated/types";
import type { SurfaceHandle } from "./handles";

export interface SurfaceClient {
  createNurbsSurface: LegacyKernelSession["createNurbsSurface"];
  surfacePointAt: LegacyKernelSession["surfacePointAt"];
  surfaceD1At: LegacyKernelSession["surfaceD1At"];
  surfaceD2At: LegacyKernelSession["surfaceD2At"];
  surfaceFrameAt: LegacyKernelSession["surfaceFrameAt"];
  surfaceTranslate: LegacyKernelSession["surfaceTranslate"];
  surfaceRotate: LegacyKernelSession["surfaceRotate"];
  surfaceScale: LegacyKernelSession["surfaceScale"];
  surfaceBakeTransform: LegacyKernelSession["surfaceBakeTransform"];
  surfaceTessellateToMesh: LegacyKernelSession["surfaceTessellateToMesh"];
  bounds: (surfaceHandle: SurfaceHandle, options?: RgmBoundsOptions) => RgmBounds3;
}

export class SurfaceClientImpl implements SurfaceClient {
  constructor(private readonly session: LegacyKernelSession) {}

  createNurbsSurface: SurfaceClient["createNurbsSurface"] = (
    desc,
    controlPoints,
    weights,
    knotsU,
    knotsV,
    tolerance,
  ) => this.session.createNurbsSurface(desc, controlPoints, weights, knotsU, knotsV, tolerance);

  surfacePointAt: SurfaceClient["surfacePointAt"] = (surfaceHandle, uvNorm) =>
    this.session.surfacePointAt(surfaceHandle, uvNorm);

  surfaceD1At: SurfaceClient["surfaceD1At"] = (surfaceHandle, uvNorm) =>
    this.session.surfaceD1At(surfaceHandle, uvNorm);

  surfaceD2At: SurfaceClient["surfaceD2At"] = (surfaceHandle, uvNorm) =>
    this.session.surfaceD2At(surfaceHandle, uvNorm);

  surfaceFrameAt: SurfaceClient["surfaceFrameAt"] = (surfaceHandle, uvNorm) =>
    this.session.surfaceFrameAt(surfaceHandle, uvNorm);

  surfaceTranslate: SurfaceClient["surfaceTranslate"] = (surfaceHandle, delta) =>
    this.session.surfaceTranslate(surfaceHandle, delta);

  surfaceRotate: SurfaceClient["surfaceRotate"] = (surfaceHandle, axis, angleRad, pivot) =>
    this.session.surfaceRotate(surfaceHandle, axis, angleRad, pivot);

  surfaceScale: SurfaceClient["surfaceScale"] = (surfaceHandle, scale, pivot) =>
    this.session.surfaceScale(surfaceHandle, scale, pivot);

  surfaceBakeTransform: SurfaceClient["surfaceBakeTransform"] = (surfaceHandle) =>
    this.session.surfaceBakeTransform(surfaceHandle);

  surfaceTessellateToMesh: SurfaceClient["surfaceTessellateToMesh"] = (surfaceHandle, options) =>
    this.session.surfaceTessellateToMesh(surfaceHandle, options);

  bounds: SurfaceClient["bounds"] = (surfaceHandle, options) =>
    this.session.objectComputeBounds(surfaceHandle, options);
}
