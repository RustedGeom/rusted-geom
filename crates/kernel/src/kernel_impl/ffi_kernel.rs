use std::sync::atomic::Ordering;

#[no_mangle]
pub extern "C" fn rgm_kernel_create(out_session: *mut RgmKernelHandle) -> RgmStatus {
    if out_session.is_null() {
        return RgmStatus::InvalidInput;
    }

    let session_id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);

    SESSIONS.insert(session_id, parking_lot::RwLock::new(SessionState::default()));

    // SAFETY: out_session is non-null by guard above.
    unsafe {
        *out_session = RgmKernelHandle(session_id);
    }

    RgmStatus::Ok
}

#[no_mangle]
pub extern "C" fn rgm_kernel_destroy(session: RgmKernelHandle) -> RgmStatus {
    match SESSIONS.remove(&session.0) {
        Some(_) => RgmStatus::Ok,
        None => RgmStatus::NotFound,
    }
}

#[no_mangle]
pub extern "C" fn rgm_object_release(
    session: RgmKernelHandle,
    object: RgmObjectHandle,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        if state.objects.remove(&object.0).is_some() {
            state.mesh_accels.remove(&object.0);
            state.bounds_cache.remove(&object.0);
            if let Some(path) = state.path_index.remove(&object.0) {
                state.stage.remove_prim(&path);
            }
            Ok(())
        } else {
            Err(RgmStatus::NotFound)
        }
    });

    match result {
        Ok(()) => {
            clear_error(session.0);
            RgmStatus::Ok
        }
        Err(status) => map_err_with_session(session, status, "Object not found in this session"),
    }
}

#[no_mangle]
pub extern "C" fn rgm_last_error_code(session: RgmKernelHandle, out_code: *mut i32) -> RgmStatus {
    if out_code.is_null() {
        return RgmStatus::InvalidInput;
    }

    let Some(entry) = SESSIONS.get(&session.0) else {
        return RgmStatus::NotFound;
    };
    let state = entry.value().read();

    // SAFETY: out_code is non-null by guard above.
    unsafe {
        *out_code = state.last_error_code as i32;
    }

    RgmStatus::Ok
}

/// Export USD stage (or a filtered subset) to USDA text.
///
/// If `object_id_count == 0`, the full stage is serialised.  Otherwise only the
/// prims registered for the provided IDs are included.
///
/// On success, `*out_ptr` is set to a heap-allocated UTF-8 string (null-terminated)
/// and `*out_len` to the byte count (excluding the null byte).  The caller must
/// free the buffer with `rgm_dealloc(ptr, len + 1, 1)`.
#[no_mangle]
pub extern "C" fn rgm_export_usda_text(
    session: RgmKernelHandle,
    object_ids: *const u64,
    object_id_count: usize,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> RgmStatus {
    if out_ptr.is_null() || out_len.is_null() {
        return RgmStatus::InvalidInput;
    }
    let ids: &[u64] = if object_ids.is_null() || object_id_count == 0 {
        &[]
    } else {
        // SAFETY: caller guarantees pointer and count are valid.
        unsafe { std::slice::from_raw_parts(object_ids, object_id_count) }
    };

    let text = match export_usda_text(session, ids) {
        Ok(t) => t,
        Err(_) => return map_err_with_session(session, RgmStatus::InternalError, "USDA export failed"),
    };

    let bytes = text.into_bytes();
    let len = bytes.len();
    let Ok(layout) = Layout::from_size_align(len + 1, 1) else {
        return RgmStatus::InternalError;
    };
    // SAFETY: layout is valid.
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        return RgmStatus::InternalError;
    }
    // SAFETY: ptr is non-null, layout covers len + 1 bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
        *ptr.add(len) = 0;
        *out_ptr = ptr;
        *out_len = len;
    }

    RgmStatus::Ok
}

#[no_mangle]
pub extern "C" fn rgm_last_error_message(
    session: RgmKernelHandle,
    buffer: *mut u8,
    buffer_len: usize,
    out_written: *mut usize,
) -> RgmStatus {
    if out_written.is_null() {
        return RgmStatus::InvalidInput;
    }

    let Some(entry) = SESSIONS.get(&session.0) else {
        return RgmStatus::NotFound;
    };
    let state = entry.value().read();

    let message_bytes = state.last_error_message.as_bytes();
    let bytes_to_copy = if buffer.is_null() || buffer_len == 0 {
        0
    } else {
        message_bytes.len().min(buffer_len.saturating_sub(1))
    };

    // SAFETY: out_written is non-null by guard. buffer writes guarded by null and length checks.
    unsafe {
        *out_written = bytes_to_copy;
        if !buffer.is_null() && buffer_len > 0 {
            std::ptr::copy_nonoverlapping(message_bytes.as_ptr(), buffer, bytes_to_copy);
            *buffer.add(bytes_to_copy) = 0;
        }
    }

    RgmStatus::Ok
}
