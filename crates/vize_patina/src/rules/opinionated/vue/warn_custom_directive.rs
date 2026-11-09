//! vue/warn-custom-directive
//!
//! Warn about custom directives usage.
//!
//! Custom directives (directives other than built-in Vue directives) should
//! be properly registered and documented. This rule warns about their usage
//! to ensure proper registration.
//!
//! ## Built-in Directives
//!
//! - `v-if`, `v-else`, `v-else-if`, `v-show`
//! - `v-for`, `v-on` (`@`), `v-bind` (`:`)
//! - `v-model`, `v-slot` (`#`)
//! - `v-pre`, `v-once`, `v-memo`, `v-cloak`
//! - `v-html`, `v-text`
//!
//! ## Examples
//!
//! ### Triggers Warning
//! ```vue
//! <div v-focus></div>
//! <input v-mask="'###-####'" />
//! <div v-click-outside="handleClose"></div>
//! ```
//!
//! ### Valid (Built-in)
//! ```vue
//! <div v-if="show"></div>
//! <input v-model="value" />
//! <button @click="onClick">Click</button>
//! ```

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, ImportDeclarationSpecifier, ObjectExpression,
    ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_ast_visit::{Visit, walk::walk_object_expression};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::{CompactString, String, cstr};
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/warn-custom-directive",
    description: "Warn about custom directives that need registration",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Built-in Vue directives
const BUILTIN_DIRECTIVES: &[&str] = &[
    "if", "else", "else-if", "show", "for", "on", "bind", "model", "slot", "pre", "once", "memo",
    "cloak", "html", "text", "is",
];

/// Warn about custom directives
pub struct WarnCustomDirective {
    ignored_directives: Vec<String>,
}

#[expect(
    non_upper_case_globals,
    reason = "the unit-like rule value intentionally shares the rule type name"
)]
/// Default `vue/warn-custom-directive` rule with no ignored directives.
pub const WarnCustomDirective: WarnCustomDirective = WarnCustomDirective {
    ignored_directives: Vec::new(),
};

impl Default for WarnCustomDirective {
    fn default() -> Self {
        WarnCustomDirective
    }
}

impl WarnCustomDirective {
    pub(crate) fn with_ignore(ignored_directives: Vec<String>) -> Self {
        Self { ignored_directives }
    }
}

impl Rule for WarnCustomDirective {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        let name = directive.name;

        // Check if this is a custom directive (not built-in). A `<script setup>`
        // binding `vFocus` (import or local const) registers `v-focus`.
        if !BUILTIN_DIRECTIVES.contains(&name)
            && !self
                .ignored_directives
                .iter()
                .any(|ignored| ignored.as_str() == name)
            && !scripts_register_directive(ctx, name)
        {
            ctx.warn_with_help(
                cstr!(
                    "Custom directive 'v-{}' detected. Ensure it is properly registered.",
                    name
                ),
                &directive.loc,
                "Register the directive globally or locally in the component's `directives` option",
            );
        }
    }
}

/// `v-focus` / `v-click-outside` resolve to script-setup bindings `vFocus` / `vClickOutside`.
fn directive_setup_binding(name: &str) -> CompactString {
    let camel = vize_l0::camelize(name);
    let mut binding = CompactString::with_capacity(camel.len() + 1);
    binding.push('v');
    let mut chars = camel.chars();
    if let Some(first) = chars.next() {
        for ch in first.to_uppercase() {
            binding.push(ch);
        }
        binding.push_str(chars.as_str());
    }
    binding
}

fn scripts_register_directive(ctx: &LintContext<'_>, directive: &str) -> bool {
    let binding = directive_setup_binding(directive);
    let Some(descriptor) = ctx.sfc_descriptor() else {
        return false;
    };
    descriptor.script_setup.as_ref().is_some_and(|block| {
        script_block_registers(
            block.content.as_ref(),
            block.lang.as_deref(),
            binding.as_str(),
            None,
        )
    }) || descriptor.script.as_ref().is_some_and(|block| {
        script_block_registers(
            block.content.as_ref(),
            block.lang.as_deref(),
            binding.as_str(),
            Some(directive),
        )
    })
}

