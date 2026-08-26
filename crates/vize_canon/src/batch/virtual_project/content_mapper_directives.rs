//! Template diagnostic directives for the TypeScript content mapper.
//!
//! Vue templates cannot carry `@ts-expect-error` comments, so Vize maps the
//! Vue-standard HTML comment directives onto the content-mapper protocol's
//! `diagnosticDirectives`: `<!-- @vue-expect-error -->` expects at least one
//! TypeScript diagnostic on the next template line (and reports `vize4` when
//! none appears), while `<!-- @vue-ignore -->` silently suppresses them.
//! Script blocks pass through verbatim, so plain `@ts-expect-error` already
//! works there without any directive mapping.

use vize_atelier_sfc::SfcTemplateBlock;
use vize_s0::String as CompactString;

use super::protocol::{
    ContentMapperDiagnosticDirective, ContentMapperDiagnosticDirectives, ContentMapperSpan,
    ContentMapperUnusedExpectDiagnostic, DIRECTIVE_POLICY_EXPECT, DIRECTIVE_POLICY_IGNORE,
};

/// Mapper diagnostic code for an unused `@vue-expect-error` directive.
const UNUSED_EXPECT_CODE: i32 = 4;

const EXPECT_TOKEN: &str = "@vue-expect-error";
const IGNORE_TOKEN: &str = "@vue-ignore";

/// Collect `@vue-expect-error` / `@vue-ignore` template comment directives and
/// project them onto the generated ranges mapped from each directive's target
/// line. Returns `None` when the template declares no directives.
pub(super) fn template_diagnostic_directives(
    source: &str,
    template: Option<&SfcTemplateBlock<'_>>,
    spans: &[ContentMapperSpan],
) -> Option<ContentMapperDiagnosticDirectives> {
    let template = template?;
    let template_range = template.loc.start..template.loc.end.min(source.len());
    if template_range.is_empty() || !source.is_char_boundary(template_range.end) {
        return None;
    }

    let mut directives = Vec::new();
    let mut expects_unused_table = false;
    for comment in template_comments(source, template_range.clone()) {
        let Some((token, token_start)) = directive_token(&source[comment.text.clone()]) else {
            continue;
        };
        let original_start = comment.text.start + token_start;
        let target = directive_target(source, comment.end, template_range.end);
        let virtual_range = mapped_virtual_range(spans, &target);
        let policy = if token == EXPECT_TOKEN {
            expects_unused_table = true;
            DIRECTIVE_POLICY_EXPECT
        } else {
            if virtual_range.is_none() {
                continue;
            }
            DIRECTIVE_POLICY_IGNORE
        };
        let (virtual_start, virtual_end) = virtual_range.unwrap_or((0, 0));
        directives.push(ContentMapperDiagnosticDirective([
            original_start,
            token.len(),
            virtual_start,
            virtual_end,
            policy,
            0,
        ]));
    }

    if directives.is_empty() {
        return None;
    }
    let unused_expect_directive_diagnostics = if expects_unused_table {
        vec![ContentMapperUnusedExpectDiagnostic {
            code: UNUSED_EXPECT_CODE,
            message_text: CompactString::from("Unused '@vue-expect-error' directive"),
        }]
    } else {
        Vec::new()
    };
    Some(ContentMapperDiagnosticDirectives {
        unused_expect_directive_diagnostics,
        directives,
    })
}

struct TemplateComment {
    /// Comment text range between `<!--` and `-->`.
    text: std::ops::Range<usize>,
    /// Absolute offset just past the closing `-->`.
    end: usize,
}

fn template_comments(
    source: &str,
    template_range: std::ops::Range<usize>,
) -> impl Iterator<Item = TemplateComment> {
    let mut cursor = template_range.start;
    std::iter::from_fn(move || {
        let text = &source[cursor..template_range.end];
        let open = text.find("<!--")?;
        let text_start = cursor + open + "<!--".len();
        let close = source[text_start..template_range.end].find("-->")?;
        let comment = TemplateComment {
            text: text_start..text_start + close,
            end: text_start + close + "-->".len(),
        };
        cursor = comment.end;
        Some(comment)
    })
}

/// Find the first directive token in a comment's text, requiring a token
/// boundary so `@vue-ignore` never matches inside a longer word.
fn directive_token(comment: &str) -> Option<(&'static str, usize)> {
    [EXPECT_TOKEN, IGNORE_TOKEN]
        .iter()
        .filter_map(|token| {
            let start = comment.find(token)?;
            let tail = comment[start + token.len()..].bytes().next();
            tail.is_none_or(|byte| !byte.is_ascii_alphanumeric() && byte != b'-')
                .then_some((*token, start))
        })
        .min_by_key(|(_, start)| *start)
}

/// The authored range a directive applies to: the remainder of the comment's
/// own line when it carries content, otherwise the next non-empty line.
fn directive_target(
    source: &str,
    comment_end: usize,
    template_end: usize,
) -> std::ops::Range<usize> {
    let mut line_start = comment_end;
    loop {
        let rest = &source[line_start..template_end];
        let line_len = rest.find('\n').unwrap_or(rest.len());
        let line = &rest[..line_len];
        if !line.trim().is_empty() {
            return line_start..line_start + line_len;
        }
        if line_start + line_len >= template_end {
            return template_end..template_end;
        }
        line_start += line_len + 1;
    }
}

/// The union of generated ranges mapped from spans intersecting the target.
fn mapped_virtual_range(
    spans: &[ContentMapperSpan],
    target: &std::ops::Range<usize>,
) -> Option<(usize, usize)> {
    let mut range: Option<(usize, usize)> = None;
    for ContentMapperSpan([gen_start, gen_len, orig_start, orig_len, _, _]) in spans {
        if *orig_start < target.end && orig_start + orig_len > target.start {
            let (start, end) = range.get_or_insert((*gen_start, gen_start + gen_len));
            *start = (*start).min(*gen_start);
            *end = (*end).max(gen_start + gen_len);
        }
    }
    range
}
