//! Complete authored callback traces, independent of the checked-view producer.

use super::{AttributeObservation, span};
use vize_l0::String;
use vize_l1::markup::{ArgSyntax, DirectiveName, DirectivePrefix};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Root(&'static str),
    Element {
        marker: &'static str,
        tag: String,
        ordinal: usize,
        parent: Option<String>,
    },
    Attribute {
        marker: &'static str,
        observation: AttributeObservation,
    },
}

pub fn element_event(
    marker: &'static str,
    tag: &str,
    ordinal: usize,
    parent: Option<&str>,
) -> Event {
    Event::Element {
        marker,
        tag: tag.into(),
        ordinal,
        parent: parent.map(Into::into),
    }
}

pub fn static_event(
    marker: &'static str,
    source: &str,
    ordinal: usize,
    name: &str,
    authored: &str,
    value: Option<&str>,
) -> Event {
    Event::Attribute {
        marker,
        observation: AttributeObservation {
            ordinal,
            name: name.into(),
            value: value.map(Into::into),
            range: span(source, authored),
            head: None,
            binding: "static",
            argument: None,
        },
    }
}

/// These controls author a Bind shorthand with no modifiers. Argument syntax
/// and ranges are explicit golden inputs; no native decomposition is reused.
pub fn binding_event(
    marker: &'static str,
    source: &str,
    ordinal: usize,
    head: &str,
    authored: &str,
    value: &str,
    argument: ArgSyntax,
) -> Event {
    let head_span = span(source, head);
    let (binding, argument_range) = match argument {
        ArgSyntax::Static(range) => ("bind", range),
        ArgSyntax::Dynamic(range) => ("dynamic-bind", range),
    };
    Event::Attribute {
        marker,
        observation: AttributeObservation {
            ordinal,
            name: head.into(),
            value: Some(value.into()),
            range: span(source, authored),
            head: Some(DirectiveName {
                prefix: DirectivePrefix::Bind,
                name: vize_l0::Span::new(head_span.start, head_span.start),
                arg: Some(argument),
                modifiers: vize_l0::Span::new(head_span.end, head_span.end),
            }),
            binding,
            argument: Some(argument_range),
        },
    }
}
