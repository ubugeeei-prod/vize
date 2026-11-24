//! Reuse only answers attached to a native immutable snapshot still acknowledged by its owner.
use crate::lsp_client::{
    CorsaProjectClient, DiagnosticFetch, PhasedDiagnostic, initial_diagnostics::AmbientWorld,
};
use std::borrow::Cow;
use vize_l0::{String, cstr};

/// One successful compiler read and its exact native source world. A new
/// caller cannot construct a receipt or install one into another session.
pub struct AmbientDiagnosticSnapshot {
    read: Box<AmbientWorld>,
    diagnostics: Option<Vec<PhasedDiagnostic>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AmbientReadOrigin {
    Previous,
    Cache,
    Compiler,
}

impl AmbientReadOrigin {
    pub(crate) fn used_cache(self) -> bool {
        self != Self::Compiler
    }

    pub(crate) fn confirmed_previous(self) -> bool {
        self == Self::Previous
    }

    fn fetched(used_cache: bool) -> Self {
        if used_cache {
            Self::Cache
        } else {
            Self::Compiler
        }
    }
}

impl AmbientDiagnosticSnapshot {
    /// Keep the native cursor after the caller has retained its complete mapped
    /// answer. A reader requesting fresh rows can still use this cursor.
    pub(crate) fn into_mapped_cursor(mut self) -> Self {
        if let Some(rows) = &self.diagnostics {
            // An actually empty native response remains a complete raw answer.
            // Release even a spare vector allocation before retaining it.
            self.diagnostics = rows.is_empty().then(Vec::new);
        }
        self
    }

    pub(crate) fn has_empty_native_rows(&self) -> bool {
        self.diagnostics.as_ref().is_some_and(Vec::is_empty)
    }

    pub(crate) fn confirm_owner(self) -> (Self, bool) {
        let confirmed = self.read.confirms_owner().unwrap_or(false);
        (self, confirmed)
    }
}

impl CorsaProjectClient {
    pub(crate) fn read_ambient_diagnostic_snapshot<'a>(
        &mut self,
        uri: &str,
        snapshot: &'a mut Option<AmbientDiagnosticSnapshot>,
        confirmed_owner: bool,
        same_projection: bool,
    ) -> Result<(Cow<'a, [PhasedDiagnostic]>, AmbientReadOrigin), String> {
        let previous = snapshot.take();
        let owned = |fetch: DiagnosticFetch| {
            (
                Cow::Owned(fetch.diagnostics),
                AmbientReadOrigin::fetched(fetch.used_cache),
            )
        };
        // This also refreshes an attached editor's acknowledged native world.
        let Some(document_uri) = self.prepare_diagnostics_read_document(uri)? else {
            return self.request_diagnostics_full(uri).map(owned);
        };
        if let Some(previous) = previous
            && (same_projection || previous.diagnostics.is_some())
            && previous.read.is_current(self, uri, &document_uri)
            && (confirmed_owner || previous.read.confirms_owner()?)
        {
            crate::corsa_bridge::native_operation::checkpoint()?;
            let previous = snapshot.insert(previous);
            let rows = if same_projection {
                &[][..]
            } else {
                previous.diagnostics.as_deref().unwrap_or_default()
            };
            return Ok((Cow::Borrowed(rows), AmbientReadOrigin::Previous));
        }
        let read = self.borrow_diagnostics_read_view(uri, &document_uri)?;
        let rows = read.run_with_provenance()?;
        if !self.borrowed_diagnostics_read_is_current(&read) {
            return Err(cstr!(
                "Corsa diagnostic snapshot is no longer current for {uri}"
            ));
        }
        let Some(rows) = rows else {
            return self.request_lsp_diagnostics_or_cache(uri).map(owned);
        };
        let world = read.into_ambient_world(self.document_texts.ambient_revision());
        let fetch = self.store_file_diagnostics(uri, rows);
        let retained = snapshot.insert(AmbientDiagnosticSnapshot {
            read: Box::new(world),
            diagnostics: None,
        });
        let rows = retained.diagnostics.insert(fetch.diagnostics);
        Ok((
            Cow::Borrowed(rows),
            AmbientReadOrigin::fetched(fetch.used_cache),
        ))
    }
}
