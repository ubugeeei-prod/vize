//! `<slot>` outlets recovered from the lossless L1 template.
//!
//! A recovered outlet creates a finding, so comments, text and `v-pre`
//! subtrees are excluded structurally. A dynamic `:name` is skipped while
//! statically named sibling outlets remain checkable.

use super::super::template_surface::{
    DirectiveArgument, directive, element_span, element_verbatim, static_attribute_value,
};
use vize_l0::CompactString;
use vize_l1::markup::{Element, SurfaceChild, SurfaceTree};

/// A `<slot>` outlet and its complete element range, relative to the template.
pub(super) struct RenderedSlot {
    pub(super) name: CompactString,
    pub(super) start: u32,
    pub(super) end: u32,
}

/// Every `<slot>` outlet the template renders.
pub(super) enum RenderedSlots {
    Known(Vec<RenderedSlot>),
}

pub(super) fn collect_rendered_slots(root: &SurfaceTree<'_>) -> RenderedSlots {
    let mut collector = SlotCollector {
        root,
        slots: Vec::new(),
    };
    collector.walk(&root.children, false);
    RenderedSlots::Known(collector.slots)
}

struct SlotCollector<'a> {
    root: &'a SurfaceTree<'a>,
    slots: Vec<RenderedSlot>,
}

impl SlotCollector<'_> {
    fn walk(&mut self, children: &[SurfaceChild<'_>], inherited_verbatim: bool) {
        for child in children {
            if let SurfaceChild::Element(element) = child {
                self.visit_element(element, inherited_verbatim);
            }
        }
    }

    fn visit_element(&mut self, element: &Element<'_>, inherited_verbatim: bool) {
        let verbatim = element_verbatim(element, inherited_verbatim);
        if !verbatim && element.tag() == "slot" {
            self.record(element);
        }
        self.walk(&element.children, verbatim);
    }

    fn record(&mut self, element: &Element<'_>) {
        let mut name = None;
        for attribute in &element.open.attrs {
            match directive(self.root, attribute, false) {
                None if attribute.name.text == "name" => {
                    if let Some(value) = static_attribute_value(self.root, attribute) {
                        name = Some(CompactString::new(value.text()));
                    }
                }
                Some(directive)
                    if directive.name == "bind"
                        && directive.argument == Some(DirectiveArgument::Static("name")) =>
                {
                    let Some(static_name) = directive
                        .value
                        .as_ref()
                        .and_then(|value| static_string_expression(value.text()))
                    else {
                        return;
                    };
                    name = Some(static_name);
                }
                _ => {}
            }
        }
        let Some(span) = element_span(self.root.source, element) else {
            return;
        };
        self.slots.push(RenderedSlot {
            name: name.unwrap_or_else(|| CompactString::new("default")),
            start: span.start,
            end: span.end,
        });
    }
}

fn static_string_expression(source: &str) -> Option<CompactString> {
    let source = source.trim();
    let quote = *source.as_bytes().first()?;
    if source.len() < 2
        || !matches!(quote, b'\'' | b'"' | b'`')
        || source.as_bytes().last() != Some(&quote)
    {
        return None;
    }

    let value = source.get(1..source.len() - 1)?;
    let is_unambiguous_literal = !value.as_bytes().contains(&b'\\')
        && !value.as_bytes().contains(&quote)
        && (quote == b'`' || !value.bytes().any(|byte| byte.is_ascii_control()))
        && (quote != b'`' || !value.contains("${"));
    is_unambiguous_literal.then(|| CompactString::new(value))
}
