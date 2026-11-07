//! Per-element lowering: attribute analysis, element/component/slot
//! dispatch, and the markup-namespace inheritance.
//!
//! # Namespace, v1 scope (recorded in the P2-8 record)
//!
//! The namespace rule is tag inheritance: SVG/MathML tags open their
//! namespace, and the SVG/MathML HTML integration points
//! (`foreignObject`/`desc`/`title`; `mi`/`mo`/`mn`/`ms`/`mtext`) return
//! their **children** to HTML. This approximates the HTML tree-construction
//! namespace algorithm the way the shipped compiler-dom logic does.

use super::features::OpFamily;
use alloc::vec::Vec as StdVec;

use vize_l0::{Box, String, Vec, cstr, is_native_tag};
use vize_l1::markup::Element;

use vize_l2::op::{Attribute, BindingOp, ComponentOp, ElementOp, Namespace, Op, Region};
use vize_l2::provenance::ProvenanceSink;

pub(super) mod identity;
mod v_pre;
use identity::{children_ns, enter_ns};
pub use v_pre::frozen_attribute_name;

use super::binding::{Owner, lower_attr};
use super::cx::{Cx, attr_slice, attr_span, element_span};
use super::directive::{AttrForm, Head, classify, split_head};
use super::slot::lower_slot;
use super::structural::lower_children;

/// Which branch of a `v-if` chain an element opens or continues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchKind {
    If,
    ElseIf,
    Else,
}

/// One element's attribute analysis, computed once per element.
#[derive(Debug)]
pub(crate) struct Analyzed<'a> {
    /// One form per authored attribute, by index.
    pub forms: StdVec<AttrForm<'a>>,
    /// The first branch directive (attr index, kind), when present.
    pub branch: Option<(usize, BranchKind)>,
    /// The first `v-for` (attr index), when present.
    pub vfor: Option<usize>,
    /// First authored static component selector, inspected during this analysis.
    pub static_is: Option<usize>,
    /// The `v-pre` spelling's attr index, when the element carries one.
    /// Vue drops the attribute itself from the output while keeping every
    /// other directive on the element as a literal attribute.
    pub v_pre: Option<usize>,
    /// Whether this element is the one that *opens* a `v-pre` subtree, as
    /// opposed to sitting inside one. The two freeze their attribute
    /// names differently — see [`frozen_attribute_name`].
    pub opens_v_pre: bool,
}

/// Analyze an element's attributes: classify every name, note the
/// structural directives.
///
/// `in_v_pre` says an ancestor carries `v-pre`; structural spellings then
/// become literal attributes, so no branch or loop is built. Default entries
/// retain the shipped opening-name rewrite and complete inherited names.
/// Only authenticated compiler custody selects literal element ownership and
/// the positional authored-name projection in [`element_core`].
pub(crate) fn analyze<'a>(element: &Element<'a>, in_v_pre: bool) -> Analyzed<'a> {
    let mut forms = StdVec::with_capacity(element.open.attrs.len());
    let mut static_is = None;
    for (index, attr) in element.open.attrs.iter().enumerate() {
        let form = classify(attr.name.text);
        if static_is.is_none() && attr.name.text == "is" && matches!(form, AttrForm::Static) {
            static_is = Some(index);
        }
        forms.push(form);
    }
    let read = structure(element, in_v_pre);
    let mut analyzed = Analyzed {
        forms,
        branch: read.branch,
        vfor: read.vfor,
        static_is,
        v_pre: read.v_pre,
        opens_v_pre: read.opens_v_pre,
    };
    if in_v_pre {
        // Inside the subtree the tokenizer never split the name, so every
        // attribute is already the plain one it was authored as.
        analyzed.freeze_as_authored();
    }
    analyzed
}

