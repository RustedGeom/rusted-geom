//! `Result<T, JsValue>` helper utilities for the wasm-bindgen layer.

use crate::RgmStatus;
use wasm_bindgen::JsValue;

/// Convert an `RgmStatus` failure code into a `JsValue` error string.
pub(crate) fn status_to_jsval(status: RgmStatus) -> JsValue {
    JsValue::from_str(&format!("{:?}", status))
}

/// Check a status; return `Ok(())` or propagate as a `JsValue`.
pub(crate) fn check(status: RgmStatus) -> Result<(), JsValue> {
    if status == RgmStatus::Ok {
        Ok(())
    } else {
        Err(status_to_jsval(status))
    }
}

/// Convert an `RgmStatus` to a `JsValue` error (convenience for match arms).
pub(crate) fn js_err(status: RgmStatus) -> JsValue {
    status_to_jsval(status)
}
