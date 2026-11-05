//! vue/jsx-uses-vars
//!
//! A variable named by a JSX tag is used. ESLint's `no-unused-vars` does not
//! see JSX identifiers, so eslint-plugin-vue marks the tag's root binding.
//! Patina applies that mark when deciding whether a registered component is
//! unused.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script setup>
//! import MyButton from './MyButton.vue'
//! </script>
//! <template><div /></template>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup>
//! import MyButton from './MyButton.vue'
//! const view = () => <MyButton />
//! </script>
//! <template><div /></template>
//! ```

use super::{ScriptRule, ScriptRuleMeta, script_source_type};
use crate::diagnostic::Severity;
use oxc_allocator::Allocator;
use oxc_ast::ast::{JSXElement, JSXElementName, JSXMemberExpressionObject, Program};
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::walk_jsx_element;
use oxc_parser::Parser;
use vize_l0::{CompactString, FxHashSet};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "vue/jsx-uses-vars",
    description: "Treat variables used in JSX element names as used",
    default_severity: Severity::Error,
};

pub struct JsxUsesVars;

impl ScriptRule for JsxUsesVars {
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
        result: &mut super::ScriptLintResult,
    ) {
        let _ = (offset, result);
        if source.contains('<') {
            let _ = referenced_in_program(program);
        }
    }
}

/// Root bindings of JSX opening tags in `source`.
///
/// `<Foo />` and `<Foo.Bar />` contribute `Foo`. Namespaced tags and `<this.Foo />`
/// contribute nothing, matching eslint-plugin-vue.
pub(crate) fn jsx_referenced_names(source: &str) -> FxHashSet<CompactString> {
    if !source.contains('<') {
        return FxHashSet::default();
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, script_source_type().with_jsx(true)).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return FxHashSet::default();
    }
    referenced_in_program(&parsed.program)
}

fn referenced_in_program(program: &Program<'_>) -> FxHashSet<CompactString> {
    let mut visitor = JsxUseVisitor {
        names: FxHashSet::default(),
    };
    visitor.visit_program(program);
    visitor.names
}

struct JsxUseVisitor {
    names: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for JsxUseVisitor {
    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        if let Some(name) = root_binding(&element.opening_element.name) {
            self.names.insert(CompactString::new(name));
        }
        walk_jsx_element(self, element);
    }
}

fn root_binding<'a>(name: &'a JSXElementName<'a>) -> Option<&'a str> {
    match name {
        JSXElementName::Identifier(identifier) => Some(identifier.name.as_str()),
        JSXElementName::IdentifierReference(identifier) => Some(identifier.name.as_str()),
        JSXElementName::MemberExpression(expression) => member_root(&expression.object),
        JSXElementName::NamespacedName(_) | JSXElementName::ThisExpression(_) => None,
    }
}

fn member_root<'a>(object: &'a JSXMemberExpressionObject<'a>) -> Option<&'a str> {
    match object {
        JSXMemberExpressionObject::IdentifierReference(identifier) => {
            Some(identifier.name.as_str())
        }
        JSXMemberExpressionObject::MemberExpression(expression) => member_root(&expression.object),
        JSXMemberExpressionObject::ThisExpression(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::jsx_referenced_names;
    use crate::Linter;

    fn names(source: &str) -> Vec<String> {
        let mut names: Vec<_> = jsx_referenced_names(source)
            .into_iter()
            .map(|name| name.to_string())
            .collect();
        names.sort();
        names
    }

    fn lint(source: &str) -> Vec<String> {
        Linter::new()
            .with_enabled_rules(Some(vec![
                "vue/no-unused-components".into(),
                "vue/jsx-uses-vars".into(),
            ]))
            .lint_sfc(source, "test.vue")
            .diagnostics
            .into_iter()
            .map(|diagnostic| format!("{}: {}", diagnostic.rule_name, diagnostic.message))
            .collect()
    }

    #[test]
    fn marks_jsx_roots_and_skips_namespaced_tags() {
        assert_eq!(names("const view = <MyButton />"), ["MyButton"]);
        assert_eq!(names("const view = <MyButton.Item />"), ["MyButton"]);
        assert_eq!(names("const view = <Foo.Bar.Baz />"), ["Foo"]);
        assert_eq!(names("const view = <div><span /></div>"), ["div", "span"]);
        assert_eq!(names("const view = <svg:path />"), Vec::<String>::new());
        assert_eq!(names("const view = <this.Foo />"), Vec::<String>::new());
        assert_eq!(names("const MyButton = 1"), Vec::<String>::new());
    }

    #[test]
    fn jsx_use_keeps_the_component_import() {
        let source = r#"<script setup>
import MyButton from './MyButton.vue'
const view = () => <MyButton />
</script>
<template><div /></template>
"#;
        assert_eq!(lint(source), Vec::<String>::new());
    }

    #[test]
    fn an_unreferenced_component_import_is_still_unused() {
        let source = r#"<script setup>
import MyButton from './MyButton.vue'
const view = () => <div />
</script>
<template><div /></template>
"#;
        assert_eq!(
            lint(source),
            [
                "vue/no-unused-components: Component 'MyButton' is registered but never used in template"
            ]
        );
    }
}
