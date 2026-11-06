//! Template expression statement generation.
//!
//! Emits TypeScript `void (...)` statements for template expressions, with
//! optional v-if narrowing, and delegates recognized v-if chains to the
//! control-flow emitter in [`super::vif_chain`].

use super::super::types::VizeMapping;
use super::binding_mapping::BindingSource;
use super::component_ref_callbacks::generate_component_ref_callback_statement;
use super::directive_values::generate_directive_value_statement;
use super::native_props::generate_native_prop_statement;
use super::reserved_props::{map_rewritten_binding_source, rewrite_reserved_template_binding};
use super::statement_guard::generate_vif_guard_expression;
use super::ts_suppression_comments::{
    expression_source_for_typecheck, inference_callback_parameters_on_ignored_lines,
};
use super::value_checks::TemplateValueChecks;
use super::vif_chain::{VifControlFlowChain, emit_vif_control_flow_chain};
use crate::virtual_ts::scope::{
    append_chained_vif_guard_close, append_chained_vif_guard_open,
    remove_enclosing_vif_guard_prefix,
};
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use std::borrow::Cow;
use vize_carton::{CompactString, FxHashSet, String, append, cstr, profile};
use vize_croquis::croquis::{TemplateExpression, TemplateExpressionKind};
use vize_croquis::drawer::{
    EventHandlerExpression, classify_event_handler, extract_identifier_refs_checked,
    strip_js_comments_with_offsets,
};

pub(crate) fn guarded_callback_reads(expr: &TemplateExpression) -> Vec<CompactString> {
    let Some(guard) = expr.vif_guard.as_deref() else {
        return Vec::new();
    };
    if expr.kind != TemplateExpressionKind::VOn
        || !matches!(
            classify_event_handler(expr.content.as_str()),
            EventHandlerExpression::Callback { .. }
        )
    {
        return Vec::new();
    }
    let Some(callback_reads) = extract_identifier_refs_checked(expr.content.as_str()) else {
        return Vec::new();
    };
    let Some(guard_reads) = extract_identifier_refs_checked(guard) else {
        return Vec::new();
    };
    let guarded: FxHashSet<_> = guard_reads.into_iter().map(|read| read.name).collect();
    let mut emitted = FxHashSet::default();
    callback_reads
        .into_iter()
        .filter_map(|read| {
            (guarded.contains(&read.name) && emitted.insert(read.name.clone())).then_some(read.name)
        })
        .collect()
}

/// Generate template expressions, compacting recognized v-if chains into
/// TypeScript control-flow blocks.
pub(crate) fn generate_expressions(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    exprs: &[&TemplateExpression],
    template_binding_access: &TemplateBindingAccess,
    context: &ExpressionListEmitContext<'_>,
) {
    let mut index = 0;
    while let Some(&expr) = exprs.get(index) {
        if context
            .skipped_expression_ranges
            .contains(&(expr.start, expr.end))
        {
            index += 1;
            continue;
        }
        if let Some(chain) = VifControlFlowChain::collect(exprs, index) {
            emit_vif_control_flow_chain(
                ts,
                mappings,
                exprs,
                &chain,
                template_binding_access,
                context,
            );
            index = chain.end;
            continue;
        }

        profile!(
            "canon.virtual_ts.generate_expression",
            generate_expression(
                ts,
                mappings,
                expr,
                template_binding_access,
                context.template_offset,
                context.indent,
                context.checks,
            )
        );
        index += 1;
    }
}

/// Generate expressions inside control flow that already enforces a common
/// v-if guard, removing only that exact top-level guard prefix first.
pub(crate) fn generate_expressions_in_enclosing_guard(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    exprs: &[&TemplateExpression],
    template_binding_access: &TemplateBindingAccess,
    context: &ExpressionListEmitContext<'_>,
    enclosing_guard: Option<&str>,
) {
    let Some(enclosing_guard) = enclosing_guard else {
        generate_expressions(ts, mappings, exprs, template_binding_access, context);
        return;
    };

    let adjusted_expressions: Vec<_> = exprs
        .iter()
        .map(|expr| {
            let mut adjusted = (**expr).clone();
            adjusted.vif_guard = adjusted.vif_guard.as_ref().and_then(|guard| {
                remove_enclosing_vif_guard_prefix(guard.as_str(), enclosing_guard)
                    .map(|guard| CompactString::new(guard.as_str()))
            });
            adjusted
        })
        .collect();
    let adjusted_expression_refs: Vec<_> = adjusted_expressions.iter().collect();
    generate_expressions(
        ts,
        mappings,
        &adjusted_expression_refs,
        template_binding_access,
        context,
    );
}

