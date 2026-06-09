//! Vue-specific script-setup semantic validation for props.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, PropertyKey, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::String;

use crate::script::ScriptCompileContext;
use crate::types::SfcError;

use super::runtime_type::resolve_prop_js_type;
use super::text_resolve::extract_prop_types_from_type_with_context;

/// Validate Vue-specific script-setup semantics that the TypeScript checker
/// cannot derive on its own (currently: prop destructure defaults whose value
/// disagrees with the declared prop type).
///
/// This runs only the analysis needed to drive the validators — no codegen,
/// no template compile — so check/LSP can call it on every SFC without paying
/// the cost of `compile_sfc`. Returns `Ok(())` when there is no `<script
/// setup>` or no actionable issue.
pub fn validate_script_setup_semantics(script_setup_content: &str) -> Result<(), SfcError> {
    // Delegating with a zero block offset yields block-relative diagnostic
    // locations. Callers that know where the `<script setup>` block sits in
    // the full SFC should prefer `validate_script_setup_semantics_located` so
    // diagnostics map to the correct document position.
    validate_script_setup_semantics_located(script_setup_content, 0, script_setup_content)
}

/// Like [`validate_script_setup_semantics`], but rebases diagnostic locations
/// onto the full SFC: `block_start` is the byte offset of the script-setup
/// block content within `sfc_source` (i.e. `script_setup.loc.start`), and
/// `sfc_source` is the whole SFC text used to resolve line/column. This lets
/// editor/check diagnostics point at the offending span (e.g. a destructure
/// default) instead of the start of the `<script setup>` block.
pub fn validate_script_setup_semantics_located(
    script_setup_content: &str,
    block_start: usize,
    sfc_source: &str,
) -> Result<(), SfcError> {
    if script_setup_content.is_empty() {
        return Ok(());
    }
    let mut ctx = ScriptCompileContext::new(script_setup_content);
    ctx.analyze();
    validate_props_destructure_default_types(&ctx, block_start, sfc_source)
}

/// Build a 1-based [`BlockLocation`] for an absolute byte span in `source`.
/// Columns count UTF-16 code units to match the LSP position convention.
fn block_location_for_span(source: &str, start: usize, end: usize) -> crate::types::BlockLocation {
    fn line_col_1based(source: &str, offset: usize) -> (usize, usize) {
        let target = offset.min(source.len());
        let mut line = 1usize;
        let mut column = 1usize;
        for (index, ch) in source.char_indices() {
            if index >= target {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += ch.len_utf16();
            }
        }
        (line, column)
    }
    let (start_line, start_column) = line_col_1based(source, start);
    let (end_line, end_column) = line_col_1based(source, end);
    crate::types::BlockLocation {
        start,
        end,
        tag_start: start,
        tag_end: end,
        start_line,
        start_column,
        end_line,
        end_column,
    }
}

/// Locate the byte span `(start, end)` of a reactive props-destructure
/// default's value expression for `prop_key`, relative to
/// `script_setup_content` (e.g. the `0` in `const { msg = 0 } = defineProps<…>()`).
///
/// Used to point the default-type-mismatch diagnostic at the offending
/// default rather than the start of the `<script setup>` block. Re-parses the
/// script-setup content; only the rare diagnostic error path calls this, so
/// the extra parse is not on the hot path. The span lives on the AST, not on
/// the public `PropsDestructureBinding`, to keep that type's API stable.
fn props_destructure_default_span(
    script_setup_content: &str,
    prop_key: &str,
) -> Option<(u32, u32)> {
    use oxc_ast::ast::BindingPattern;

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script_setup_content, SourceType::ts()).parse();
    for stmt in &parsed.program.body {
        let Statement::VariableDeclaration(var_decl) = stmt else {
            continue;
        };
        for decl in &var_decl.declarations {
            // Match `const { … } = defineProps(…)`.
            let BindingPattern::ObjectPattern(obj_pat) = &decl.id else {
                continue;
            };
            let is_define_props = matches!(
                &decl.init,
                Some(Expression::CallExpression(call))
                    if matches!(&call.callee, Expression::Identifier(id) if id.name == "defineProps")
            );
            if !is_define_props {
                continue;
            }
            for prop in obj_pat.properties.iter() {
                let key = match &prop.key {
                    PropertyKey::StaticIdentifier(id) => id.name.as_str(),
                    PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                    _ => continue,
                };
                if key != prop_key {
                    continue;
                }
                if let BindingPattern::AssignmentPattern(assign) = &prop.value {
                    let span = assign.right.span();
                    return Some((span.start, span.end));
                }
            }
        }
    }
    None
}

