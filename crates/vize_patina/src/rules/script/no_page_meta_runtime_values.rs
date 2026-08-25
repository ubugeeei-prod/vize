//! nuxt/no-page-meta-runtime-values
//!
//! `definePageMeta()` is extracted into an eager build-time chunk. Nuxt/Vue
//! context APIs, `this`, and `await` are therefore invalid at the macro's eager
//! level, but remain valid inside callbacks such as `middleware` and
//! `validate`. This ports `@nuxt/eslint-plugin` 1.16.0's syntax-only boundary
//! and direct-callee checks exactly.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    ArrowFunctionExpression, AwaitExpression, CallExpression, Expression, Function, Program,
    ThisExpression,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_arrow_function_expression, walk_await_expression, walk_call_expression, walk_function,
    },
};
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;
use vize_carton::cstr;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "nuxt/no-page-meta-runtime-values",
    description: "Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup",
    default_severity: Severity::Error,
};

/// Disallow runtime-only values at the eager level of `definePageMeta()`.
pub struct NoPageMetaRuntimeValues;

impl ScriptRule for NoPageMetaRuntimeValues {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let mut visitor = PageMetaVisitor {
            inside_define_page_meta: false,
            function_depth: 0,
            offset,
            result,
        };
        visitor.visit_program(program);
    }
}

struct PageMetaVisitor<'result> {
    inside_define_page_meta: bool,
    function_depth: u32,
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for PageMetaVisitor<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if !self.inside_define_page_meta && is_define_page_meta_call(call) {
            self.inside_define_page_meta = true;
            walk_call_expression(self, call);
            self.inside_define_page_meta = false;
            self.function_depth = 0;
            return;
        }

        if self.is_at_eager_level()
            && let Some(name) = identifier_callee_name(call)
            && is_runtime_context_api(name)
        {
            self.report_context_call(call.span, name);
        }
        walk_call_expression(self, call);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if self.inside_define_page_meta {
            self.function_depth += 1;
            walk_arrow_function_expression(self, arrow);
            self.function_depth -= 1;
        } else {
            walk_arrow_function_expression(self, arrow);
        }
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        if self.inside_define_page_meta {
            self.function_depth += 1;
            walk_function(self, function, flags);
            self.function_depth -= 1;
        } else {
            walk_function(self, function, flags);
        }
    }

    fn visit_this_expression(&mut self, this: &ThisExpression) {
        if self.is_at_eager_level() {
            self.report(
                this.span,
                "`definePageMeta()` is extracted at build time and runs before component setup. `this` is not available in the extracted context.",
            );
        }
    }

    fn visit_await_expression(&mut self, await_expression: &AwaitExpression<'a>) {
        if self.is_at_eager_level() {
            self.report(
                await_expression.span,
                "`definePageMeta()` is extracted at build time. `await` is not supported inside `definePageMeta`.",
            );
        }
        walk_await_expression(self, await_expression);
    }
}

impl PageMetaVisitor<'_> {
    #[inline]
    fn is_at_eager_level(&self) -> bool {
        self.inside_define_page_meta && self.function_depth == 0
    }

    fn report_context_call(&mut self, span: Span, name: &str) {
        self.report(
            span,
            cstr!("`definePageMeta()` is extracted at build time and runs before component setup. `{name}()` requires a Nuxt/Vue runtime context that is not available here. Move it inside a `middleware` or `validate` function."),
        );
    }

    fn report(&mut self, span: Span, message: impl Into<vize_carton::String>) {
        self.result.add_diagnostic(LintDiagnostic::error(
            META.name,
            message,
            self.offset as u32 + span.start,
            self.offset as u32 + span.end,
        ));
    }
}

fn is_define_page_meta_call(call: &CallExpression<'_>) -> bool {
    identifier_callee_name(call) == Some("definePageMeta")
}

fn identifier_callee_name<'call>(call: &'call CallExpression<'_>) -> Option<&'call str> {
    let Expression::Identifier(identifier) = &call.callee else {
        return None;
    };
    Some(identifier.name.as_str())
}

fn is_runtime_context_api(name: &str) -> bool {
    matches!(
        name,
        "ref"
            | "shallowRef"
            | "customRef"
            | "computed"
            | "reactive"
            | "shallowReactive"
            | "readonly"
            | "shallowReadonly"
            | "toRef"
            | "toRefs"
            | "watch"
            | "watchEffect"
            | "watchPostEffect"
            | "watchSyncEffect"
            | "effectScope"
            | "onScopeDispose"
            | "onBeforeMount"
            | "onMounted"
            | "onBeforeUpdate"
            | "onUpdated"
            | "onBeforeUnmount"
            | "onUnmounted"
            | "onActivated"
            | "onDeactivated"
            | "onErrorCaptured"
            | "onRenderTracked"
            | "onRenderTriggered"
            | "onServerPrefetch"
            | "inject"
            | "getCurrentInstance"
    ) || name.starts_with("use") && name.as_bytes().get(3).is_some_and(u8::is_ascii_uppercase)
}

#[cfg(test)]
mod tests;
