//! One source-coordinate conversion and one ordered set of original use sites.
use super::{Model, NavigationRefusal, RetainedNavigation};
use tower_lsp::lsp_types::{DocumentHighlight, Position};

impl RetainedNavigation<'_, '_> {
    pub(super) fn symbol_sites(
        &self,
        position: Position,
    ) -> Result<Vec<DocumentHighlight>, NavigationRefusal> {
        let offset = self.offset(position)?;
        let mut sites = Vec::new();
        let binding = if let Some(native) = &self.native {
            native
                .for_each_reference_at_binding(offset, |r| sites.push((r.span(), Some(r.usage()))))
                .map_err(NavigationRefusal::NativeQuery)?
        } else {
            let model = Model::new(self.file);
            let binding = model
                .binding_at_offset(offset)
                .map_err(NavigationRefusal::Query)?;
            if let Some(binding) = binding {
                model.for_each_reference_to(binding, |r| {
                    let row = r.reference();
                    sites.push((row.span, Some(row.usage)));
                });
            }
            binding
        };
        let Some(binding) = binding else {
            return Ok(Vec::new());
        };
        let declaration = binding.declaration().ok_or(NavigationRefusal::Projection)?;
        sites.push((declaration.span, None));
        super::super::highlights::owned_highlights(self.snapshot.source(), self.lines, sites)
    }
}
