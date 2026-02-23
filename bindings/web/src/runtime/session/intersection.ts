import type { KernelSession as LegacyKernelSession } from "./core";

export interface IntersectionClient {
  intersectCurvePlane: LegacyKernelSession["intersectCurvePlane"];
  intersectCurveCurve: LegacyKernelSession["intersectCurveCurve"];
  intersectMeshPlane: LegacyKernelSession["intersectMeshPlane"];
  intersectMeshMesh: LegacyKernelSession["intersectMeshMesh"];
  intersectSurfaceSurface: LegacyKernelSession["intersectSurfaceSurface"];
  intersectSurfacePlane: LegacyKernelSession["intersectSurfacePlane"];
  intersectSurfaceCurve: LegacyKernelSession["intersectSurfaceCurve"];
  intersectionBranchCount: LegacyKernelSession["intersectionBranchCount"];
  intersectionBranchSummary: LegacyKernelSession["intersectionBranchSummary"];
  intersectionBranchPoints: LegacyKernelSession["intersectionBranchPoints"];
  intersectionBranchUvA: LegacyKernelSession["intersectionBranchUvA"];
  intersectionBranchUvB: LegacyKernelSession["intersectionBranchUvB"];
  intersectionBranchCurveT: LegacyKernelSession["intersectionBranchCurveT"];
  intersectionBranchToNurbs: LegacyKernelSession["intersectionBranchToNurbs"];
}

export class IntersectionClientImpl implements IntersectionClient {
  constructor(private readonly session: LegacyKernelSession) {}

  intersectCurvePlane: IntersectionClient["intersectCurvePlane"] = (curveHandle, plane) =>
    this.session.intersectCurvePlane(curveHandle, plane);

  intersectCurveCurve: IntersectionClient["intersectCurveCurve"] = (curveA, curveB) =>
    this.session.intersectCurveCurve(curveA, curveB);

  intersectMeshPlane: IntersectionClient["intersectMeshPlane"] = (meshHandle, plane) =>
    this.session.intersectMeshPlane(meshHandle, plane);

  intersectMeshMesh: IntersectionClient["intersectMeshMesh"] = (meshA, meshB) =>
    this.session.intersectMeshMesh(meshA, meshB);

  intersectSurfaceSurface: IntersectionClient["intersectSurfaceSurface"] = (surfaceA, surfaceB) =>
    this.session.intersectSurfaceSurface(surfaceA, surfaceB);

  intersectSurfacePlane: IntersectionClient["intersectSurfacePlane"] = (surface, plane) =>
    this.session.intersectSurfacePlane(surface, plane);

  intersectSurfaceCurve: IntersectionClient["intersectSurfaceCurve"] = (surface, curve) =>
    this.session.intersectSurfaceCurve(surface, curve);

  intersectionBranchCount: IntersectionClient["intersectionBranchCount"] = (intersection) =>
    this.session.intersectionBranchCount(intersection);

  intersectionBranchSummary: IntersectionClient["intersectionBranchSummary"] = (
    intersection,
    branchIndex,
  ) => this.session.intersectionBranchSummary(intersection, branchIndex);

  intersectionBranchPoints: IntersectionClient["intersectionBranchPoints"] = (
    intersection,
    branchIndex,
  ) => this.session.intersectionBranchPoints(intersection, branchIndex);

  intersectionBranchUvA: IntersectionClient["intersectionBranchUvA"] = (intersection, branchIndex) =>
    this.session.intersectionBranchUvA(intersection, branchIndex);

  intersectionBranchUvB: IntersectionClient["intersectionBranchUvB"] = (intersection, branchIndex) =>
    this.session.intersectionBranchUvB(intersection, branchIndex);

  intersectionBranchCurveT: IntersectionClient["intersectionBranchCurveT"] = (
    intersection,
    branchIndex,
  ) => this.session.intersectionBranchCurveT(intersection, branchIndex);

  intersectionBranchToNurbs: IntersectionClient["intersectionBranchToNurbs"] = (
    intersection,
    branchIndex,
    tolerance,
  ) => this.session.intersectionBranchToNurbs(intersection, branchIndex, tolerance);
}
