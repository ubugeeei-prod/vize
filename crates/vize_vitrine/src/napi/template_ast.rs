//! `parseTemplateAst`: the full authored template tree for JavaScript tools.
//!
//! The tree is read from the L1 component surface, the lossless authored
//! level every Davinci tool parses from: comments stay in place, a
//! self-closing element is one element (as Vue's compiler parses it), and
//! every location is exact. Node kinds and field names follow
//! `@vue/compiler-core` where they have the same meaning, with string `type`
//! discriminants (`"ELEMENT"`, `"DIRECTIVE"`, ...).
//!
//! Locations are JavaScript string coordinates: `offset` and `column` count
//! UTF-16 code units, `line` and `column` are 1-based, so
//! `source.slice(loc.start.offset, loc.end.offset)` is a node's source text.
//!
//! The tree crosses the boundary as one JSON text that `JSON.parse` turns
//! into objects: on a 0.5 MB template that is about 2.5 times faster than
//! creating each node object through N-API. Parser diagnostics are returned,
//! never thrown.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![expect(
    clippy::disallowed_types,
    reason = "N-API values cross the boundary as std `String`s"
)]

mod diagnostics;
mod json;
mod position;
mod props;
#[cfg(feature = "napi")]
mod schema;
#[cfg(test)]
mod tests;
mod tree;

#[cfg(feature = "napi")]
use napi::bindgen_prelude::{
    Env, Error, Function, JsObjectValue as _, Object, Result, Status, Unknown,
};
#[cfg(feature = "napi")]
use napi_derive::napi;
#[cfg(feature = "napi")]
pub use schema::*;
use vize_atelier_core::options::CustomElementMatcher;
use vize_l0::Allocator;
use vize_l1::markup::{
    ComponentSourceError, SurfaceParseOptions, parse_component_with_diagnostics,
};

/// Options for `parseTemplateAst`.
#[cfg_attr(feature = "napi", napi(object))]
#[derive(Debug, Clone, Default)]
pub struct TemplateAstOptions {
    /// Tag patterns that are custom elements (`ELEMENT`, never `COMPONENT`),
    /// as `compile`'s `customElements`; `*` matches any substring.
    pub custom_elements: Option<Vec<String>>,
    /// Accept Vue's experimental `//` comments inside opening tags.
    pub experimental_in_tag_comments: Option<bool>,
}

/// The `{ ast, errors, warnings }` JSON text of one template.
pub(crate) fn template_ast_json(
    source: &str,
    options: &TemplateAstOptions,
) -> core::result::Result<Vec<u8>, ComponentSourceError> {
    let allocator = Allocator::new();
    let parse = parse_component_with_diagnostics(
        &allocator,
        source,
        SurfaceParseOptions {
            experimental_in_tag_comments: options.experimental_in_tag_comments.unwrap_or(false),
            ..SurfaceParseOptions::default()
        },
    )?;
    let custom_elements = CustomElementMatcher::from_patterns(
        crate::types::custom_element_patterns(options.custom_elements.as_deref()),
    );
    let mut writer = tree::Writer::new(source, custom_elements);
    writer.root(&parse);
    Ok(writer.json.bytes)
}

/// Parse a Vue template into its full authored tree. Parser diagnostics are
/// in `errors` and `warnings`; the call throws only for invalid options or a
/// source past the 4 GiB coordinate range.
#[cfg(feature = "napi")]
#[napi(js_name = "parseTemplateAst", ts_return_type = "TemplateAstResult")]
pub fn parse_template_ast<'env>(
    env: &'env Env,
    source: String,
    options: Option<TemplateAstOptions>,
) -> Result<Unknown<'env>> {
    let json = template_ast_json(&source, &options.unwrap_or_default()).map_err(|_| {
        Error::new(
            Status::InvalidArg,
            "parseTemplateAst: the template is larger than 4 GiB",
        )
    })?;
    let text = core::str::from_utf8(&json)
        .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))?;
    let json_namespace: Object<'env> = env.get_global()?.get_named_property("JSON")?;
    let parse: Function<'env, &str, Unknown<'env>> = json_namespace.get_named_property("parse")?;
    parse.call(text)
}
