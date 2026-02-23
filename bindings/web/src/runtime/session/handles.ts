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

/** Union of all kernel object handle types. Used for releaseObject. */
export type ObjectHandle = CurveHandle | SurfaceHandle | MeshHandle | FaceHandle | IntersectionHandle;
