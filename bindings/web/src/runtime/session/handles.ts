// Nominal branded handle types for the kernel session API.
//
// Each domain has its own opaque brand so TypeScript prevents accidentally
// passing a CurveHandle to a method that expects a SurfaceHandle, etc.
// There is zero runtime overhead — brands exist only in the type system.

declare const _curveBrand: unique symbol;
/** Opaque handle to a curve object in the kernel session. */
export type CurveHandle = bigint & { readonly [_curveBrand]: void };

declare const _surfaceBrand: unique symbol;
/** Opaque handle to a surface object in the kernel session. */
export type SurfaceHandle = bigint & { readonly [_surfaceBrand]: void };

declare const _meshBrand: unique symbol;
/** Opaque handle to a mesh object in the kernel session. */
export type MeshHandle = bigint & { readonly [_meshBrand]: void };

declare const _faceBrand: unique symbol;
/** Opaque handle to a trimmed face object in the kernel session. */
export type FaceHandle = bigint & { readonly [_faceBrand]: void };

declare const _intersectionBrand: unique symbol;
/** Opaque handle to an intersection result in the kernel session. */
export type IntersectionHandle = bigint & { readonly [_intersectionBrand]: void };

declare const _brepBrand: unique symbol;
/** Opaque handle to a BREP object in the kernel session. */
export type BrepHandle = bigint & { readonly [_brepBrand]: void };

declare const _brepFaceIdBrand: unique symbol;
export type BrepFaceId = number & { readonly [_brepFaceIdBrand]: void };

declare const _brepEdgeIdBrand: unique symbol;
export type BrepEdgeId = number & { readonly [_brepEdgeIdBrand]: void };

declare const _brepLoopIdBrand: unique symbol;
export type BrepLoopId = number & { readonly [_brepLoopIdBrand]: void };

declare const _brepShellIdBrand: unique symbol;
export type BrepShellId = number & { readonly [_brepShellIdBrand]: void };

declare const _brepSolidIdBrand: unique symbol;
export type BrepSolidId = number & { readonly [_brepSolidIdBrand]: void };

/** Union of all kernel object handle types. Used for releaseObject. */
export type ObjectHandle =
  | CurveHandle
  | SurfaceHandle
  | MeshHandle
  | FaceHandle
  | IntersectionHandle
  | BrepHandle;
