use super::{LintResult, Linter};
use crate::diagnostic::LintDiagnostic;
use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::{CallExpression, ChainElement, Expression, Statement};
use oxc_parser::Parser as OxcParser;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::operator::UnaryOperator;
use std::path::Path;
use vize_armature::Parser as TemplateParser;
use vize_canon::{offset_to_position, LspHover, LspHoverContents, LspMarkedString};
use vize_carton::{CompactString, String, ToCompactString};
use vize_croquis::{
    virtual_ts::{generate_virtual_ts, VirtualTsOutput},
    Analyzer, AnalyzerOptions,
};

const RULE_REQUIRE_TYPED_PROPS: &str = "type/require-typed-props";
const RULE_REQUIRE_TYPED_EMITS: &str = "type/require-typed-emits";
const RULE_NO_FLOATING_PROMISES: &str = "type/no-floating-promises";

pub(crate) fn has_active_type_aware_rules(linter: &Linter) -> bool {
    [
        RULE_REQUIRE_TYPED_PROPS,
        RULE_REQUIRE_TYPED_EMITS,
        RULE_NO_FLOATING_PROMISES,
    ]
    .into_iter()
    .any(|rule_name| linter.registry.has_rule(rule_name) && linter.is_rule_enabled(rule_name))
}

pub(crate) fn lint_sfc_with_tsgo(linter: &Linter, source: &str, filename: &str) -> LintResult {
    let mut result = match super::script_rules::parse_sfc_for_lint(source, filename) {
        Ok(descriptor) => lint_with_descriptor(linter, source, filename, descriptor),
        Err(_) => {
            if let Some((content, byte_offset)) = super::engine::extract_template_fast(source) {
                let mut fallback = linter.lint_template(&content, filename);
                if byte_offset > 0 {
                    for diag in &mut fallback.diagnostics {
                        diag.start += byte_offset;
                        diag.end += byte_offset;
                        for label in &mut diag.labels {
                            label.start += byte_offset;
                            label.end += byte_offset;
                        }
                    }
                }
                fallback
            } else {
                LintResult {
                    filename: filename.to_compact_string(),
                    diagnostics: Vec::new(),
                    error_count: 0,
                    warning_count: 0,
                }
            }
        }
    };

    result.filename = filename.to_compact_string();
    result
}

