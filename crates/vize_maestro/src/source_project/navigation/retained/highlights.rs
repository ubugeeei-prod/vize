//! Same original Program/SFC facts supply usage, never response-name scans.

use super::{NavigationRefusal, ReferenceTarget, RetainedNavigation};
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
            sites.extend(self.file.references().iter().filter_map(|site| {
                (site.target == ReferenceTarget::Resolved(binding.id()))
                    .then_some((site.span, Some(site.usage)))
            }));
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
