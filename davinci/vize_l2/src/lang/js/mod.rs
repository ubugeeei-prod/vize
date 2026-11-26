//! Shared JS/TS semantic production over actual retained language syntax.

mod conditional;
mod model;
pub use model::{NativeModelInput, NativeModelInputError};
pub(crate) mod file;
pub use conditional::{NativeConditionalInput, NativeConditionalInputError};
mod for_head;
mod handler;
pub mod interpolation;
pub use crate::file::region::native::NativeTemplateWalk;
pub use file::native::{
    NativeScopedTemplateIssue, NativeScopedTemplateIssueKind, NativeScopedTemplateView,
    NativeSelectedSetup, NativeSetupIssue, NativeSetupIssueKind, NativeTemplateIssue,
    NativeTemplateIssueKind, NativeTemplateL2Ir, NativeTemplateOwner, NativeTemplateView,
    RejectedNativeTemplateOwner,
};
pub use file::observer::{
    CallEvent, DeclaredEvent, InvocationEvent, L2IrObserver, StatementEvent, SyntaxEvent,
};
pub use file::ordinary::{OrdinaryEmptyScript, OrdinaryIssue, OrdinaryIssueKind, VueOrdinaryEmpty};
pub use file::setup::{SetupAnnotation, SetupIssue, SetupIssueKind, SetupPrimitiveType, VueSetup};
pub use file::{L2IrProducer, ProgramInput, ProgramInputError, ProgramScope};
pub use file::{
    ModuleSourceError, ModuleSourceErrorKind, ModuleSourceKind, OriginalModuleSource,
    OriginalModuleSources,
};
pub use for_head::{ForHeadInput, NativeForInput, RejectedForHeadInput, RejectedNativeForInput};
pub use interpolation::{
    NativeInterpolationInput, NativeInterpolationInputError, NativeInterpolationInputView,
};

mod jsx_body;
pub use handler::{
    HandlerInput, HandlerInputErrorKind, NativeHandlerInput, RejectedHandlerInput,
    RejectedNativeHandlerInput,
};
pub use jsx_body::{JsxChildren, JsxL2Ir, JsxL2IrError, JsxL2IrProducer, JsxNode, RejectedJsxL2Ir};
pub use jsx_body::{JsxMarkupIssue, JsxMarkupIssueKind, JsxMarkupL2Ir, RejectedJsxMarkupL2Ir};