fn lint_with_descriptor<'a>(
    linter: &Linter,
    source: &str,
    filename: &str,
    descriptor: vize_atelier_sfc::SfcDescriptor<'a>,
) -> LintResult {
    let mut result = super::script_rules::lint_with_descriptor(linter, filename, &descriptor);

    let Some(script_block) = descriptor
        .script_setup
        .as_ref()
        .or(descriptor.script.as_ref())
    else {
        return result;
    };

    let script_content = script_block.content.as_ref();
    if script_content.is_empty() {
        return result;
    }

    let allocator =
        vize_carton::Allocator::with_capacity((source.len() * 4).max(linter.initial_capacity));
    let template_ast = descriptor.template.as_ref().map(|template| {
        let parser = TemplateParser::new(allocator.as_bump(), &template.content);
        let (root, _) = parser.parse();
        (root, template.loc.start as u32)
    });

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        let generic = script_setup
            .attrs
            .get("generic")
            .map(|value| value.as_ref());
        analyzer.analyze_script_setup_with_generic(script_setup.content.as_ref(), generic);
    } else if let Some(script) = descriptor.script.as_ref() {
        analyzer.analyze_script_plain(script.content.as_ref());
    }
    if let Some((template_ast, _)) = template_ast.as_ref() {
        analyzer.analyze_template(template_ast);
    }
    let analysis = analyzer.finish();

    let template_offset = template_ast
        .as_ref()
        .map(|(_, offset)| *offset)
        .unwrap_or(0);

    let from_file = Path::new(filename).parent();
    let mut virtual_ts = generate_virtual_ts(
        Some(script_content),
        template_ast.as_ref().map(|(root, _)| root),
        &analysis.bindings,
        None,
        from_file,
        template_offset,
    );

    let mut macro_queries = Vec::new();
    if linter.registry.has_rule(RULE_REQUIRE_TYPED_PROPS)
        && linter.is_rule_enabled(RULE_REQUIRE_TYPED_PROPS)
    {
        if let Some(call) = analysis.macros.define_props() {
            push_macro_marker(
                &mut virtual_ts,
                script_content,
                call.start,
                call.end,
                "__vize_patina_props_",
                RULE_REQUIRE_TYPED_PROPS,
                &mut macro_queries,
            );
        }
    }
    if linter.registry.has_rule(RULE_REQUIRE_TYPED_EMITS)
        && linter.is_rule_enabled(RULE_REQUIRE_TYPED_EMITS)
    {
        if let Some(call) = analysis.macros.define_emits() {
            push_macro_marker(
                &mut virtual_ts,
                script_content,
                call.start,
                call.end,
                "__vize_patina_emits_",
                RULE_REQUIRE_TYPED_EMITS,
                &mut macro_queries,
            );
        }
    }

    if linter.registry.has_rule(RULE_NO_FLOATING_PROMISES)
        && linter.is_rule_enabled(RULE_NO_FLOATING_PROMISES)
    {
        for candidate in collect_floating_candidates(script_content) {
            push_macro_marker(
                &mut virtual_ts,
                script_content,
                candidate.start,
                candidate.end,
                "__vize_patina_promise_",
                RULE_NO_FLOATING_PROMISES,
                &mut macro_queries,
            );
        }
    }

    let virtual_uri = build_virtual_uri(filename);
    let open_result = with_tsgo_client(linter, filename, |client| {
        client.did_open(&virtual_uri, &virtual_ts.content)?;

        for query in &macro_queries {
            let hover_result = hover_at_offset(
                client,
                &virtual_uri,
                &virtual_ts.content,
                query.generated_offset,
            );
            let Some(hover) = hover_result? else {
                continue;
            };
            let hover_text = flatten_hover(&hover);
            if query.rule_name == RULE_REQUIRE_TYPED_PROPS && is_untyped_props_hover(&hover_text) {
                push_warning(
                    &mut result,
                    LintDiagnostic::warn(
                        RULE_REQUIRE_TYPED_PROPS,
                        "Prop should have a type definition",
                        script_block.loc.start as u32 + query.source_start,
                        script_block.loc.start as u32 + query.source_end,
                    )
                    .with_help("Use `defineProps<Props>()` or a runtime prop object with concrete constructor types."),
                );
            } else if query.rule_name == RULE_REQUIRE_TYPED_EMITS
                && is_untyped_emits_hover(&hover_text)
            {
                push_warning(
                    &mut result,
                    LintDiagnostic::warn(
                        RULE_REQUIRE_TYPED_EMITS,
                        "Emit should have a type definition",
                        script_block.loc.start as u32 + query.source_start,
                        script_block.loc.start as u32 + query.source_end,
                    )
                    .with_help("Use `defineEmits<...>()` or a validator object with typed payload parameters."),
                );
            } else if query.rule_name == RULE_NO_FLOATING_PROMISES
                && is_promise_like_hover(&hover_text)
            {
                push_warning(
                    &mut result,
                    LintDiagnostic::warn(
                        RULE_NO_FLOATING_PROMISES,
                        "Floating Promise must be awaited, returned, or explicitly ignored with `void`",
                        script_block.loc.start as u32 + query.source_start,
                        script_block.loc.start as u32 + query.source_end,
                    )
                    .with_help("Add `await`, return the Promise, or prefix it with `void` when the fire-and-forget behavior is intentional."),
                );
            }
        }

        client.did_close(&virtual_uri)?;
        Ok(())
    });

    if open_result.is_err() {
        let _ = with_tsgo_client(linter, filename, |client| client.did_close(&virtual_uri));
    }

    result
}

fn push_warning(result: &mut LintResult, diagnostic: LintDiagnostic) {
    result.warning_count += 1;
    result.diagnostics.push(diagnostic);
}

