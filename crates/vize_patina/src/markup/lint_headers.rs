//! Borrowed normal-surface attributes for source-shaped document rules.
//!
//! The normal L1 tree retains the established repairs and frozen spellings.
//! Embedded markup keeps its existing attribute projection; no Relief nodes,
//! expression indexes, value decoders or attribute vectors are constructed.

use super::l2::surface::{offset_in, token_range};
use super::{MarkupDocument, MarkupDocumentInner, MarkupElement};
use crate::ir::ByteRange;
use vize_l1::framework::vue::directive::{DirectivePrefix, DirectiveSyntax, VueDirectives};
use vize_l1::markup::{Attribute, Element, SurfaceChild};

#[derive(Clone, Copy)]
enum HeaderInner<'a> {
    Surface {
        element: &'a Element<'a>,
        source: &'a str,
        frozen: bool,
    },
    Markup(MarkupElement<'a>),
}

/// One borrowed header, never a reconstructed element or attribute vector.
#[derive(Clone, Copy)]
pub(crate) struct LintHeader<'a> {
    inner: HeaderInner<'a>,
}

/// The raw static attribute name and both established report ranges.
pub(crate) struct LintHeaderAttribute<'a> {
    pub(crate) name: &'a str,
    pub(crate) name_range: ByteRange,
    pub(crate) range: ByteRange,
}

impl LintHeader<'_> {
    pub(crate) fn walk_static_attributes(
        &self,
        markup_source: &str,
        visitor: &mut impl FnMut(LintHeaderAttribute<'_>),
    ) {
        match self.inner {
            HeaderInner::Surface {
                element,
                source,
                frozen,
            } => {
                for attribute in &element.open.attrs {
                    let directive = VueDirectives
                        .decompose(attribute.name.text, 0)
                        .ok()
                        .flatten();
                    if (frozen && is_v_pre(attribute.name.text)) || (!frozen && directive.is_some())
                    {
                        continue;
                    }
                    visitor(LintHeaderAttribute {
                        name: attribute.name.text,
                        name_range: token_range(source, &attribute.name),
                        range: attribute_range(source, attribute),
                    });
                }
            }
            HeaderInner::Markup(element) => element.walk_attributes(&mut |attribute| {
                if !attribute.is_plain_static() {
                    return;
                }
                let name_range = attribute.name_range();
                let name =
                    match markup_source.get(name_range.start as usize..name_range.end as usize) {
                        Some(name) if !name.is_empty() => name,
                        _ => attribute.name(),
                    };
                visitor(LintHeaderAttribute {
                    name,
                    name_range,
                    range: attribute.range(),
                });
            }),
        }
    }
}

impl<'a> MarkupDocument<'a> {
    /// Visit the normal L1 attribute plane used by the compatibility linter.
    pub(crate) fn walk_lint_headers(&self, visitor: &mut impl FnMut(LintHeader<'a>)) {
        if let MarkupDocumentInner::L2(doc) = self.inner
            && let Some(surface) = doc.surface
        {
            walk_surface(&surface.children, doc.source, false, visitor);
            return;
        }
        self.walk_tree(
            &mut |element| {
                visitor(LintHeader {
                    inner: HeaderInner::Markup(element),
                })
            },
            &mut |_| {},
        );
    }
}

fn walk_surface<'a>(
    children: &'a [SurfaceChild<'a>],
    source: &'a str,
    inherited_frozen: bool,
    visitor: &mut impl FnMut(LintHeader<'a>),
) {
    for child in children {
        let SurfaceChild::Element(element) = child else {
            continue;
        };
        vize_l0::ensure_sufficient_stack(|| {
            if element.open.is_ignored_start() {
                walk_surface(&element.children, source, inherited_frozen, visitor);
                return;
            }
            let frozen = inherited_frozen
                || element.open.is_verbatim()
                || element
                    .open
                    .attrs
                    .iter()
                    .any(|attribute| is_v_pre(attribute.name.text));
            visitor(LintHeader {
                inner: HeaderInner::Surface {
                    element,
                    source,
                    frozen,
                },
            });
            walk_surface(&element.children, source, frozen, visitor);
        });
    }
}

fn is_v_pre(name: &str) -> bool {
    VueDirectives
        .decompose(name, 0)
        .ok()
        .flatten()
        .is_some_and(|directive| {
            directive.prefix == DirectivePrefix::Full && directive.name.slice(name) == "pre"
        })
}

fn attribute_range(source: &str, attribute: &Attribute<'_>) -> ByteRange {
    let start = offset_in(source, attribute.name.text);
    let end = attribute.value.as_ref().map_or_else(
        || start + attribute.name.text.len() as u32,
        |value| {
            value.close_quote.as_ref().map_or_else(
                || token_range(source, &value.content).end,
                |quote| token_range(source, quote).end,
            )
        },
    );
    ByteRange::new(start, end)
}
