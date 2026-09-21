//! The shipped transform's expression rewrite, published for string backends.
//!
//! The DOM lane consumes the transform's prefixed text through DOM codegen
//! rules (`//` comment conversion, slot-param strips). The SSR string plan
//! consumes the transform's text directly, so it needs the rewrite without
//! that DOM consumption. This module is the narrow, typed door: it runs the
//! same byte-proven `process_expression` port the DOM lane uses and returns
//! the rewritten text unconsumed ([`prefix::Site::Raw`]).

use vize_s0::String;
use vize_s2::expr::ExprRef;

use super::js::RawJs;
use super::options::BindingTable;
use super::prefix::{self, PrefixScope, ScopeMark, Site};

/// How the shipped parser handed an expression position to the transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformContent {
    /// The whitespace-padded text between the attribute quotes (or the
    /// source itself outside quotes): interpolations and most directives.
    Padded,
    /// [`TransformContent::Padded`] after HTML entity decoding: `v-bind`
    /// values, which the shipped parser decodes before the transform.
    Decoded,
}

/// A transform-rewritten expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformedExpr {
    /// The rewritten JavaScript text.
    pub text: String,
    /// Whether the rewrite read a binding through the `_unref` helper.
    pub used_unref: bool,
}

/// Why an expression has no transform rewrite this door can reproduce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformRefusal {
    /// A foreign or filter expression: no shipped rewrite to mirror.
    ExpressionKind,
    /// The shipped transform reports `X_INVALID_EXPRESSION` here.
    InvalidExpression,
}

/// Opaque mark restoring a [`TransformExpressions`] scope.
#[derive(Clone, Copy)]
pub struct TransformScopeMark(ScopeMark);

/// The shipped transform's `process_expression` with `prefix_identifiers`
/// on, over one template's S2 expressions.
pub struct TransformExpressions<'b> {
    source: &'b str,
    scope: PrefixScope<'b>,
}

impl<'b> TransformExpressions<'b> {
    /// A rewriter for expressions spanned in `source` under the script's
    /// binding table, TypeScript erasure, and inline render mode.
    #[must_use]
    pub fn new(
        source: &'b str,
        bindings: Option<&'b BindingTable>,
        is_ts: bool,
        inline: bool,
    ) -> Self {
        Self {
            source,
            scope: PrefixScope::new(bindings, true, is_ts, inline),
        }
    }

    /// The transform's rewrite of `expr` at its authored position.
    pub fn expr(
        &self,
        expr: &ExprRef<'_>,
        content: TransformContent,
    ) -> Result<TransformedExpr, TransformRefusal> {
        let (source, js) = match expr {
            ExprRef::Js(js) => (js.source, Some(*js)),
            ExprRef::Opaque(opaque) => (opaque.source, None),
            ExprRef::Foreign(_) | ExprRef::Filter(_) => {
                return Err(TransformRefusal::ExpressionKind);
            }
        };
        let content = match content {
            TransformContent::Padded => prefix::node_content(self.source, source, expr.span()),
            TransformContent::Decoded => {
                prefix::node_content_decoded(self.source, source, expr.span())
            }
        };
        prefix::prefix_expression(&self.scope, &content, js, Site::Raw)
            .map(|prefixed| TransformedExpr {
                text: prefixed.text,
                used_unref: prefixed.used_unref,
            })
            .map_err(|_| TransformRefusal::InvalidExpression)
    }

    /// The transform's rewrite of fact-derived text with no retained AST,
    /// such as a dynamic part of a merged text/interpolation run.
    pub fn text(&self, text: &str) -> Result<TransformedExpr, TransformRefusal> {
        let content = prefix::Content {
            text: RawJs::Borrowed(text),
            offset: None,
        };
        prefix::prefix_expression(&self.scope, &content, None, Site::Raw)
            .map(|prefixed| TransformedExpr {
                text: prefixed.text,
                used_unref: prefixed.used_unref,
            })
            .map_err(|_| TransformRefusal::InvalidExpression)
    }

    /// The transform's `process_inline_handler` over a `v-on` value: prefixed,
    /// and wrapped as `$event => (...)` unless it is a function or a callable
    /// reference.
    pub fn handler(
        &mut self,
        expr: &ExprRef<'_>,
        content: TransformContent,
    ) -> Result<TransformedExpr, TransformRefusal> {
        let (source, js) = match expr {
            ExprRef::Js(js) => (js.source, Some(*js)),
            ExprRef::Opaque(opaque) => (opaque.source, None),
            ExprRef::Foreign(_) | ExprRef::Filter(_) => {
                return Err(TransformRefusal::ExpressionKind);
            }
        };
        let content = match content {
            TransformContent::Padded => prefix::node_content(self.source, source, expr.span()),
            TransformContent::Decoded => {
                prefix::node_content_decoded(self.source, source, expr.span())
            }
        };
        prefix::prefix_inline_handler(&mut self.scope, &content, js)
            .map(|prefixed| TransformedExpr {
                text: prefixed.text,
                used_unref: prefixed.used_unref,
            })
            .map_err(|_| TransformRefusal::InvalidExpression)
    }

    /// Whether processed handler text is a function or a callable reference.
    #[must_use]
    pub fn handler_is_callable(text: &str) -> bool {
        prefix::handler_text_is_callable(text)
    }

    /// Enter a `v-for` scope: the aliases stop being prefixed.
    pub fn enter_for(&mut self, aliases: [Option<&str>; 3]) -> TransformScopeMark {
        let mark = TransformScopeMark(self.scope.mark());
        self.scope.push_for(aliases);
        mark
    }

    /// Enter a scoped-slot scope: the slot props stop being prefixed.
    pub fn enter_slot(&mut self, params: &str) -> TransformScopeMark {
        let mark = TransformScopeMark(self.scope.mark());
        if !params.trim().is_empty() {
            self.scope.push_slot(params);
        }
        mark
    }

    /// Leave the scope entered when `mark` was taken.
    pub fn leave(&mut self, mark: TransformScopeMark) {
        self.scope.pop(mark.0);
    }
}

/// Decode HTML entities the way the shipped parser does for template text
/// and attribute values.
#[must_use]
pub fn decode_template_entities(text: &str) -> String {
    super::entity::decode_html_entities(text)
}
