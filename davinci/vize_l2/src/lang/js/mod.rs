//! Shared JS/TS semantic production over actual retained language syntax.

mod conditional;
mod model;
pub use model::{ModelInput, ModelInputError};
pub(crate) mod file;
pub use conditional::{ConditionalInput, ConditionalInputError};
mod for_head;
mod handler;
pub mod interpolation;
pub use crate::file::region::template::FileTemplateWalk;
pub use file::observer::{
    CallEvent, DeclaredEvent, FileObserver, InvocationEvent, StatementEvent, SyntaxEvent,
};
pub use file::ordinary::{OrdinaryEmptyScript, OrdinaryIssue, OrdinaryIssueKind, VueOrdinaryEmpty};
pub use file::setup::{SetupAnnotation, SetupIssue, SetupIssueKind, SetupPrimitiveType, VueSetup};
pub use file::template::{
    RejectedTemplateOwner, ScopedTemplateIssue, ScopedTemplateIssueKind, ScopedTemplateView,
    SelectedSetup, SelectedSetupIssue, SelectedSetupIssueKind, TemplateFile, TemplateFileView,
    TemplateOwner, TemplateRouteIssue, TemplateRouteIssueKind,
};
pub use file::{FileProducer, ProgramInput, ProgramInputError, ProgramScope};
pub use file::{
    ModuleSourceError, ModuleSourceErrorKind, ModuleSourceKind, OriginalModuleSource,
    OriginalModuleSources,
};
pub use for_head::{FileForInput, ForHeadInput, RejectedFileForInput, RejectedForHeadInput};
pub use interpolation::{InterpolationInput, InterpolationInputError, InterpolationInputView};

mod jsx_body;
pub use handler::{
    FileHandlerInput, HandlerInput, HandlerInputErrorKind, RejectedFileHandlerInput,
    RejectedHandlerInput,
};
pub use jsx_body::{JsxChildren, JsxFile, JsxFileError, JsxFileProducer, JsxNode, RejectedJsxFile};
pub use jsx_body::{JsxMarkupFile, JsxMarkupIssue, JsxMarkupIssueKind, RejectedJsxMarkupFile};
