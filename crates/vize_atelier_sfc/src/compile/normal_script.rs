//! Functions for processing normal `<script>` blocks when both
//! `<script>` and `<script setup>` exist.

use vize_carton::{String, ToCompactString, profile};

use crate::module_map::{Runs, reprint_points};
use crate::types::SfcDescriptor;

use super::helpers::is_ts_lang;

/// The normal `<script>` content kept beside `<script setup>`, and, when
/// `trace` asks for it, that content's provenance in `.vue` offsets (Davinci
/// P3-9).
pub(super) fn extract_for_setup(
    descriptor: &SfcDescriptor<'_>,
    output_is_ts: bool,
    trace: bool,
) -> (Option<String>, Option<Runs>) {
    let Some(script) = descriptor.script.as_ref() else {
        return (None, None);
    };
    let source_is_ts = is_ts_lang(script.lang.as_deref());
    let (content, runs) = profile!(
        "atelier.sfc.normal_script.extract",
        extract_traced(&script.content, source_is_ts, output_is_ts, trace)
    );
    (
        Some(content),
        runs.map(|runs| runs.offset_origin(script.loc.start)),
    )
}

/// Extract content from normal script block that should be preserved when both
/// `<script>` and `<script setup>` exist.
/// This includes imports, type definitions, interfaces, but excludes `export default`.
///
/// Parameters:
/// - `content`: The script content
/// - `source_is_ts`: Whether the source script is TypeScript (has lang="ts")
/// - `output_is_ts`: Whether to preserve TypeScript in output (false = transpile to JS)
#[cfg(test)]
pub(super) fn extract_normal_script_content(
    content: &str,
    source_is_ts: bool,
    output_is_ts: bool,
) -> String {
    extract_traced(content, source_is_ts, output_is_ts, false).0
}

