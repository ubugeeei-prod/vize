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

use super::{ScriptRule, ScriptRuleMeta};
use crate::diagnostic::Severity;
use oxc_ast::ast::{JSXElement, JSXElementName, JSXMemberExpressionObject, Program};
use oxc_ast_visit::Visit;
use oxc_ast_visit::walk::walk_jsx_element;
use oxc_parser::ParserReturn;
use vize_l0::{CompactString, FxHashSet};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "vue/jsx-uses-vars",
    description: "Treat variables used in JSX element names as used",
    default_severity: Severity::Error,
};

/// Implements the `script/jsx-uses-vars` lint rule.
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

/// Root bindings of JSX opening tags in the lint pass's retained script parse.
///
/// `<Foo />` and `<Foo.Bar />` contribute `Foo`. Namespaced tags and `<this.Foo />`
/// contribute nothing, matching eslint-plugin-vue.
pub(crate) fn jsx_referenced_names(parsed: &ParserReturn<'_>) -> FxHashSet<CompactString> {
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
mod tests;
