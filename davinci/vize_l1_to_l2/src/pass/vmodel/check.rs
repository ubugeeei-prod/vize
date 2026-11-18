//! The two live-lane `v-model` validations, in the live lane's order.

use alloc::vec::Vec as StdVec;

use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::diag::{Diagnostic, Stage};
use vize_l0::id::NodeId;
use vize_l0::{Allocator, String};
use vize_l2::op::ModelOp;
use vize_l2::provenance::ProvenanceSink;

use super::{
    ARG_ON_ELEMENT_MESSAGE, Channels, ModelFacts, ModelFault, ON_SCOPE_MESSAGE, ScopeName,
};

/// The two legacy checks, in the legacy order, first failure wins.
pub(super) fn check_model(
    channels: &mut Channels<'_, impl ProvenanceSink>,
    env: &StdVec<ScopeName<'_>>,
    id: Option<NodeId>,
    model: &ModelOp<'_>,
) {
    let read = model.contract.read.source();
    let allocator = channels.allocator;
    let (fault, message, rule) = if env.iter().any(|scope| declares(allocator, scope, read)) {
        (
            ModelFault::OnScope,
            ON_SCOPE_MESSAGE,
            "error.v-model-on-scope",
        )
    } else {
        let component = model.attributes.iter().any(|attribute| {
            attribute.name == "element-kind" && attribute.value == Some("component")
        });
        let argument = model.argument.is_some();
        if component || !argument {
            return;
        }
        (
            ModelFault::ArgOnElement,
            ARG_ON_ELEMENT_MESSAGE,
            "error.v-model-arg-on-element",
        )
    };
    channels.diagnostics.push(Diagnostic::legacy_error(
        &crate::exemptions::V_MODEL,
        Stage::Semantic,
        model.span,
        String::from(message),
    ));
    channels
        .provenance
        .record(|| vize_l2::provenance::ProvenanceRecord {
            rule: String::from(rule),
            node: id,
            before: String::from(read),
            after: String::default(),
            span: model.span,
        });
    if let Some(id) = id {
        channels.facts.insert(id, ModelFacts { fault });
    }
}

/// Whether a scope frame entry declares `read`.
fn declares(allocator: &Allocator, scope: &ScopeName<'_>, read: &str) -> bool {
    match scope {
        ScopeName::Text(name) => name.as_str() == read,
        ScopeName::Pattern(pattern) => pattern_declares(allocator, pattern, read),
    }
}

/// Whether the slot-props `pattern`, parsed as the slot's parameter goal,
/// declares `name` (the lowering's own walk: nested, default-valued and
/// rest positions). A pattern that does not parse declares nothing.
fn pattern_declares(allocator: &Allocator, pattern: &str, name: &str) -> bool {
    // A declared name is spelled in the pattern unless it is escaped, so
    // the parse runs only for a read the pattern can declare.
    if name.is_empty() || (!pattern.contains('\\') && !pattern.contains(name)) {
        return false;
    }
    let Ok(parameters) =
        Parser::new(allocator.as_oxc(), pattern, SourceType::ts()).parse_slot_parameters()
    else {
        return false;
    };
    !crate::lower::visit_bindings(&parameters, &mut |identifier| identifier.name != name)
}
