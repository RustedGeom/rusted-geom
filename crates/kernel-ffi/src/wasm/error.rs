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
