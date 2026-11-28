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
//! with the same name (template names come from the parsed template, see
//! [`template_refs`]).
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
//! The callee is Vue's `ref` / `shallowRef` under any local name or through a
//! Vue namespace, and the call may sit inside parentheses or a TypeScript
//! wrapper (`as Ref<T>`, `!`). A callee imported from another module is Vue's
//! when that module re-exports it: a package whose published entry re-exports
//! Vue's runtime, or a project module whose re-export chain reaches Vue (see
//! [`reexports`]). A ref whose own binding has its `.value` assigned is
//! skipped: `useTemplateRef` returns a readonly ref, so the suggestion would
//! break the write. A nested binding with the same name writes its own value
//! and does not count. The suggested replacement keeps the declaration keyword
//! and the authored element type without `| null`.
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
    BindingPattern, Expression, ImportDeclarationSpecifier, ObjectExpression, ObjectPropertyKind,
    Program, PropertyKey, Statement, VariableDeclaration,
};
use oxc_ast_visit::{Visit, walk::walk_object_expression};

use declarations::{Callee, RefCallees, ValueWrites, WrittenBindings};
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::Span;
use vize_l0::{CompactString, FxHashSet, cstr};

use super::vue_object::VueObjects;
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};

mod declarations;
mod native;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod reexport_tests;
mod reexports;
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

    fn as_native_sfc_setup_rule(&self) -> Option<&dyn crate::native::sfc::NativeSfcSetupRule> {
        Some(self)
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    /// Template ref names create findings, so they come from the parsed
    /// template, never from comment or text bytes.
    #[inline]
    fn uses_template_ast(&self) -> bool {
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
        source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        let template_ref_names = match (sfc.template_surface, sfc.template_source) {
            (Some(surface), _) => template_refs::surface_ref_names(surface),
            (None, Some(template)) => template_refs::source_ref_names(template),
            (None, None) => return,
        };
        if template_ref_names.is_empty() {
            return;
        }

        let mut callees = RefCallees::default();
        callees.add_imports(program);
        if let Some(sibling) = sfc.sibling_program {
            callees.add_imports(sibling);
        }
        let mut collector = DeclarationCollector {
            callees: &callees,
            template_ref_names: &template_ref_names,
            source,
            declarations: Vec::new(),
        };
        let options_semantic = (!sfc.is_script_setup)
            .then(|| SemanticBuilder::new().with_build_nodes(true).build(program));
        if let Some(built) = &options_semantic {
            let objects = VueObjects::collect(
                program,
                source,
                &built.semantic,
                sfc.is_sfc && !sfc.is_script_setup,
            );
            let mut setup = SetupRefCollector {
                collector: &mut collector,
                objects: &objects,
            };
            setup.visit_program(program);
        } else {
            collector.collect(&program.body);
        }

        let mut declarations = collector.declarations;
        declarations.retain(|declaration| template_ref_names.contains(&declaration.name));
        if declarations.is_empty() {
            return;
        }
        // `useTemplateRef` returns a readonly ref: a ref the script writes
        // keeps its `ref()`.
        let mut writes = ValueWrites::default();
        writes.visit_program(program);
        let owned_semantic;
        let written = if declarations
            .iter()
            .any(|declaration| writes.names.contains(&declaration.name))
        {
            let scoping: &Scoping = match (&options_semantic, sfc.semantic_of(program)) {
                (Some(built), _) => built.semantic.scoping(),
                (None, Some(shared)) => shared.semantic.scoping(),
                (None, None) => {
                    owned_semantic = SemanticBuilder::new().build(program);
                    owned_semantic.semantic.scoping()
                }
            };
            Some(WrittenBindings::collect(program, scoping, &writes.names))
        } else {
            None
        };
        for declaration in declarations {
            if written.as_ref().is_some_and(|written| {
                written.contains(&declaration.name, declaration.binding_span)
            }) {
                continue;
            }
            let callee = match &declaration.callee {
                Callee::Vue(callee) => Some(*callee),
                Callee::Imported { local, member } => imported_factory(
                    program,
                    sfc,
                    local.as_str(),
                    member.as_ref().map(CompactString::as_str),
                ),
            };
            if let Some(callee) = callee {
                report(&declaration, callee, offset, result);
            }
        }
    }
}

/// The Vue factory a callee imported from another module (`local`, or
/// `local.member` through a namespace import) re-exports.
fn imported_factory(
    program: &Program<'_>,
    sfc: SfcScriptContext<'_>,
    local: &str,
    member: Option<&str>,
) -> Option<&'static str> {
    let file = sfc.importing_file?;
    let import = core::iter::once(program)
        .chain(sfc.sibling_program)
        .flat_map(|program| &program.body)
        .find_map(|statement| match statement {
            Statement::ImportDeclaration(import) if !import.import_kind.is_type() => import
                .specifiers
                .iter()
                .flatten()
                .find(|specifier| specifier.local().name == local)
                .map(|specifier| (import.source.value.as_str(), specifier)),
            _ => None,
        });
    let (source, specifier) = import?;
    let export = match (specifier, member) {
        (ImportDeclarationSpecifier::ImportSpecifier(specifier), None)
            if !specifier.import_kind.is_type() =>
        {
            specifier.imported.name().as_str()
        }
        (ImportDeclarationSpecifier::ImportNamespaceSpecifier(_), Some(member)) => member,
        _ => return None,
    };
    reexports::imported_factory(file, source, export)
}

