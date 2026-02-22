import { RgmStatus } from "../generated/types";

export class KernelRuntimeError extends Error {
  constructor(
    message: string,
    public readonly status: RgmStatus,
    public readonly details?: string,
  ) {
    super(details ? `${message}: ${details}` : message);
    this.name = "KernelRuntimeError";
  }
}

export function statusToName(status: RgmStatus): string {
  return RgmStatus[status] ?? `Unknown(${status})`;
}
