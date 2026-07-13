//! Production-neutral JavaScript and TypeScript module frontend.
//!
//! OXC values are consumed while their allocator is alive. Only owned facts
//! and an owned CFG escape into Atlas, keeping this crate independent from
//! template syntax, Vue semantics, and compiler frontends.

mod facts;
mod flow;
mod frontend;
mod model;
mod operations;
mod provider;

pub use flow::{append_module_flow, project_module_flow};
pub use frontend::{snapshot_module, snapshot_program};
pub use model::{
    ModuleBindingKind, ModuleBlock, ModuleCfg, ModuleDeclaration, ModuleDiagnostic, ModuleDocument,
    ModuleEdge, ModuleEdgeKind, ModuleExport, ModuleExpression, ModuleExpressionKind,
    ModuleFunction, ModuleImport, ModuleImportBinding, ModuleImportBindingKind, ModuleInstruction,
    ModuleInstructionKind, ModuleLanguage, ModuleLiteralKind, ModuleObjectBinding,
    ModuleObjectProperty, ModuleOperation, ModuleOperationKind, ModuleOperations, ModulePattern,
    ModuleReference, ModuleSpan, ModuleSyntax,
};
pub use provider::{
    MODULE_SOURCE_KIND, ModuleFlowProvider, ModuleSyntaxProduct, RawModuleSyntaxProvider,
    register_raw_providers,
};
