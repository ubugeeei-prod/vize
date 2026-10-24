//! AST visitor collecting every module specifier literal the rewriter may
//! replace, with the span of the literal's *contents* (quotes excluded).

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    CallExpression, ExportAllDeclaration, ExportNamedDeclaration, Expression, ImportAttributeKey,
    ImportDeclaration, ImportExpression, ObjectExpression, ObjectPropertyKind, PropertyKey,
    Statement, StringLiteral, TSExternalModuleReference, TSImportType, TSModuleDeclarationName,
    WithClause,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::String;

pub(super) fn collect_specifier_occurrences(
    source: &str,
    source_type: SourceType,
) -> Vec<(String, crate::PackageResolutionMode)> {
    let mut occurrences = Vec::new();
    for (_, _, path, mode) in collect_specifier_spans(source, source_type) {
        if !occurrences.contains(&(path.clone(), mode)) {
            occurrences.push((path, mode));
        }
    }
    occurrences
}

pub(super) fn collect_specifier_spans(
    source: &str,
    source_type: SourceType,
) -> Vec<(u32, u32, String, crate::PackageResolutionMode)> {
    let allocator = Allocator::default();
    let result =
        vize_croquis::script_parser::parse_program_for_analysis(&allocator, source, source_type);
    let mut collector = ModuleSpecifierCollector::new();
    collector.visit_program(&result.program);
    if result.panicked {
        // A fatal error after otherwise valid imports can leave OXC's partial
        // program without its leading statements. Recover only facts from the
        // longest line boundary before the fatal parser error that parses as a
        // complete program. Generated Vue documents can append hundreds of
        // lines after an authored hole, so starting at EOF would spend the
        // entire bounded search inside the generated suffix.
        // The original source is still rewritten at these exact spans; no
        // repaired body or inferred dependency is admitted.
        // Keep editor latency bounded even for a very large broken buffer.
        let recovery_end = result
            .diagnostics
            .last()
            .and_then(|diagnostic| {
                diagnostic
                    .labels
                    .iter()
                    .find(|label| label.primary())
                    .or_else(|| diagnostic.labels.first())
            })
            .map_or(source.len(), |label| label.offset() as usize)
            .min(source.len());
        let recovery_source = source.get(..recovery_end).unwrap_or(source);
        for (line_end, _) in recovery_source.match_indices('\n').rev().take(128) {
            let Some(prefix) = source.get(..line_end + 1) else {
                continue;
            };
            let parsed = Parser::new(&allocator, prefix, source_type).parse();
            if parsed.panicked || !parsed.diagnostics.is_empty() {
                continue;
            }
            // The prefix parsed cleanly, so it has the same specifiers a
            // complete program would. Declaration-only collection drops
            // `import()`, `require()`, import types, and import-equals.
            let mut prefix_collector = ModuleSpecifierCollector::new();
            prefix_collector.visit_program(&parsed.program);
            if prefix_collector.specifiers.is_empty() {
                continue;
            }
            for occurrence in prefix_collector.specifiers {
                if !collector.specifiers.iter().any(|existing| {
                    existing.0 == occurrence.0
                        && existing.1 == occurrence.1
                        && existing.3 == occurrence.3
                }) {
                    collector.specifiers.push(occurrence);
                }
            }
            break;
        }
    }
    collector.specifiers.sort_by_key(|occurrence| occurrence.0);
    collector.specifiers
}

pub(super) struct BareStarExport {
    pub(super) header: oxc_span::Span,
    pub(super) literal: oxc_span::Span,
    pub(super) is_type: bool,
    pub(super) specifier: String,
    pub(super) mode: crate::PackageResolutionMode,
}

pub(super) struct ModuleSpecifierCollector {
    pub(super) specifiers: Vec<(u32, u32, String, crate::PackageResolutionMode)>,
    pub(super) empty_stars: Vec<BareStarExport>,
    capture_stars: bool,
    statement_depth: usize,
}

impl ModuleSpecifierCollector {
    pub(super) fn new() -> Self {
        Self {
            specifiers: Vec::new(),
            empty_stars: Vec::new(),
            capture_stars: false,
            statement_depth: 0,
        }
    }

    pub(super) fn with_stars() -> Self {
        Self {
            capture_stars: true,
            ..Self::new()
        }
    }

    fn push(&mut self, start: u32, end: u32, specifier: &str, mode: crate::PackageResolutionMode) {
        self.specifiers
            .push((start + 1, end - 1, specifier.into(), mode));
    }

    fn push_literal(&mut self, lit: &StringLiteral<'_>, mode: crate::PackageResolutionMode) {
        self.push(lit.span.start, lit.span.end, lit.value.as_str(), mode);
    }
}

