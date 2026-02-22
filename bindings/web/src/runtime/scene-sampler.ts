import type { RustedGeomApi } from "../generated/native";
import type { RgmPoint3 } from "../generated/types";
import { RgmStatus } from "../generated/types";
import { KernelRuntimeError, statusToName } from "./errors";
import { KERNEL_LAYOUT, type KernelMemory } from "./memory";

export interface CurveSampleContext {
  api: RustedGeomApi;
  memory: KernelMemory;
  session: bigint;
  getLastErrorMessage: () => string;
}

export function sampleCurvePolyline(
  context: CurveSampleContext,
  curve: bigint,
  sampleCount: number,
): RgmPoint3[] {
  const count = Math.max(2, Math.floor(sampleCount));
  const points: RgmPoint3[] = [];
  const pointPtr = context.memory.alloc(KERNEL_LAYOUT.POINT3_BYTES, 8);

  try {
    for (let idx = 0; idx < count; idx += 1) {
      const tNorm = count === 1 ? 0 : idx / (count - 1);
      const status = context.api.curvePointAt(
        context.session,
        curve,
        tNorm,
        pointPtr,
      ) as RgmStatus;
      if (status !== RgmStatus.Ok) {
        const details = context.getLastErrorMessage();
        throw new KernelRuntimeError(
          `Curve sampling failed (${statusToName(status)})`,
          status,
          details,
        );
      }

      points.push(context.memory.readPoint(pointPtr));
    }
  } finally {
    context.memory.free(pointPtr, KERNEL_LAYOUT.POINT3_BYTES, 8);
  }

  return points;
}
