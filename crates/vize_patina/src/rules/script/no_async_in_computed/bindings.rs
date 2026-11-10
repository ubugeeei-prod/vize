//! Vue `computed` import resolution for the async-computed rule.

use oxc_ast::ast::{Expression, ImportDeclarationSpecifier, Program, Statement};
use vize_l0::{CompactString, FxHashSet};

use super::member_expression;

#[derive(Default)]
pub(super) struct ComputedBindings {
    named: FxHashSet<CompactString>,
    namespaces: FxHashSet<CompactString>,
    bare_computed_is_shadowed: bool,
}

impl ComputedBindings {
    pub(super) fn collect(program: &Program<'_>) -> Self {
        let mut bindings = Self::default();
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            if import.import_kind.is_type() {
                continue;
            }
            let is_vue = matches!(import.source.value.as_str(), "vue" | "@vue/composition-api");
            for specifier in import.specifiers.iter().flatten() {
                if !is_vue {
                    let shadows_computed = match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                            !specifier.import_kind.is_type() && specifier.local.name == "computed"
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                            specifier.local.name == "computed"
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                            specifier.local.name == "computed"
                        }
                    };
                    bindings.bare_computed_is_shadowed |= shadows_computed;
                    continue;
                }
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier)
                        if !specifier.import_kind.is_type()
                            && specifier.imported.name() == "computed" =>
                    {
                        bindings
                            .named
                            .insert(CompactString::new(specifier.local.name.as_str()));
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        bindings
                            .namespaces
                            .insert(CompactString::new(specifier.local.name.as_str()));
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        bindings
                            .namespaces
                            .insert(CompactString::new(specifier.local.name.as_str()));
                    }
                    _ => {}
                }
            }
        }
        bindings
    }

    pub(super) fn resolves(&self, callee: &Expression<'_>) -> bool {
        let callee = callee.get_inner_expression();
        if let Expression::Identifier(identifier) = callee {
            return self.named.contains(identifier.name.as_str())
                || (identifier.name == "computed" && !self.bare_computed_is_shadowed);
        }
        let Some(member) = member_expression(callee) else {
            return false;
        };
        member.static_property_name() == Some("computed")
            && matches!(member.object().get_inner_expression(), Expression::Identifier(identifier)
                if self.namespaces.contains(identifier.name.as_str()))
    }
}