/// A `const <name> = ref(...)` / `shallowRef(...)` declaration that could be a
/// template ref, with the span of the `ref(...)` call to report at (matching
/// upstream, which reports the `CallExpression`).
struct RefDeclaration {
    name: CompactString,
    /// The declared binding, which a `.value` write must reach to keep the ref.
    binding_span: Span,
    /// `ref` / `shallowRef`, echoed in the diagnostic message, or an import
    /// whose module decides it.
    callee: Callee<CompactString>,
    span: Span,
    /// The declaration keyword, kept by the suggested replacement.
    keyword: &'static str,
    /// The authored element type, without `| null`.
    type_argument: Option<CompactString>,
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
struct DeclarationCollector<'c, 's> {
    callees: &'c RefCallees,
    template_ref_names: &'c FxHashSet<CompactString>,
    source: &'s str,
    declarations: Vec<RefDeclaration>,
}

impl DeclarationCollector<'_, '_> {
    fn collect(&mut self, statements: &[Statement<'_>]) {
        for statement in statements {
            if let Statement::VariableDeclaration(declaration) = statement {
                self.collect_from_variable_declaration(declaration);
            }
        }
    }

    /// `const name = ref(...)`, also through parentheses and TypeScript
    /// wrappers, and with `ref` imported under another name or read from a
    /// Vue namespace.
    fn collect_from_variable_declaration(&mut self, declaration: &VariableDeclaration<'_>) {
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            let Some(call) = declarator
                .init
                .as_ref()
                .and_then(declarations::initializer_call)
            else {
                continue;
            };
            let Some(callee) = self.callees.factory(&call.callee) else {
                continue;
            };
            // Most imported calls are not factories: keep only those a
            // template ref pairs with before resolving their module.
            if matches!(callee, Callee::Imported { .. })
                && !self.template_ref_names.contains(id.name.as_str())
            {
                continue;
            }
            self.declarations.push(RefDeclaration {
                name: CompactString::new(id.name.as_str()),
                binding_span: id.span,
                callee: callee.owned(),
                span: call.span,
                keyword: declaration.kind.as_str(),
                type_argument: declarations::template_ref_type_argument(call, self.source),
            });
        }
    }
}

/// Collects candidates from the body of every `setup` property, covering the
/// Options API form (`export default { setup() { ... } }`, `defineComponent({
/// setup: () => { ... } })`) the way upstream's `onSetupFunctionEnter` hook
/// does.
///
/// The containing object must be one of the Vue component objects recognized
/// by the same analysis as eslint-plugin-vue. A custom factory whose argument
/// happens to contain a `setup` property is not a Vue component by itself.
///
/// An expression-bodied arrow (`setup: () => ({ ... })`) declares nothing, and
/// upstream skips it as well.
struct SetupRefCollector<'out, 'c, 's> {
    collector: &'out mut DeclarationCollector<'c, 's>,
    objects: &'out VueObjects,
}

impl<'a> Visit<'a> for SetupRefCollector<'_, '_, '_> {
    fn visit_object_expression(&mut self, it: &ObjectExpression<'a>) {
        if self.objects.contains(it) {
            for property in &it.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    continue;
                };
                if !property.computed && property_key_is_setup(&property.key) {
                    match &property.value {
                        Expression::FunctionExpression(function) => {
                            if let Some(body) = &function.body {
                                self.collector.collect(&body.statements);
                            }
                        }
                        Expression::ArrowFunctionExpression(arrow) if !arrow.expression => {
                            self.collector.collect(&arrow.body.statements);
                        }
                        _ => {}
                    }
                }
            }
        }
        walk_object_expression(self, it);
    }
}

fn property_key_is_setup(key: &PropertyKey<'_>) -> bool {
    match key {
        PropertyKey::StaticIdentifier(identifier) => identifier.name.as_str() == "setup",
        PropertyKey::StringLiteral(literal) => literal.value.as_str() == "setup",
        _ => false,
    }
}

fn report(
    declaration: &RefDeclaration,
    callee: &'static str,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let start = offset as u32 + declaration.span.start;
    let end = offset as u32 + declaration.span.end;
    let name = declaration.name.as_str();
    let keyword = declaration.keyword;
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
        .with_help(match &declaration.type_argument {
            Some(ty) => cstr!("Replace with: `{keyword} {name} = useTemplateRef<{ty}>('{name}')`"),
            None => cstr!("Replace with: `{keyword} {name} = useTemplateRef('{name}')`"),
        }),
    );
}
