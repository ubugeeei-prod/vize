//! Project evidence for whether server rendering is possible.
//!
//! A client build is also part of a universal application. Vite build/compile
//! target flags therefore do not establish a client-only application.

use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

pub(super) fn ssr_enabled(root: &Path) -> bool {
    let mut directory = Some(root);
    while let Some(root) = directory {
        for name in [
            "nuxt.config.ts",
            "nuxt.config.js",
            "nuxt.config.mts",
            "nuxt.config.mjs",
            "nuxt.config.cts",
            "nuxt.config.cjs",
        ] {
            let path = root.join(name);
            if path.exists() {
                // Unreadable, dynamic or invalid configuration cannot prove that
                // an otherwise reachable browser read never runs on a server.
                return fs::read_to_string(path)
                    .ok()
                    .and_then(|source| nuxt_ssr(&source))
                    != Some(false);
            }
        }
        // A command launched in src/ still belongs to its application. Stop at
        // a nested package or Vite project instead of borrowing an outer SPA's
        // policy for a separately owned application.
        if root.join("package.json").is_file()
            || [
                "vite.config.ts",
                "vite.config.js",
                "vite.config.mts",
                "vite.config.mjs",
                "vite.config.cts",
                "vite.config.cjs",
            ]
            .iter()
            .any(|name| root.join(name).is_file())
        {
            break;
        }
        directory = root.parent();
    }
    true
}

fn nuxt_ssr(source: &str) -> Option<bool> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let export = parsed
        .program
        .body
        .iter()
        .find_map(|statement| match statement {
            Statement::ExportDefaultDeclaration(export) => Some(export),
            _ => None,
        })?;
    let expression = unwrap(export.declaration.as_expression()?);
    let expression = match expression {
        Expression::CallExpression(call) if matches!(&call.callee, Expression::Identifier(name) if name.name == "defineNuxtConfig") =>
        {
            // A locally defined function with this name is not the Nuxt
            // configuration identity. Keep the result unknown in that case.
            if shadows_nuxt_config(&parsed.program) {
                return None;
            }
            unwrap(call.arguments.first()?.as_expression()?)
        }
        expression => expression,
    };
    let Expression::ObjectExpression(object) = expression else {
        return None;
    };
    let mut ssr = None;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            // Later literal properties may override a spread's unknown key.
            ssr = None;
            continue;
        };
        let key = match &property.key {
            PropertyKey::StaticIdentifier(name) if !property.computed => Some(name.name.as_str()),
            PropertyKey::StringLiteral(name) => Some(name.value.as_str()),
            _ => None,
        };
        match key {
            // Nuxt environment-specific blocks can replace root settings.
            Some(key) if key.starts_with('$') => return None,
            Some("ssr") => {
                ssr = match unwrap(&property.value) {
                    Expression::BooleanLiteral(value) => Some(value.value),
                    _ => None,
                };
            }
            Some(_) => {}
            None => ssr = None,
        }
    }
    ssr
}

fn unwrap<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(inner) => unwrap(&inner.expression),
        Expression::TSAsExpression(inner) => unwrap(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => unwrap(&inner.expression),
        Expression::TSNonNullExpression(inner) => unwrap(&inner.expression),
        expression => expression,
    }
}

fn shadows_nuxt_config(program: &oxc_ast::ast::Program<'_>) -> bool {
    use oxc_ast::ast::{BindingIdentifier, ImportDeclaration, ImportDeclarationSpecifier};
    use oxc_ast_visit::Visit;

    #[derive(Default)]
    struct Bindings(bool);
    impl<'a> Visit<'a> for Bindings {
        fn visit_binding_identifier(&mut self, binding: &BindingIdentifier<'a>) {
            self.0 |= binding.name == "defineNuxtConfig";
        }

        fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
            for specifier in import.specifiers.iter().flatten() {
                if matches!(specifier, ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if import.source.value == "nuxt/config"
                        && specifier.local.name == "defineNuxtConfig"
                        && specifier.imported.name() == "defineNuxtConfig")
                {
                    continue;
                }
                self.visit_import_declaration_specifier(specifier);
            }
        }
    }
    let mut bindings = Bindings::default();
    bindings.visit_program(program);
    bindings.0
}

#[cfg(test)]
mod tests {
    use super::{nuxt_ssr, ssr_enabled};

    #[test]
    fn source_directories_inherit_app_policy_but_nested_projects_do_not() {
        let project = tempfile::tempdir().unwrap();
        std::fs::write(
            project.path().join("nuxt.config.ts"),
            "export default defineNuxtConfig({ssr:false})",
        )
        .unwrap();
        let source = project.path().join("src");
        std::fs::create_dir_all(&source).unwrap();
        assert!(!ssr_enabled(&source));
        std::fs::write(source.join("package.json"), "{}").unwrap();
        assert!(ssr_enabled(&source));
    }

    #[test]
    fn only_complete_default_export_configuration_can_disable_ssr() {
        for source in [
            "export default defineNuxtConfig({ ssr: false })",
            "export default ({ 'ssr': false } as const)",
            "export default defineNuxtConfig({ ...base, ssr: false } satisfies NuxtConfig)",
            "export default defineNuxtConfig({ ssr: true, ssr: false })",
            "import { defineNuxtConfig } from 'nuxt/config'; export default defineNuxtConfig({ ssr: false })",
        ] {
            assert_eq!(nuxt_ssr(source), Some(false), "{source}");
        }
        for source in [
            "export default defineNuxtConfig({ ssr: false, ...base })",
            "export default defineNuxtConfig({ ssr: false, [key]: true })",
            "export default defineNuxtConfig({ ssr: false, $production: { ssr: true } })",
            "export default defineNuxtConfig({ ssr: process.env.SSR })",
            "export default transform({ ssr: false })",
            "const defineNuxtConfig = () => ({ssr:true}); export default defineNuxtConfig({ssr:false})",
            "import { defineNuxtConfig } from 'unrelated'; export default defineNuxtConfig({ssr:false})",
            "export const ssr = false; export default defineNuxtConfig({})",
            "export default defineNuxtConfig({ app: { ssr: false } })",
            "// ssr: false\nexport default defineNuxtConfig({})",
        ] {
            assert_eq!(nuxt_ssr(source), None, "{source}");
        }
        assert_eq!(
            nuxt_ssr("export default defineNuxtConfig({ ssr: false, ssr: true })"),
            Some(true)
        );
    }
}
