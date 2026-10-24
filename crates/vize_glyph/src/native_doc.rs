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

mod directive;
mod document;
mod expression;
mod printer;
mod selected_template;
mod sfc;
mod template;

pub use document::{Doc, Line};
pub use expression::{
    ExpressionDocument, ExpressionRefusal, MAX_EXPRESSION_DOCUMENT_DEPTH, expression_document,
};
pub use printer::{LineEnding, PrintOptions, print};
pub use selected_template::{
    NativeTemplateDocument, NativeTemplateRefusal, NativeTemplateValuePolicy,
    ObservedNativeTemplateDocument, ObservedNativeTemplateFailure,
    ObservedNativeTemplateFailureParts, ObservedNativeTemplateRefusal, native_template_document,
    observed_native_template_document, observed_native_template_document_with_policy,
};
pub use selected_template::{
    ObservedNativeTemplateBindingFailureParts, ObservedNativeTemplateBindingParts,
};
pub use sfc::NativeSfcDirectivePolicy;
pub use sfc::{
    NativeSfcBlockRole, NativeSfcObservation, NativeSfcOptions, NativeSfcRefusal,
    observe_native_sfc_in,
};
pub use template::{TemplateDocument, TemplateRefusal, UnsupportedSyntax, template_document};

mod vue2_text;
pub use vue2_text::{Vue2TextDocument, Vue2TextDocumentRefusal, vue2_text_document};

mod vue1_text;
pub use vue1_text::{
    Vue1TextDocument, Vue1TextDocumentRefusal, Vue1TextPrintRefusal, vue1_text_document,
};

mod vue2_sfc;
mod vue2_template;
pub use vue2_sfc::{
    NativeVue2SfcNode, NativeVue2SfcObservation, NativeVue2SfcOptions, NativeVue2SfcRefusal,
    observe_native_vue2_sfc_in,
};
