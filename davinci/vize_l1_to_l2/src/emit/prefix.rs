//! Identifier prefixing (`_ctx.`) for the L2 DOM lane — P2-11
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

mod aliases;
mod codegen_visitor;
mod collector;
mod compat;
pub(in crate::emit) mod globals;
mod handler;
mod params;
mod rewrite;
mod scope;
mod scope_walk;
mod shape;
#[cfg(test)]
mod shorthand_scope;
mod slot_defaults;
mod splice;
mod strip;
mod targets;
#[cfg(feature = "typescript")]
mod typescript;
mod var_bindings;

pub(super) use globals::{is_constant_global, is_global_allowed, is_simple_identifier};
pub(super) use rewrite::slot_params_syntax_valid;
pub(super) use scope::{PrefixScope, ScopeMark};
pub(super) use slot_defaults::{prefix_slot_defaults, prefix_slot_defaults_with_render_prefixes};
#[cfg(feature = "typescript")]
pub(in crate::emit) use typescript::{erase_typescript, erase_typescript_params};

/// Without the `typescript` feature the emit refuses an `is_ts` request up
/// front (`emit_dom_with_options`), so nothing reaches an eraser.
#[cfg(not(feature = "typescript"))]
pub(in crate::emit) fn erase_typescript(content: &str, _proven_js: bool) -> String {
    String::from(content)
}

#[cfg(not(feature = "typescript"))]
pub(in crate::emit) fn erase_typescript_params(_params: &str) -> Option<String> {
    None
}

/// Whether the whole handler is a reference, rather than a statement body.
pub(super) fn handler_source_is_reference(source: &str) -> bool {
    shape::is_event_handler_reference_expression(source)
}

/// Whether the shipped codegen's prefix parse admits the text as an
/// expression at all (`foo() // c` does, `return false` does not).
pub(super) fn handler_source_is_expression(source: &str) -> bool {
    rewrite::with_prefix_parse(source, |_| true).unwrap_or(false)
}

#[cfg(feature = "typescript")]
use vize_l0::FxHashMap;
use vize_l0::{Span, String};
use vize_l2::expr::{ExprRef, JsExpr};

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

/// Whether an existing key AST can reach the retained rewrite without a parse.
pub(super) fn retained_expression_is_js(js: &JsExpr<'_>) -> bool {
    compat::js_module_compatible(js.ast, js.source)
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

/// An authored expression past the lane's one TypeScript boundary: every
/// reader below takes its JavaScript from here, never from the node.
struct Erased<'c, 'r, 'a> {
    /// The node's own text.
    authored: &'c str,
    /// Its JavaScript spelling, when erasure changed the bytes.
    erased: Option<String>,
    /// The retained AST of `authored`, where it sits in that text.
    retained: Option<Retained<'r, 'a>>,
    /// Whether `retained` passed the JS-module dialect gate, when known.
    proven_js: Option<bool>,
}

impl Erased<'_, '_, '_> {
    fn js(&self) -> &str {
        self.erased.as_deref().unwrap_or(self.authored)
    }

    fn changed(&self) -> bool {
        self.erased.is_some()
    }

    fn proven_js(&self) -> bool {
        self.proven_js.unwrap_or_else(|| {
            self.retained
                .is_some_and(|js| compat::js_module_compatible(js.ast, js.source))
        })
    }
}

/// One remembered strip per `(content, proven_js)` on this worker. The
/// constness reader and [`erase`] used to each pay for the oxc transform.
#[cfg(feature = "typescript")]
const ERASURE_MEMO_CAP: usize = 2048;

#[cfg(all(test, feature = "typescript"))]
std::thread_local! {
    static ERASURE_CALLS: core::cell::Cell<u32> = const { core::cell::Cell::new(0) };
}

#[cfg(feature = "typescript")]
std::thread_local! {
    static ERASURES: core::cell::RefCell<FxHashMap<(String, bool), String>> =
        core::cell::RefCell::new(FxHashMap::<(String, bool), String>::default());
}

#[cfg(feature = "typescript")]
fn recall_erasure(key: &(String, bool)) -> Option<String> {
    ERASURES
        .try_with(|memo| memo.borrow().get(key).cloned())
        .ok()
        .flatten()
}

