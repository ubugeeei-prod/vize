//! One request may reuse its exact cold native project, after normal graph validation.

use std::sync::{Arc, Mutex, Weak};

use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaInitialVueDocument};

use super::IdeContext;
use crate::server::state::CorsaRequestStamp;

pub(super) type OwnedInitialDocument = InitialCanonicalDocument<CorsaInitialVueDocument>;

/// The payload is opaque Canon authority. These guards only bind its consumer;
/// Canon still validates the complete current graph before skipping native sync.
pub(super) struct InitialCanonicalDocument<T> {
    bridge: Weak<CorsaBridge>,
    uri: Url,
    source: Arc<String>,
    stamp: CorsaRequestStamp,
    receipt: Mutex<Option<T>>,
}

impl<T> InitialCanonicalDocument<T> {
    fn new(
        ctx: &IdeContext<'_>,
        bridge: &Arc<CorsaBridge>,
        receipt: T,
        stamp: CorsaRequestStamp,
    ) -> Self {
        Self {
            bridge: Arc::downgrade(bridge),
            uri: ctx.uri.clone(),
            source: Arc::clone(&ctx.content),
            stamp,
            receipt: Mutex::new(Some(receipt)),
        }
    }

    fn take(&self, ctx: &IdeContext<'_>, bridge: &CorsaBridge) -> Option<T> {
        if !std::ptr::eq(self.bridge.as_ptr(), bridge)
            || self.uri != *ctx.uri
            || !Arc::ptr_eq(&self.source, &ctx.content)
            || !self.stamp.is_current(ctx.state)
            || !ctx
                .state
                .documents
                .text_snapshot(ctx.uri)
                .is_some_and(|source| Arc::ptr_eq(&source, &self.source))
        {
            return None;
        }
        self.receipt
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
}

impl IdeContext<'_> {
    pub(crate) fn attach_initial_canonical_document(
        &mut self,
        bridge: Option<&Arc<CorsaBridge>>,
        receipt: Option<CorsaInitialVueDocument>,
        stamp: CorsaRequestStamp,
    ) {
        if let (Some(bridge), Some(receipt)) = (bridge, receipt) {
            self.initial_canonical_document = Some(Box::new(InitialCanonicalDocument::new(
                self, bridge, receipt, stamp,
            )));
        }
    }

    pub(crate) fn take_initial_canonical_document(
        &self,
        bridge: &CorsaBridge,
    ) -> Option<CorsaInitialVueDocument> {
        self.initial_canonical_document.as_ref()?.take(self, bridge)
    }
}

#[cfg(test)]
mod tests;
