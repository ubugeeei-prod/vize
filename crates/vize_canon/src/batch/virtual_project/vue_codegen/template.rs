//! The existing template parser and diagnostic owner for Vue projections.

use std::path::Path;

use vize_atelier_core::{ParserOptions, parser::parse_with_options_and_template_syntax};
use vize_carton::{Allocator, cstr, profile};
use vize_relief::RootNode;

use super::{VueCodegenOptions, diagnostic_for_offset};
use crate::batch::{Diagnostic, SfcBlockType};

pub(super) fn parse<'a>(
    allocator: &'a Allocator,
    template_content: &'a str,
    template_offset: u32,
    path: &Path,
    source: &str,
    codegen_options: VueCodegenOptions<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> (Option<RootNode<'a>>, bool) {
    let mut template_hard_error = false;
    let template_ast = profile!("canon.template.parse", {
        let (root, errors) = parse_with_options_and_template_syntax(
            allocator,
            template_content,
            ParserOptions {
                experimental_in_tag_comments: codegen_options.experimental_in_tag_comments,
                ..ParserOptions::default()
            },
            codegen_options.template_syntax,
        );
        for error in errors {
            if error.code.is_recovery() {
                continue;
            }
            // A documented recovery still yields a complete tree, so the
            // defect is reported without suppressing the rest of the file.
            if !error.code.has_documented_parse_recovery() {
                template_hard_error = true;
            }
            let start = error
                .loc
                .as_ref()
                .map(|loc| template_offset + loc.span.start)
                .unwrap_or(template_offset);
            diagnostics.push(diagnostic_for_offset(
                path,
                source,
                start,
                cstr!("Template parse error: {}", error.message),
                SfcBlockType::Template,
            ));
        }
        // Drop the AST only when a hard error occurred; recovery-level
        // diagnostics leave a fully usable tree.
        (!template_hard_error).then_some(root)
    });
    (template_ast, template_hard_error)
}
