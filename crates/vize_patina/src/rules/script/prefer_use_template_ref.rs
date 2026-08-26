//! script/prefer-use-template-ref
//!
//! Recommend `useTemplateRef` over `ref` / `shallowRef` for template
//! references.
//!
//! Since Vue 3.5, `useTemplateRef()` is the recommended way to obtain a
//! template ref: it takes the ref name directly, so it cannot silently
//! disagree with the template, and it infers the element type.
//!
//! ## Template awareness
//!
//! Whether a `ref()` declaration is a *template* ref is not decidable from the
//! script alone — it depends on the template binding a matching `ref="name"`
//! attribute. This rule therefore pairs the two, exactly like upstream
//! [`vue/prefer-use-template-ref`](https://eslint.vuejs.org/rules/prefer-use-template-ref.html):
//! a declaration is reported only when the template binds a static template ref
//! with the same name (template names come from
//! [`template_refs::collect_template_ref_names`]).
//!
//! The initializer argument is deliberately *not* part of the test. A template
//! ref is written `ref(null)`, `ref()`, `ref<HTMLInputElement | null>(null)` or
//! `shallowRef(null)` interchangeably, so keying on `ref(null)` both missed the
//! other spellings and — far worse — reported every nullable data ref
//! (`const error = ref(null)`) in files whose template never mentions it. Only
//! the declared name and the callee matter.
//!
//! Without a template (a standalone script, an inline HTML `<script>`, a bare
//! `.ts` module) there are no template refs to pair against, so nothing is
//! reported.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script setup lang="ts">
//! import { ref } from 'vue'
//! const input = ref<HTMLInputElement | null>(null)
//! </script>
//! <template>
//!   <input ref="input" />
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup lang="ts">
//! import { ref, useTemplateRef } from 'vue'
//! // Paired with the template ref below.
//! const input = useTemplateRef<HTMLInputElement>('input')
//! // A nullable data ref the template never binds as a ref.
//! const error = ref(null)
//! </script>
//! <template>
//!   <input ref="input" />
//!   <p>{{ error }}</p>
//! </template>
//! ```

use oxc_ast::ast::{
    BindingPattern, Expression, ObjectProperty, Program, PropertyKey, Statement,
    VariableDeclaration,
};
use oxc_ast_visit::{Visit, walk::walk_object_property};
use oxc_span::Span;
use vize_s0::{CompactString, cstr};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};

mod template_refs;
#[cfg(test)]
mod tests;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/prefer-use-template-ref",
    description: "Recommend useTemplateRef over ref for template references (Vue 3.5+)",
    default_severity: Severity::Warning,
};

/// Prefer `useTemplateRef` for template references.
pub struct PreferUseTemplateRef;

impl ScriptRule for PreferUseTemplateRef {
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
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        // Keep the parse-owning `check` path functional: without SFC context
        // there is no template to pair declarations against.
        self.check_program_with_sfc(program, source, offset, SfcScriptContext::default(), result);
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        let Some(template) = sfc.template_source else {
            return;
        };
        let template_ref_names = template_refs::collect_template_ref_names(template);
        if template_ref_names.is_empty() {
            return;
        }

        let mut declarations = Vec::new();
        collect_ref_declarations(&program.body, &mut declarations);
        let mut setup = SetupRefCollector {
            declarations: &mut declarations,
        };
        setup.visit_program(program);

        for declaration in declarations {
            if template_ref_names.contains(&declaration.name) {
                report(&declaration, offset, result);
            }
        }
    }
}

/// A `const <name> = ref(...)` / `shallowRef(...)` declaration that could be a
/// template ref, with the span of the `ref(...)` call to report at (matching
/// upstream, which reports the `CallExpression`).
struct RefDeclaration {
    name: CompactString,
    /// `"ref"` or `"shallowRef"`, echoed in the diagnostic message.
    callee: &'static str,
    span: Span,
}

/// Collect `ref` / `shallowRef` declarations from a statement list.
///
/// Only declarations directly in the list are collected. Upstream reads the
/// `<script setup>` program body and the `setup()` body the same way, so a
/// `ref()` buried inside a nested function is not a template-ref candidate for
/// either implementation.
///
/// Unlike upstream, which inspects only `declarations[0]`, every declarator of
/// a multi-declarator statement is considered: `const a = 1, root = ref(null)`
/// is the same template ref as its single-declarator spelling. This is a
/// deliberate superset in the "patina reports more" direction.
fn collect_ref_declarations(statements: &[Statement<'_>], declarations: &mut Vec<RefDeclaration>) {
    for statement in statements {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        collect_from_variable_declaration(declaration, declarations);
    }
}

fn collect_from_variable_declaration(
    declaration: &VariableDeclaration<'_>,
    declarations: &mut Vec<RefDeclaration>,
) {
    for declarator in &declaration.declarations {
        let BindingPattern::BindingIdentifier(id) = &declarator.id else {
            continue;
        };
        let Some(Expression::CallExpression(call)) = declarator.init.as_ref() else {
            continue;
        };
        let Expression::Identifier(identifier) = &call.callee else {
            continue;
        };
        let callee = match identifier.name.as_str() {
            "ref" => "ref",
            "shallowRef" => "shallowRef",
            _ => continue,
        };
        declarations.push(RefDeclaration {
            name: CompactString::new(id.name.as_str()),
            callee,
            span: call.span,
        });
    }
}

/// Collects candidates from the body of every `setup` property, covering the
/// Options API form (`export default { setup() { ... } }`, `defineComponent({
/// setup: () => { ... } })`) the way upstream's `onSetupFunctionEnter` hook
/// does.
///
/// Matching the property *key* rather than resolving the enclosing component
/// object keeps this independent of how the component is declared
/// (`export default {}`, `defineComponent({})`, a re-exported object). A stray
/// object property named `setup` is the only over-match, and its declarations
/// still have to pair with a template ref name to be reported.
///
/// An expression-bodied arrow (`setup: () => ({ ... })`) declares nothing, and
/// upstream skips it as well.
struct SetupRefCollector<'out> {
    declarations: &'out mut Vec<RefDeclaration>,
}

impl<'a> Visit<'a> for SetupRefCollector<'_> {
    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        if !it.computed && property_key_is_setup(&it.key) {
            match &it.value {
                Expression::FunctionExpression(function) => {
                    if let Some(body) = &function.body {
                        collect_ref_declarations(&body.statements, self.declarations);
                    }
                }
                Expression::ArrowFunctionExpression(arrow) if !arrow.expression => {
                    collect_ref_declarations(&arrow.body.statements, self.declarations);
                }
                _ => {}
            }
        }
        walk_object_property(self, it);
    }
}

fn property_key_is_setup(key: &PropertyKey<'_>) -> bool {
    match key {
        PropertyKey::StaticIdentifier(identifier) => identifier.name.as_str() == "setup",
        PropertyKey::StringLiteral(literal) => literal.value.as_str() == "setup",
        _ => false,
    }
}

fn report(declaration: &RefDeclaration, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + declaration.span.start;
    let end = offset as u32 + declaration.span.end;
    let name = declaration.name.as_str();
    let callee = declaration.callee;
    result.add_diagnostic(
        LintDiagnostic::warn(
            META.name,
            cstr!(
                "Template ref '{name}' is declared with {callee}(); use useTemplateRef() instead (Vue 3.5+)."
            ),
            start,
            end,
        )
        .with_label("declared as a template ref", start, end)
        .with_help(cstr!(
            "Replace with: `const {name} = useTemplateRef('{name}')`"
        )),
    );
}
