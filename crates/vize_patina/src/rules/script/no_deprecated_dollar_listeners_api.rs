//! script/no-deprecated-dollar-listeners-api
//!
//! Disallow the `$listeners` instance property.
//!
//! ## Rationale
//!
//! `$listeners` was removed in Vue 3. In Vue 2 it exposed the parent-scope
//! event listeners separately from `$attrs`; in Vue 3 listeners are plain
//! props prefixed with `on`, so they are merged into `$attrs`. Any code that
//! still reads the instance property (or the unbound template `$listeners`)
//! must migrate it to `$attrs` for Vue 3.
//!
//! Built-in Vue 3 presets enable this migration rule; Vue 2 presets leave it off.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script>
//! export default {
//!   mounted() {
//!     const vm = this
//!     return vm.$listeners
//!   }
//! }
//! </script>
//! <template><Child v-on="$listeners" /></template>
//! ```
//!
//! ### Valid
//! ```vue
//! <script>
//! export default {
//!   mounted() {
//!     return this.$attrs
//!   }
//! }
//! </script>
//! <template><Child v-bind="$attrs" /></template>
//! ```

mod template;

use super::vue_instance::VueInstance;
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, ScriptTemplateRule};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{ComputedMemberExpression, Expression, Program, StaticMemberExpression};
use oxc_ast_visit::{
    Visit,
    walk::{walk_computed_member_expression, walk_static_member_expression},
};
use oxc_span::Span;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-deprecated-dollar-listeners-api",
    description: "Disallow the $listeners instance property removed in Vue 3 (merged into $attrs)",
    default_severity: Severity::Error,
};

/// The deprecated instance property name removed in Vue 3.
const DOLLAR_LISTENERS: &str = "$listeners";

const MESSAGE: &str = "$listeners was removed in Vue 3 and merged into $attrs";
const HELP: &str = "Replace $listeners with $attrs. In Vue 3 listeners are passed as on-prefixed props and are \
     exposed through $attrs (or useAttrs() in the Composition API).";

/// Disallow the `$listeners` instance property (Vue 3 migration).
pub struct NoDeprecatedDollarListenersApi;

impl ScriptRule for NoDeprecatedDollarListenersApi {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn template_check(&self) -> Option<&dyn ScriptTemplateRule> {
        Some(self)
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        if !(source.contains("$listeners") || source.contains('\\')) {
            return;
        }
        let instance = VueInstance::new(program, source);
        let mut visitor = NoDeprecatedDollarListenersApiVisitor {
            offset,
            result,
            instance: &instance,
        };
        visitor.visit_program(program);
    }
}

struct NoDeprecatedDollarListenersApiVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    instance: &'result VueInstance,
}

impl<'a> Visit<'a> for NoDeprecatedDollarListenersApiVisitor<'_> {
    fn visit_computed_member_expression(&mut self, it: &ComputedMemberExpression<'a>) {
        // The provider admits an Identifier property even in brackets, while a
        // computed string literal is outside this rule's script selector.
        if self.instance.contains(it.span)
            && let Expression::Identifier(property) = &it.expression
            && property.name.as_str() == DOLLAR_LISTENERS
            && self.instance.is_this(&it.object)
        {
            self.push_diagnostic(property.span);
        }
        walk_computed_member_expression(self, it);
    }

    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        if self.instance.contains(it.span)
            && it.property.name.as_str() == DOLLAR_LISTENERS
            && self.instance.is_this(&it.object)
        {
            self.push_diagnostic(it.property.span);
        }
        walk_static_member_expression(self, it);
    }
}

impl NoDeprecatedDollarListenersApiVisitor<'_> {
    fn push_diagnostic(&mut self, span: Span) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result
            .add_diagnostic(LintDiagnostic::error(META.name, MESSAGE, start, end).with_help(HELP));
    }
}

impl ScriptTemplateRule for NoDeprecatedDollarListenersApi {
    fn check_template(
        &self,
        root: &vize_relief::RootNode<'_>,
        offset: u32,
        result: &mut ScriptLintResult,
    ) {
        for span in template::references(root) {
            result.add_diagnostic(
                LintDiagnostic::error(META.name, MESSAGE, offset + span.start, offset + span.end)
                    .with_help(HELP),
            );
        }
    }
}
