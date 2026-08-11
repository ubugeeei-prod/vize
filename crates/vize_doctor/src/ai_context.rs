//! Provider-neutral, budgeted context packets for AI consumers.
//!
//! The packet contract deliberately contains no vendor SDK types, prompt
//! syntax, timestamps, process identifiers, or terminal text. Connectors can
//! serialize the same packet into any model API while retaining stable source,
//! evidence, edit-safety, and verification semantics.
//! Packets are untrusted data: connectors must not execute edit plans or
//! verification commands without the caller's explicit authorization.

mod builder;
mod contract;
mod snippet;
mod validation;

#[cfg(test)]
mod tests;

pub use builder::build_ai_context;
pub use contract::{
    AiContextBudget, AiContextOmissions, AiContextPacket, AiEditOperation, AiEditPlan,
    AiEvidenceEdge, AiEvidenceGraph, AiEvidenceNode, AiEvidenceNodeKind, AiEvidenceRelation,
    AiFindingContext, AiSourceSnippet, AiVerificationStep, DOCTOR_AI_CONTEXT_FORMAT_VERSION,
};
pub use validation::AiContextError;
