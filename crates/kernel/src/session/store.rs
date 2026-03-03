//! Session lifecycle: global session registry, accessor helpers, and
//! object-insertion utilities.
//!
//! `SESSIONS` is a sharded global map from session ID to `SessionState`
//! protected by per-session read/write locks. All mutable access goes through
//! [`with_session_mut`], which acquires a write lock for the target session.

use crate::elements::brep::types::BrepData;
use crate::session::objects::{
    CurveData, FaceData, GeometryObject, IntersectionData, LandXmlDocData, MeshData, SessionState,
    SurfaceData,
};
use crate::{RgmKernelHandle, RgmObjectHandle, RgmStatus};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);
pub(crate) static NEXT_OBJECT_ID: AtomicU64 = AtomicU64::new(1);
pub(crate) static SESSIONS: Lazy<DashMap<u64, RwLock<SessionState>>> = Lazy::new(DashMap::new);

pub(crate) fn set_error(session_id: u64, status: RgmStatus, message: impl Into<String>) {
    if let Some(entry) = SESSIONS.get(&session_id) {
        let mut session = entry.value().write();
        session.last_error_code = status;
        session.last_error_message = message.into();
    }
}

fn clear_error_state(state: &mut SessionState) {
    state.last_error_code = RgmStatus::Ok;
    state.last_error_message.clear();
}

// P1: This function is kept for compatibility with existing export callsites outside brep_ops.
// In brep_ops.rs, finish() no longer calls clear_error; with_session_mut clears on success.
pub(crate) fn clear_error(session_id: u64) {
    if let Some(entry) = SESSIONS.get(&session_id) {
        let mut session = entry.value().write();
        clear_error_state(&mut session);
    }
}

// P1: Error is cleared inside the write lock on success — no second lock acquisition.
pub(crate) fn with_session_mut<T>(
    session: RgmKernelHandle,
    f: impl FnOnce(&mut SessionState) -> Result<T, RgmStatus>,
) -> Result<T, RgmStatus> {
    let entry = SESSIONS.get(&session.0).ok_or(RgmStatus::NotFound)?;
    let mut state = entry.value().write();
    let result = f(&mut state);
    if result.is_ok() {
        clear_error_state(&mut state);
    }
    result
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
    state
        .objects
        .insert(object_id, GeometryObject::Curve(curve));
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

pub(crate) fn insert_brep(state: &mut SessionState, brep: BrepData) -> RgmObjectHandle {
    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);
    state.objects.insert(object_id, GeometryObject::Brep(brep));
    RgmObjectHandle(object_id)
}

pub(crate) fn insert_landxml_doc(
    state: &mut SessionState,
    doc: LandXmlDocData,
) -> RgmObjectHandle {
    let object_id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);
    state
        .objects
        .insert(object_id, GeometryObject::LandXmlDoc(doc));
    RgmObjectHandle(object_id)
}

// S1: insert_brep_in_progress removed; callers use insert_brep with finalized=false.
