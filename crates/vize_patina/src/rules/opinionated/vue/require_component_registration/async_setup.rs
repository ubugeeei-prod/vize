//! Resolve local `<script setup>` components created by Vue's async factory.
//!
//! Vue exposes top-level setup bindings to the template, so
//! `const Panel = defineAsyncComponent(...)` is a component even though
//! `Panel` itself is not imported. Only a call to the value imported from
//! `vue` counts; a coincidentally named local helper does not.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Expression, ImportDeclarationSpecifier, Statement, VariableDeclarationKind,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::SfcScriptBlock;
use vize_s0::{CompactString, FxHashSet};

pub(super) fn async_component_names(script: &SfcScriptBlock<'_>) -> Vec<CompactString> {
    let source = script.content.as_ref();
    if !source.contains("defineAsyncComponent") {
        return Vec::new();
    }

    let allocator = Allocator::default();
    let path = if matches!(script.lang.as_deref(), Some("tsx" | "jsx")) {
        "script.tsx"
    } else {
        "script.ts"
    };
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked {
        return Vec::new();
    }

    let mut named_imports = FxHashSet::default();
    let mut namespaces = FxHashSet::default();
    for statement in &parsed.program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.source.value != "vue" || import.import_kind.is_type() {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if !specifier.import_kind.is_type()
                        && specifier.imported.name() == "defineAsyncComponent" =>
                {
                    named_imports.insert(CompactString::new(specifier.local.name.as_str()));
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    namespaces.insert(CompactString::new(specifier.local.name.as_str()));
                }
                _ => {}
            }
        }
    }
    if named_imports.is_empty() && namespaces.is_empty() {
        return Vec::new();
    }

    let mut components = Vec::new();
    for statement in &parsed.program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        if declaration.kind != VariableDeclarationKind::Const {
            continue;
        }
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
                continue;
            };
            let Some(Expression::CallExpression(call)) = declarator.init.as_ref() else {
                continue;
            };
            if call.arguments.is_empty() {
                continue;
            }
            let is_vue_factory = match &call.callee {
                Expression::Identifier(identifier) => named_imports.contains(identifier.name.as_str()),
                callee if callee.is_member_expression() => callee
                    .as_member_expression()
                    .is_some_and(|member| {
                        member.static_property_name() == Some("defineAsyncComponent")
                            && matches!(member.object(), Expression::Identifier(identifier) if namespaces.contains(identifier.name.as_str()))
                    }),
                _ => false,
            };
            if is_vue_factory {
                components.push(CompactString::new(binding.name.as_str()));
            }
        }
    }
    components
}