pub(crate) struct ExpressionListEmitContext<'a> {
    pub(crate) skipped_expression_ranges: &'a FxHashSet<(u32, u32)>,
    pub(crate) template_offset: u32,
    pub(crate) indent: &'a str,
    pub(crate) checks: TemplateValueChecks<'a>,
}

impl<'a> ExpressionListEmitContext<'a> {
    pub(crate) fn new(
        skipped_expression_ranges: &'a FxHashSet<(u32, u32)>,
        template_offset: u32,
        indent: &'a str,
        checks: TemplateValueChecks<'a>,
    ) -> Self {
        Self {
            skipped_expression_ranges,
            template_offset,
            indent,
            checks,
        }
    }
}

/// Generate a template expression, wrapping guarded expressions so TypeScript
/// can narrow them.
pub(crate) fn generate_expression(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expr: &vize_croquis::TemplateExpression,
    template_binding_access: &TemplateBindingAccess,
    template_offset: u32,
    indent: &str,
    checks: TemplateValueChecks<'_>,
) {
    let scoped_access = template_binding_access.for_scope(expr.scope_id);
    let template_binding_access = &scoped_access;
    if let Some(ref guard) = expr.vif_guard {
        if expr.kind == TemplateExpressionKind::VIf {
            generate_vif_guard_expression(
                ts,
                mappings,
                expr,
                guard.as_str(),
                template_binding_access,
                template_offset,
                indent,
            );
            return;
        }

        let trimmed_guard = guard.as_str().trim();
        let rewritten_guard =
            rewrite_reserved_template_binding(trimmed_guard, template_binding_access);
        let generated_guard = rewritten_guard
            .as_ref()
            .map_or_else(|| guard.as_str(), |s| s.as_str());
        // The authored condition has its own projection. This duplicate guard
        // only carries narrowing; mapping it onto the body would rename body
        // tokens when the condition's symbol is renamed.
        append_chained_vif_guard_open(ts, indent, generated_guard, "Narrowing-only guard");
        generate_expression_statement(
            ts,
            mappings,
            expr,
            template_binding_access,
            template_offset,
            &cstr!("{indent}  "),
            checks,
        );
        append_chained_vif_guard_close(ts, indent);
    } else {
        generate_expression_statement(
            ts,
            mappings,
            expr,
            template_binding_access,
            template_offset,
            indent,
            checks,
        );
    }
}

pub(super) fn generate_expression_statement(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expr: &TemplateExpression,
    template_binding_access: &TemplateBindingAccess,
    template_offset: u32,
    indent: &str,
    checks: TemplateValueChecks<'_>,
) {
    let scoped_access = template_binding_access.for_scope(expr.scope_id);
    let template_binding_access = &scoped_access;
    let first_mapping = mappings.len();
    let generated_start = ts.len();
    emit_expression_statement(
        ts,
        mappings,
        expr,
        template_binding_access,
        template_offset,
        indent,
        checks,
    );
    if let Some(rows) = mappings.get(first_mapping..) {
        super::read_ownership::record(expr, rows, template_offset, checks);
    }
    if !template_binding_access.is_empty() {
        let expression = expression_source_for_typecheck(expr.content.as_str());
        // The same stripped text was emitted above. Retain its original byte
        // offsets rather than treating removed comments as affine source bytes.
        // Borrowed suppression comments remain identity input, as before.
        let (expression, offsets) = if matches!(expression, Cow::Owned(_)) {
            strip_js_comments_with_offsets(expr.content.as_str())
        } else {
            (expression, None)
        };
        let trimmed = expression.trim();
        let leading = expression.len() - expression.trim_start().len();
        let authored_start = (template_offset + expr.start) as usize;
        let (start, offsets) = if let Some(offsets) = offsets.as_deref() {
            let Some(offsets) = offsets.get(leading..leading + trimmed.len()) else {
                return;
            };
            (authored_start, Some(offsets))
        } else {
            (authored_start + leading, None)
        };
        map_rewritten_binding_source(
            ts,
            mappings,
            generated_start..ts.len(),
            BindingSource {
                expression: trimmed,
                start,
                offsets,
            },
            template_binding_access,
        );
    }
}

