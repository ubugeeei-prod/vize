//! Frontend-neutral render HIR shared by Vize producers and consumers.
//!
//! Rendu is an owned, indexed compilation product. A frontend lowers directly
//! into [`RenduBuilder`]; a backend reads [`RenduRoot`] and never needs the AST
//! that produced it. Source provenance and required render capabilities travel
//! with the HIR.

mod builder;
mod capability;
mod expression;
mod ids;
mod node;
mod product;
mod property;
mod root;
mod source;
mod validate;
mod walk;

pub use builder::RenduBuilder;
pub use capability::{RenduCapabilities, RenduCapability};
pub use expression::{RenduExpression, RenduExpressionKind};
pub use ids::{RenduExpressionId, RenduNodeId, RenduSourceId};
pub use node::{
    RenduBinding, RenduEscapeMode, RenduIfBranch, RenduName, RenduNamespace, RenduNode,
};
pub use product::{
    RenderCapabilities, RenderCapabilitiesInput, RenderEmitSettings, RenderEmitSettingsInput,
    RenderOutputMode, RenduModule, RenduProduct,
};
pub use property::{RenduAttribute, RenduAttributeValue, RenduDirective, RenduProperty};
pub use root::RenduRoot;
pub use source::{RenduPosition, RenduProvenance, RenduSource, RenduSpan};
pub use validate::{RenduValidationError, RenduValidationErrors};
pub use walk::{RenduWalkEvent, RenduWalkSummary, RenduWalker, summarize_rendu, walk_rendu};