impl<'a> Visit<'a> for ModuleSpecifierCollector {
    fn visit_statement(&mut self, statement: &Statement<'a>) {
        if !self.capture_stars {
            walk::walk_statement(self, statement);
            return;
        }
        self.statement_depth += 1;
        walk::walk_statement(self, statement);
        self.statement_depth -= 1;
    }

    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        self.push_literal(
            &decl.source,
            mode_from_with_clause(decl.with_clause.as_deref()),
        );
        walk::walk_import_declaration(self, decl);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &decl.source {
            self.push_literal(source, mode_from_with_clause(decl.with_clause.as_deref()));
        }
        walk::walk_export_named_declaration(self, decl);
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        if self.capture_stars && self.statement_depth == 1 && decl.exported.is_none() {
            self.empty_stars.push(BareStarExport {
                header: oxc_span::Span::new(decl.span.start, decl.source.span.start),
                literal: oxc_span::Span::new(decl.source.span.start + 1, decl.source.span.end - 1),
                is_type: decl.export_kind.is_type(),
                specifier: decl.source.value.as_str().into(),
                mode: mode_from_with_clause(decl.with_clause.as_deref()),
            });
        }
        self.push_literal(
            &decl.source,
            mode_from_with_clause(decl.with_clause.as_deref()),
        );
        walk::walk_export_all_declaration(self, decl);
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        if let Expression::StringLiteral(lit) = &expr.source {
            self.push_literal(lit, mode_from_options(expr.options.as_ref()));
        }
        walk::walk_import_expression(self, expr);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Some(lit) = expr.common_js_require() {
            self.push(
                lit.span.start,
                lit.span.end,
                lit.value.as_str(),
                crate::PackageResolutionMode::Require,
            );
        }
        walk::walk_call_expression(self, expr);
    }

    fn visit_ts_import_type(&mut self, import_type: &TSImportType<'a>) {
        self.push_literal(
            &import_type.source,
            import_type
                .options
                .as_deref()
                .and_then(mode_from_options_object)
                .unwrap_or(crate::PackageResolutionMode::Contextual),
        );
        walk::walk_ts_import_type(self, import_type);
    }

    fn visit_ts_external_module_reference(&mut self, reference: &TSExternalModuleReference<'a>) {
        self.push_literal(&reference.expression, crate::PackageResolutionMode::Require);
        walk::walk_ts_external_module_reference(self, reference);
    }

    fn visit_ts_module_declaration_name(&mut self, name: &TSModuleDeclarationName<'a>) {
        if let TSModuleDeclarationName::StringLiteral(lit) = name {
            self.push_literal(lit, crate::PackageResolutionMode::Contextual);
        }
        walk::walk_ts_module_declaration_name(self, name);
    }
}

fn mode_from_with_clause(clause: Option<&WithClause<'_>>) -> crate::PackageResolutionMode {
    clause
        .into_iter()
        .flat_map(|clause| &clause.with_entries)
        .find_map(|attribute| {
            let key = match &attribute.key {
                ImportAttributeKey::Identifier(key) => key.name.as_str(),
                ImportAttributeKey::StringLiteral(key) => key.value.as_str(),
            };
            (key == "resolution-mode")
                .then(|| {
                    crate::PackageResolutionMode::from_explicit_attribute(&attribute.value.value)
                })
                .flatten()
        })
        .unwrap_or(crate::PackageResolutionMode::Contextual)
}

fn mode_from_options(options: Option<&Expression<'_>>) -> crate::PackageResolutionMode {
    options
        .and_then(expression_object)
        .and_then(mode_from_options_object)
        .unwrap_or(crate::PackageResolutionMode::Import)
}

fn mode_from_options_object(
    options: &ObjectExpression<'_>,
) -> Option<crate::PackageResolutionMode> {
    let attributes = expression_object(
        object_property(options, "with").or_else(|| object_property(options, "assert"))?,
    )?;
    let Expression::StringLiteral(value) = object_property(attributes, "resolution-mode")? else {
        return None;
    };
    crate::PackageResolutionMode::from_explicit_attribute(&value.value)
}

fn expression_object<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    let Expression::ObjectExpression(object) = expression else {
        return None;
    };
    Some(object)
}

fn object_property<'a>(object: &'a ObjectExpression<'a>, name: &str) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        let key = match &property.key {
            PropertyKey::StaticIdentifier(key) => key.name.as_str(),
            PropertyKey::StringLiteral(key) => key.value.as_str(),
            _ => return None,
        };
        (key == name).then_some(&property.value)
    })
}
