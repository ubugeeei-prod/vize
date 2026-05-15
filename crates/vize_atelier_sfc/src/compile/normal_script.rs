//! Functions for processing normal `<script>` blocks when both
//! `<script>` and `<script setup>` exist.

use vize_carton::{profile, String, ToCompactString};

/// Extract content from normal script block that should be preserved when both
/// `<script>` and `<script setup>` exist.
/// This includes imports, type definitions, interfaces, but excludes `export default`.
///
/// Parameters:
/// - `content`: The script content
/// - `source_is_ts`: Whether the source script is TypeScript (has lang="ts")
/// - `output_is_ts`: Whether to preserve TypeScript in output (false = transpile to JS)
pub(super) fn extract_normal_script_content(
    content: &str,
    source_is_ts: bool,
    output_is_ts: bool,
) -> String {
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

    if !ret.errors.is_empty() {
        // If parsing fails, return original content minus any obvious export default
        return content
            .lines()
            .filter(|line| !line.trim().starts_with("export default"))
            .collect::<Vec<_>>()
            .join("\n")
            .into();
    }

    let program = ret.program;
    let mut output = String::default();
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
        output.push_str(&content[last_end..*start as usize]);
        if let Some(repl) = replacement {
            output.push_str(repl);
        }
        last_end = *end as usize;
    }
    if last_end < content.len() {
        output.push_str(&content[last_end..]);
    }

    let extracted = output.trim().to_compact_string();

    // If source is TypeScript and we need JavaScript output, transpile
    if source_is_ts && !output_is_ts {
        // Re-parse the extracted content
        let allocator2 = Allocator::default();
        let ret2 = profile!(
            "atelier.normal_script.extract.ts_parse",
            Parser::new(&allocator2, &extracted, SourceType::ts()).parse()
        );
        if ret2.errors.is_empty() {
            let mut program2 = ret2.program;

            // Run semantic analysis
            let semantic_ret = profile!(
                "atelier.normal_script.extract.ts_semantic",
                SemanticBuilder::new().build(&program2)
            );
            if semantic_ret.errors.is_empty() {
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

                if transform_ret.errors.is_empty() {
                    // Generate JavaScript code
                    return profile!(
                        "atelier.normal_script.extract.ts_codegen",
                        Codegen::new().build(&program2)
                    )
                    .code
                    .into();
                }
            }
        }
    }

    extracted
}
