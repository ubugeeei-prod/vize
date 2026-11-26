//! Select one genuine binding and visit its script and template uses together.
use super::{
    BindingRef, Model, NativePositionQueryError, NativeReferenceRef, NativeSfc, PositionQueryError,
};

impl<'o, 'a> NativeSfc<'o, 'a> {
    /// Return the original declaration target and visit its references. A late
    /// refusal invalidates any partial results. Uses retain their original
    /// Program or template occurrence, scope, usage kind and source span.
    pub fn for_each_reference_at_binding(
        &self,
        offset: u32,
        mut visit: impl FnMut(NativeReferenceRef<'o, 'a>),
    ) -> Result<Option<BindingRef<'o, 'a>>, NativePositionQueryError> {
        let script = Model::new(self.file().l2ir())
            .binding_at_offset(offset)
            .map_err(NativePositionQueryError::File)?;
        let Some(binding) = script else {
            let Some(site) = self.reference_at_offset(offset)? else {
                return Ok(None);
            };
            let binding = site.binding();
            self.for_each_reference_to(binding, visit)?;
            return Ok(Some(binding));
        };
        // The original script unit has already selected the target. Certify all
        // remaining original owners and gather this target's uses in one visit.
        let mut at_site = false;
        let mut ambiguous = false;
        self.for_each_reference(|site| {
            let span = site.span();
            if span.start <= offset && offset < span.end {
                ambiguous |= at_site || site.template().is_some();
                at_site = true;
            }
            if site.binding().id() == binding.id() {
                visit(site);
            }
        })?;
        if ambiguous {
            return Err(NativePositionQueryError::File(
                PositionQueryError::AmbiguousSite,
            ));
        }
        Ok(Some(binding))
    }
}
