//! Retained expression ASTs for native generation.
//!
//! S2 parses each template expression once. The native route moves exactly the
//! admitted, non-reference ASTs into the output arena (a structural copy, not a
//! parse) and hands them to the shared generator as `js_ast`, so emitted
//! expressions are resolved from the retained tree instead of reparsed text.
//! Compound text parts were never parsed by S2; they take their single parse
//! here through S2's own admission rule. Anything the generator's retained
//! path cannot consume byte-equivalently stays on the legacy lane.

use oxc_allocator::CloneIn;
use oxc_allocator::HashMap;
use vize_atelier_core::{JsExpression, retained::js_module_compatible};
use vize_carton::{Allocator, Span, Vec};
use vize_s2::{
    expr::{ExprRef, JsExpr},
    op::{BindingOp, Op, Region},
};

pub(crate) struct Retained<'s, 'a> {
    allocator: &'a Allocator,
    // Arena tables: one compile owns them, so they never touch the heap.
    source: HashMap<'a, (u32, u32), &'s JsExpr<'s>>,
    parsed: HashMap<'a, (u32, u32), JsExpression<'a>>,
}

impl<'s, 'a> Retained<'s, 'a> {
    pub(crate) fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            source: HashMap::new_in(allocator.as_oxc()),
            parsed: HashMap::new_in(allocator.as_oxc()),
        }
    }

    /// The output arena, which also holds the admitted native payload.
    pub(crate) const fn allocator(&self) -> &'a Allocator {
        self.allocator
    }

    /// Index every retained expression in positions the native route reads.
    pub(crate) fn collect(allocator: &'a Allocator, root: &'s Region<'s>) -> Self {
        let mut retained = Self::new(allocator);
        let mut pending = Vec::from_value_in(root, &allocator);
        while let Some(region) = pending.pop() {
            for op in &region.ops {
                match op {
                    Op::Element(element) => {
                        retained.bindings(&element.bindings);
                        pending.push(&element.children);
                    }
                    Op::Interpolation(interpolation) => retained.insert(interpolation.expression),
                    Op::If(branches) => {
                        for branch in &branches.branches {
                            if let Some(condition) = branch.condition {
                                retained.insert(condition);
                            }
                            pending.push(&branch.region);
                        }
                    }
                    Op::For(each) => {
                        retained.insert(each.binding.source);
                        pending.push(&each.region);
                    }
                    Op::Component(component) => {
                        retained.bindings(&component.bindings);
                        pending.push(&component.children);
                    }
                    Op::Slot(slot) => {
                        retained.bindings(&slot.bindings);
                        pending.push(&slot.fallback);
                    }
                    Op::Text(_) | Op::Comment(_) => {}
                }
            }
        }
        retained
    }

    fn bindings(&mut self, bindings: &'s [BindingOp<'s>]) {
        for binding in bindings {
            let value = match binding {
                BindingOp::Bind(bind) => bind.value,
                BindingOp::On(on) => on.handler,
                BindingOp::VueShow(show) => Some(show.value),
                BindingOp::VueHtml(html) => html.value,
                BindingOp::VueText(text) => text.value,
                _ => None,
            };
            if let Some(value) = value {
                self.insert(value);
            }
        }
    }

    fn insert(&mut self, expression: ExprRef<'s>) {
        if let ExprRef::Js(js) = expression {
            self.source.insert((js.span.start, js.span.end), js);
        }
    }

    /// Parse a compound text part once, with S2's admission rule.
    pub(crate) fn parse(&mut self, text: &'a str, span: Span) -> bool {
        let Ok(js) = JsExpr::parse_in(self.allocator, text, span) else {
            return false;
        };
        let js = JsExpression {
            ast: js.ast,
            raw: text,
        };
        let admitted = js_module_compatible(&js);
        if admitted {
            self.parsed.insert((span.start, span.end), js);
        }
        admitted
    }

    /// The retained AST describing exactly `text` at `span`, in the output
    /// arena, when the generator's retained resolver can consume it.
    pub(crate) fn expression(&self, text: &'a str, span: Span) -> Option<JsExpression<'a>> {
        let key = (span.start, span.end);
        if let Some(js) = self.parsed.get(&key) {
            return (js.raw == text).then_some(*js);
        }
        let js = self.source.get(&key)?;
        if js.source != text {
            return None;
        }
        let oxc = self.allocator.as_oxc();
        let js = JsExpression {
            ast: oxc.alloc(js.ast.clone_in(oxc)),
            raw: text,
        };
        js_module_compatible(&js).then_some(js)
    }
}
