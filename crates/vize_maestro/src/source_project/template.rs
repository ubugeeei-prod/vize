//! Read the existing grouped passes against the current open editor revision.
use super::{SnapshotRefusal, SourceSnapshot};
use crate::server::ServerState;
use vize_incremental::template::{TemplateReadError, TemplateReport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateRequestError {
    Host(SnapshotRefusal),
    Language,
    Analysis(TemplateReadError),
}

impl SourceSnapshot {
    /// Read the template's source range, decision rows and diagnostics for this snapshot.
    /// The normal source query checks freshness again when publishing the response.
    pub fn template_report(
        &self,
        state: &ServerState,
    ) -> Result<Option<TemplateReport>, TemplateRequestError> {
        let passes = self
            .with_current(&state.documents, |document| {
                if self.language_id() != "vue"
                    || crate::utils::is_standalone_html_path(self.uri().path())
                {
                    return Err(TemplateRequestError::Language);
                }
                Ok(document
                    .template_passes
                    .get_or_init(Default::default)
                    .clone())
            })
            .map_err(TemplateRequestError::Host)??;
        let result = {
            let mut passes = if let Some(available) = passes.try_lock() {
                available
            } else {
                let available = passes.lock();
                self.check_current(&state.documents)
                    .map_err(TemplateRequestError::Host)?;
                available
            };
            passes
                .read(self.source_owner(), &mut vize_l0::pass::NoObserver)
                .and_then(|view| view.map(|read| read.report()).transpose())
        };
        self.check_current(&state.documents)
            .map_err(TemplateRequestError::Host)?;
        result.map_err(TemplateRequestError::Analysis)
    }
}

#[cfg(test)]
mod tests;
