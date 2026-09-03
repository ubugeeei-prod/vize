//! The type-erased view of a bind value under `is_ts`.
//!
//! The shipped transform rewrites an expression node's content *once*
//! and marks it processed; every later reader — the patch-flag static
//! check, the hoist decision, the hoisted text itself — sees that
//! rewritten text. The S2 emitter reads the authored bytes instead, so
//! wherever such a reader exists it takes this view: the stripped text
//! and a parse of it, when stripping actually changed the bytes.

use vize_s0::{Allocator, String};
use vize_s2::expr::JsExpr;

use super::super::prefix::strip_typescript_from_expression;
use super::static_expr::is_static_bound_expr;

pub(in crate::emit) struct TsView {
    text: String,
}

impl TsView {
    /// `is_static_bound_expression` over the erased text.
    pub(in crate::emit) fn is_static(&self) -> bool {
        let allocator = Allocator::new();
        let mut wrapped = String::with_capacity(self.text.len() + 2);
        wrapped.push('(');
        wrapped.push_str(self.text.as_str());
        wrapped.push(')');
        oxc_parser::Parser::new(
            allocator.as_oxc(),
            wrapped.as_str(),
            oxc_span::SourceType::default().with_module(true),
        )
        .parse_expression()
        .is_ok_and(|expr| is_static_bound_expr(&expr))
    }

    pub(in crate::emit) fn into_text(self) -> String {
        self.text
    }
}

/// `Some` only when `is_ts` erased something: an unchanged strip leaves
/// every decision on the retained AST, which is the byte proof.
pub(in crate::emit) fn ts_view(js: &JsExpr<'_>, is_ts: bool) -> Option<TsView> {
    if !is_ts {
        return None;
    }
    let source = super::super::js::js_expr_source(js);
    let text = strip_typescript_from_expression(source.as_str());
    (text.as_str() != source.as_str()).then_some(TsView { text })
}