fn with_tsgo_client<T>(
    linter: &Linter,
    filename: &str,
    f: impl FnOnce(&mut vize_canon::lsp_client::TsgoLspClient) -> Result<T, String>,
) -> Result<T, String> {
    let mut guard = linter
        .native_tsgo
        .lock()
        .map_err(|_| "Failed to lock tsgo client".to_compact_string())?;

    if guard.is_none() {
        let fallback_dir = std::env::current_dir()
            .ok()
            .and_then(|path| path.to_str().map(|value| value.to_compact_string()));
        let working_dir = Path::new(filename)
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .and_then(|path| path.to_str())
            .or(fallback_dir.as_deref())
            .unwrap_or(".");
        *guard = Some(vize_canon::lsp_client::TsgoLspClient::new(
            None,
            Some(working_dir),
        )?);
    }

    let client = guard
        .as_mut()
        .ok_or_else(|| "Failed to initialize tsgo client".to_compact_string())?;

    let result = f(client);
    if result.is_err() {
        let _ = client.shutdown();
        *guard = None;
    }
    result
}

fn build_virtual_uri(filename: &str) -> String {
    let mut path = std::env::current_dir().unwrap_or_else(|_| ".".into());
    path.push("__agent_only");
    path.push("vize-patina");
    let _ = std::fs::create_dir_all(&path);

    let base_name = Path::new(filename)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("Component.vue");
    path.push(base_name);
    path.set_extension("patina.ts");

    let mut uri = String::with_capacity(path.as_os_str().len() + 8);
    uri.push_str("file://");
    uri.push_str(path.to_string_lossy().as_ref());
    uri
}

fn hover_at_offset(
    client: &mut vize_canon::lsp_client::TsgoLspClient,
    uri: &str,
    generated_source: &str,
    generated_offset: u32,
) -> Result<Option<LspHover>, String> {
    let position = offset_to_position(generated_source, generated_offset);
    client.hover(uri, position.line, position.column)
}

fn flatten_hover(hover: &LspHover) -> CompactString {
    match &hover.contents {
        LspHoverContents::Markup(markup) => markup.value.as_str().to_compact_string(),
        LspHoverContents::String(value) => value.as_str().to_compact_string(),
        LspHoverContents::Array(values) => {
            let mut text = String::default();
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    text.push('\n');
                }
                match value {
                    LspMarkedString::String(value) => text.push_str(value),
                    LspMarkedString::LanguageString { value, .. } => text.push_str(value),
                }
            }
            text
        }
    }
}

fn is_untyped_props_hover(text: &str) -> bool {
    text.contains(": unknown")
        || text.contains(": any")
        || text.contains("?: any")
        || text.contains(": {}")
        || text.ends_with("unknown")
        || text.ends_with("{}")
}

fn is_untyped_emits_hover(text: &str) -> bool {
    text.contains("...args: any[]") || text.contains(": unknown") || text.ends_with("unknown")
}

fn is_promise_like_hover(text: &str) -> bool {
    text.contains("Promise<") || text.contains("PromiseLike<") || text.contains("Thenable<")
}

struct MacroHoverQuery {
    rule_name: &'static str,
    generated_offset: u32,
    source_start: u32,
    source_end: u32,
}

fn push_macro_marker(
    virtual_ts: &mut VirtualTsOutput,
    script_content: &str,
    source_start: u32,
    source_end: u32,
    prefix: &str,
    rule_name: &'static str,
    queries: &mut Vec<MacroHoverQuery>,
) {
    let Some(insert_offset) = marker_insert_offset(&virtual_ts.content) else {
        return;
    };
    let source_range = source_start as usize..source_end as usize;
    let Some(expression_source) = script_content.get(source_range) else {
        return;
    };

    let mut marker_name = String::with_capacity(prefix.len() + 8);
    marker_name.push_str(prefix);
    let query_index = queries.len().to_compact_string();
    marker_name.push_str(query_index.as_str());

    let mut line = String::with_capacity(marker_name.len() + expression_source.len() + 24);
    line.push_str("    const ");
    let name_offset = line.len() as u32;
    line.push_str(&marker_name);
    line.push_str(" = (");
    line.push_str(expression_source);
    line.push_str(");\n");

    let generated_offset = insert_offset as u32 + name_offset;
    virtual_ts.content.insert_str(insert_offset, &line);
    queries.push(MacroHoverQuery {
        rule_name,
        generated_offset,
        source_start,
        source_end,
    });
}

fn marker_insert_offset(content: &str) -> Option<usize> {
    content
        .rfind("\n}\n\n// Invoke setup")
        .map(|index| index + 1)
}

