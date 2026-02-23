#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_alloc(byte_len: usize, align: usize, out_ptr: *mut *mut u8) -> RgmStatus {
    if out_ptr.is_null() {
        return RgmStatus::InvalidInput;
    }

    if byte_len == 0 || align == 0 || !align.is_power_of_two() {
        return RgmStatus::InvalidInput;
    }

    let Ok(layout) = Layout::from_size_align(byte_len, align) else {
        return RgmStatus::InvalidInput;
    };

    // SAFETY: layout is validated above.
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        return RgmStatus::InternalError;
    }

    // SAFETY: out_ptr is non-null by guard above.
    unsafe {
        *out_ptr = ptr;
    }

    RgmStatus::Ok
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_alloc_addr(byte_len: usize, align: usize) -> usize {
    if byte_len == 0 || align == 0 || !align.is_power_of_two() {
        return 0;
    }

    let Ok(layout) = Layout::from_size_align(byte_len, align) else {
        return 0;
    };

    // SAFETY: layout is validated above.
    let ptr = unsafe { alloc(layout) };
    ptr as usize
}

#[rgm_export]
#[no_mangle]
pub extern "C" fn rgm_dealloc(ptr: *mut u8, byte_len: usize, align: usize) -> RgmStatus {
    if ptr.is_null() {
        return RgmStatus::InvalidInput;
    }

    if byte_len == 0 || align == 0 || !align.is_power_of_two() {
        return RgmStatus::InvalidInput;
    }

    let Ok(layout) = Layout::from_size_align(byte_len, align) else {
        return RgmStatus::InvalidInput;
    };

    // SAFETY: ptr and layout originate from rgm_alloc contract.
    unsafe {
        dealloc(ptr, layout);
    }

    RgmStatus::Ok
}
