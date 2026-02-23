//! Session lifecycle: global session registry, accessor helpers, and
//! object-insertion utilities.
//!
//! `SESSIONS` is the global map from session ID to `SessionState`.  All
//! mutable access goes through [`with_session_mut`], which holds the mutex for
//! the duration of the closure.

use crate::session::objects::{
    CurveData, FaceData, GeometryObject, IntersectionData, MeshData, SessionState, SurfaceData,
};
use crate::{RgmKernelHandle, RgmObjectHandle, RgmStatus};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

pub(crate) static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);
pub(crate) static NEXT_OBJECT_ID: AtomicU64 = AtomicU64::new(1);
pub(crate) static SESSIONS: Lazy<Mutex<HashMap<u64, SessionState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub(crate) fn set_error(session_id: u64, status: RgmStatus, message: impl Into<String>) {
    if let Ok(mut sessions) = SESSIONS.lock() {
        if let Some(session) = sessions.get_mut(&session_id) {
            session.last_error_code = status;
            session.last_error_message = message.into();
        }
    }
}

pub(crate) fn clear_error(session_id: u64) {
    if let Ok(mut sessions) = SESSIONS.lock() {
        if let Some(session) = sessions.get_mut(&session_id) {
            session.last_error_code = RgmStatus::Ok;
            session.last_error_message.clear();
        }
    }
}

pub(crate) fn with_session_mut<T>(
    session: RgmKernelHandle,
    f: impl FnOnce(&mut SessionState) -> Result<T, RgmStatus>,
) -> Result<T, RgmStatus> {
    let mut sessions = SESSIONS.lock().map_err(|_| RgmStatus::InternalError)?;
    let state = sessions.get_mut(&session.0).ok_or(RgmStatus::NotFound)?;
    f(state)
}

pub(crate) fn map_err_with_session(
    session: RgmKernelHandle,
    status: RgmStatus,
    message: &str,
) -> RgmStatus {
    set_error(session.0, status, message);
    status
}

// ─── Object Insert Helpers ────────────────────────────────────────────────────

pub(crate) fn insert_curve(state: &mut SessionState, curve: CurveData) -> RgmObjectHandle {
    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);
    state.objects.insert(object_id, GeometryObject::Curve(curve));
    RgmObjectHandle(object_id)
}

pub(crate) fn insert_mesh(state: &mut SessionState, mesh: MeshData) -> RgmObjectHandle {
    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);
    state.objects.insert(object_id, GeometryObject::Mesh(mesh));
    RgmObjectHandle(object_id)
}

pub(crate) fn insert_surface(state: &mut SessionState, surface: SurfaceData) -> RgmObjectHandle {
    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);
    state
        .objects
        .insert(object_id, GeometryObject::Surface(surface));
    RgmObjectHandle(object_id)
}

pub(crate) fn insert_face(state: &mut SessionState, face: FaceData) -> RgmObjectHandle {
    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);
    state.objects.insert(object_id, GeometryObject::Face(face));
    RgmObjectHandle(object_id)
}

pub(crate) fn insert_intersection(
    state: &mut SessionState,
    intersection: IntersectionData,
) -> RgmObjectHandle {
    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);
    state
        .objects
        .insert(object_id, GeometryObject::Intersection(intersection));
    RgmObjectHandle(object_id)
}
