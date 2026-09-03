//! Identifier prefixing (`_ctx.`) for the S2 DOM lane — P2-11
//! installment 85, the `prefix_identifiers` half of the production option
//! surface.
//!
//! The shipped lane prefixes at two moments that this emitter has to
//! replay in one walk: the transform's `process_expression` /
//! `process_inline_handler` (with the transform's scope chain), then the
//! codegen's slot-param strips and dynamic-argument special cases (with
//! the codegen's slot params). Every submodule here is a port of the
//! shipped file it names, with binding metadata, `inline` and `is_ts`
//! left out until their installments; byte-identical output against the
//! shipped lane is the bar, so the ports keep the shipped quirks (the
//! second `$event =>` wrap, the two different strips, the prefix parse).

mod codegen_visitor;
mod collector;
mod compat;
mod globals;
mod handler;
mod params;
mod rewrite;
mod scope;
mod shape;
mod slot_defaults;
mod splice;
mod strip;
mod targets;

pub(super) use globals::is_global_allowed;
pub(super) use scope::{PrefixScope, ScopeMark};
pub(super) use slot_defaults::prefix_slot_defaults;

/// `is_event_handler_reference_expression`: the shipped codegen's prefix
/// parse of a handler text, which reads `a; b` as the reference `a`.
pub(super) fn handler_source_is_reference(source: &str) -> bool {
    shape::is_event_handler_reference_expression(source)
}

/// Whether the shipped codegen's prefix parse admits the text as an
/// expression at all (`foo() // c` does, `return false` does not).
pub(super) fn handler_source_is_expression(source: &str) -> bool {
    rewrite::with_prefix_parse(source, |_| true).unwrap_or(false)
}

use vize_s0::{Span, String};
use vize_s2::expr::JsExpr;

use super::js::RawJs;
use super::js_comment::convert_line_comments_to_block;
use rewrite::Retained;

/// How the shipped codegen consumed the transform's prefixed text.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Site {
    /// `generate_simple_expression`: `//` comments become block comments,
    /// then the scope-prefix scan strip under slot params.
    Expression,
    /// Pushed verbatim (hoisted values, custom-directive arguments).
    Raw,
    /// `generate_slot_expression`: the `_ctx.<param>` replace strip, no
    /// comment conversion.
    SlotText,
}

/// The shipped node's content for one expression position: the raw
/// quoted attribute value (entity-decoded) for attribute positions, the
/// trimmed source otherwise, plus where the retained source sits in it.
pub(super) struct Content<'a> {
    pub(super) text: RawJs<'a>,
    /// Byte offset of the retained source inside `text`, when the text
    /// is that source plus surrounding whitespace only.
    pub(super) offset: Option<usize>,
}

impl Content<'_> {
    fn retained<'r, 'a>(&self, js: Option<&'r JsExpr<'a>>) -> Option<Retained<'r, 'a>> {
        let js = js?;
        let offset = self.offset?;
        Some(Retained {
            ast: js.ast,
            source: js.source,
            offset,
        })
    }
}

/// Recover the shipped content for an expression authored at `span`
/// inside `file`: the whitespace-padded text between the enclosing
/// attribute quotes when the span is quote-delimited, else the source.
pub(super) fn node_content<'a>(file: &'a str, source: &'a str, span: Span) -> Content<'a> {
    let Some(padded) = quote_padded(file, source, span) else {
        return Content {
            text: RawJs::Borrowed(source),
            offset: Some(0),
        };
    };
    let leading = padded.len() - padded.trim_start().len();
    Content {
        text: RawJs::Borrowed(padded),
        offset: Some(leading),
    }
}

/// [`node_content`] for the bind-value positions the shipped lane
/// entity-decodes (`bind_js_source`, `innerHTML`).
pub(super) fn node_content_decoded<'a>(file: &'a str, source: &'a str, span: Span) -> Content<'a> {
    decoded(node_content(file, source, span), source)
}

