//! vue/no-multi-spaces
//!
//! Disallow multiple consecutive spaces in template.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div  class="foo"></div>
//! <div class="foo"  id="bar"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div class="foo"></div>
//! <div class="foo" id="bar"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::ir::ByteRange;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, InterpolationNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-multi-spaces",
    description: "Disallow multiple consecutive spaces",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Disallow multiple spaces
#[derive(Default)]
pub struct NoMultiSpaces {
    /// Ignore properties (v-if, v-for expressions)
    pub ignore_properties: bool,
}

impl Rule for NoMultiSpaces {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        self.check_element(ctx, &MarkupElement::new(element), false);
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        self.check_expression_range(
            ctx,
            ByteRange::new(interpolation.loc.span.start, interpolation.loc.span.end),
        );
    }
}

impl MarkupRule for NoMultiSpaces {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        let is_jsx = ctx.is_jsx() || is_jsx_filename(ctx.lint().filename);
        self.check_element(ctx.lint(), element, is_jsx);
    }

    fn enter_interpolation(&self, ctx: &mut MarkupContext<'_, '_>, range: ByteRange) {
        self.check_expression_range(ctx.lint(), range);
    }
}

impl NoMultiSpaces {
    fn check_element(&self, ctx: &mut LintContext<'_>, element: &MarkupElement<'_>, is_jsx: bool) {
        let mut previous_binding_end = None;

        element.walk_opening_item_ranges(&mut |range| {
            let gap_start = previous_binding_end
                .unwrap_or_else(|| tag_name_end_hint(element.range(), element.tag()));
            self.check_gap(ctx, gap_start, range.start);
            if !is_jsx && !self.ignore_properties {
                self.check_structural_directive_expression(ctx, range);
            }
            previous_binding_end = Some(range.end);
        });

        let opening = element.open_tag_range(ctx.source);
        let closing_start = opening_closing_start(ctx.source, opening);
        let gap_start = previous_binding_end
            .unwrap_or_else(|| tag_name_end_hint(element.range(), element.tag()));
        self.check_gap(ctx, gap_start, closing_start);

        if !self.ignore_properties {
            element.walk_bindings(&mut |binding| {
                let Some(expression) = binding.expression() else {
                    return;
                };
                let range = binding.range();
                let Some(authored) = ctx.source.get(range.start as usize..range.end as usize)
                else {
                    return;
                };
                let Some(relative) = authored.find(expression) else {
                    return;
                };
                self.check_expression(ctx, range.start as usize + relative, expression.as_bytes());
            });
        }
    }

    fn check_gap(&self, ctx: &mut LintContext<'_>, gap_start: u32, gap_end: u32) {
        let gap_start = gap_start as usize;
        let gap_end = gap_end as usize;
        let gap_start = first_whitespace_offset(ctx.source, gap_start, gap_end);
        if gap_end <= gap_start {
            return;
        }

        let Some(gap) = ctx.source.get(gap_start..gap_end) else {
            return;
        };
        if !is_invalid_gap(gap) {
            return;
        }

        let fix = Fix::new(
            "Replace multiple spaces with single space",
            TextEdit::replace(gap_start as u32, gap_end as u32, " "),
        );

        ctx.report(
            LintDiagnostic::warn(
                META.name,
                "Multiple consecutive spaces",
                gap_start as u32,
                gap_end as u32,
            )
            .with_fix(fix),
        );
    }

    fn check_structural_directive_expression(&self, ctx: &mut LintContext<'_>, range: ByteRange) {
        let Some(item) = ctx.source.get(range.start as usize..range.end as usize) else {
            return;
        };
        let Some((raw_name, raw_value)) = item.split_once('=') else {
            return;
        };
        let name = raw_name.trim();
        if !matches!(name, "v-if" | "v-else-if" | "v-for") {
            return;
        }
        let raw_value = raw_value.trim_start();
        let Some(quote @ ('\'' | '"')) = raw_value.chars().next() else {
            return;
        };
        let Some(value) = raw_value
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
        else {
            return;
        };
        let value_start =
            range.start as usize + item.len().saturating_sub(raw_value.len()) + quote.len_utf8();
        self.check_expression(ctx, value_start, value.as_bytes());
    }

