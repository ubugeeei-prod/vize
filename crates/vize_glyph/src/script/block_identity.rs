use super::format_script_content_stable;
use crate::error::FormatError;
use crate::options::FormatOptions;
use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::Statement;
use oxc_formatter::parse_for_format;
use oxc_span::SourceType;
use vize_s0::{Allocator, String, ToCompactString};

/// Formats an SFC script without letting cleanup erase its block identity.
///
/// Oxfmt deliberately removes empty statements. That remains desirable for a
/// standalone script, but Vue's official parser ignores an SFC script block
/// that becomes empty. Only a comment-free, directive-free sequence of empty
/// statements is replaced with one canonical statement. If a future formatter
/// unexpectedly erases any other non-empty program, preserve the authored body
/// rather than silently changing the SFC descriptor.
pub(crate) fn format_sfc_script_content_stable(
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
    source_type: SourceType,
) -> Result<String, FormatError> {
    let formatted = format_script_content_stable(source, options, allocator, source_type)?;
    if source.trim().is_empty() || !formatted.trim().is_empty() {
        return Ok(formatted);
    }
    if is_empty_statement_only_program(source, source_type) {
        return Ok(";".into());
    }
    Ok(source.trim().to_compact_string())
}

fn is_empty_statement_only_program(source: &str, source_type: SourceType) -> bool {
    let allocator = OxcAllocator::default();
    let parsed = parse_for_format(&allocator, source, source_type);
    parsed.diagnostics.is_empty()
        && parsed.program.comments.is_empty()
        && parsed.program.directives.is_empty()
        && !parsed.program.body.is_empty()
        && parsed
            .program
            .body
            .iter()
            .all(|statement| matches!(statement, Statement::EmptyStatement(_)))
}

#[cfg(test)]
mod tests {
    use super::is_empty_statement_only_program;
    use oxc_span::SourceType;

    #[test]
    fn classifies_only_comment_free_empty_statements() {
        for source_type in [SourceType::ts().with_module(true), SourceType::mjs()] {
            for source in [";", ";;;", "\n; ;\n;"] {
                assert!(is_empty_statement_only_program(source, source_type));
            }
            for source in [
                "",
                "// keep\n;",
                "/* keep */ ;",
                "\"use strict\";",
                "const value = 1;",
            ] {
                assert!(!is_empty_statement_only_program(source, source_type));
            }
        }
    }
}
