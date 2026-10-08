//! Binding metadata accessors for the projection mapping.

use super::{ProjectionMapping, VizeMapping, VizeSemanticLink};

impl ProjectionMapping {
    /// Semantic links between generated ranges.
    #[inline]
    pub fn semantic_links(&self) -> &[VizeSemanticLink] {
        &self.semantic_links
    }

    /// The rows and semantic links, dropping metadata and base.
    pub fn into_parts(self) -> (Vec<VizeMapping>, Vec<VizeSemanticLink>) {
        (self.spans, self.semantic_links)
    }

    /// Additional AST-owned inline default keys joined to bare prop bindings.
    /// Core semantic links and their existing cardinality stay unchanged.
    pub fn prop_default_key_links(&self) -> &[VizeSemanticLink] {
        &self.prop_default_key_links
    }

    pub(crate) fn set_prop_default_key_links(&mut self, links: Vec<VizeSemanticLink>) {
        self.prop_default_key_links = links;
    }

    /// Consume spans, core links and supplemental default-key links without
    /// cloning. The original two-part accessor retains its existing contract.
    pub fn into_binding_parts(
        self,
    ) -> (
        Vec<VizeMapping>,
        Vec<VizeSemanticLink>,
        Vec<VizeSemanticLink>,
    ) {
        (self.spans, self.semantic_links, self.prop_default_key_links)
    }
}

#[cfg(test)]
mod tests {
    use super::{ProjectionMapping, VizeMapping, VizeSemanticLink};
    use crate::virtual_ts::VizeSemanticLinkKind;

    fn mapping(spans: Vec<VizeMapping>) -> ProjectionMapping {
        let core = VizeSemanticLink {
            source_range: 10..14,
            target_range: 60..64,
            kind: VizeSemanticLinkKind::VueTemplatePropBinding,
        };
        let key = VizeSemanticLink {
            source_range: 10..14,
            target_range: 50..54,
            kind: VizeSemanticLinkKind::VueTemplatePropBinding,
        };
        let mut mapping = ProjectionMapping::from_parts(spans, vec![core]);
        mapping.set_prop_default_key_links(vec![key]);
        mapping
    }

    #[test]
    fn core_parts_keep_their_contract_while_full_parts_keep_auxiliary_ownership() {
        let spans = vec![VizeMapping::new(0..70, 100..170)];
        let mapping = mapping(spans.clone());
        let (old_spans, core) = mapping.clone().into_parts();
        assert_eq!(old_spans, spans);
        assert_eq!(core.len(), 1);
        assert_eq!(core[0].target_range, 60..64);
        let (all_spans, all_core, auxiliary) = mapping.into_binding_parts();
        assert_eq!((all_spans, all_core), (old_spans, core));
        assert_eq!(auxiliary.len(), 1);
        assert_eq!(auxiliary[0].source_range, 10..14);
        assert_eq!(auxiliary[0].target_range, 50..54);
    }

    #[test]
    fn both_replacement_routes_shift_auxiliary_and_core_endpoints_together() {
        for spans in [
            vec![VizeMapping::new(20..70, 100..150)],
            vec![VizeMapping::new(0..70, 100..170)],
        ] {
            let mut mapping = mapping(spans);
            mapping.note_generated_replacement(5, 0, 3);
            assert_eq!(mapping.semantic_links()[0].source_range, 13..17);
            assert_eq!(mapping.semantic_links()[0].target_range, 63..67);
            assert_eq!(mapping.prop_default_key_links()[0].source_range, 13..17);
            assert_eq!(mapping.prop_default_key_links()[0].target_range, 53..57);
        }
    }

    #[test]
    fn expression_retarget_preserves_both_exact_binding_endpoints() {
        let mut mapping = mapping(Vec::new());
        mapping.retarget_expression_binding(5..40, 10..14, 5, 8);
        assert_eq!(mapping.semantic_links()[0].source_range, 15..19);
        assert_eq!(mapping.semantic_links()[0].target_range, 68..72);
        assert_eq!(mapping.prop_default_key_links()[0].source_range, 15..19);
        assert_eq!(mapping.prop_default_key_links()[0].target_range, 58..62);
    }
}
