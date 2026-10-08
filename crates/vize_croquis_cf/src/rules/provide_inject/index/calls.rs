use vize_croquis::provide::{InjectEntry, ProvideEntry, ProvideKey};

/// Extract provide/inject calls from a component's analysis.
/// Uses the ProvideInjectTracker for precise static analysis - no heuristics.
#[inline]
pub(super) fn extract_provide_inject(
    analysis: &vize_croquis::Croquis,
) -> (Vec<ProvideEntry>, Vec<InjectEntry>) {
    // Use the actual provide/inject tracker data - precise static analysis
    let provides = vize_croquis::facts::provide_entries(analysis);
    let injects = vize_croquis::facts::inject_entries(analysis);
    (provides, injects)
}
pub(super) fn matching_provider<'a>(
    component_provides: &'a [ProvideEntry],
    key: &ProvideKey,
) -> Option<&'a ProvideEntry> {
    component_provides
        .iter()
        .rev()
        .find(|provide| provide.key == *key)
}
impl super::ProvideInjectIndex {
    pub(crate) fn tree_receiver(&self, file: crate::registry::FileId) -> bool {
        self.slot_scopes.is_receiver(file)
    }
}
