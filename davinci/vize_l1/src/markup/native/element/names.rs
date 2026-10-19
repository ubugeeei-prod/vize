//! Authored tag names borrowed from one original Element CST projection.

use super::{NativeComponent, NativeElement, Parent};
use crate::{Element, ElementClose, SurfaceChild, Token};
use vize_l0::{SourceBlock, Span};

/// The original construction outcome, without an invented closing name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeElementClosingName {
    Present(Span),
    Missing,
    Implicit,
    SelfClosing,
    Void,
}

/// No source or completed grammar is inferred from recovered syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeElementNameRefusal {
    Recovered { offset: u32 },
    UnsupportedComponent,
    IncompleteOpening,
    IncompleteClosing,
    SourceMismatch,
}

/// Exact original Component, Element and direct-child identity remain borrowed.
/// Spans are authored UTF-8 bytes and exclude tag delimiters and gap bytes.
/// This is lexical CST custody, not namespace, runtime or File admission.
///
/// Raw elements and source coordinates cannot mint this receipt:
/// ```compile_fail
/// use vize_l1::markup::NativeElementNames;
/// fn forge() { let _ = NativeElementNames { opening: Default::default() }; }
/// ```
/// The actual original owner must remain alive:
/// ```compile_fail
/// use vize_l0::{Allocator, SourceRoot};
/// use vize_l1::markup::NativeComponent;
/// let arena = Allocator::default();
/// let owner = NativeComponent::parse_in(&arena, SourceRoot::new("<p></p>").unwrap().whole_block()).unwrap();
/// let element = owner.children().next().unwrap().into_element().unwrap();
/// let names = element.names().unwrap();
/// drop(owner);
/// let _ = names.opening();
/// ```
pub struct NativeElementNames<'o, 'a> {
    component: &'o NativeComponent<'a>,
    element: &'o Element<'a>,
    parent: Parent<'o, 'a>,
    ordinal: usize,
    opening: Span,
    closing: NativeElementClosingName,
}

impl core::fmt::Debug for NativeElementNames<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NativeElementNames")
            .field("opening", &self.opening)
            .field("closing", &self.closing)
            .finish_non_exhaustive()
    }
}

impl<'o, 'a> NativeElementNames<'o, 'a> {
    #[must_use]
    pub fn component(&self) -> &'o NativeComponent<'a> {
        self.component
    }
    #[must_use]
    pub fn element(&self) -> &'o Element<'a> {
        self.element
    }
    #[must_use]
    pub fn block(&self) -> SourceBlock<'a> {
        self.component.block()
    }
    #[must_use]
    pub const fn opening(&self) -> Span {
        self.opening
    }
    #[must_use]
    pub const fn closing(&self) -> NativeElementClosingName {
        self.closing
    }
    /// Rejoin the same actual element; equal source buffers/ordinals do not suffice.
    #[must_use]
    pub fn accepts(&self, element: &NativeElement<'_, 'a>) -> bool {
        core::ptr::eq(self.component, element.component)
            && core::ptr::eq(self.element, element.element)
            && self.ordinal == element.ordinal
            && match (self.parent, element.parent) {
                (Parent::Root, Parent::Root) => true,
                (Parent::Element(left), Parent::Element(right)) => core::ptr::eq(left, right),
                _ => false,
            }
    }
}

