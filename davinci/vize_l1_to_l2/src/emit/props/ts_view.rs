//! The type-erased view of a bind value under `is_ts`.
//!
//! The shipped transform rewrites an expression node's content *once*
//! and marks it processed; every later reader — the patch-flag static
//! check, the hoist decision, the hoisted text itself — sees that
//! rewritten text. The prefix boundary stores that erasure. This view
//! reads it back, so a second strip cannot disagree with emission.

use vize_l0::{Allocator, String};
use vize_l2::expr::JsExpr;

use super::super::prefix::{retained_expression_is_js, shared_erasure};
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
    let text = shared_erasure(source.as_str(), retained_expression_is_js(js));
    (text.as_str() != source.as_str()).then_some(TsView { text })
}

#[cfg(all(test, feature = "typescript"))]
mod tests {
    use vize_l0::{Allocator, Span};
    use vize_l2::expr::JsExpr;

    use super::super::super::prefix;
    use super::ts_view;

    /// The constness reader and a second reader of the same bind share one strip.
    #[test]
    fn ts_view_strips_a_typed_literal_once() {
        let source = "{ n: 1 } as const";
        let allocator = Allocator::new();
        let end = u32::try_from(source.len()).unwrap();
        let js = JsExpr::parse_in(&allocator, source, Span::new(0, end)).unwrap();
        let before = prefix::typescript_erasure_calls();
        let first = ts_view(js, true).expect("typed literal erases");
        let after_first = prefix::typescript_erasure_calls();
        let second = ts_view(js, true).expect("second reader reuses the strip");
        assert_eq!(after_first - before, 1);
        assert_eq!(prefix::typescript_erasure_calls(), after_first);
        assert!(first.is_static());
        assert_eq!(first.into_text().as_str(), second.into_text().as_str());
    }
}
