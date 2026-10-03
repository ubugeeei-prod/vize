//! Opt-in L1 template formatting with a separate document IR and printer.
//!
//! This facility leaves all existing formatter entry points unchanged. The
//! native template consumer lays out plain and typed static directive opening tags and preserves
//! content, comments, entities, attribute order and quoted values verbatim.
//! Directives without static arguments, dynamic arguments, embed formatting and the other
//! Vue dialects remain unsupported.

#[path = "native_doc/directive.rs"]
mod directive;
#[path = "native_doc/document.rs"]
mod document;
#[path = "native_doc/printer.rs"]
mod printer;
#[path = "native_doc/template.rs"]
mod template;

pub use document::{Doc, Line};
pub use printer::{LineEnding, PrintOptions, print};
pub use template::{TemplateDocument, TemplateRefusal, UnsupportedSyntax, template_document};
