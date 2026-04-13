//! AST-based top-level statement extraction for script setup compilation.
//!
//! This keeps imports, TypeScript declarations, and setup code separated using
//! precise OXC statement spans instead of line-based heuristics.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, Expression, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};

use vize_carton::{String, ToCompactString};
use vize_croquis::macros::is_builtin_macro;

enum StatementBucket {
    Import,
    TypeDeclaration,
    Macro,
    Setup,
}

pub(crate) fn extract_script_sections(
    content: &str,
    is_ts: bool,
) -> Option<(Vec<String>, Vec<String>, Vec<String>)> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, content, source_type).parse();

    if ret.panicked {
        return None;
    }

    let mut user_imports = Vec::new();
    let mut setup_lines = Vec::new();
    let mut ts_declarations = Vec::new();

    let mut prev_end = 0usize;
    let mut pending_gap = String::default();

    for stmt in ret.program.body.iter() {
        let span = stmt.span();
        let start = span.start as usize;
        let end = span.end as usize;

        if start < prev_end || end > content.len() || start > end {
            return None;
        }

        pending_gap.push_str(&content[prev_end..start]);

        let slice = &content[start..end];
        match classify_statement(stmt, slice) {
            StatementBucket::Import => {
                let mut segment = std::mem::take(&mut pending_gap);
                segment.push_str(slice);
                user_imports.push(normalize_statement_segment(&segment));
            }
            StatementBucket::TypeDeclaration => {
                let mut segment = std::mem::take(&mut pending_gap);
                segment.push_str(slice);
                if is_ts {
                    ts_declarations.push(normalize_preserved_segment(&segment));
                }
            }
            StatementBucket::Setup => {
                let mut segment = std::mem::take(&mut pending_gap);
                segment.push_str(slice);
                push_non_empty_lines(&mut setup_lines, &segment);
            }
            StatementBucket::Macro => {}
        }

        prev_end = end;
    }

    pending_gap.push_str(&content[prev_end..]);
    if !pending_gap.trim().is_empty() {
        push_non_empty_lines(&mut setup_lines, &pending_gap);
    }

    Some((user_imports, setup_lines, ts_declarations))
}

fn classify_statement(stmt: &Statement<'_>, slice: &str) -> StatementBucket {
    let trimmed = slice.trim_start();

    if trimmed.starts_with("declare ") {
        return StatementBucket::TypeDeclaration;
    }

    match stmt {
        Statement::ImportDeclaration(_) => StatementBucket::Import,
        Statement::TSInterfaceDeclaration(_) | Statement::TSTypeAliasDeclaration(_) => {
            StatementBucket::TypeDeclaration
        }
        Statement::ExportNamedDeclaration(export_decl) => {
            if export_decl.export_kind.is_type()
                || export_decl.declaration.as_ref().is_some_and(|decl| {
                    matches!(
                        decl,
                        Declaration::TSInterfaceDeclaration(_)
                            | Declaration::TSTypeAliasDeclaration(_)
                    )
                })
                || trimmed.starts_with("export type ")
                || trimmed.starts_with("export interface ")
            {
                StatementBucket::TypeDeclaration
            } else {
                StatementBucket::Setup
            }
        }
        Statement::ExpressionStatement(expr_stmt) => {
            if unwrap_call_expression(&expr_stmt.expression).is_some_and(is_macro_call) {
                StatementBucket::Macro
            } else {
                StatementBucket::Setup
            }
        }
        Statement::VariableDeclaration(var_decl) => {
            if var_decl.declarations.iter().any(|decl| {
                decl.init
                    .as_ref()
                    .and_then(unwrap_call_expression)
                    .is_some_and(is_macro_call)
            }) {
                StatementBucket::Macro
            } else {
                StatementBucket::Setup
            }
        }
        _ => StatementBucket::Setup,
    }
}

fn unwrap_call_expression<'a>(
    expr: &'a Expression<'a>,
) -> Option<&'a oxc_ast::ast::CallExpression<'a>> {
    match expr {
        Expression::CallExpression(call) => Some(call),
        Expression::TSAsExpression(ts_as) => unwrap_call_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            unwrap_call_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            unwrap_call_expression(&ts_non_null.expression)
        }
        Expression::ParenthesizedExpression(paren) => unwrap_call_expression(&paren.expression),
        _ => None,
    }
}

fn is_macro_call(call: &oxc_ast::ast::CallExpression<'_>) -> bool {
    match &call.callee {
        Expression::Identifier(id) => is_builtin_macro(id.name.as_str()),
        _ => false,
    }
}

fn normalize_statement_segment(segment: &str) -> String {
    let trimmed = normalize_preserved_segment(segment);
    let mut normalized = trimmed;
    normalized.push('\n');
    normalized
}

fn normalize_preserved_segment(segment: &str) -> String {
    segment
        .trim_start_matches(['\n', '\r'])
        .trim_end_matches(['\n', '\r'])
        .to_compact_string()
}

fn push_non_empty_lines(lines: &mut Vec<String>, segment: &str) {
    for line in segment.lines() {
        if !line.trim().is_empty() {
            lines.push(line.to_compact_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::extract_script_sections;

    #[test]
    fn test_extract_script_sections_skips_next_line_macro_assignment() {
        let content = r#"const props =
  defineProps<{
    name: string
  }>()

const count = 1
"#;

        let (_, setup_lines, ts_declarations) =
            extract_script_sections(content, true).expect("sections should parse");

        insta::assert_debug_snapshot!((&setup_lines, &ts_declarations));
    }

    #[test]
    fn test_extract_script_sections_trims_leading_blank_lines_from_type_declarations() {
        let content = r#"import { useStore } from 'vuex'

interface RootState {
  count: number
}

const store = useStore<RootState>()
"#;

        let (_, _, ts_declarations) =
            extract_script_sections(content, true).expect("sections should parse");

        assert_eq!(ts_declarations.len(), 1);
        assert_eq!(
            ts_declarations[0].as_str(),
            "interface RootState {\n  count: number\n}"
        );
    }
}
