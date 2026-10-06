//! The existing template parser and diagnostic owner for Vue projections.

use std::path::Path;

use vize_atelier_core::{
    CompilerError, ErrorCode, ParserOptions, parser::parse_with_options_and_template_syntax,
};
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
) -> (Option<RootNode<'a>>, bool, bool) {
    let mut template_hard_error = false;
    let (template_ast, incomplete_tag) = profile!("canon.template.parse", {
        let (root, errors) = parse_with_options_and_template_syntax(
            allocator,
            template_content,
            ParserOptions {
                experimental_in_tag_comments: codegen_options.experimental_in_tag_comments,
                ..ParserOptions::default()
            },
            codegen_options.template_syntax,
        );
        let incomplete_tag = is_incomplete_tag(&errors);
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
        ((!template_hard_error).then_some(root), incomplete_tag)
    });
    (template_ast, template_hard_error, incomplete_tag)
}

// Preserve scripts only when the parser identified an unfinished tag. A
// complete but malformed template keeps the existing whole-file fallback.
fn is_incomplete_tag(errors: &[CompilerError]) -> bool {
    errors.iter().any(|error| error.code == ErrorCode::EofInTag)
        && errors.iter().all(|error| {
            error.code.is_recovery()
                || error.code.has_documented_parse_recovery()
                || matches!(error.code, ErrorCode::EofInTag | ErrorCode::MissingEndTag)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_parser_owned_unfinished_tags_admit_script_preservation() {
        for (codes, expected) in [
            (vec![], false),
            (vec![ErrorCode::MissingEndTag], false),
            (vec![ErrorCode::EofInTag], true),
            (vec![ErrorCode::EofInTag, ErrorCode::MissingEndTag], true),
            (vec![ErrorCode::EofInTag, ErrorCode::InvalidEndTag], false),
            (vec![ErrorCode::EofInTag, ErrorCode::EofInComment], true),
            (
                vec![ErrorCode::EofInTag, ErrorCode::MissingEndTagName],
                false,
            ),
        ] {
            let errors = codes
                .into_iter()
                .map(|code| CompilerError::new(code, None))
                .collect::<Vec<_>>();
            assert_eq!(is_incomplete_tag(&errors), expected);
        }
    }
}