/// [`extract_normal_script_content`] plus, when `trace` is set, the result's
/// provenance in `content` (`None` when a fallback path cannot say).
fn extract_traced(
    content: &str,
    source_is_ts: bool,
    output_is_ts: bool,
    trace: bool,
) -> (String, Option<Runs>) {
    use oxc_allocator::Allocator;
    use oxc_ast::ast::Statement;
    use oxc_codegen::Codegen;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::{GetSpan, SourceType};
    use oxc_transformer::{TransformOptions, Transformer, TypeScriptOptions};

    // Always parse as TypeScript if source is TypeScript
    let source_type = if source_is_ts {
        SourceType::ts()
    } else {
        SourceType::mjs()
    };

    let allocator = Allocator::default();
    let ret = profile!(
        "atelier.normal_script.extract.parse",
        Parser::new(&allocator, content, source_type).parse()
    );

    if !ret.diagnostics.is_empty() {
        // If parsing fails, return original content minus any obvious export default
        let kept = content
            .lines()
            .filter(|line| !line.trim().starts_with("export default"))
            .collect::<Vec<_>>()
            .join("\n")
            .into();
        return (kept, None);
    }

    let program = ret.program;
    let mut output = String::default();
    let mut runs = Runs::default();
    let mut last_end = 0;

    // Collect spans of statements to skip (export default declarations)
    let mut skip_spans: Vec<(u32, u32)> = Vec::new();

    // Collect spans to rewrite: (start, end, replacement)
    let mut rewrites: Vec<(u32, u32, String)> = Vec::new();

    for stmt in program.body.iter() {
        match stmt {
            // Rewrite export default declarations to const __default__ = ...
            Statement::ExportDefaultDeclaration(decl) => {
                // Find the span of "export default" keyword portion
                let stmt_start = stmt.span().start;
                let stmt_end = stmt.span().end;
                let stmt_text = &content[stmt_start as usize..stmt_end as usize];
                // Replace "export default" with "const __default__ ="
                let rewritten: String = stmt_text
                    .replacen("export default", "const __default__ =", 1)
                    .into();
                rewrites.push((stmt_start, stmt_end, rewritten));
                let _ = decl; // suppress unused
            }
            // Skip named exports that include default: export { foo as default }
            Statement::ExportNamedDeclaration(decl) => {
                let has_default_export = decl.specifiers.iter().any(|s| {
                    matches!(&s.exported, oxc_ast::ast::ModuleExportName::IdentifierName(name) if name.name == "default")
                        || matches!(&s.exported, oxc_ast::ast::ModuleExportName::IdentifierReference(name) if name.name == "default")
                });
                if has_default_export {
                    skip_spans.push((stmt.span().start, stmt.span().end));
                }
            }
            _ => {}
        }
    }

    // Build output by copying content, applying rewrites and skipping as needed
    // Merge all modifications into a sorted list
    let mut modifications: Vec<(u32, u32, Option<String>)> = Vec::new();
    for (start, end, replacement) in rewrites {
        modifications.push((start, end, Some(replacement)));
    }
    for (start, end) in &skip_spans {
        modifications.push((*start, *end, None));
    }
    modifications.sort_by_key(|m| m.0);

    for (start, end, replacement) in &modifications {
        let start = *start as usize;
        runs.copy(output.len(), last_end, start.saturating_sub(last_end));
        output.push_str(&content[last_end..start]);
        if let Some(repl) = replacement {
            trace_default_rewrite(
                &mut runs,
                output.len(),
                &content[start..*end as usize],
                start,
            );
            output.push_str(repl);
        }
        last_end = *end as usize;
    }
    if last_end < content.len() {
        runs.copy(output.len(), last_end, content.len() - last_end);
        output.push_str(&content[last_end..]);
    }

    let lead = output.len() - output.trim_start().len();
    let extracted = output.trim().to_compact_string();
    let runs = runs.slice(lead, extracted.len());

    // If source is TypeScript and we need JavaScript output, transpile
    if source_is_ts && !output_is_ts {
        // Re-parse the extracted content
        let allocator2 = Allocator::default();
        let ret2 = profile!(
            "atelier.normal_script.extract.ts_parse",
            Parser::new(&allocator2, &extracted, SourceType::ts()).parse()
        );
        if ret2.diagnostics.is_empty() {
            let mut program2 = ret2.program;

            // Run semantic analysis
            // `with_enum_eval(true)`: OXC 0.142's TypeScript enum transform
            // panics unless the `Scoping` it is handed carries evaluated enum
            // member values.
            let semantic_ret = profile!(
                "atelier.normal_script.extract.ts_semantic",
                SemanticBuilder::new().with_enum_eval(true).build(&program2)
            );
            if semantic_ret.diagnostics.is_empty() {
                let scoping = semantic_ret.semantic.into_scoping();

                // Transform TypeScript to JavaScript
                // Use only_remove_type_imports to preserve imports that might be used in template
                let transform_options = TransformOptions {
                    typescript: TypeScriptOptions {
                        only_remove_type_imports: true,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let transform_ret = profile!(
                    "atelier.normal_script.extract.ts_transform",
                    Transformer::new(&allocator2, std::path::Path::new(""), &transform_options)
                        .build_with_scoping(scoping, &mut program2)
                );

                if transform_ret.diagnostics.is_empty() {
                    // Generate JavaScript code
                    let options = oxc_codegen::CodegenOptions {
                        source_map_path: trace.then(|| std::path::PathBuf::from("script.ts")),
                        ..oxc_codegen::CodegenOptions::default()
                    };
                    let printed = profile!(
                        "atelier.normal_script.extract.ts_codegen",
                        Codegen::new().with_options(options).build(&program2)
                    );
                    let printed_runs = printed
                        .map
                        .as_ref()
                        .map(|map| reprint_points(map, &printed.code, &extracted).compose(&runs));
                    return (printed.code.into(), printed_runs);
                }
            }
        }
    }

    (extracted, trace.then_some(runs))
}

/// Provenance of `export default` rewritten to `const __default__ =` in
/// `statement` (at `start` of the content), written at `out`: the keyword is
/// anchored at the authored keyword and the rest of the statement is a copy.
fn trace_default_rewrite(runs: &mut Runs, out: usize, statement: &str, start: usize) {
    const FROM: &str = "export default";
    const TO: &str = "const __default__ =";
    let Some(keyword) = statement.find(FROM) else {
        return;
    };
    runs.copy(out, start, keyword);
    runs.point(out + keyword, start + keyword);
    let rest = keyword + FROM.len();
    runs.copy(
        out + keyword + TO.len(),
        start + rest,
        statement.len() - rest,
    );
}
