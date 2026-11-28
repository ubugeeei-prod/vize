//! Classify both rule requests as each original op or attached binding arrives.
use super::{NativeCanonicalLintRefusal as Refusal, Spans};
use crate::attribute_policy::{ImageAltPolicy, positive_tabindex};
use vize_l0::Span;
use vize_l2::{
    op::{Attribute, BindingOp, DynamicName, Namespace, Op},
    walk::{NodeLimit, NodeRef},
};

#[derive(Default)]
pub(super) struct Candidate {
    pub(super) spans: Spans,
    refusal: Option<Refusal>,
}

impl Candidate {
    pub(super) fn outcome(&self) -> Result<(), Refusal> {
        self.refusal.map_or(Ok(()), Err)
    }

    fn tabindex_attributes(&mut self, attributes: &[Attribute<'_>]) {
        if self.refusal.is_none() {
            for attr in attributes {
                if attr.name.eq_ignore_ascii_case("tabindex")
                    && attr.value.is_some_and(positive_tabindex)
                {
                    self.spans.push(attr.span);
                }
            }
        }
    }
}

#[derive(Default)]
pub(super) struct Candidates {
    pub(super) tabindex: Candidate,
    pub(super) images: Candidate,
    pending_image: Option<(ImageAltPolicy, Span, bool)>,
}

impl Candidates {
    pub(super) fn collect(&mut self, l2ir: &vize_l2::ir::L2Ir<'_>) {
        let artifact = l2ir.artifact();
        // The validated count includes attached bindings and nested regions.
        // An isolated root is therefore the complete input to both rules.
        if artifact.node_count() == 1
            && let [only] = artifact.root().ops.as_slice()
        {
            self.observe(NodeRef::Op(only));
            self.finish_image();
            return;
        }
        let walk = artifact.visit_markup_preorder(&mut |_, node| self.observe(node));
        self.finish(walk);
    }

    fn observe(&mut self, node: NodeRef<'_, '_>) {
        match node {
            NodeRef::Op(op) => {
                self.finish_image();
                match op {
                    Op::Element(element) => {
                        self.tabindex.tabindex_attributes(&element.attributes);
                        if self.images.refusal.is_some() {
                            return;
                        }
                        let policy = ImageAltPolicy::for_tag(element.tag);
                        if policy.applies() {
                            if element.namespace == Namespace::Html {
                                let has_alt = element
                                    .attributes
                                    .iter()
                                    .any(|a| a.name.eq_ignore_ascii_case("alt"));
                                self.pending_image = Some((policy, element.span, has_alt));
                            } else {
                                self.images.refusal =
                                    Some(Refusal::UnsupportedImageElement { span: element.span });
                            }
                        }
                    }
                    Op::Component(component) => {
                        self.tabindex.tabindex_attributes(&component.attributes);
                        // Component identity does not determine the original image tag.
                        self.images
                            .refusal
                            .get_or_insert(Refusal::UnsupportedImageElement {
                                span: component.span,
                            });
                    }
                    Op::Comment(comment) => {
                        let refusal = Refusal::MarkupComment { span: comment.span };
                        self.tabindex.refusal.get_or_insert(refusal);
                        self.images.refusal.get_or_insert(refusal);
                    }
                    _ => {}
                }
            }
            NodeRef::Binding(BindingOp::Bind(binding)) if self.images.refusal.is_none() => {
                match binding.name {
                    Some(DynamicName::Static(name)) => {
                        if let Some((_, _, has_alt)) = self.pending_image.as_mut() {
                            *has_alt |= name.eq_ignore_ascii_case("alt");
                        }
                    }
                    None | Some(DynamicName::Dynamic(_)) => {
                        self.images.refusal =
                            Some(Refusal::IndeterminateAttribute { span: binding.span });
                        self.pending_image = None;
                    }
                }
            }
            _ => {}
        }
    }

    fn finish_image(&mut self) {
        if let Some((policy, span, has_alt)) = self.pending_image.take()
            && policy.is_missing(has_alt)
        {
            self.images.spans.push(span);
        }
    }

    fn finish(&mut self, walk: Result<(), NodeLimit>) {
        if walk.is_err() {
            self.tabindex.refusal = Some(Refusal::NodeLimit);
            self.images.refusal = Some(Refusal::NodeLimit);
        } else {
            self.finish_image();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_walk_overrides_each_rule_and_leaves_the_last_image_unfinished() {
        let mut group = Candidates::default();
        let span = Span::new(18, 23);
        group.images.refusal = Some(Refusal::IndeterminateAttribute { span });
        group.tabindex.refusal = Some(Refusal::MarkupComment { span });
        group.tabindex.spans.push(span);
        group.pending_image = Some((ImageAltPolicy::for_tag("img"), span, false));
        group.finish(Err(NodeLimit));
        assert_eq!(group.images.outcome(), Err(Refusal::NodeLimit));
        assert_eq!(group.tabindex.outcome(), Err(Refusal::NodeLimit));
        assert!(group.images.spans.is_empty());
        assert_eq!(group.tabindex.spans.as_slice(), &[span]);
    }
}