/// Whether a later plain attribute replaces this one under the historical
/// parser's case-insensitive attribute-map semantics.
pub(super) fn static_attribute_is_shadowed(
    cx: &Cx<'_, impl ProvenanceSink>,
    element: &Element<'_>,
    analyzed: &Analyzed<'_>,
    index: usize,
) -> bool {
    if !cx.caps.duplicate_static_attrs_keep_last {
        return false;
    }
    let Some(attr) = element.open.attrs.get(index) else {
        return false;
    };
    element
        .open
        .attrs
        .iter()
        .zip(&analyzed.forms)
        .skip(index + 1)
        .any(|(later, form)| {
            matches!(form, AttrForm::Static) && later.name.text.eq_ignore_ascii_case(attr.name.text)
        })
}

/// The structural reading the lowering gives one element's attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Structure {
    /// The first branch directive (attribute index, kind), when present.
    pub branch: Option<(usize, BranchKind)>,
    /// The first `v-for` (attribute index), when present.
    pub vfor: Option<usize>,
    /// Whether the element opens a `v-pre` subtree.
    pub opens_v_pre: bool,
    /// Whether the element carries a `v-slot` / `#` spelling.
    pub slot_spelling: bool,
    /// The opening `v-pre` spelling's attribute index.
    v_pre: Option<usize>,
}

impl Structure {
    /// The branch directive's attribute and kind, when one is authored.
    pub fn branch_attr<'t, 'a>(
        &self,
        element: &'t Element<'a>,
    ) -> Option<(&'t vize_l1::markup::Attribute<'a>, BranchKind)> {
        let (index, kind) = self.branch?;
        Some((element.open.attrs.get(index)?, kind))
    }

    /// Whether the lowering unwraps `element` into its scope's region.
    /// `v-for` decides first: a loop with a value unwraps its template; a
    /// blank one builds no loop. Otherwise a chain branch unwraps.
    pub fn unwraps(&self, element: &Element<'_>, in_chain: bool) -> bool {
        match self.vfor {
            Some(index) => {
                element
                    .open
                    .attrs
                    .get(index)
                    .and_then(super::forop::v_for_value)
                    .is_some()
                    && template_unwraps(element, self.slot_spelling)
            }
            None => in_chain && template_unwraps(element, self.slot_spelling),
        }
    }
}

/// Whether a loop or branch carrier lowers into its scope's region: a
/// `<template>` without a `v-slot` / `#` spelling, which must keep its op
/// so the slot name survives.
pub(crate) fn template_unwraps(element: &Element<'_>, slot_spelling: bool) -> bool {
    element.tag() == "template" && !slot_spelling
}

/// The structural reading of `element`; `in_v_pre` when an ancestor
/// carries `v-pre`. Reads only directive heads.
///
/// Inside a `v-pre` subtree every attribute is the plain one it was
/// authored as. On the element that opens the subtree the structural
/// directives are forgotten while its other spellings stand.
pub fn structure(element: &Element<'_>, in_v_pre: bool) -> Structure {
    let none = Structure {
        branch: None,
        vfor: None,
        opens_v_pre: false,
        slot_spelling: false,
        v_pre: None,
    };
    if in_v_pre {
        return none;
    }
    let mut read = none;
    for (index, attr) in element.open.attrs.iter().enumerate() {
        let Some(split) = split_head(attr.name.text) else {
            continue;
        };
        let kind = match split.head {
            Head::If => Some(BranchKind::If),
            Head::ElseIf => Some(BranchKind::ElseIf),
            Head::Else => Some(BranchKind::Else),
            _ => None,
        };
        if let Some(kind) = kind
            && read.branch.is_none()
        {
            read.branch = Some((index, kind));
        }
        match split.head {
            Head::For if read.vfor.is_none() => read.vfor = Some(index),
            Head::Pre if read.v_pre.is_none() => read.v_pre = Some(index),
            Head::Slot => read.slot_spelling = true,
            _ => {}
        }
    }
    if read.v_pre.is_some() {
        // On the opening element the tokenizer had not yet entered
        // `v-pre` when it read these attributes, so they arrive split and
        // the element lowering rebuilds each name from the parts.
        read.opens_v_pre = true;
        read.branch = None;
        read.vfor = None;
    }
    read
}