    fn check_expression(&self, ctx: &mut LintContext<'_>, start: usize, expression: &[u8]) {
        let mut cursor = 0;
        while cursor < expression.len() {
            let Some(&byte) = expression.get(cursor) else {
                break;
            };
            match byte {
                b'`' => {
                    cursor = skip_template_literal(expression, cursor);
                    continue;
                }
                b'\'' | b'"' => {
                    cursor = skip_quoted(expression, cursor, byte);
                    continue;
                }
                b'/' if expression.get(cursor + 1) == Some(&b'/') => {
                    cursor = skip_line_comment(expression, cursor + 2);
                    continue;
                }
                b'/' if expression.get(cursor + 1) == Some(&b'*') => {
                    cursor = skip_block_comment(expression, cursor + 2);
                    continue;
                }
                _ => {}
            }
            if !expression
                .get(cursor)
                .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
            {
                cursor += 1;
                continue;
            }
            let run_start = cursor;
            while expression
                .get(cursor)
                .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
            {
                cursor += 1;
            }
            let starts_line = run_start == 0
                || expression
                    .get(run_start - 1)
                    .is_some_and(|byte| matches!(byte, b'\n' | b'\r'));
            let ends_line = expression
                .get(cursor)
                .is_some_and(|byte| matches!(byte, b'\n' | b'\r'));
            if cursor - run_start > 1 && !starts_line && !ends_line {
                self.check_gap(ctx, (start + run_start) as u32, (start + cursor) as u32);
            }
        }
    }

    fn check_expression_range(&self, ctx: &mut LintContext<'_>, range: ByteRange) {
        let Some(expression) = ctx
            .source
            .get(range.start as usize..range.end as usize)
            .map(str::as_bytes)
        else {
            return;
        };
        self.check_expression(ctx, range.start as usize, expression);
    }
}

fn skip_quoted(source: &[u8], mut cursor: usize, delimiter: u8) -> usize {
    cursor += 1;
    while let Some(byte) = source.get(cursor).copied() {
        match byte {
            b'\\' => cursor = (cursor + 2).min(source.len()),
            byte if byte == delimiter => return cursor + 1,
            _ => cursor += 1,
        }
    }
    source.len()
}

fn skip_template_literal(source: &[u8], mut cursor: usize) -> usize {
    cursor += 1;
    while let Some(byte) = source.get(cursor).copied() {
        match byte {
            b'\\' => cursor = (cursor + 2).min(source.len()),
            b'`' => return cursor + 1,
            b'$' if source.get(cursor + 1) == Some(&b'{') => {
                cursor = skip_template_expression(source, cursor + 2);
            }
            _ => cursor += 1,
        }
    }
    source.len()
}

fn skip_template_expression(source: &[u8], mut cursor: usize) -> usize {
    let mut depth = 1usize;
    while let Some(byte) = source.get(cursor).copied() {
        match byte {
            b'\\' => cursor = (cursor + 2).min(source.len()),
            b'\'' | b'"' => cursor = skip_quoted(source, cursor, byte),
            b'`' => cursor = skip_template_literal(source, cursor),
            b'/' if source.get(cursor + 1) == Some(&b'/') => {
                cursor = skip_line_comment(source, cursor + 2);
            }
            b'/' if source.get(cursor + 1) == Some(&b'*') => {
                cursor = skip_block_comment(source, cursor + 2);
            }
            b'{' => {
                depth += 1;
                cursor += 1;
            }
            b'}' => {
                depth -= 1;
                cursor += 1;
                if depth == 0 {
                    return cursor;
                }
            }
            _ => cursor += 1,
        }
    }
    source.len()
}

fn skip_line_comment(source: &[u8], mut cursor: usize) -> usize {
    while source
        .get(cursor)
        .is_some_and(|byte| !matches!(byte, b'\n' | b'\r'))
    {
        cursor += 1;
    }
    cursor
}

fn skip_block_comment(source: &[u8], mut cursor: usize) -> usize {
    while cursor < source.len() {
        if source.get(cursor) == Some(&b'*') && source.get(cursor + 1) == Some(&b'/') {
            return cursor + 2;
        }
        cursor += 1;
    }
    source.len()
}

