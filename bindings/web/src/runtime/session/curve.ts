import type { KernelSession as LegacyKernelSession } from "./core";

export interface CurveClient {
  buildCurveFromPreset: LegacyKernelSession["buildCurveFromPreset"];
  createLine: LegacyKernelSession["createLine"];
  createArc: LegacyKernelSession["createArc"];
  createCircle: LegacyKernelSession["createCircle"];
  createPolyline: LegacyKernelSession["createPolyline"];
  createPolycurve: LegacyKernelSession["createPolycurve"];
  sampleCurvePolyline: LegacyKernelSession["sampleCurvePolyline"];
  pointAt: LegacyKernelSession["pointAt"];
  curveLength: LegacyKernelSession["curveLength"];
  curveLengthAt: LegacyKernelSession["curveLengthAt"];
}

export class CurveClientImpl implements CurveClient {
  constructor(private readonly session: LegacyKernelSession) {}

  buildCurveFromPreset: CurveClient["buildCurveFromPreset"] = (preset) =>
    this.session.buildCurveFromPreset(preset);

  createLine: CurveClient["createLine"] = (line, tolerance) =>
    this.session.createLine(line, tolerance);

  createArc: CurveClient["createArc"] = (arc, tolerance) =>
    this.session.createArc(arc, tolerance);

  createCircle: CurveClient["createCircle"] = (circle, tolerance) =>
    this.session.createCircle(circle, tolerance);

  createPolyline: CurveClient["createPolyline"] = (points, closed, tolerance) =>
    this.session.createPolyline(points, closed, tolerance);

  createPolycurve: CurveClient["createPolycurve"] = (segments, tolerance) =>
    this.session.createPolycurve(segments, tolerance);

  sampleCurvePolyline: CurveClient["sampleCurvePolyline"] = (curveHandle, sampleCount) =>
    this.session.sampleCurvePolyline(curveHandle, sampleCount);

  pointAt: CurveClient["pointAt"] = (curveHandle, tNorm) =>
    this.session.pointAt(curveHandle, tNorm);

  curveLength: CurveClient["curveLength"] = (curveHandle) => this.session.curveLength(curveHandle);

  curveLengthAt: CurveClient["curveLengthAt"] = (curveHandle, tNorm) =>
    this.session.curveLengthAt(curveHandle, tNorm);
}
