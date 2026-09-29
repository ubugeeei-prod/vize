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
use oxc_ast::ast::{BindingPattern, Declaration, ImportDeclarationSpecifier, Program, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::{CompactString, cstr};
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
#[derive(Default)]
pub struct WarnCustomDirective;

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
        if !BUILTIN_DIRECTIVES.contains(&name) && !script_setup_registers_directive(ctx, name) {
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

fn script_setup_registers_directive(ctx: &LintContext<'_>, directive: &str) -> bool {
    let binding = directive_setup_binding(directive);
    let Some(setup) = ctx
        .sfc_descriptor()
        .and_then(|descriptor| descriptor.script_setup.as_ref())
    else {
        return false;
    };
    let source = setup.content.as_ref();
    if !contains_identifier(source, binding.as_str()) {
        return false;
    }
    let source_type = match setup.lang.as_deref() {
        Some("tsx") => SourceType::tsx(),
        Some("jsx") => SourceType::jsx(),
        _ => SourceType::ts(),
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked {
        return false;
    }
    top_level_value_binding(&parsed.program, binding.as_str())
}

fn contains_identifier(source: &str, name: &str) -> bool {
    let bytes = source.as_bytes();
    source.match_indices(name).any(|(index, _)| {
        let before = index.checked_sub(1).and_then(|at| bytes.get(at)).copied();
        let after = bytes.get(index + name.len()).copied();
        !is_ident_byte(before) && !is_ident_byte(after)
    })
}

fn is_ident_byte(byte: Option<u8>) -> bool {
    byte.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$')
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
            Statement::ExportNamedDeclaration(export) if !export.export_kind.is_type() => {
                match &export.declaration {
                    Some(Declaration::VariableDeclaration(declaration)) => {
                        for declarator in &declaration.declarations {
                            if binding_declares(&declarator.id, name) {
                                return true;
                            }
                        }
                    }
                    Some(Declaration::FunctionDeclaration(function))
                        if function
                            .id
                            .as_ref()
                            .is_some_and(|id| id.name.as_str() == name) =>
                    {
                        return true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    false
}

fn binding_declares(pattern: &BindingPattern<'_>, name: &str) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => identifier.name.as_str() == name,
        BindingPattern::AssignmentPattern(assignment) => binding_declares(&assignment.left, name),
        _ => false,
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
        // `vue/no-unused-properties` is what makes the SFC path attach the
        // descriptor. This rule reads `<script setup>` from that descriptor.
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(WarnCustomDirective));
        registry.register(Box::new(crate::rules::vue::NoUnusedProperties::default()));
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
}