fn opening_closing_start(source: &str, opening: ByteRange) -> u32 {
    let start = opening.start as usize;
    let end = opening.end as usize;
    let Some(authored) = source.get(start..end) else {
        return opening.end;
    };
    let Some(close) = authored.rfind('>') else {
        return opening.end;
    };
    let mut close = start + close;
    if close > start && source.as_bytes().get(close - 1) == Some(&b'/') {
        close -= 1;
    }
    close as u32
}

fn tag_name_end_hint(range: ByteRange, tag: &str) -> u32 {
    // For JSX member/namespaced tags the facade tag can be shorter than the
    // written tag. Starting early is fine because `first_whitespace_offset`
    // advances to the actual gap before checking it.
    range.start.saturating_add(1 + tag.len() as u32)
}

fn is_invalid_gap(gap: &str) -> bool {
    gap.len() > 1
        && gap
            .as_bytes()
            .iter()
            .all(|byte| matches!(byte, b' ' | b'\t'))
        && !gap
            .as_bytes()
            .iter()
            .any(|byte| matches!(byte, b'\n' | b'\r'))
}

fn first_whitespace_offset(source: &str, start: usize, end: usize) -> usize {
    let mut offset = start;
    let bytes = source.as_bytes();
    while offset < end
        && bytes
            .get(offset)
            .is_some_and(|byte| !matches!(byte, b' ' | b'\t' | b'\n' | b'\r'))
    {
        offset += 1;
    }
    offset
}

fn is_jsx_filename(filename: &str) -> bool {
    filename.ends_with(".jsx") || filename.ends_with(".tsx")
}

#[cfg(test)]
mod tests {
    use super::NoMultiSpaces;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoMultiSpaces::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_single_space() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="foo" id="bar"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_multiple_spaces() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="foo"  id="bar"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].has_fix());
    }

    #[test]
    fn test_invalid_multiple_spaces_before_first_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div  class="foo"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].has_fix());
    }

    #[test]
    fn test_valid_multiline_attributes() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button
  class="btn"
  :disabled="isDisabled"
>
</button>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_multiple_spaces_before_self_closing_end() {
        let result = create_linter().lint_template(r#"<Input fluid  />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_multiple_spaces_inside_expression() {
        let result = create_linter().lint_template(
            r#"<Input :aria-controls="expanded ?  id : undefined" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn quoted_expression_content_and_comments_are_not_rewritten() {
        let linter = create_linter();
        let clean = r#"<Input :one="'a  b'" :two="`c  d`" :three="value /* x  y */" />"#;
        assert_eq!(linter.lint_template(clean, "test.vue").warning_count, 0);

        let source = r#"<Input :one="'a  b'" :value="left  + right" />"#;
        let result = linter.lint_template(source, "test.vue");
        let start = source.find("left  +").unwrap() as u32 + 4;
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
        assert_eq!(
            (result.diagnostics[0].start, result.diagnostics[0].end),
            (start, start + 2)
        );
    }

    #[test]
    fn nested_template_literal_content_is_not_template_spacing() {
        let source = r#"<style>{{ `outer ${items.map(item => `  --color-${item}: red;`).join("\n")}` }}</style>"#;
        let result = create_linter().lint_template(source, "test.vue");
        assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn reports_spacing_before_a_string_literal_in_a_directive() {
        let source = r#"<span v-if="related && level ===  'error'" />"#;
        let result = create_linter().lint_template(source, "test.vue");
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    }

    #[test]
    fn reports_multiple_spaces_in_interpolations_and_slot_patterns() {
        for source in [
            r#"<template><p>{{ value ?  'yes' : 'no' }}</p></template>"#,
            r#"<template><p>{{  }}</p></template>"#,
            r#"<template><Child><template #item="{ row,  expanded }">{{ row }}</template></Child></template>"#,
        ] {
            let result = create_linter().lint_sfc(source, "test.vue");
            assert_eq!(
                result.warning_count, 1,
                "{source}: {:?}",
                result.diagnostics
            );
        }
    }
}
