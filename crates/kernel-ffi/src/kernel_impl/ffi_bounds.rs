#[no_mangle]
pub extern "C" fn rgm_object_compute_bounds(
    session: RgmKernelHandle,
    object: RgmObjectHandle,
    options: *const RgmBoundsOptions,
    out_bounds: *mut RgmBounds3,
) -> RgmStatus {
    let options = if options.is_null() {
        None
    } else {
        // SAFETY: pointer is non-null by guard above.
        Some(unsafe { *options })
    };
    rgm_object_compute_bounds_impl(session, object, options, out_bounds)
}
