//! Shared JS/TS semantic production over actual retained language syntax.

mod conditional;
mod model;
pub use model::{NativeModelInput, NativeModelInputError};
pub(crate) mod ir;
pub use conditional::{NativeConditionalInput, NativeConditionalInputError};
mod for_head;
mod handler;
pub mod interpolation;
pub use crate::ir::region::native::NativeTemplateWalk;
pub use for_head::{ForHeadInput, NativeForInput, RejectedForHeadInput, RejectedNativeForInput};
pub use interpolation::{
    NativeInterpolationInput, NativeInterpolationInputError, NativeInterpolationInputView,
};
pub use ir::native::{
    NativeScopedTemplateIssue, NativeScopedTemplateIssueKind, NativeScopedTemplateView,
    NativeSelectedSetup, NativeSetupIssue, NativeSetupIssueKind, NativeTemplateIssue,
    NativeTemplateIssueKind, NativeTemplateL2Ir, NativeTemplateOwner, NativeTemplateView,
    RejectedNativeTemplateOwner,
};
pub use ir::observer::{
    CallEvent, DeclaredEvent, InvocationEvent, L2IrObserver, StatementEvent, SyntaxEvent,
};
pub use ir::ordinary::{OrdinaryEmptyScript, OrdinaryIssue, OrdinaryIssueKind, VueOrdinaryEmpty};
pub use ir::setup::{SetupAnnotation, SetupIssue, SetupIssueKind, SetupPrimitiveType, VueSetup};
pub use ir::{L2IrProducer, ProgramInput, ProgramInputError, ProgramScope};
pub use ir::{
    ModuleSourceError, ModuleSourceErrorKind, ModuleSourceKind, OriginalModuleSource,
    OriginalModuleSources,
};

mod jsx_body;
pub use handler::{
    HandlerInput, HandlerInputErrorKind, NativeHandlerInput, RejectedHandlerInput,
    RejectedNativeHandlerInput,
};
pub use jsx_body::{JsxChildren, JsxL2Ir, JsxL2IrError, JsxL2IrProducer, JsxNode, RejectedJsxL2Ir};
pub use jsx_body::{JsxMarkupIssue, JsxMarkupIssueKind, JsxMarkupL2Ir, RejectedJsxMarkupL2Ir};