fn script_block_registers(
    source: &str,
    lang: Option<&str>,
    binding: &str,
    options_name: Option<&str>,
) -> bool {
    let source_type = match lang {
        Some("tsx") => SourceType::tsx(),
        Some("jsx") => SourceType::jsx(),
        _ => SourceType::ts(),
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked {
        return false;
    }
    top_level_value_binding(&parsed.program, binding)
        || options_name
            .is_some_and(|name| options_register_directive(&parsed.program, source, name))
}

fn top_level_value_binding(program: &Program<'_>, name: &str) -> bool {
    for statement in &program.body {
        match statement {
            Statement::ImportDeclaration(import) if !import.import_kind.is_type() => {
                let Some(specifiers) = &import.specifiers else {
                    continue;
                };
                for specifier in specifiers {
                    let local = match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(spec)
                            if !spec.import_kind.is_type() =>
                        {
                            spec.local.name.as_str()
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                            spec.local.name.as_str()
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                            spec.local.name.as_str()
                        }
                        _ => continue,
                    };
                    if local == name {
                        return true;
                    }
                }
            }
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    if binding_declares(&declarator.id, name) {
                        return true;
                    }
                }
            }
            Statement::FunctionDeclaration(function) if !function.declare => {
                if function
                    .id
                    .as_ref()
                    .is_some_and(|id| id.name.as_str() == name)
                {
                    return true;
                }
            }
            Statement::ClassDeclaration(class) if !class.declare => {
                if class.id.as_ref().is_some_and(|id| id.name.as_str() == name) {
                    return true;
                }
            }
            Statement::TSEnumDeclaration(enumeration) if !enumeration.declare => {
                if enumeration.id.name.as_str() == name {
                    return true;
                }
            }
            Statement::ExportNamedDeclaration(export)
                if !export.export_kind.is_type()
                    && export
                        .declaration
                        .as_ref()
                        .is_some_and(|declaration| declaration_declares(declaration, name)) =>
            {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn binding_declares(pattern: &BindingPattern<'_>, name: &str) -> bool {
    pattern
        .get_binding_identifiers()
        .into_iter()
        .any(|identifier| identifier.name.as_str() == name)
}

fn declaration_declares(declaration: &Declaration<'_>, name: &str) -> bool {
    match declaration {
        Declaration::VariableDeclaration(declaration) if !declaration.declare => declaration
            .declarations
            .iter()
            .any(|declarator| binding_declares(&declarator.id, name)),
        Declaration::FunctionDeclaration(function) if !function.declare => function
            .id
            .as_ref()
            .is_some_and(|id| id.name.as_str() == name),
        Declaration::ClassDeclaration(class) if !class.declare => {
            class.id.as_ref().is_some_and(|id| id.name.as_str() == name)
        }
        Declaration::TSEnumDeclaration(enumeration) if !enumeration.declare => {
            enumeration.id.name.as_str() == name
        }
        _ => false,
    }
}

fn options_register_directive(program: &Program<'_>, source: &str, name: &str) -> bool {
    let built = SemanticBuilder::new().with_build_nodes(true).build(program);
    let objects = crate::rules::script::vue_object::VueObjects::collect(
        program,
        source,
        &built.semantic,
        true,
    );
    let mut finder = DirectiveOptionFinder {
        objects: &objects,
        name,
        found: false,
    };
    finder.visit_program(program);
    finder.found
}

struct DirectiveOptionFinder<'o, 'n> {
    objects: &'o crate::rules::script::vue_object::VueObjects,
    name: &'n str,
    found: bool,
}

impl<'a> Visit<'a> for DirectiveOptionFinder<'_, '_> {
    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        if self.objects.contains(object) {
            self.found |= object.properties.iter().any(|property| {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return false;
                };
                !property.computed
                    && property_key_name(&property.key) == Some("directives")
                    && matches!(property.value.without_parentheses(), Expression::ObjectExpression(directives) if directives.properties.iter().any(|entry| {
                        matches!(entry, ObjectPropertyKind::ObjectProperty(entry) if !entry.computed && property_key_name(&entry.key) == Some(self.name))
                    }))
            });
        }
        walk_object_expression(self, object);
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::WarnCustomDirective;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(WarnCustomDirective));
        Linter::with_registry(registry)
    }

    #[test]
    fn configured_global_directive_is_not_reported() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(WarnCustomDirective::with_ignore(vec![
            "hover-tooltip".into(),
        ])));
        let result = Linter::with_registry(registry).lint_template(
            r#"<button v-hover-tooltip="tip"></button><button v-focus></button>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
        assert_eq!(
            result.diagnostics[0].message,
            "Custom directive 'v-focus' detected. Ensure it is properly registered."
        );
    }

    #[test]
    fn test_valid_builtin_v_if() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-if="show"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_builtin_v_model() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input v-model="value" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_custom_directive() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-focus></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_warns_custom_directive_with_arg() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div v-click-outside="handler"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    fn lint_sfc(source: &str) -> crate::linter::LintResult {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(WarnCustomDirective));
        Linter::with_registry(registry).lint_sfc(source, "directive.vue")
    }

    fn custom_directive_warnings(source: &str) -> usize {
        lint_sfc(source)
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/warn-custom-directive")
            .count()
    }

    #[test]
    fn script_setup_local_const_registers_the_directive() {
        let source = r#"<script setup lang="ts">
import type { Directive } from "vue";

const focus: Directive<HTMLElement> = { mounted: el => el.focus() };
const vFocus = focus;
</script>

<template>
  <input v-focus type="text" />
</template>
"#;
        assert_eq!(custom_directive_warnings(source), 0);
    }

    #[test]
    fn script_setup_import_registers_the_directive() {
        let source = r#"<script setup lang="ts">
import { vFocus } from "./focus";
</script>

<template>
  <input v-focus type="text" />
</template>
"#;
        assert_eq!(custom_directive_warnings(source), 0);
    }

    #[test]
    fn unregistered_directive_in_script_setup_is_still_reported() {
        let source = r#"<script setup lang="ts">
const focus = {};
</script>

<template>
  <input v-focus type="text" />
</template>
"#;
        assert_eq!(custom_directive_warnings(source), 1);
    }

    #[test]
    fn accepts_all_compiler_visible_directive_bindings() {
        for source in [
            r#"<script lang="ts">
import { defineComponent } from "vue";
export default defineComponent({ directives: { focus: {} } });
</script>
<template><input v-focus /></template>"#,
            r#"<script lang="ts">import { vFocus } from "./focus";</script>
<script setup lang="ts">const label = "Name";</script>
<template><input v-focus :aria-label="label" /></template>"#,
            r#"<script setup lang="ts">const { vFocus } = useDirectives();</script>
<template><input v-focus /></template>"#,
            r#"<script setup lang="ts">class vFocus {}; enum vTip { mounted = "mounted" }</script>
<template><input v-focus /><b v-tip /></template>"#,
        ] {
            assert_eq!(custom_directive_warnings(source), 0, "{source}");
        }
    }
}
