//! Shared JS/TS semantic production over actual retained language syntax.

pub(crate) mod file;
mod for_head;
pub mod interpolation;
pub use crate::file::region::native::NativeTemplateWalk;
pub use file::native::{
    NativeTemplateFile, NativeTemplateIssue, NativeTemplateIssueKind, NativeTemplateOwner,
    NativeTemplateView, RejectedNativeTemplateOwner,
};
pub use file::observer::{
    CallEvent, DeclaredEvent, FileObserver, InvocationEvent, StatementEvent, SyntaxEvent,
};
pub use file::setup::{SetupAnnotation, SetupIssue, SetupIssueKind, SetupPrimitiveType, VueSetup};
pub use file::{FileProducer, ProgramInput, ProgramInputError, ProgramScope};
pub use for_head::{ForHeadInput, NativeForInput, RejectedForHeadInput, RejectedNativeForInput};
pub use interpolation::{
    NativeInterpolationInput, NativeInterpolationInputError, NativeInterpolationInputView,
};

mod jsx_body;
pub use jsx_body::{JsxChildren, JsxFile, JsxFileError, JsxFileProducer, JsxNode, RejectedJsxFile};