impl<'o, 'a> NativeElement<'o, 'a> {
    /// Project the original CST at this existing child visit, without a walk,
    /// parser, source scan, allocation or stored per-node name record.
    ///
    /// Only `ElementClose::Present` grants a second name. Its original builder
    /// matched the authored end-tag event; its name is the token suffix of the
    /// opening name's byte length. Existing ASCII-insensitive matching preserves
    /// that length, including non-ASCII bytes. Delimiter/gap bytes are excluded
    /// without rescanning them or changing the original matching policy.
    /// Missing/implicit/void/self-closing outcomes remain distinct. Every
    /// original spelling survives; no namespace or dialect case fold is added.
    pub fn names(&self) -> Result<NativeElementNames<'o, 'a>, NativeElementNameRefusal> {
        let component = self.component;
        let block = component.block();
        let carrier = component.carrier();
        if let Some(error) = carrier.errors.first() {
            return Err(NativeElementNameRefusal::Recovered {
                offset: block
                    .start()
                    .checked_add(error.offset)
                    .ok_or(NativeElementNameRefusal::SourceMismatch)?,
            });
        }
        if carrier.authored.is_some() || !carrier.unsupported.is_empty() {
            return Err(NativeElementNameRefusal::UnsupportedComponent);
        }
        let original = match self.parent {
            Parent::Root => component.root_child_at(self.ordinal),
            Parent::Element(parent) => parent.children.get(self.ordinal),
        };
        if !core::ptr::eq(carrier.tree.source, block.source())
            || !matches!(original, Some(SurfaceChild::Element(element))
                if core::ptr::eq(element.as_ref(), self.element))
        {
            return Err(NativeElementNameRefusal::SourceMismatch);
        }
        let open = &self.element.open;
        if open.lt_name.is_missing() || open.gt.is_missing() {
            return Err(NativeElementNameRefusal::IncompleteOpening);
        }
        let opening_token = token_span(block, &open.lt_name)?;
        let opening = open
            .lt_name
            .text
            .strip_prefix('<')
            .filter(|name| !name.is_empty())
            .ok_or(NativeElementNameRefusal::SourceMismatch)?;
        let opening = block
            .span_of(opening)
            .filter(|span| block.contains_block_span(*span))
            .ok_or(NativeElementNameRefusal::SourceMismatch)?;
        let open_gt = token_span(block, &open.gt)?;
        if opening.start != opening_token.start + 1
            || open.gt.text != ">"
            || opening.end > open_gt.start
        {
            return Err(NativeElementNameRefusal::SourceMismatch);
        }
        let closing = match &self.element.close {
            ElementClose::Missing => NativeElementClosingName::Missing,
            ElementClose::Implicit => NativeElementClosingName::Implicit,
            ElementClose::NotExpected if open.slash.is_some() => {
                let slash = open
                    .slash
                    .as_ref()
                    .ok_or(NativeElementNameRefusal::SourceMismatch)?;
                let span = token_span(block, slash)?;
                if slash.is_missing() || slash.text != "/" || span.end > open_gt.start {
                    return Err(NativeElementNameRefusal::IncompleteOpening);
                }
                NativeElementClosingName::SelfClosing
            }
            ElementClose::NotExpected => NativeElementClosingName::Void,
            ElementClose::Present(close) => {
                if close.lt_slash_name.is_missing() || close.gt.is_missing() {
                    return Err(NativeElementNameRefusal::IncompleteClosing);
                }
                let token = token_span(block, &close.lt_slash_name)?;
                let gt = token_span(block, &close.gt)?;
                let start = close
                    .lt_slash_name
                    .text
                    .len()
                    .checked_sub((opening.end - opening.start) as usize)
                    .filter(|start| *start >= 2)
                    .ok_or(NativeElementNameRefusal::SourceMismatch)?;
                let name = close
                    .lt_slash_name
                    .text
                    .get(start..)
                    .ok_or(NativeElementNameRefusal::SourceMismatch)?;
                let span = block
                    .span_of(name)
                    .filter(|span| block.contains_block_span(*span))
                    .ok_or(NativeElementNameRefusal::SourceMismatch)?;
                if !close.lt_slash_name.text.starts_with("</")
                    || close.gt.text != ">"
                    || span.end != token.end
                    || token.start < open_gt.end
                    || span.end > gt.start
                {
                    return Err(NativeElementNameRefusal::SourceMismatch);
                }
                NativeElementClosingName::Present(span)
            }
        };
        Ok(NativeElementNames {
            component,
            element: self.element,
            parent: self.parent,
            ordinal: self.ordinal,
            opening,
            closing,
        })
    }
}

fn token_span(block: SourceBlock<'_>, token: &Token<'_>) -> Result<Span, NativeElementNameRefusal> {
    block
        .span_of(token.text)
        .filter(|span| block.contains_block_span(*span))
        .ok_or(NativeElementNameRefusal::SourceMismatch)
}

#[cfg(test)]
mod tests {
    use super::{NativeElementClosingName as Closing, NativeElementNameRefusal as Refusal};
    use crate::container::{Vue, vue::DescriptorOptions};
    use crate::markup::{NativeComponent, NativeTemplateComponent};
    use crate::{ElementClose, check_fidelity};
    use vize_l0::{
        Allocator, SourceRoot, Span,
        config::{VueDialect, VueVersion},
        line_index::LineBreaks,
    };

    fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
        let descriptor = Vue.observe_descriptor(
            arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: crate::SurfaceParseOptions::default(),
            },
        );
        NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
            .unwrap()
            .unwrap()
    }

    #[test]
    fn original_nested_duplicate_names_keep_parent_ordinal_and_token_order() {
        let arena = Allocator::default();
        let source = "<Card><Card title=\"</Card>\">x</Card></Card>";
        let owner =
            NativeComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block())
                .unwrap();
        let outer = owner.children().next().unwrap().into_element().unwrap();
        let inner = outer.children().next().unwrap().into_element().unwrap();
        let outer_names = outer.names().unwrap();
        let inner_names = inner.names().unwrap();
        assert_eq!(outer_names.opening(), Span::new(1, 5));
        assert_eq!(outer_names.closing(), Closing::Present(Span::new(38, 42)));
        assert_eq!(inner_names.opening(), Span::new(7, 11));
        assert_eq!(inner_names.closing(), Closing::Present(Span::new(31, 35)));
        assert!(outer_names.accepts(&outer));
        assert!(inner_names.accepts(&inner));
        assert!(!outer_names.accepts(&inner));
        assert!(!inner_names.accepts(&outer));
        assert_eq!(check_fidelity(&owner.carrier().tree), Ok(()));
    }

    #[test]
    fn selected_full_source_utf8_spans_exclude_attribute_script_and_gap_lookalikes() {
        let arena = Allocator::default();
        let source = "<!--😀-->\r\n<template>\r\n<Card-é title=\"</Card-é>\">😀</Card-é>\r\n</template><script>const fake=\"<Card-é></Card-é>\"</script>";
        let owner = selected(&arena, source);
        let element = owner.children().nth(1).unwrap().into_element().unwrap();
        let names = element.names().unwrap();
        assert_eq!(names.opening(), Span::new(26, 33));
        assert_eq!(names.closing(), Closing::Present(Span::new(59, 66)));
        assert_eq!(source.get(26..33), Some("Card-é"));
        assert_eq!(source.get(59..66), Some("Card-é"));
        assert_eq!(LineBreaks::Lsp.offset_to_position(source, 26), (2, 1));
        assert_eq!(LineBreaks::Lsp.offset_to_position(source, 59), (2, 30));
        assert!(core::ptr::eq(names.block().root_source(), source));
        assert!(core::ptr::eq(names.component(), owner.component()));
        assert!(core::ptr::eq(names.element(), element.surface()));
    }

    #[test]
    fn original_closing_token_preserves_case_and_excludes_accepted_gap_bytes() {
        let arena = Allocator::default();
        for source in ["<Widget></wIDGET>", "<Widget></ Widget >"] {
            let owner =
                NativeComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block())
                    .unwrap();
            let element = owner.children().next().unwrap().into_element().unwrap();
            let names = element.names().unwrap();
            let Closing::Present(close) = names.closing() else {
                panic!("original present closing tag");
            };
            assert_eq!(source.get(1..7), Some("Widget"));
            assert_eq!(names.opening(), Span::new(1, 7));
            assert_eq!(
                source.get(close.start as usize..close.end as usize),
                Some(if source.contains("wIDGET") {
                    "wIDGET"
                } else {
                    "Widget"
                })
            );
            assert_eq!(check_fidelity(&owner.carrier().tree), Ok(()));
        }
    }

    #[test]
    fn equal_original_buffer_parses_cannot_rejoin_foreign_element_names() {
        let arena = Allocator::default();
        let source = "<p></p>";
        let block = SourceRoot::new(source).unwrap().whole_block();
        let owner = NativeComponent::parse_in(&arena, block).unwrap();
        let foreign = NativeComponent::parse_in(&arena, block).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let other = foreign.children().next().unwrap().into_element().unwrap();
        let names = element.names().unwrap();
        assert_eq!(names.opening(), other.names().unwrap().opening());
        assert_eq!(names.closing(), other.names().unwrap().closing());
        assert!(names.accepts(&element));
        assert!(!names.accepts(&other));
        // Even a dev-only fabricated direct tuple cannot bypass the original slot.
        let fabricated = super::NativeElement::from_child(
            &owner,
            super::Parent::Root,
            other.ordinal(),
            other.surface(),
        );
        assert_eq!(fabricated.names().unwrap_err(), Refusal::SourceMismatch);
    }

    #[test]
    fn equal_text_from_distinct_source_allocations_never_grants_original_membership() {
        let arena = Allocator::default();
        let source = vize_l0::String::from("<p></p>");
        let other_source = vize_l0::String::from("<p></p>");
        let owner =
            NativeComponent::parse_in(&arena, SourceRoot::new(&source).unwrap().whole_block())
                .unwrap();
        let foreign = NativeComponent::parse_in(
            &arena,
            SourceRoot::new(&other_source).unwrap().whole_block(),
        )
        .unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let other = foreign.children().next().unwrap().into_element().unwrap();
        assert!(!element.names().unwrap().accepts(&other));
    }

    #[test]
    fn void_and_self_closing_originals_never_invent_a_paired_name() {
        let arena = Allocator::default();
        let source = "<br><br/><Widget />";
        let owner =
            NativeComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block())
                .unwrap();
        let names: alloc::vec::Vec<_> = owner
            .children()
            .map(|child| child.into_element().unwrap().names().unwrap())
            .collect();
        assert_eq!(names[0].opening(), Span::new(1, 3));
        assert_eq!(names[0].closing(), Closing::Void);
        assert_eq!(names[1].closing(), Closing::SelfClosing);
        assert_eq!(names[2].closing(), Closing::SelfClosing);
    }

    #[test]
    fn original_missing_and_implicit_closes_remain_distinct_from_present_tokens() {
        let arena = Allocator::default();
        for (source, expected) in [
            ("<p>text", Closing::Missing),
            ("<a><a>inner</a></a>", Closing::Implicit),
        ] {
            let owner =
                NativeComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block())
                    .unwrap();
            let element = owner.children().next().unwrap().into_element().unwrap();
            assert_eq!(element.names().unwrap().closing(), expected);
            assert_eq!(check_fidelity(&owner.carrier().tree), Ok(()));
        }
    }

    #[test]
    fn recovered_original_component_refuses_all_names_without_partial_pairing() {
        let arena = Allocator::default();
        let source = "<p></p><div title='unterminated";
        let owner =
            NativeComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block())
                .unwrap();
        let error = owner.carrier().errors.first().unwrap();
        let expected = Refusal::Recovered {
            offset: error.offset,
        };
        for child in owner.children() {
            if let Some(element) = child.into_element() {
                assert_eq!(element.names().unwrap_err(), expected);
            }
        }
        assert_eq!(check_fidelity(&owner.carrier().tree), Ok(()));
    }

    #[test]
    fn original_verbatim_body_keeps_lexical_names_without_semantic_admission() {
        let arena = Allocator::default();
        let source =
            "<template><div v-pre><Widget :x='{{not syntax}}'>{{text}}</Widget></div></template>";
        let owner = selected(&arena, source);
        let root = owner.children().next().unwrap().into_element().unwrap();
        let element = root.children().next().unwrap().into_element().unwrap();
        assert!(element.surface().open.is_verbatim());
        let names = element.names().unwrap();
        assert!(matches!(names.closing(), Closing::Present(_)));
        assert!(names.accepts(&element));
        assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
    }

    #[test]
    fn owner_move_reborrow_and_query_order_preserve_original_names_without_arena_growth() {
        let arena = Allocator::default();
        let source = "<p></p><Widget></Widget>";
        let owner =
            NativeComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block())
                .unwrap();
        let owner = core::hint::black_box(owner);
        let before = arena.allocated_bytes();
        let (original, names) = {
            let first = owner.children().next().unwrap().into_element().unwrap();
            (first.surface(), first.names().unwrap())
        };
        let second = owner.children().nth(1).unwrap().into_element().unwrap();
        let next = second.names().unwrap();
        for _ in 0..16 {
            let projection = owner.children().next().unwrap().into_element().unwrap();
            let repeated = projection.names().unwrap();
            assert!(names.accepts(&projection));
            assert_eq!(names.opening(), repeated.opening());
            assert_eq!(names.closing(), repeated.closing());
            assert!(!next.accepts(&projection));
            assert!(core::ptr::eq(names.element(), original));
        }
        assert_eq!(arena.allocated_bytes(), before);
    }

    #[test]
    fn actual_ancestor_close_leaves_missing_child_without_claiming_its_end_tag() {
        let arena = Allocator::default();
        let source = "<div><p>text</div>";
        let owner =
            NativeComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block())
                .unwrap();
        let root = owner.children().next().unwrap().into_element().unwrap();
        let child = root.children().next().unwrap().into_element().unwrap();
        assert!(matches!(child.surface().close, ElementClose::Missing));
        assert_eq!(child.names().unwrap().closing(), Closing::Missing);
        assert_eq!(
            root.names().unwrap().closing(),
            Closing::Present(Span::new(14, 17))
        );
    }

    #[test]
    fn original_unsupported_directive_observation_cannot_grant_name_admission() {
        let arena = Allocator::default();
        let source = "<template><div v-pre:[([([([([([([([([([([([([([([([([([([([([([([([([([([([([([([([([key])])])])])])])])])])])])])])])])])])])])])])])])])])])])])])])])]>body</div></template>";
        let owner = selected(&arena, source);
        assert_eq!(owner.component().carrier().unsupported.len(), 1);
        let element = owner.children().next().unwrap().into_element().unwrap();
        assert_eq!(element.names().unwrap_err(), Refusal::UnsupportedComponent);
        assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
    }
}
