//! Opt-in L1 template formatting with a separate document IR and printer.
//!
//! This facility leaves all existing formatter entry points unchanged. The
//! native template consumer lays out plain and typed directive opening tags and preserves
//! content, comments, entities, attribute order and quoted values verbatim.
//! The selected-template consumer joins genuine original interpolation owners
//! to the bounded native expression document provider. Incomplete directive
//! heads, missing shorthand arguments, other embed shapes and Vue dialects
//! remain unsupported.
//!
//! An independent retained-expression consumer builds documents directly from
//! genuine compiler-profile atoms, parentheses and binary/logical nodes with
//! checked original source/decode-map custody. Its template integration remains
//! a separate dependent consumer; existing formatter routes are unchanged.

#[path = "native_doc/directive.rs"]
mod directive;
#[path = "native_doc/document.rs"]
mod document;
#[path = "native_doc/expression.rs"]
mod expression;
#[path = "native_doc/printer.rs"]
mod printer;
#[path = "native_doc/selected_template.rs"]
mod selected_template;
#[path = "native_doc/template.rs"]
mod template;

pub use document::{Doc, Line};
pub use expression::{
    ExpressionDocument, ExpressionRefusal, MAX_EXPRESSION_DOCUMENT_DEPTH, expression_document,
};
pub use printer::{LineEnding, PrintOptions, print};
pub use selected_template::{
    NativeTemplateDocument, NativeTemplateRefusal, ObservedNativeTemplateDocument,
    ObservedNativeTemplateFailure, ObservedNativeTemplateRefusal, native_template_document,
    observed_native_template_document,
};
pub use template::{TemplateDocument, TemplateRefusal, UnsupportedSyntax, template_document};
