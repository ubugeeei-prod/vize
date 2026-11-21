//! One explicitly requested public packet for a position in an open document.
use super::{NativeNavigationProject, NavigationRefusal};
use tower_lsp::lsp_types::{DocumentHighlight, DocumentHighlightKind, Location, Position, Url};

#[derive(Debug, Default, PartialEq)]
pub struct SymbolNavigationProducts {
    pub definition: Option<Location>,
    pub references: Vec<Location>,
    pub highlights: Vec<DocumentHighlight>,
}

impl SymbolNavigationProducts {
    // The worker sends the ordered ranges and usage kinds once. URI-bearing
    // products are owned by the request and allocated after its mailbox reply.
    fn from_highlights(uri: &Url, highlights: Vec<DocumentHighlight>, include: bool) -> Self {
        let mut definition = None;
        let mut references =
            Vec::with_capacity(highlights.len().saturating_sub(usize::from(!include)));
        for site in &highlights {
            let declaration = site.kind == Some(DocumentHighlightKind::TEXT);
            if declaration {
                definition = Some(Location::new(uri.clone(), site.range));
            }
            if !declaration || include {
                references.push(Location::new(uri.clone(), site.range));
            }
        }
        Self {
            definition,
            references,
            highlights,
        }
    }
}

impl NativeNavigationProject<'_> {
    /// Resolve once on the retained script or Vue File. Each range belongs to
    /// the same source revision and configuration at publication.
    pub async fn symbol_products(
        &self,
        uri: &Url,
        position: Position,
        include_declaration: bool,
    ) -> Result<SymbolNavigationProducts, NavigationRefusal> {
        let (query, _) = self
            .source
            .begin_query(uri)
            .map_err(NavigationRefusal::Host)?;
        let ready = query
            .run(|snapshot| async move {
                let worker = self.worker(snapshot)?;
                let result = worker.symbol_sites(position).await.map(|sites| {
                    SymbolNavigationProducts::from_highlights(uri, sites, include_declaration)
                });
                Ok((worker.profile(), result))
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

#[cfg(test)]
mod tests;
