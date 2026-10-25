//! Image-alt presence over one admitted canonical File, without source scans.

use super::{NativeCanonicalLint, NativeCanonicalLintRefusal};
use crate::{
    attribute_policy::{ImageAltMessages, ImageAltPolicy},
    native::{IMG_ALT_RULE, NativeLintFinding},
};
use vize_l0::{Span, diag::MessageLookup};
use vize_l2::{
    op::{BindingOp, DynamicName, Namespace, Op},
    walk::NodeRef,
};

struct PendingImage {
    policy: ImageAltPolicy,
    span: Span,
    has_alt: bool,
}

impl NativeCanonicalLint<'_, '_> {
    /// Check image-alt presence on genuine native Vue/JSX/TSX markup.
    ///
    /// Static and statically named bound attributes count, including empty alt
    /// values. Markup comments, component operations, non-HTML image elements,
    /// indeterminate attribute names and node-limit failures refuse the complete
    /// query. This does not enable a configured product route or infer component
    /// rendering. Native producer refusals remain separate from query admission.
    pub fn img_alt(
        &self,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeCanonicalLintRefusal> {
        vize_l0::profile!(
            "patina.native.canonical.img_alt",
            self.check_img_alt(messages)
        )
    }

    fn check_img_alt(
        &self,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeCanonicalLintRefusal> {
        let mut findings = Vec::new();
        let mut rendered = None;
        let mut pending = None;
        let mut refusal = None;
        // Attached bindings immediately follow their owner in canonical page
        // order. Finish the previous header on the next op, so every binding
        // is inspected through the existing single traversal, never rescanned.
        self.file
            .artifact()
            .visit_nodes(&mut |_, node| {
                if refusal.is_some() {
                    return;
                }
                match node {
                    NodeRef::Op(op) => {
                        finish(&mut pending, &mut findings, &mut rendered, messages);
                        match op {
                            Op::Element(element) => {
                                let policy = ImageAltPolicy::for_tag(element.tag);
                                if !policy.applies() {
                                    return;
                                }
                                if element.namespace != Namespace::Html {
                                    refusal =
                                        Some(NativeCanonicalLintRefusal::UnsupportedImageElement {
                                            span: element.span,
                                        });
                                    return;
                                }
                                pending = Some(PendingImage {
                                    policy,
                                    span: element.span,
                                    has_alt: element.attributes.iter().any(|attribute| {
                                        attribute.name.eq_ignore_ascii_case("alt")
                                    }),
                                });
                            }
                            Op::Component(component) => {
                                // A semantic component name is not authority
                                // for its original tag: `img is='vue:Other'`
                                // no longer has an image-shaped name here.
                                refusal =
                                    Some(NativeCanonicalLintRefusal::UnsupportedImageElement {
                                        span: component.span,
                                    });
                            }
                            Op::Comment(comment) => {
                                refusal = Some(NativeCanonicalLintRefusal::MarkupComment {
                                    span: comment.span,
                                });
                            }
                            _ => {}
                        }
                    }
                    NodeRef::Binding(BindingOp::Bind(binding)) => match binding.name {
                        Some(DynamicName::Static(name)) => {
                            if let Some(image) = pending.as_mut() {
                                image.has_alt |= name.eq_ignore_ascii_case("alt");
                            }
                        }
                        None | Some(DynamicName::Dynamic(_)) => {
                            refusal = Some(NativeCanonicalLintRefusal::IndeterminateAttribute {
                                span: binding.span,
                            });
                        }
                    },
                    _ => {}
                }
            })
            .map_err(|_| NativeCanonicalLintRefusal::NodeLimit)?;
        if let Some(refusal) = refusal {
            return Err(refusal);
        }
        finish(&mut pending, &mut findings, &mut rendered, messages);
        findings.sort_unstable_by_key(|finding| {
            let span = finding.diagnostic().span;
            (span.start, span.end)
        });
        Ok(findings)
    }
}

fn finish(
    pending: &mut Option<PendingImage>,
    findings: &mut Vec<NativeLintFinding>,
    rendered: &mut Option<ImageAltMessages>,
    messages: &impl MessageLookup,
) {
    if let Some(image) = pending.take()
        && image.policy.is_missing(image.has_alt)
    {
        let text = rendered.get_or_insert_with(|| ImageAltMessages::new(messages));
        findings.push(NativeLintFinding {
            rule_name: IMG_ALT_RULE,
            diagnostic: text.diagnostic(image.span),
        });
    }
}
