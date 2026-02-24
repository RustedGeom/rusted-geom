use std::sync::atomic::Ordering;

#[rgm_export(ts = "create", receiver = "kernel_static")]
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

#[rgm_export(ts = "dispose", receiver = "kernel")]
#[no_mangle]
pub extern "C" fn rgm_kernel_destroy(session: RgmKernelHandle) -> RgmStatus {
    match SESSIONS.remove(&session.0) {
        Some(_) => RgmStatus::Ok,
        None => RgmStatus::NotFound,
    }
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_object_release(
    session: RgmKernelHandle,
    object: RgmObjectHandle,
) -> RgmStatus {
    let result = with_session_mut(session, |state| {
        if state.objects.remove(&object.0).is_some() {
            state.mesh_accels.remove(&object.0);
            state.bounds_cache.remove(&object.0);
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

#[rgm_export]
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

#[rgm_export]
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