#[cfg(feature = "typescript")]
fn remember_erasure(key: (String, bool), erased: &str) {
    let _ = ERASURES.try_with(|memo| {
        let mut memo = memo.borrow_mut();
        if memo.len() >= ERASURE_MEMO_CAP {
            memo.clear();
        }
        memo.insert(key, String::from(erased));
    });
}

/// The lane's one type erasure of `content`. Later readers, including the
/// bind constness view, take this text instead of stripping again.
#[cfg(feature = "typescript")]
pub(in crate::emit) fn shared_erasure(content: &str, proven_js: bool) -> String {
    let key = (String::from(content), proven_js);
    if let Some(hit) = recall_erasure(&key) {
        return hit;
    }
    #[cfg(test)]
    {
        let _ = ERASURE_CALLS.try_with(|calls| calls.set(calls.get().saturating_add(1)));
    }
    let text = erase_typescript(content, proven_js);
    remember_erasure(key, text.as_str());
    text
}

#[cfg(not(feature = "typescript"))]
pub(in crate::emit) fn shared_erasure(content: &str, proven_js: bool) -> String {
    erase_typescript(content, proven_js)
}

#[cfg(all(test, feature = "typescript"))]
pub(in crate::emit) fn typescript_erasure_calls() -> u32 {
    ERASURE_CALLS.try_with(core::cell::Cell::get).unwrap_or(0)
}

/// Type-erase `content` once, under `is_ts`. The result is the same text
/// [`shared_erasure`] hands to the constness reader.
fn erase<'c, 'r, 'a>(
    scope: &PrefixScope<'_>,
    content: &'c Content<'_>,
    js: Option<&'r JsExpr<'a>>,
) -> Erased<'c, 'r, 'a> {
    let authored = content.text.as_str();
    let mut erased = Erased {
        authored,
        erased: None,
        retained: content.retained(js),
        proven_js: None,
    };
    if scope.is_ts() {
        let proven_js = erased.proven_js();
        let text = shared_erasure(authored, proven_js);
        erased.erased = (text.as_str() != authored).then_some(text);
        erased.proven_js = Some(proven_js);
    }
    erased
}

/// Recover the shipped content for an expression authored at `span`
/// inside `file`: the whitespace-padded text between the enclosing
/// attribute quotes when the span is quote-delimited, else the source.
pub(super) fn node_content<'a>(file: &'a str, source: &'a str, span: Span) -> Content<'a> {
    let Some(padded) = quote_padded(file, source, span) else {
        if let Some(content) = decoded_padding(file, source, span) {
            return content;
        }
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

/// A lowering-decoded expression keeps the padding of its authored value:
/// the shipped node holds the whole quoted value, entity-decoded.
fn decoded_padding<'a>(file: &'a str, source: &str, span: Span) -> Option<Content<'a>> {
    let authored = file.get(usize::try_from(span.start).ok()?..usize::try_from(span.end).ok()?)?;
    let padded = quote_padded(file, authored, span)?;
    let text = super::entity::decode_html_entities(padded);
    (text.trim() == source).then(|| Content {
        offset: Some(text.len() - text.trim_start().len()),
        text: RawJs::Owned(text),
    })
}

fn quote_padded<'a>(file: &'a str, source: &str, span: Span) -> Option<&'a str> {
    let start = usize::try_from(span.start).ok()?;
    let end = usize::try_from(span.end).ok()?;
    if start > end || end > file.len() || file.get(start..end)? != source {
        return None;
    }
    let before = file.get(..start)?;
    let quote_pos = before.rfind(|c: char| !c.is_ascii_whitespace())?;
    let quote = *before.as_bytes().get(quote_pos)?;
    if !matches!(quote, b'"' | b'\'') {
        return None;
    }
    let after = file.get(end..)?;
    let close_rel = after.find(|c: char| !c.is_ascii_whitespace())?;
    if after.as_bytes().get(close_rel) != Some(&quote) {
        return None;
    }
    file.get(quote_pos + 1..end + close_rel)
}

/// Refused: the shipped lane reports a non-recoverable
/// `X_INVALID_EXPRESSION` here, so no comparison exists to win.
pub(super) struct Refused;

