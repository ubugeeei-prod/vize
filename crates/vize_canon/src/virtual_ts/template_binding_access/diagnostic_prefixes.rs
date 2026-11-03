//! Generated receiver anchors belong to emission, independently of instance admission.

use std::cell::RefCell;

use super::TemplateBindingAccess;
use crate::virtual_ts::types::VizeMapping;

#[derive(Debug)]
pub(super) enum DiagnosticPrefixes<'a> {
    Owned(RefCell<Vec<VizeMapping>>),
    Borrowed(&'a RefCell<Vec<VizeMapping>>),
}

impl Default for DiagnosticPrefixes<'_> {
    fn default() -> Self {
        Self::Owned(RefCell::default())
    }
}

impl DiagnosticPrefixes<'_> {
    pub(super) fn get(&self) -> &RefCell<Vec<VizeMapping>> {
        match self {
            Self::Owned(value) => value,
            Self::Borrowed(value) => value,
        }
    }
}

impl TemplateBindingAccess<'_> {
    pub(crate) fn record_instance_diagnostic_prefix(&self, mapping: VizeMapping) {
        self.diagnostic_prefixes.get().borrow_mut().push(mapping);
        vize_carton::profiler::global_profiler()
            .record_counter("canon.instance_global_rewrite.diagnostic_prefixes", 1);
    }

    pub(crate) fn append_instance_diagnostic_prefixes(&self, output: &mut Vec<VizeMapping>) {
        output.append(&mut self.diagnostic_prefixes.get().borrow_mut());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_views_share_diagnostic_positions_without_an_instance_owner() {
        let owner = TemplateBindingAccess::from_props(["static".into()].into_iter().collect());
        let scope = owner.for_scope(vize_croquis::ScopeId::ROOT);
        assert!(!scope.has_instance_globals());
        scope.record_instance_diagnostic_prefix(VizeMapping::new(100..107, 20..26));
        let mut mappings = Vec::new();
        owner.append_instance_diagnostic_prefixes(&mut mappings);
        assert_eq!(mappings, vec![VizeMapping::new(100..107, 20..26)]);
        scope.append_instance_diagnostic_prefixes(&mut mappings);
        assert_eq!(mappings.len(), 1, "the emission positions are drained once");
    }
}
