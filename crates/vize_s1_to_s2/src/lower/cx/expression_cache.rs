//! Compile-local reuse of the four most recent successful TS expression parses.
//!
//! Consumers of lowered expressions borrow ASTs without semantic mutation.
//! Every hit still receives its own authored source/span wrapper. Rejections,
//! foreign expressions and Vue filters retain their canonical admission paths.

use core::cell::{Cell, RefCell};

use vize_s0::{Allocator, Span};
use vize_s2::expr::{ExprRef, JsExpr};

#[cfg(test)]
mod tests;

pub(crate) struct ExpressionCache<'a> {
    entries: RefCell<[Option<JsExpr<'a>>; 4]>,
    next: Cell<usize>,
}

impl<'a> ExpressionCache<'a> {
    pub(crate) const fn new() -> Self {
        Self {
            entries: RefCell::new([None; 4]),
            next: Cell::new(0),
        }
    }

    pub(crate) fn parse(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        span: Span,
    ) -> ExprRef<'a> {
        let cached = self
            .entries
            .borrow()
            .iter()
            .flatten()
            .find(|entry| entry.source == source)
            .copied();
        if let Some(cached) = cached {
            #[cfg(feature = "davinci-benchmark-profile")]
            record("davinci.s1_to_s2.expression_cache.hit");
            return ExprRef::Js(allocator.alloc(JsExpr {
                ast: cached.ast,
                source,
                span,
            }));
        }
        #[cfg(feature = "davinci-benchmark-profile")]
        record("davinci.s1_to_s2.expression_cache.miss");
        #[cfg(feature = "davinci-benchmark-profile")]
        let parsed = vize_s0::profile!(
            "atelier.s1_to_s2.expression_parse",
            ExprRef::parse_js_in(allocator, source, span)
        );
        #[cfg(not(feature = "davinci-benchmark-profile"))]
        let parsed = ExprRef::parse_js_in(allocator, source, span);
        if let ExprRef::Js(js) = parsed {
            #[cfg(feature = "davinci-benchmark-profile")]
            record("davinci.s1_to_s2.expression_cache.admitted");
            let next = self.next.get();
            self.entries.borrow_mut()[next] = Some(*js);
            self.next.set((next + 1) % 4);
        }
        parsed
    }
}

#[cfg(feature = "davinci-benchmark-profile")]
fn record(key: &'static str) {
    let profiler = vize_s0::profiler::global_profiler();
    if profiler.is_enabled() {
        profiler.record_counter_enabled(key, 1);
    }
}