/// Prefixed text, with whether reaching it needed the `_unref` helper.
pub(super) struct Prefixed {
    pub(super) text: String,
    pub(super) used_unref: bool,
    pub(super) used_is_ref: bool,
    pub(super) has_identifier_references: bool,
}

/// `process_expression` then the codegen consumption for `site`.
pub(super) fn prefix_expression(
    scope: &PrefixScope<'_>,
    content: &Content<'_>,
    js: Option<&JsExpr<'_>>,
    site: Site,
) -> Result<Prefixed, Refused> {
    // `process_expression`: with `prefix_identifiers` off the TS lane only
    // type-erases, and the node is still marked rewritten — so the codegen
    // consumption runs either way.
    let js = erase(scope, content, js);
    if !scope.prefixes_identifiers() {
        return Ok(Prefixed {
            text: consume(scope, String::from(js.js()), site),
            used_unref: false,
            used_is_ref: false,
            has_identifier_references: false,
        });
    }
    let rewritten = rewrite::rewrite_expression(&js, scope, false);
    if rewritten.parse_error {
        return Err(Refused);
    }
    Ok(Prefixed {
        text: consume(scope, rewritten.code, site),
        used_unref: rewritten.used_unref,
        used_is_ref: rewritten.used_is_ref,
        has_identifier_references: rewritten.has_identifier_references,
    })
}

/// The codegen consumption alone (text the transform did not rewrite).
pub(super) fn consume(scope: &PrefixScope<'_>, code: String, site: Site) -> String {
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
    scope: &mut PrefixScope<'_>,
    content: &Content<'_>,
    js: Option<&JsExpr<'_>>,
    for_caching: bool,
) -> Result<Prefixed, Refused> {
    let js = erase(scope, content, js);
    let (processed, is_reference) = handler::process_classified_handler(&js, scope);
    if processed.parse_error {
        return Err(Refused);
    }
    Ok(Prefixed {
        text: handler::finish_event_handler(processed.code, scope, for_caching, is_reference),
        used_unref: processed.used_unref,
        used_is_ref: processed.used_is_ref,
        has_identifier_references: processed.has_identifier_references,
    })
}

/// `process_inline_handler` alone: the transform's handler text, before any
/// backend's codegen consumption.
pub(super) fn prefix_inline_handler(
    scope: &mut PrefixScope<'_>,
    content: &Content<'_>,
    js: Option<&JsExpr<'_>>,
) -> Result<Prefixed, Refused> {
    let js = erase(scope, content, js);
    let processed = handler::process_inline_handler(&js, scope);
    if processed.parse_error {
        return Err(Refused);
    }
    Ok(Prefixed {
        text: processed.code,
        used_unref: processed.used_unref,
        used_is_ref: processed.used_is_ref,
        has_identifier_references: processed.has_identifier_references,
    })
}

/// A component receives its method reference unchanged, except an Options API
/// method keeps its live-lookup wrapper. Read the same erased text as emission.
pub(super) fn handler_bypasses_component_cache(
    scope: &PrefixScope<'_>,
    file: &str,
    expr: &ExprRef<'_>,
) -> bool {
    let (source, js) = match expr {
        ExprRef::Js(js) => (js.source, Some(*js)),
        ExprRef::Opaque(opaque) => (opaque.source, None),
        ExprRef::Foreign(_) | ExprRef::Filter(_) => return false,
    };
    let content = node_content(file, source, expr.span());
    let erased = erase(scope, &content, js);
    let content = erased.js().trim();
    let name = content.strip_prefix("$options.").unwrap_or(content);
    let options_reference = is_simple_identifier(name)
        && scope.bindings().and_then(|table| table.kind(name))
            == Some(super::options::BindingKind::Options);
    !options_reference
        && handler::is_reference(
            erased.js(),
            if erased.changed() {
                None
            } else {
                erased.retained
            },
        )
}

/// Whether processed handler text is already a function or a callable
/// reference (the codegen passes such text through unwrapped).
pub(super) fn handler_text_is_callable(text: &str) -> bool {
    shape::is_function_expression(text) || shape::is_event_handler_reference_expression(text)
}

/// `emit_dynamic_directive_arg` under `prefix_identifiers`.
pub(super) fn prefix_dynamic_arg(scope: &PrefixScope<'_>, js: &JsExpr<'_>) -> String {
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