fn decoded<'a>(content: Content<'a>, source: &str) -> Content<'a> {
    if !content.text.as_str().contains('&') {
        return content;
    }
    let text = super::entity::decode_html_entities(content.text.as_str());
    let offset = (text.trim() == source).then(|| text.len() - text.trim_start().len());
    Content {
        text: RawJs::Owned(text),
        offset,
    }
}

fn quote_padded<'a>(file: &'a str, source: &str, span: Span) -> Option<&'a str> {
    let start = usize::try_from(span.start).ok()?;
    let end = usize::try_from(span.end).ok()?;
    if start > end || end > file.len() || file.get(start..end)? != source {
        return None;
    }
    let before = file.get(..start)?;
    let quote_pos = before.rfind(|c: char| !c.is_ascii_whitespace())?;
    let quote = before.as_bytes()[quote_pos];
    if !matches!(quote, b'"' | b'\'') {
        return None;
    }
    let after = file.get(end..)?;
    let close_rel = after.find(|c: char| !c.is_ascii_whitespace())?;
    if after.as_bytes()[close_rel] != quote {
        return None;
    }
    file.get(quote_pos + 1..end + close_rel)
}

/// Refused: the shipped lane reports a non-recoverable
/// `X_INVALID_EXPRESSION` here, so no comparison exists to win.
pub(super) struct Refused;

/// `process_expression` then the codegen consumption for `site`.
pub(super) fn prefix_expression(
    scope: &PrefixScope,
    content: &Content<'_>,
    js: Option<&JsExpr<'_>>,
    site: Site,
) -> Result<String, Refused> {
    let retained = content.retained(js);
    let rewritten = rewrite::rewrite_expression(content.text.as_str(), retained, scope, false);
    if rewritten.parse_error {
        return Err(Refused);
    }
    Ok(consume(scope, rewritten.code, site))
}

/// The codegen consumption alone (text the transform did not rewrite).
pub(super) fn consume(scope: &PrefixScope, code: String, site: Site) -> String {
    match site {
        Site::Expression => {
            let code = if code.contains("//") {
                convert_line_comments_to_block(code.as_str())
            } else {
                code
            };
            if scope.has_slot_params() && strip::contains_slot_param_scope_prefix(code.as_str()) {
                strip::strip_scope_prefixes_for_slot_params(scope, code.as_str())
            } else {
                code
            }
        }
        Site::Raw => code,
        Site::SlotText => strip::strip_ctx_prefix_for_slot_params(scope, code.as_str()),
    }
}

/// `process_inline_handler` + `generate_event_handler`.
pub(super) fn prefix_handler(
    scope: &PrefixScope,
    content: &Content<'_>,
    js: Option<&JsExpr<'_>>,
) -> Result<String, Refused> {
    let retained = content.retained(js);
    let processed = handler::process_inline_handler(content.text.as_str(), retained, scope);
    if processed.parse_error {
        return Err(Refused);
    }
    Ok(handler::finish_event_handler(processed.code, scope))
}

/// `emit_dynamic_directive_arg` under `prefix_identifiers`.
pub(super) fn prefix_dynamic_arg(scope: &PrefixScope, js: &JsExpr<'_>) -> String {
    let content = js.source;
    if let Some(local) = content
        .strip_prefix("_ctx.")
        .filter(|local| scope.is_slot_param(local))
    {
        return String::from(local);
    }
    if scope.is_slot_param(content) {
        return String::from(content);
    }
    if globals::is_simple_identifier(content) {
        let mut out = String::with_capacity(5 + content.len());
        out.push_str("_ctx.");
        out.push_str(content);
        return out;
    }
    let retained = Some(Retained {
        ast: js.ast,
        source: js.source,
        offset: 0,
    });
    if content.starts_with('_') || content.starts_with('$') {
        return consume(scope, String::from(content), Site::Expression);
    }
    if content.starts_with('`') {
        let mut out = String::from("(");
        out.push_str(
            codegen_visitor::prefix_identifiers_with_context_node(content, retained, scope)
                .as_str(),
        );
        out.push(')');
        return out;
    }
    codegen_visitor::prefix_identifiers_with_context_node(content, retained, scope)
}