#[derive(Clone, Copy)]
struct FloatingCandidate {
    start: u32,
    end: u32,
}

fn collect_floating_candidates(source: &str) -> Vec<FloatingCandidate> {
    let allocator = OxcAllocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let parsed = OxcParser::new(&allocator, source, source_type).parse();
    if parsed.panicked {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for statement in parsed.program.body.iter() {
        if let Statement::ExpressionStatement(expression_statement) = statement {
            if is_explicitly_handled(&expression_statement.expression) {
                continue;
            }
            let span = expression_statement.expression.span();
            candidates.push(FloatingCandidate {
                start: span.start,
                end: span.end,
            });
        }
    }
    candidates
}

fn is_explicitly_handled(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::AwaitExpression(_) => true,
        Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::Void,
        Expression::ParenthesizedExpression(paren) => is_explicitly_handled(&paren.expression),
        Expression::TSAsExpression(ts_as) => is_explicitly_handled(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            is_explicitly_handled(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            is_explicitly_handled(&ts_non_null.expression)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => is_handled_call(call),
            ChainElement::TSNonNullExpression(non_null) => {
                is_explicitly_handled(&non_null.expression)
            }
            _ => false,
        },
        Expression::CallExpression(call) => is_handled_call(call),
        _ => false,
    }
}

fn is_handled_call(call: &CallExpression<'_>) -> bool {
    call.callee
        .as_member_expression()
        .and_then(|member| member.static_property_name())
        .is_some_and(|name| matches!(name, "then" | "catch" | "finally"))
}

#[cfg(test)]
mod tests {
    use super::{
        has_active_type_aware_rules, lint_sfc_with_tsgo, RULE_NO_FLOATING_PROMISES,
        RULE_REQUIRE_TYPED_EMITS, RULE_REQUIRE_TYPED_PROPS,
    };
    use crate::{LintPreset, Linter};

    fn tsgo_available() -> bool {
        let mut client = match vize_canon::lsp_client::TsgoLspClient::new(
            None,
            Some(env!("CARGO_MANIFEST_DIR")),
        ) {
            Ok(client) => client,
            Err(_) => return false,
        };
        let _ = client.shutdown();
        true
    }

    #[test]
    fn opinionated_preset_enables_native_type_aware_rules() {
        let linter = Linter::with_preset(LintPreset::Opinionated);
        assert!(has_active_type_aware_rules(&linter));
    }

    #[test]
    fn require_typed_props_uses_tsgo() {
        if !tsgo_available() {
            return;
        }

        let linter = Linter::with_preset(LintPreset::Opinionated);
        let source = r#"<script setup lang="ts">
defineProps(['msg', 'count'])
</script>"#;
        let result = lint_sfc_with_tsgo(&linter, source, "Component.vue");
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.rule_name == RULE_REQUIRE_TYPED_PROPS));
    }

    #[test]
    fn require_typed_emits_uses_tsgo() {
        if !tsgo_available() {
            return;
        }

        let linter = Linter::with_preset(LintPreset::Opinionated);
        let source = r#"<script setup lang="ts">
defineEmits(['save'])
</script>"#;
        let result = lint_sfc_with_tsgo(&linter, source, "Component.vue");
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.rule_name == RULE_REQUIRE_TYPED_EMITS));
    }

    #[test]
    fn no_floating_promises_uses_tsgo() {
        if !tsgo_available() {
            return;
        }

        let linter = Linter::with_preset(LintPreset::Opinionated);
        let source = r#"<script setup lang="ts">
async function loadData(): Promise<number> {
  return 1
}

loadData()
</script>"#;
        let result = lint_sfc_with_tsgo(&linter, source, "Component.vue");
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
    }

    #[test]
    fn voided_promises_are_ignored() {
        if !tsgo_available() {
            return;
        }

        let linter = Linter::with_preset(LintPreset::Opinionated);
        let source = r#"<script setup lang="ts">
async function loadData(): Promise<number> {
  return 1
}

void loadData()
</script>"#;
        let result = lint_sfc_with_tsgo(&linter, source, "Component.vue");
        assert!(!result
            .diagnostics
            .iter()
            .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
    }
}
