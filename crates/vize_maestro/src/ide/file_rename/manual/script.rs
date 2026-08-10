//! Syntax-aware module specifier collection and edit generation.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{CallExpression, Expression, ImportExpression, Statement, TSImportType};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use tower_lsp::lsp_types::TextEdit;

use crate::server::ServerState;

use super::{RenameTarget, offset_range};

struct SpecifierOccurrence {
    start: usize,
    end: usize,
    specifier: std::string::String,
}

struct ScriptEditContext<'a> {
    state: &'a ServerState,
    current_path: &'a Path,
    future_path: &'a Path,
    full_source: &'a str,
    rename_targets: &'a [RenameTarget],
}

#[derive(Default)]
struct ModuleSpecifierCollector {
    specifiers: Vec<SpecifierOccurrence>,
}

impl ModuleSpecifierCollector {
    fn push(&mut self, start: u32, end: u32, specifier: &str) {
        self.specifiers.push(SpecifierOccurrence {
            start: start as usize,
            end: end as usize,
            specifier: specifier.to_string(),
        });
    }
}

impl<'a> Visit<'a> for ModuleSpecifierCollector {
    fn visit_program(&mut self, program: &oxc_ast::ast::Program<'a>) {
        for statement in &program.body {
            match statement {
                Statement::ImportDeclaration(decl) => {
                    self.push(
                        decl.source.span.start + 1,
                        decl.source.span.end - 1,
                        decl.source.value.as_str(),
                    );
                }
                Statement::ExportNamedDeclaration(decl) => {
                    if let Some(source) = &decl.source {
                        self.push(
                            source.span.start + 1,
                            source.span.end - 1,
                            source.value.as_str(),
                        );
                    }
                }
                Statement::ExportAllDeclaration(decl) => {
                    self.push(
                        decl.source.span.start + 1,
                        decl.source.span.end - 1,
                        decl.source.value.as_str(),
                    );
                }
                _ => {}
            }
        }

        walk::walk_program(self, program);
    }

    fn visit_import_expression(&mut self, expression: &ImportExpression<'a>) {
        if let Expression::StringLiteral(literal) = &expression.source {
            self.push(
                literal.span.start + 1,
                literal.span.end - 1,
                literal.value.as_str(),
            );
        }

        walk::walk_import_expression(self, expression);
    }

    fn visit_call_expression(&mut self, expression: &CallExpression<'a>) {
        if let Expression::Identifier(identifier) = &expression.callee
            && identifier.name.as_str() == "require"
            && let Some(oxc_ast::ast::Argument::StringLiteral(literal)) =
                expression.arguments.first()
        {
            self.push(
                literal.span.start + 1,
                literal.span.end - 1,
                literal.value.as_str(),
            );
        }

        walk::walk_call_expression(self, expression);
    }

    fn visit_ts_import_type(&mut self, import_type: &TSImportType<'a>) {
        self.push(
            import_type.source.span.start + 1,
            import_type.source.span.end - 1,
            import_type.source.value.as_str(),
        );

        walk::walk_ts_import_type(self, import_type);
    }
}

pub(super) fn collect_vue_edits(
    state: &ServerState,
    path: &Path,
    future_path: &Path,
    source: &str,
    rename_targets: &[RenameTarget],
) -> Vec<TextEdit> {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: path.to_string_lossy().to_string().into(),
        ..Default::default()
    };

    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(source, options) else {
        return Vec::new();
    };

    let mut edits = Vec::new();
    let edit_context = ScriptEditContext {
        state,
        current_path: path,
        future_path,
        full_source: source,
        rename_targets,
    };

    if let Some(script) = descriptor.script.as_ref() {
        edits.extend(collect_script_content_edits(
            &edit_context,
            script.content.as_ref(),
            script_source_type(script.lang.as_deref()),
            script.loc.start,
        ));
    }

    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        edits.extend(collect_script_content_edits(
            &edit_context,
            script_setup.content.as_ref(),
            script_source_type(script_setup.lang.as_deref()),
            script_setup.loc.start,
        ));
    }
    edits.extend(super::super::resources::collect_vue_resource_edits(
        state,
        path,
        future_path,
        source,
        rename_targets,
    ));

    edits
}

pub(super) fn collect_script_file_edits(
    state: &ServerState,
    path: &Path,
    future_path: &Path,
    source: &str,
    rename_targets: &[RenameTarget],
) -> Vec<TextEdit> {
    let Some(source_type) = SourceType::from_path(path).ok() else {
        return Vec::new();
    };

    let edit_context = ScriptEditContext {
        state,
        current_path: path,
        future_path,
        full_source: source,
        rename_targets,
    };

    collect_script_content_edits(&edit_context, source, source_type, 0)
}

fn collect_script_content_edits(
    context: &ScriptEditContext<'_>,
    script_source: &str,
    source_type: SourceType,
    base_offset: usize,
) -> Vec<TextEdit> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script_source, source_type).parse();

    let mut collector = ModuleSpecifierCollector::default();
    collector.visit_program(&parsed.program);

    let Some(future_dir) = context.future_path.parent() else {
        return Vec::new();
    };

    collector
        .specifiers
        .into_iter()
        .filter_map(|specifier| {
            let new_text = super::super::alias::rewrite_specifier(
                context.state,
                context.current_path,
                future_dir,
                &specifier.specifier,
                context.rename_targets,
            )?;

            if new_text == specifier.specifier {
                return None;
            }

            let start_offset = base_offset + specifier.start;
            let end_offset = base_offset + specifier.end;
            let range = offset_range(context.full_source, start_offset, end_offset)?;

            Some(TextEdit { range, new_text })
        })
        .collect()
}

fn script_source_type(lang: Option<&str>) -> SourceType {
    match lang.unwrap_or("js") {
        "ts" => SourceType::ts(),
        "tsx" => SourceType::tsx(),
        "jsx" => SourceType::jsx(),
        _ => SourceType::mjs(),
    }
}
