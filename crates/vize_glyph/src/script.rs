//! High-performance Script/TypeScript formatting using oxc_formatter.
//!
//! This module provides Prettier-compatible formatting for JavaScript/TypeScript
//! code using OXC's formatter (oxfmt).

mod block_identity;
mod expression;
mod expression_wrapper;
mod format;

use crate::error::FormatError;
use crate::options::FormatOptions;
use oxc_span::{FileExtension, SourceType};
use vize_l0::{Allocator, String};

pub(crate) use block_identity::format_sfc_script_content_stable;
pub use expression::format_js_expression;
pub(crate) use expression::{
    format_js_expression_in_attribute, format_js_expression_in_attribute_with_layout,
    format_js_expression_with_quote_style,
};
pub(crate) use expression_wrapper::FormattedExpression;

const MAX_SCRIPT_STABILIZATION_PASSES: usize = 6;

/// Format JavaScript/TypeScript content using oxc_formatter
///
/// Uses arena allocation for efficient memory management.
#[inline]
#[cfg(test)]
pub fn format_script_content(
    source: &str,
    options: &FormatOptions,
    _allocator: &Allocator,
) -> Result<String, FormatError> {
    format::format_script_content_with_source_type(
        source,
        options,
        _allocator,
        SourceType::ts().with_module(true),
    )
}

pub(crate) fn format_script_content_stable(
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
    source_type: SourceType,
    sort_imports: Option<&crate::ImportSortOptions>,
) -> Result<String, FormatError> {
    if options.end_of_line == crate::EndOfLine::Auto {
        return options.format_with_source_line_ending(source, |options| {
            format_script_content_stable(source, options, allocator, source_type, sort_imports)
        });
    }
    let mut current = format::format_script_content_with_sort_imports(
        source,
        options,
        allocator,
        source_type,
        sort_imports,
    )?;
    if options.skip_script_stabilization {
        return Ok(current);
    }
    let mut current_trimmed_len = current.trim_end().len();
    let source_trimmed = source.trim_end();
    if source_trimmed.len() == current_trimmed_len && current.starts_with(source_trimmed) {
        return Ok(current);
    }
    for _ in 1..MAX_SCRIPT_STABILIZATION_PASSES {
        let next = match format::format_script_content_with_sort_imports(
            current.as_str(),
            options,
            allocator,
            source_type,
            sort_imports,
        ) {
            Ok(next) => next,
            Err(_) => return Ok(current),
        };
        let next_trimmed = next.trim_end();
        if next_trimmed.len() == current_trimmed_len && current.starts_with(next_trimmed) {
            return Ok(next);
        }
        current_trimmed_len = next_trimmed.len();
        current = next;
    }
    Ok(current)
}

pub(crate) fn format_ts_script_content_stable(
    source: &str,
    options: &FormatOptions,
    allocator: &Allocator,
) -> Result<String, FormatError> {
    format_script_content_stable(
        source,
        options,
        allocator,
        SourceType::ts().with_module(true),
        None,
    )
}

pub(crate) fn source_type_for_script_lang(lang: Option<&str>) -> SourceType {
    match lang {
        Some("ts") => SourceType::from(FileExtension::Ts).with_module(true),
        Some("jsx") => SourceType::jsx().with_module(true),
        Some("tsx") => SourceType::tsx().with_module(true),
        _ => SourceType::ts().with_module(true),
    }
}

#[cfg(test)]
mod tests {
    use super::format::format_script_content_with_source_type;
    use super::{Allocator, FormatOptions, format_js_expression, format_script_content};
    use oxc_span::SourceType;
    use vize_l0::String;

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
    fn test_format_tsx_component_script() {
        let source =
            "const Comp=(props:{msg:string})=><section class=\"box\">{props.msg}</section>";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content_with_source_type(
            source,
            &options,
            &allocator,
            SourceType::tsx().with_module(true),
        )
        .unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_jsx_component_script() {
        let source = "const Comp=({msg})=><><span data-id=\"x\">{msg}</span></>";
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content_with_source_type(
            source,
            &options,
            &allocator,
            SourceType::jsx().with_module(true),
        )
        .unwrap();

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
    fn test_format_unicode_whitespace_only() {
        let options = FormatOptions::default();
        let allocator = Allocator::default();
        let result = format_script_content("\u{a0}\u{2003}\u{2028}", &options, &allocator).unwrap();
        assert_eq!(result.as_str(), "");
    }

    #[test]
    fn test_absent_sorting_override_keeps_pinned_none_default() {
        assert!(
            oxc_formatter::JsFormatOptions::default()
                .sort_imports
                .is_none()
        );
        assert!(
            FormatOptions::default()
                .to_oxc_format_options()
                .sort_imports
                .is_none()
        );
    }

    #[test]
    fn test_expression_scratch_keeps_prefix_after_long_and_rejected_inputs() {
        let options = FormatOptions::default();
        let long = format_js_expression("longIdentifier+otherLongIdentifier", &options).unwrap();
        assert_eq!(long.as_str(), "longIdentifier + otherLongIdentifier");
        assert_eq!(format_js_expression(")", &options), None);
        let short = format_js_expression("a+b", &options).unwrap();
        assert_eq!(short.as_str(), "a + b");
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