fn emit_expression_statement(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expr: &TemplateExpression,
    template_binding_access: &TemplateBindingAccess,
    template_offset: u32,
    indent: &str,
    checks: TemplateValueChecks<'_>,
) {
    let src_start = (template_offset + expr.start) as usize;
    let src_end = (template_offset + expr.end) as usize;
    let is_component_ref_callback = checks
        .component_ref_callbacks
        .contains_key(&(expr.start, expr.end));
    let expression = if is_component_ref_callback {
        Cow::Borrowed(expr.content.as_str())
    } else {
        profile!(
            "canon.virtual_ts.expression.strip_comments",
            expression_source_for_typecheck(expr.content.as_str())
        )
    };
    let trimmed_expression = expression.as_ref().trim();
    if trimmed_expression.is_empty() {
        return;
    }
    let statement_expression =
        if expr.kind == TemplateExpressionKind::VOn && trimmed_expression.starts_with(';') {
            trimmed_expression
                .trim_start_matches(|character: char| character == ';' || character.is_whitespace())
        } else {
            expression.as_ref()
        };
    if statement_expression.is_empty() {
        return;
    }
    let rewritten_expression =
        rewrite_reserved_template_binding(statement_expression.trim(), template_binding_access);
    let generated_expression = rewritten_expression
        .as_ref()
        .map_or_else(|| statement_expression, |s| s.as_str());
    let native_prop = checks.native_props.get(&(expr.start, expr.end));
    let isolated_expression = if native_prop.is_some() {
        super::incomplete::isolate_diagnostic_expression(generated_expression)
    } else {
        super::incomplete::isolate_incomplete_expression(generated_expression)
    };
    let map_recovery_terminator = isolated_expression.has_unrepaired_syntax_error();
    let generated_expression = isolated_expression.as_str();
    let mapped_bounds = isolated_expression.mapped_bounds();

    if let Some(native_prop) = native_prop {
        generate_native_prop_statement(
            ts,
            mappings,
            expr,
            native_prop,
            (
                generated_expression,
                mapped_bounds.0,
                mapped_bounds.1,
                map_recovery_terminator,
            ),
            template_offset,
            indent,
        );
        return;
    }

    if is_component_ref_callback {
        generate_component_ref_callback_statement(
            ts,
            mappings,
            expr,
            generated_expression,
            mapped_bounds,
            template_offset,
            indent,
        );
        return;
    }

    if expr.kind == TemplateExpressionKind::CustomDirective
        && let Some(directive_value) = checks.directive_values.values.get(&(expr.start, expr.end))
    {
        generate_directive_value_statement(
            ts,
            mappings,
            expr,
            directive_value,
            (generated_expression, mapped_bounds.0, mapped_bounds.1),
            template_offset,
            (indent, checks.check_unknown_directives),
        );
        return;
    }

    if expr.kind == TemplateExpressionKind::DynamicDirectiveArgument
        && !checks
            .directive_values
            .dynamic_arguments
            .contains(&(expr.start, expr.end))
    {
        super::dynamic_arguments::generate(
            ts,
            mappings,
            expr,
            (generated_expression, mapped_bounds.0, mapped_bounds.1),
            template_offset,
            indent,
            checks.template_source,
        );
        return;
    }
    let gen_stmt_start = ts.len();
    let projected_listener_callback = expr.kind == TemplateExpressionKind::VOn
        && matches!(
            classify_event_handler(expr.content.as_str()),
            EventHandlerExpression::Callback { .. }
        );
    let gen_expr_range = if projected_listener_callback {
        // This render-time copy owns only listener construction and usage.
        // This copy keeps render-time reads live for inference and narrowing,
        // but it must not own diagnostics. The typed event-scope primary still
        // checks every parameter and callback-body diagnostic (#11965, #11977).
        let inference_expression =
            inference_callback_parameters_on_ignored_lines(generated_expression);
        let inference_expression = inference_expression
            .as_ref()
            .map_or(generated_expression, String::as_str);
        append!(
            *ts,
            "{indent}// @ts-ignore Inference-only listener value; the typed event scope owns diagnostics.\n{indent}void (("
        );
        ts.push_str(inference_expression);
        append!(
            *ts,
            ") as ((...__vize_args: any[]) => unknown)); // {}\n",
            expr.kind.as_str()
        );
        // An arrow value itself does not read its captures. End the render
        // projection with guarded free-variable reads so they remain the last
        // uses after the arrow closure; otherwise TypeScript still treats the
        // repeated inner guard as an always-true final condition (#11965).
        // These are liveness-only copies: the typed event-scope primary owns
        // every authored diagnostic.
        for capture in guarded_callback_reads(expr) {
            let rewritten =
                rewrite_reserved_template_binding(capture.as_str(), template_binding_access);
            let capture = rewritten.as_deref().unwrap_or(capture.as_str());
            append!(
                *ts,
                "{indent}void ({capture}); // Listener capture liveness\n"
            );
        }
        // Keep the whole projection unmapped, including later callback-body
        // lines that a single TypeScript suppression directive cannot cover.
        // The typed event-scope copy below is the sole navigation and
        // diagnostic owner for the authored listener.
        None
    } else {
        append!(
            *ts,
            "{indent}void ({generated_expression}); // {}\n",
            expr.kind.as_str()
        );
        let start = gen_stmt_start + indent.len() + "void (".len() + mapped_bounds.0;
        Some(start..start + mapped_bounds.1)
    };
    if let Some(gen_range) = gen_expr_range {
        mappings.push(VizeMapping {
            gen_range,
            src_range: src_start..src_end,
            sub_spans: Vec::new(),
        });
    }
    append!(*ts, "{indent}// @vize-map: expr -> {src_start}:{src_end}\n",);
}

