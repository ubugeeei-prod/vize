//! High-performance Script/TypeScript formatting using oxc_formatter.
//!
//! This module provides Prettier-compatible formatting for JavaScript/TypeScript
//! code using OXC's formatter (oxfmt).

use crate::error::FormatError;
use crate::options::FormatOptions;
use oxc_allocator::Allocator as OxcAllocator;
use oxc_formatter::{Formatter as OxcFormatter, get_parse_options};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{Allocator, String, ToCompactString};

/// Format JavaScript/TypeScript content using oxc_formatter
///
/// Uses arena allocation for efficient memory management.
#[inline]
pub fn format_script_content(
    source: &str,
    options: &FormatOptions,
    _allocator: &Allocator,
) -> Result<String, FormatError> {
    // Fast path for empty content
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(String::default());
    }

    // Use OXC's allocator for parsing (required by oxc_parser)
    let oxc_allocator = OxcAllocator::default();

    // Determine source type (default to TypeScript module)
    let source_type = SourceType::ts().with_module(true);

    // Parse the source with formatter-compatible options
    let parsed = Parser::new(&oxc_allocator, source, source_type)
        .with_options(get_parse_options())
        .parse();

    if !parsed.errors.is_empty() {
        let error_messages: Vec<String> = parsed
            .errors
            .iter()
            .map(|e| e.to_compact_string())
            .collect();
        return Err(FormatError::ScriptParseError(
            error_messages.join("; ").into(),
        ));
    }

    // Convert options and format
    let oxc_options = options.to_oxc_format_options();
    let formatted = OxcFormatter::new(&oxc_allocator, oxc_options).build(&parsed.program);

    Ok(formatted.into())
}

thread_local! {
    /// Per-thread scratch reused across template-expression formats. A single
    /// template can call `format_js_expression` thousands of times (once per
    /// interpolation / directive value); reusing the arena (reset between calls)
    /// avoids a bumpalo chunk alloc+teardown per call, and reusing the `void (…)`
    /// wrapper buffer avoids a heap allocation per call. The CLI formats files in
    /// parallel, so per-thread state keeps each worker independent and lock-free.
    static EXPR_SCRATCH: core::cell::RefCell<(OxcAllocator, String)> =
        core::cell::RefCell::new((OxcAllocator::default(), String::default()));
}

/// Format a JS expression (for use in template directive values and interpolations).
/// Returns None if the expression cannot be parsed/formatted.
pub fn format_js_expression(expr: &str, options: &FormatOptions) -> Option<String> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Some(String::default());
    }

    EXPR_SCRATCH.with(|cell| {
        let mut scratch = cell.borrow_mut();
        let (oxc_allocator, wrapped) = &mut *scratch;
        // Recycle the arena memory instead of allocating/freeing a fresh one.
        oxc_allocator.reset();

        let source_type = SourceType::ts().with_module(true);

        // Wrap the expression in a `void (…)` statement so it parses as a complete
        // statement the formatter can emit cleanly; we extract the inner part
        // below. Build the wrapper in the reused buffer (no per-call allocation).
        wrapped.clear();
        wrapped.push_str("void (");
        wrapped.push_str(trimmed);
        wrapped.push(')');
        let parsed = Parser::new(oxc_allocator, wrapped.as_str(), source_type)
            .with_options(get_parse_options())
            .parse();

        if !parsed.errors.is_empty() {
            return None;
        }

        let oxc_options = options.to_oxc_format_options();
        let formatted = OxcFormatter::new(oxc_allocator, oxc_options).build(&parsed.program);

        // Extract the expression back from the formatted output.
        // preserve_parens is false, so the formatter may remove the wrapping parens.
        // Expected forms:  "void expression;\n"  or  "void (expression);\n"
        let formatted = formatted.trim();
        let formatted = formatted.strip_suffix(';').unwrap_or(formatted);
        let inner = formatted.strip_prefix("void ").unwrap_or(formatted);

        // Strip outer parens if the formatter kept them
        let inner = if inner.starts_with('(') && inner.ends_with(')') {
            &inner[1..inner.len() - 1]
        } else {
            inner
        };

        Some(inner.trim().to_compact_string())
    })
}

#[cfg(test)]
mod tests {
    use super::{Allocator, FormatOptions, format_js_expression, format_script_content};
    use vize_carton::String;

    #[test]
    fn test_format_simple_script() {
        let source = "const x=1";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_with_imports() {
        let source = "import {ref,computed} from 'vue'";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_object() {
        let source = "const obj={a:1,b:2}";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_empty_source() {
        let source = "";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_format_whitespace_only() {
        let source = "   \n\t  ";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content(source, &options, &allocator).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_format_js_expression_simple() {
        let options = FormatOptions::default();
        let result = format_js_expression("count+1", &options);
        assert!(result.is_some());
        let expr = result.unwrap();
        insta::assert_snapshot!(expr.as_str());
    }

    #[test]
    fn test_format_js_expression_with_optional_chaining() {
        let options = FormatOptions::default();
        let expr = format_js_expression("user?.profile?.name??'Guest'", &options).unwrap();

        insta::assert_snapshot!(expr.as_str());
    }

    #[test]
    fn test_format_js_expression_empty() {
        let options = FormatOptions::default();
        let result = format_js_expression("", &options);
        assert_eq!(result, Some(String::default()));
    }
}
