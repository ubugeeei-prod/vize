use napi_derive::napi;

/// One explicit filesystem-stability window for imported-type resolution.
///
/// Keep the handle alive while issuing related synchronous `compileSfc` calls.
/// Calling `close` (or allowing the handle to be collected) ends the window, so
/// later calls strongly revalidate imported source bytes again.
#[napi(js_name = "TypeResolutionBatch")]
pub struct TypeResolutionBatchNapi {
    guard: Option<vize_atelier_sfc::TypeResolutionBatchGuard>,
}

#[napi]
impl TypeResolutionBatchNapi {
    /// End this stability window. Repeated calls are harmless.
    #[napi]
    pub fn close(&mut self) -> bool {
        self.guard.take().is_some()
    }
}

/// Begin an explicit imported-type filesystem-stability window.
///
/// The package entrypoint uses this to share validation across synchronous
/// `compileSfc` calls in one JavaScript turn. Direct consumers may also hold the
/// returned handle across a known-stable build phase and close it afterwards.
#[napi(js_name = "beginTypeResolutionBatch")]
pub fn begin_type_resolution_batch() -> TypeResolutionBatchNapi {
    TypeResolutionBatchNapi {
        guard: Some(vize_atelier_sfc::begin_type_resolution_batch()),
    }
}

#[cfg(test)]
mod tests {
    use super::begin_type_resolution_batch;

    #[test]
    fn close_is_idempotent() {
        let mut batch = begin_type_resolution_batch();
        assert!(batch.close());
        assert!(!batch.close());
    }
}
