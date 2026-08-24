//! The two live-lane `v-model` validations, in the live lane's order.

use alloc::vec::Vec as StdVec;

use vize_davinci::diagnostic::{Diagnostic, Severity, Stage};
use vize_davinci::id::NodeId;
use vize_s0::String;
use vize_s2::op::ModelOp;

use super::{ARG_ON_ELEMENT_MESSAGE, Channels, ModelFacts, ModelFault, ON_SCOPE_MESSAGE};

/// The two legacy checks, in the legacy order, first failure wins.
pub(super) fn check_model(
    channels: &mut Channels<'_>,
    env: &StdVec<String>,
    id: Option<NodeId>,
    model: &ModelOp<'_>,
) {
    let read = model.contract.read.source();
    let (fault, message, rule) = if env.iter().any(|name| name.as_str() == read) {
        (
            ModelFault::OnScope,
            ON_SCOPE_MESSAGE,
            "error.v-model-on-scope",
        )
    } else {
        let component = model.attributes.iter().any(|attribute| {
            attribute.name == "element-kind" && attribute.value == Some("component")
        });
        let argument = model
            .attributes
            .iter()
            .any(|attribute| attribute.name == "argument");
        if component || !argument {
            return;
        }
        (
            ModelFault::ArgOnElement,
            ARG_ON_ELEMENT_MESSAGE,
            "error.v-model-arg-on-element",
        )
    };
    channels.diagnostics.push(Diagnostic::new(
        Severity::Error,
        Stage::Semantic,
        model.span,
        String::from(message),
    ));
    channels
        .provenance
        .push(vize_s2::provenance::ProvenanceRecord {
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
