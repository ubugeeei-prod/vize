//! Same original Program/SFC facts supply usage, never response-name scans.

use super::{Model, NavigationRefusal, RetainedNavigation};
use tower_lsp::lsp_types::{DocumentHighlight, Position};

impl RetainedNavigation<'_, '_> {
    pub(super) fn highlights(
        &self,
        position: Position,
    ) -> Result<Vec<DocumentHighlight>, NavigationRefusal> {
        let Some(binding) = self.binding(self.offset(position)?)? else {
            return Ok(Vec::new());
        };
        let mut sites = Vec::new();
        if let Some(native) = &self.native {
            native
                .for_each_reference_to(binding, |site| {
                    sites.push((site.span(), Some(site.usage())));
                })
                .map_err(NavigationRefusal::NativeQuery)?;
        } else {
            Model::new(self.l2ir).for_each_reference_to(binding, |site| {
                let row = site.reference();
                sites.push((row.span, Some(row.usage)));
            });
        }
        sites.push((
            binding
                .declaration()
                .ok_or(NavigationRefusal::Projection)?
                .span,
            None,
        ));
        super::super::highlights::owned_highlights(self.snapshot.source(), self.lines, sites)
    }
}
