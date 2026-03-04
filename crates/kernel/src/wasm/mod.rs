//! wasm-bindgen public API for rusted-geom.
//!
//! This module replaces the C ABI (`extern "C"` / `#[no_mangle]`) layer with
//! typed, GC-safe `#[wasm_bindgen]` structs.  Each `KernelSession` owns its
//! underlying session; `Drop` destroys it automatically.  Each handle struct
//! (`CurveHandle`, `SurfaceHandle`, …) owns a reference-counted object slot
//! and releases it on `Drop` (or when JS GC collects it via FinalizationRegistry).

mod bounds;
mod brep;
mod curve;
mod error;
mod face;
mod intersection;
mod io;
mod landxml;
mod mesh;
mod surface;

pub use bounds::Bounds3;
pub use brep::BrepValidationResult;
pub use intersection::BranchSummary;
pub use surface::SurfaceEvalResult;

use crate::{
    rgm_kernel_create, rgm_kernel_destroy, rgm_object_release, RgmKernelHandle, RgmObjectHandle,
    RgmToleranceContext,
};
use wasm_bindgen::prelude::*;

// ─── Session ──────────────────────────────────────────────────────────────────

/// A geometry session.  Create one per scene or worker thread.
///
/// All objects created within a session share the same tolerance context and
/// lifetime.  The session (and all its objects) is destroyed when `.free()` is
/// called or when the JS GC finalises it.
#[wasm_bindgen]
pub struct KernelSession {
    pub(crate) session_id: u64,
    pub(crate) abs_tol: f64,
    pub(crate) rel_tol: f64,
    pub(crate) angle_tol: f64,
}

#[wasm_bindgen]
impl KernelSession {
    /// Create a new kernel session with default tolerances (abs=1e-6, rel=1e-4, angle=1e-6 rad).
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<KernelSession, JsValue> {
        let mut handle = RgmKernelHandle(0);
        let status = rgm_kernel_create(&mut handle as *mut _);
        error::check(status)?;
        Ok(KernelSession {
            session_id: handle.0,
            abs_tol: 1e-6,
            rel_tol: 1e-4,
            angle_tol: 1e-6,
        })
    }

    /// Absolute distance tolerance (linear units matching world coordinates).
    pub fn abs_tol(&self) -> f64 {
        self.abs_tol
    }
    /// Relative distance tolerance (fraction of characteristic geometry length).
    pub fn rel_tol(&self) -> f64 {
        self.rel_tol
    }
    /// Angular tolerance in radians.
    pub fn angle_tol(&self) -> f64 {
        self.angle_tol
    }

    /// Set the absolute distance tolerance.
    pub fn set_abs_tol(&mut self, v: f64) {
        self.abs_tol = v;
    }
    /// Set the relative distance tolerance.
    pub fn set_rel_tol(&mut self, v: f64) {
        self.rel_tol = v;
    }
    /// Set the angular tolerance.
    pub fn set_angle_tol(&mut self, v: f64) {
        self.angle_tol = v;
    }

    /// Return the last error message recorded in this session.
    pub fn last_error(&self) -> String {
        let Some(entry) = crate::session::store::SESSIONS.get(&self.session_id) else {
            return String::new();
        };
        let msg = entry.value().read().last_error_message.clone();
        msg
    }
}

impl Drop for KernelSession {
    fn drop(&mut self) {
        rgm_kernel_destroy(RgmKernelHandle(self.session_id));
    }
}

impl KernelSession {
    pub(crate) fn handle(&self) -> RgmKernelHandle {
        RgmKernelHandle(self.session_id)
    }

    pub(crate) fn tol(&self) -> RgmToleranceContext {
        RgmToleranceContext {
            abs_tol: self.abs_tol,
            rel_tol: self.rel_tol,
            angle_tol: self.angle_tol,
        }
    }
}

// ─── Handle types ─────────────────────────────────────────────────────────────

macro_rules! define_handle {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[wasm_bindgen]
        pub struct $name {
            pub(crate) session_id: u64,
            pub(crate) object_id: u64,
        }

        #[wasm_bindgen]
        impl $name {
            /// The raw object ID within the session (useful for `compute_bounds`).
            pub fn object_id(&self) -> f64 {
                self.object_id as f64
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                if self.object_id != 0 {
                    rgm_object_release(
                        RgmKernelHandle(self.session_id),
                        RgmObjectHandle(self.object_id),
                    );
                }
            }
        }

        impl $name {
            pub(crate) fn new(session_id: u64, object_id: u64) -> Self {
                Self { session_id, object_id }
            }
        }
    };
}

define_handle!(CurveHandle, "Handle to a curve object.");
define_handle!(SurfaceHandle, "Handle to a surface object.");
define_handle!(MeshHandle, "Handle to a mesh object.");
define_handle!(FaceHandle, "Handle to a trimmed face object.");
define_handle!(IntersectionHandle, "Handle to an intersection result.");
define_handle!(BrepHandle, "Handle to a B-rep solid.");
define_handle!(LandXmlDocHandle, "Handle to a parsed LandXML document.");

// ─── Helpers shared across domain modules ─────────────────────────────────────

/// Convert a flat `[x, y, z, x, y, z, …]` slice to a `Vec<RgmPoint3>`.
pub(crate) fn flat_to_points(flat: &[f64]) -> Vec<crate::RgmPoint3> {
    flat.chunks_exact(3)
        .map(|c| crate::RgmPoint3 { x: c[0], y: c[1], z: c[2] })
        .collect()
}

/// Convert a `Vec<RgmPoint3>` to a flat `[x, y, z, …]` `Vec<f64>`.
pub(crate) fn points_to_flat(pts: &[crate::RgmPoint3]) -> Vec<f64> {
    pts.iter().flat_map(|p| [p.x, p.y, p.z]).collect()
}

/// Convert a flat `[u, v, u, v, …]` slice to `Vec<RgmUv2>`.
pub(crate) fn flat_to_uv(flat: &[f64]) -> Vec<crate::RgmUv2> {
    flat.chunks_exact(2)
        .map(|c| crate::RgmUv2 { u: c[0], v: c[1] })
        .collect()
}

pub(crate) fn uv_to_flat(uvs: &[crate::RgmUv2]) -> Vec<f64> {
    uvs.iter().flat_map(|uv| [uv.u, uv.v]).collect()
}