impl<'a> Analyzed<'a> {
    /// The branch directive's attribute and kind, when one is authored.
    pub(crate) fn branch_attr<'t>(
        &self,
        element: &'t Element<'a>,
    ) -> Option<(&'t vize_l1::markup::Attribute<'a>, BranchKind)> {
        let (index, kind) = self.branch?;
        Some((element.open.attrs.get(index)?, kind))
    }

    /// Whether the element carries a `v-slot` / `#` spelling. A
    /// `<template v-if #name>` / `<template v-for #name>` must keep the
    /// template op so the slot name survives unwrap (P2-11 createSlots).
    pub(crate) fn has_slot_spelling(&self) -> bool {
        self.forms.iter().any(
            |form| matches!(form, AttrForm::Directive(directive) if directive.head == Head::Slot),
        )
    }

    /// Re-read every attribute as the plain one it was authored as, and
    /// forget the structural directives — the reading for an element
    /// *inside* a `v-pre` subtree.
    ///
    /// [`Analyzed::v_pre`] stays `None` here even when one of these is a
    /// `v-pre` spelling: only the element that *opens* the subtree drops
    /// its spelling. A nested `<span v-pre>` is already frozen, so its
    /// `v-pre` is an ordinary attribute and Vue emits it as one.
    fn freeze_as_authored(&mut self) {
        for form in self.forms.iter_mut() {
            if matches!(form, AttrForm::Directive(_)) {
                *form = AttrForm::Static;
            }
        }
        self.branch = None;
        self.vfor = None;
    }
}

/// The authored value text of an attribute, when it has a value node
/// (a `Missing` value hole is a zero-width slice, present but empty).
pub(crate) fn attr_value_text<'a>(element: &Element<'a>, index: usize) -> Option<&'a str> {
    attr_text(element.open.attrs.get(index)?)
}

/// [`attr_value_text`] for an attribute already in hand.
pub(crate) fn attr_text<'a>(attr: &vize_l1::markup::Attribute<'a>) -> Option<&'a str> {
    attr.value.as_ref().map(|value| value.content.text)
}

