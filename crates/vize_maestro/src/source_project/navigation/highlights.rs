//! Explicit symbol highlights preserve original usage and declaration roles.

use super::{NativeNavigationProject, NavigationRefusal, coordinates};
use tower_lsp::lsp_types::{DocumentHighlight, DocumentHighlightKind, Position, Url};
use vize_l0::Span;
use vize_l2::resolution::Usage;

impl NativeNavigationProject<'_> {
    pub async fn highlights(
        &self,
        uri: &Url,
        position: Position,
    ) -> Result<Vec<DocumentHighlight>, NavigationRefusal> {
        self.symbol_highlights(uri, position, false).await
    }

    pub async fn template_highlights(
        &self,
        uri: &Url,
        position: Position,
    ) -> Result<Vec<DocumentHighlight>, NavigationRefusal> {
        self.symbol_highlights(uri, position, true).await
    }

    async fn symbol_highlights(
        &self,
        uri: &Url,
        position: Position,
        selected: bool,
    ) -> Result<Vec<DocumentHighlight>, NavigationRefusal> {
        let (query, _) = self
            .source
            .begin_query(uri)
            .map_err(NavigationRefusal::Host)?;
        let ready = query
            .run(|snapshot| async move {
                let worker = self.worker_for(snapshot, selected)?;
                Ok((worker.profile(), worker.highlights(position).await))
            })
            .await
            .map_err(NavigationRefusal::Host)?;
        let result = ready
            .publish(|response| self.checked_response(response))
            .map_err(NavigationRefusal::Host)?;
        if matches!(result, Err(NavigationRefusal::ConfigurationChanged)) {
            self.retire_changed_configuration(uri);
        }
        result
    }
}

pub(super) fn owned_highlights(
    source: &str,
    lines: &[usize],
    mut sites: Vec<(Span, Option<Usage>)>,
) -> Result<Vec<DocumentHighlight>, NavigationRefusal> {
    sites.sort_unstable_by_key(|(span, _)| (span.start, span.end));
    sites
        .into_iter()
        .map(|(span, usage)| {
            let kind = match usage {
                None => DocumentHighlightKind::TEXT,
                Some(Usage::Read) => DocumentHighlightKind::READ,
                Some(Usage::Write | Usage::ReadWrite) => DocumentHighlightKind::WRITE,
            };
            Ok(DocumentHighlight {
                range: coordinates::range(source, lines, span)?,
                kind: Some(kind),
            })
        })
        .collect()
}
