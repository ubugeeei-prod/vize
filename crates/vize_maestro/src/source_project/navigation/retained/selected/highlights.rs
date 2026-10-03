//! Sealed original handler sites retain each actual use and declaration role.

use super::{NavigationRefusal, SelectedNavigation};
use tower_lsp::lsp_types::{DocumentHighlight, Position};

impl SelectedNavigation<'_, '_> {
    pub(super) fn highlights(
        &self,
        position: Position,
    ) -> Result<Vec<DocumentHighlight>, NavigationRefusal> {
        let Some(symbol) = self.symbol(position)? else {
            return Ok(Vec::new());
        };
        let mut sites = Vec::new();
        // Discard all callback work on a late original provider refusal.
        self.file
            .for_each_template_reference_to(symbol, |site| sites.push((site.span(), site.usage())))
            .map_err(NavigationRefusal::TemplateQuery)?;
        sites.extend(
            self.declarations(symbol)?
                .into_iter()
                .map(|span| (span, None)),
        );
        super::super::super::highlights::owned_highlights(self.snapshot.source(), self.lines, sites)
    }
}