/// Lower one element (its structural directives already consumed by the
/// caller): `<slot>` outlet, component, or native element.
pub(crate) fn element_core<'a>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    element: &Element<'a>,
    analyzed: &Analyzed<'a>,
    ns: Namespace,
) -> Op<'a> {
    cx.report_missing_close(element);
    let authored_tag = element.tag();
    let cast = identity::vue_is(cx, element, analyzed);
    let tag = cast.map_or(authored_tag, |(_, name)| name);
    let in_v_pre = cx.v_pre_suppressed() || analyzed.opens_v_pre;
    let frozen = cx.frozen_literal(element, in_v_pre);
    if authored_tag == "slot" && !frozen {
        return lower_slot(cx, element, analyzed, ns);
    }
    let own_ns = enter_ns(ns, authored_tag);
    let child_ns = children_ns(own_ns, authored_tag);
    let span = element_span(cx, element);
    let node = cx.mint_op();
    let component =
        !frozen && (cast.is_some() || (!is_native_tag(tag) && !cx.is_custom_element(tag)));

    let open_end = cx.token_span(&element.open.gt).end;
    let open_slice = cx
        .source
        .get(span.start as usize..open_end as usize)
        .unwrap_or(tag);
    if component {
        cx.observe(OpFamily::SlotCarrier);
        cx.record(
            "lower.component",
            node,
            open_slice,
            || cstr!("ui.component {tag}"),
            span,
        );
    } else {
        let after = || match own_ns {
            Namespace::Html => cstr!("ui.element {tag}"),
            Namespace::Svg => cstr!("ui.element {tag} ns=svg"),
            Namespace::MathMl => cstr!("ui.element {tag} ns=mathml"),
        };
        cx.record("lower.element", node, open_slice, after, span);
    }

    let take_scope = super::sugar::should_take(cx, element, analyzed);
    let scope_index = take_scope
        .then(|| super::sugar::scope_attr_index(element, analyzed))
        .flatten();
    let companion_slot = take_scope
        .then(|| super::sugar::companion_slot_index(element, analyzed))
        .flatten();

    let mut attributes: Vec<'a, Attribute<'a>> = Vec::new_in(&cx.allocator);
    let mut bindings: Vec<'a, BindingOp<'a>> = Vec::new_in(&cx.allocator);
    for (index, (attr, form)) in element.open.attrs.iter().zip(&analyzed.forms).enumerate() {
        if Some(index) == analyzed.branch.map(|(idx, _)| idx) || Some(index) == analyzed.vfor {
            continue;
        }
        if cast.is_some_and(|(ordinal, _)| ordinal == index) {
            identity::consume(cx, attr);
            continue;
        }
        if Some(index) == companion_slot {
            continue;
        }
        if Some(index) == analyzed.v_pre {
            // The spelling itself leaves the output — Vue emits the
            // frozen attributes without it — but the bytes are still
            // accounted for, like any other dropped node.
            cx.record(
                "drop.v-pre",
                None,
                attr_slice(cx, attr),
                String::default,
                attr_span(cx, attr),
            );
            continue;
        }
        if matches!(form, AttrForm::Static)
            && static_attribute_is_shadowed(cx, element, analyzed, index)
        {
            continue;
        }
        match form {
            AttrForm::Static if Some(index) == scope_index => {
                bindings.push(super::sugar::lower_slot_scope(
                    cx,
                    element,
                    attr,
                    companion_slot,
                ));
            }
            AttrForm::Static => attributes.push(Attribute {
                name: if frozen && cx.v_pre_suppressed() {
                    v_pre::literal_attribute_name(cx.allocator, attr.name.text)
                } else {
                    attr.name.text
                },
                value: attr.value.as_ref().map(|value| value.content.text),
                span: attr_span(cx, attr),
            }),
            AttrForm::Directive(directive) if analyzed.opens_v_pre => {
                attributes.push(Attribute {
                    name: if !frozen {
                        v_pre::frozen_name(cx.allocator, attr.name.text, directive)
                    } else if analyzed.v_pre.is_some_and(|ordinal| index < ordinal) {
                        attr.name.text
                    } else {
                        v_pre::literal_attribute_name(cx.allocator, attr.name.text)
                    },
                    value: attr.value.as_ref().map(|value| value.content.text),
                    span: attr_span(cx, attr),
                });
            }
            AttrForm::Directive(directive) => {
                let owner = Owner { tag, component };
                lower_attr(cx, attr, directive, &owner, &mut bindings);
            }
        }
    }

    // `<pre>` keeps its bytes: condensing is suppressed for the whole
    // subtree (`lower::text`, the shipped `is_pre_tag` configuration).
    let suppress = super::text::suppresses_condense(authored_tag);
    // `analyze` has already rewritten the forms, so ask the recorded
    // spelling rather than the (now empty) directive list.
    let suppress_v_pre = analyzed.v_pre.is_some();
    if suppress {
        cx.push_condense_suppression();
    }
    if suppress_v_pre {
        cx.push_v_pre_suppression();
    }
    let previous_text_parent =
        cx.enter_text_parent(element, authored_tag, own_ns == Namespace::Html);
    let children = Region {
        ops: if component || own_ns != Namespace::Html {
            lower_children(cx, &element.children, child_ns)
        } else {
            super::table::lower_element_children(cx, tag, &element.children, child_ns)
        },
    };
    (cx.text_whitespace, cx.ignore_newline_at) = previous_text_parent;
    if suppress_v_pre {
        cx.pop_v_pre_suppression();
    }
    if suppress {
        cx.pop_condense_suppression();
    }
    if component {
        Op::Component(Box::new_in(
            ComponentOp {
                name: tag,
                attributes,
                bindings,
                children,
                span,
            },
            &cx.allocator,
        ))
    } else {
        Op::Element(Box::new_in(
            ElementOp {
                tag,
                namespace: own_ns,
                attributes,
                bindings,
                children,
                span,
            },
            &cx.allocator,
        ))
    }
}