/// Validate reactive props destructure defaults against inferred runtime prop types.
pub(crate) fn validate_props_destructure_default_types(
    ctx: &ScriptCompileContext,
    block_start: usize,
    sfc_source: &str,
) -> Result<(), SfcError> {
    let Some(destructure) = ctx.macros.props_destructure.as_ref() else {
        return Ok(());
    };
    let Some(props_macro) = ctx.macros.define_props.as_ref() else {
        return Ok(());
    };
    let Some(type_args) = props_macro.type_args.as_ref() else {
        return Ok(());
    };

    let resolved_type_args =
        crate::script::resolve_type_args(type_args, &ctx.interfaces, &ctx.type_aliases);
    let prop_types = extract_prop_types_from_type_with_context(
        &resolved_type_args,
        Some(&ctx.interfaces),
        Some(&ctx.type_aliases),
    );

    for (name, prop_type) in &prop_types {
        let Some(binding) = destructure.bindings.get(name.as_str()) else {
            continue;
        };
        let Some(default_value) = binding.default.as_deref() else {
            continue;
        };

        let resolved_js_type = if prop_type.js_type == "null" {
            prop_type
                .ts_type
                .as_ref()
                .and_then(|ts_type| {
                    resolve_prop_js_type(ts_type, &ctx.interfaces, &ctx.type_aliases)
                })
                .unwrap_or_else(|| prop_type.js_type.clone())
        } else {
            prop_type.js_type.clone()
        };

        if resolved_js_type == "null" || prop_type.nullable {
            continue;
        }

        let Some(default_type) =
            infer_default_value_runtime_type(default_value, resolved_js_type.as_str())
        else {
            continue;
        };

        if !runtime_type_includes(resolved_js_type.as_str(), default_type) {
            // Point the diagnostic at the offending default expression when we
            // can locate it, rebasing the block-relative span onto the full
            // SFC. Falls back to no location (callers then pick a block-start
            // fallback) only when the span cannot be resolved.
            let loc =
                props_destructure_default_span(&ctx.source, name).map(|(rel_start, rel_end)| {
                    block_location_for_span(
                        sfc_source,
                        block_start + rel_start as usize,
                        block_start + rel_end as usize,
                    )
                });
            return Err(SfcError {
                message: {
                    let mut message = String::from("Default value of prop \"");
                    message.push_str(name);
                    message.push_str("\" does not match declared type.");
                    message
                },
                code: Some("DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE".into()),
                loc,
            });
        }
    }

    Ok(())
}

fn infer_default_value_runtime_type(
    default_value: &str,
    expected_runtime_type: &str,
) -> Option<&'static str> {
    const WRAP_PREFIX: &str = "const __vize_default__ = ";
    let mut wrapped = String::with_capacity(WRAP_PREFIX.len() + default_value.len() + 1);
    wrapped.push_str(WRAP_PREFIX);
    wrapped.push_str(default_value);
    wrapped.push(';');

    let allocator = Allocator::default();
    let parse_result = Parser::new(
        &allocator,
        &wrapped,
        SourceType::default().with_typescript(true),
    )
    .parse();
    if !parse_result.errors.is_empty() {
        return None;
    }

    let Some(Statement::VariableDeclaration(var_decl)) = parse_result.program.body.first() else {
        return None;
    };
    let declarator = var_decl.declarations.first()?;
    let value = declarator.init.as_ref()?;

    infer_expression_runtime_type(unwrap_ts_expression(value), expected_runtime_type)
}

fn unwrap_ts_expression<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    match expr {
        Expression::ParenthesizedExpression(paren) => unwrap_ts_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => unwrap_ts_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            unwrap_ts_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            unwrap_ts_expression(&ts_non_null.expression)
        }
        _ => expr,
    }
}

fn infer_expression_runtime_type(
    expr: &Expression<'_>,
    expected_runtime_type: &str,
) -> Option<&'static str> {
    match expr {
        Expression::StringLiteral(_) => Some("String"),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => Some("String"),
        Expression::NumericLiteral(_) => Some("Number"),
        Expression::BooleanLiteral(_) => Some("Boolean"),
        Expression::ObjectExpression(_) => Some("Object"),
        Expression::ArrayExpression(_) => Some("Array"),
        Expression::ArrowFunctionExpression(arrow) => {
            if runtime_type_includes(expected_runtime_type, "Function") {
                return Some("Function");
            }
            arrow_return_expression(arrow)
                .and_then(|expr| infer_expression_runtime_type(expr, expected_runtime_type))
        }
        Expression::FunctionExpression(func) => {
            if runtime_type_includes(expected_runtime_type, "Function") {
                return Some("Function");
            }
            function_body_return_expression(&func.body.as_ref()?.statements)
                .and_then(|expr| infer_expression_runtime_type(expr, expected_runtime_type))
        }
        _ => None,
    }
}

fn arrow_return_expression<'a>(
    arrow: &'a oxc_ast::ast::ArrowFunctionExpression<'a>,
) -> Option<&'a Expression<'a>> {
    if !arrow.expression {
        return function_body_return_expression(&arrow.body.statements);
    }

    let Statement::ExpressionStatement(expr_stmt) = arrow.body.statements.first()? else {
        return None;
    };
    Some(&expr_stmt.expression)
}

fn function_body_return_expression<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
) -> Option<&'a Expression<'a>> {
    for stmt in statements.iter() {
        if let Statement::ReturnStatement(ret) = stmt
            && let Some(argument) = &ret.argument
        {
            return Some(argument);
        }
    }
    None
}

fn runtime_type_includes(runtime_type: &str, value_type: &str) -> bool {
    runtime_type
        .trim_matches(|c| c == '[' || c == ']')
        .split(',')
        .map(str::trim)
        .any(|part| part == value_type)
}
