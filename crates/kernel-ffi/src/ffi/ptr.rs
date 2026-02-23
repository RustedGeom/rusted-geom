// Generic pointer helpers for the FFI boundary.
// Replaces eight type-specific write_* helpers with a single monomorphised
// write_out<T>.  All unsafe is contained here with SAFETY contracts so
// call sites remain safe.
// RgmStatus is in scope from foundation.rs (included first in ffi_impl.rs).

/// Write a `Copy` value through a non-null C output pointer.
///
/// Returns `Err(InvalidInput)` if `out` is null; otherwise writes `value` and
/// returns `Ok(())`.
///
/// # Safety contract for callers
/// The caller must ensure `out` points to writable, properly-aligned memory
/// that is live for the duration of this call.  The null-check in this function
/// satisfies the Rust `unsafe` requirement; no additional preconditions beyond
/// non-null alignment are assumed.
#[inline]
pub(crate) fn write_out<T: Copy>(out: *mut T, value: T) -> Result<(), RgmStatus> {
    if out.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    // SAFETY: non-null checked above; caller guarantees alignment and lifetime.
    unsafe {
        *out = value;
    }
    Ok(())
}

/// Read a `Copy` value through a non-null C input pointer.
///
/// Returns `Err(InvalidInput)` if `ptr` is null; otherwise reads and returns
/// the value.
#[allow(dead_code)]
#[inline]
pub(crate) fn read_in<T: Copy>(ptr: *const T) -> Result<T, RgmStatus> {
    if ptr.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    // SAFETY: non-null checked above; caller guarantees alignment and lifetime.
    Ok(unsafe { *ptr })
}

/// Write a slice of `Copy` values into a caller-provided output buffer.
///
/// Always writes the total count into `*out_count` (even when `capacity == 0`).
/// Copies up to `capacity` elements into `out_values`.  If `capacity > 0` and
/// `out_values` is null, returns `Err(InvalidInput)`.
#[inline]
pub(crate) fn write_slice<T: Copy>(
    out_values: *mut T,
    capacity: u32,
    values: &[T],
    out_count: *mut u32,
) -> Result<(), RgmStatus> {
    if out_count.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    // SAFETY: out_count non-null by guard above.
    unsafe {
        *out_count = values.len().try_into().unwrap_or(u32::MAX);
    }
    if capacity == 0 {
        return Ok(());
    }
    if out_values.is_null() {
        return Err(RgmStatus::InvalidInput);
    }
    let copy_count = values.len().min(capacity as usize);
    for (idx, value) in values.iter().take(copy_count).enumerate() {
        // SAFETY: out_values is non-null and caller guarantees at least `capacity` elements.
        unsafe {
            *out_values.add(idx) = *value;
        }
    }
    Ok(())
}
