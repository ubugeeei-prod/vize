use oxc_allocator::Allocator;
use oxc_ast::ast::{
    CallExpression, ExportAllDeclaration, ExportNamedDeclaration, Expression, ImportDeclaration,
    ImportExpression, StringLiteral, TSExternalModuleReference, TSImportType,
    TSModuleDeclarationName,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{String, ToCompactString};

pub(super) fn collect_relative_ts_specifiers(code: &str, source_type: SourceType) -> Vec<String> {
    let allocator = Allocator::default();
    let result = Parser::new(&allocator, code, source_type).parse();
    let mut collector = RelativeTsSpecifierCollector::default();
    collector.visit_program(&result.program);
    collector.specifiers
}

#[derive(Default)]
struct RelativeTsSpecifierCollector {
    specifiers: Vec<String>,
}

impl RelativeTsSpecifierCollector {
    fn push(&mut self, specifier: &str) {
        if (specifier.starts_with("./") || specifier.starts_with("../"))
            && !specifier.ends_with(".vue")
            && !specifier.ends_with(".vue.ts")
            && !specifier.ends_with(".vue.tsx")
            && !self
                .specifiers
                .iter()
                .any(|known| known.as_str() == specifier)
        {
            self.specifiers.push(specifier.to_compact_string());
        }
    }

    fn push_literal(&mut self, lit: &StringLiteral<'_>) {
        self.push(lit.value.as_str());
    }
}

impl<'a> Visit<'a> for RelativeTsSpecifierCollector {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        self.push_literal(&decl.source);
        walk::walk_import_declaration(self, decl);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &decl.source {
            self.push_literal(source);
        }
        walk::walk_export_named_declaration(self, decl);
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        self.push_literal(&decl.source);
        walk::walk_export_all_declaration(self, decl);
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        if let Expression::StringLiteral(lit) = &expr.source {
            self.push_literal(lit);
        }
        walk::walk_import_expression(self, expr);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Some(lit) = expr.common_js_require() {
            self.push(lit.value.as_str());
        }
        walk::walk_call_expression(self, expr);
    }

    fn visit_ts_import_type(&mut self, import_type: &TSImportType<'a>) {
        self.push_literal(&import_type.source);
        walk::walk_ts_import_type(self, import_type);
    }

    fn visit_ts_external_module_reference(&mut self, reference: &TSExternalModuleReference<'a>) {
        self.push_literal(&reference.expression);
        walk::walk_ts_external_module_reference(self, reference);
    }

    fn visit_ts_module_declaration_name(&mut self, name: &TSModuleDeclarationName<'a>) {
        if let TSModuleDeclarationName::StringLiteral(lit) = name {
            self.push_literal(lit);
        }
        walk::walk_ts_module_declaration_name(self, name);
    }
}

#[cfg(test)]
mod tests {
    use oxc_span::SourceType;

    use super::collect_relative_ts_specifiers;

    #[test]
    fn collects_type_only_and_require_dependency_specifiers() {
        let source = r#"import type { User } from "./user";
export type { Model } from "../model";
export type Lazy = import("./lazy").Lazy;
import Common = require("./common");
const runtime = require("./runtime");
type VirtualTs = typeof import("./App.vue.ts");
type VirtualTsx = typeof import("./App.vue.tsx");
declare module "./augment" {}
type App = typeof import("./App.vue");
"#;

        assert_eq!(
            collect_relative_ts_specifiers(source, SourceType::ts()),
            vec![
                "./user",
                "../model",
                "./lazy",
                "./common",
                "./runtime",
                "./augment"
            ]
        );
    }
}