#[cfg(test)]
mod tests {
    use super::guarded_callback_reads;
    use crate::virtual_ts::generate_virtual_ts;
    use vize_carton::CompactString;
    use vize_croquis::{
        Analyzer, AnalyzerOptions, ScopeId, TemplateExpression, TemplateExpressionKind,
    };

    fn expression(content: &str, guard: Option<&str>) -> TemplateExpression {
        TemplateExpression {
            content: CompactString::new(content),
            kind: TemplateExpressionKind::VOn,
            start: 0,
            end: content.len() as u32,
            scope_id: ScopeId::new(0),
            vif_guard: guard.map(CompactString::new),
        }
    }

    #[test]
    fn projected_callbacks_read_only_free_names_also_used_by_the_guard() {
        let reads = guarded_callback_reads(&expression(
            "(event) => (handler || (() => fallback))(event, actions)",
            Some("actions && handler && actions.size"),
        ));
        assert_eq!(
            reads.iter().map(CompactString::as_str).collect::<Vec<_>>(),
            ["actions", "handler"]
        );
        assert!(
            guarded_callback_reads(&expression("(handler) => handler", Some("handler"))).is_empty()
        );
        assert!(guarded_callback_reads(&expression("handler", Some("handler"))).is_empty());
        assert!(guarded_callback_reads(&expression("() => handler", None)).is_empty());
    }

    #[test]
    fn guarded_listener_projection_does_not_own_nested_parameter_diagnostics() {
        let script =
            "import Child from './Child.vue';\nconst ready = true;\nlet ids: string[] = [];";
        let template = r#"<div v-if="ready"><Child @change="
          v => ids = v.map(
            x => x.id
          )
        " /></div>"#;
        let allocator = vize_l0::Allocator::new();
        let (root, _) = vize_armature::parse(&allocator, template);
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

        assert_eq!(
            output
                .code
                .matches(
                    "// @ts-ignore Inference-only listener value; the typed event scope owns diagnostics."
                )
                .count(),
            1,
            "the render-time copy must not report nested implicit-any diagnostics:\n{}",
            output.code
        );
        assert_eq!(
            output
                .mapping
                .spans()
                .iter()
                .filter(|mapping| output
                    .code
                    .get(mapping.gen_range.clone())
                    .is_some_and(|mapped| mapped.contains("ids = v.map")))
                .count(),
            1,
            "only the typed event scope may map the authored listener:\n{}",
            output.code
        );
        assert_eq!(
            output
                .code
                .matches("// @ts-ignore Inference-only nested callback parameter.")
                .count(),
            1,
            "a later-line nested callback parameter needs its own suppression:\n{}",
            output.code
        );
    }
}
