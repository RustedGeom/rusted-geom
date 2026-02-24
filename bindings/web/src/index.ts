export * from "./generated/native";
export * from "./generated/types";
export * from "./runtime/errors";
export * from "./runtime/kernel-session";
export * from "./runtime/memory";
export * from "./runtime/scene-sampler";
export * from "./runtime/wasm-loader";
// Explicit re-export resolves the CurveHandle ambiguity between generated/native (class)
// and session/handles (branded bigint type). The type-only export wins.
export type {
  BrepEdgeId,
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
} from "./runtime/session/handles";
