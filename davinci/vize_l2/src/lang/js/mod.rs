//! Shared JS/TS semantic production over actual retained language syntax.

pub(crate) mod file;
pub use file::observer::{
    CallEvent, DeclaredEvent, FileObserver, InvocationEvent, StatementEvent, SyntaxEvent,
};
pub use file::{FileProducer, ProgramInput, ProgramInputError, ProgramScope};
mod jsx_body;
pub use jsx_body::{JsxChildren, JsxFile, JsxFileError, JsxFileProducer, JsxNode, RejectedJsxFile};
