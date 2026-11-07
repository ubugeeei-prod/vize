//! vue/html-quotes
//!
//! Enforce consistent use of quotes in HTML attributes.
//!
//! ## Options
//!
//! - `"double"` (default): Require double quotes
//! - `"single"`: Require single quotes
//!
//! ## Reported span
//!
//! Upstream reports the attribute *value node*, whose range includes the
//! delimiters, and it applies to every attribute with a value — plain or
//! directive, on a native element or a component — plus unquoted values, which
//! are enclosed by no quote at all. The span therefore comes from
//! [`value_range`], not from the inner text relief exposes.
//!
//! ## The one difference from upstream
//!
//! A directive value that already contains the required quote is left alone,
//! which is upstream's `avoidEscape` behaviour applied to directives only.
//! Upstream's default reports it and escapes the quote as `&quot;`, but glyph
//! prints a directive expression with double-quoted JS strings and then has to
//! delimit the attribute with single quotes — so reporting it would make `vize
//! fmt` output that `vize lint` rejects. Plain attribute values, which glyph
//! never rewrites, follow upstream exactly. Tracked for removal once glyph
//! keeps the attribute delimiter stable.
//!
//! ## Examples
//!
//! ### Invalid (with double option)
//! ```vue
//! <div class='foo'></div>
//! <div class=foo></div>
//! <div v-if='foo'></div>
//! ```
//!
//! ### Valid (with double option)
//! ```vue
//! <div class="foo"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::cstr;
use vize_relief::{ElementNode, PropNode};

mod value_range;

use value_range::{ValueRange, value_range};

static META: RuleMeta = RuleMeta {
    name: "vue/html-quotes",
    description: "Enforce quotes style of HTML attributes",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Quote style preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HtmlQuotesOption {
    #[default]
    Double,
    Single,
}

impl HtmlQuotesOption {
    const fn quote(self) -> u8 {
        match self {
            Self::Double => b'"',
            Self::Single => b'\'',
        }
    }
}

/// Enforce HTML attribute quote style
pub struct HtmlQuotes {
    pub style: HtmlQuotesOption,
    pub avoid_escape: bool,
}

impl Default for HtmlQuotes {
    fn default() -> Self {
        Self {
            style: HtmlQuotesOption::Double,
            avoid_escape: false,
        }
    }
}

impl Rule for HtmlQuotes {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let expected = self.style.quote();
        for prop in &element.props {
            // A same-name `v-bind` shorthand (`:foo`) writes no value at all;
            // upstream skips it explicitly and `value_range` finds no `=`.
            let is_directive = match prop {
                PropNode::Directive(directive) if directive.shorthand => continue,
                PropNode::Directive(_) => true,
                PropNode::Attribute(_) => false,
            };
            let Some(range) = value_range(ctx.source, prop.loc()) else {
                continue;
            };
            if range.quote == Some(expected) {
                continue;
            }
            if (is_directive || (self.avoid_escape && range.quote.is_some()))
                && holds(ctx.source, range, expected)
            {
                continue;
            }
            self.report(ctx, range, expected);
        }
    }

    fn run_on_sfc(&self, ctx: &mut LintContext<'_>) {
        let expected = self.style.quote();
        let content_start = {
            let Some(descriptor) = ctx.sfc_descriptor() else {
                return;
            };
            descriptor.template.as_ref().map(|block| block.loc.start)
        };
        for range in sfc_template_attribute_values(ctx.source, content_start) {
            if range.quote == Some(expected)
                || (self.avoid_escape
                    && range.quote.is_some()
                    && holds(ctx.source, range, expected))
            {
                continue;
            }
            self.report(ctx, range, expected);
        }
    }
}

impl HtmlQuotes {
    fn report(&self, ctx: &mut LintContext<'_>, range: ValueRange, expected: u8) {
        let diagnostic = self.diagnostic(ctx, ctx.source, range, expected);
        ctx.report(diagnostic);
    }

    fn diagnostic(
        &self,
        ctx: &LintContext<'_>,
        source: &str,
        range: ValueRange,
        expected: u8,
    ) -> LintDiagnostic {
        let message = match self.style {
            HtmlQuotesOption::Double => ctx.t("vue/html-quotes.message_double"),
            HtmlQuotesOption::Single => ctx.t("vue/html-quotes.message_single"),
        };
        let help = ctx.t("vue/html-quotes.help");
        let fix_message = if expected == b'"' {
            "Use double quotes"
        } else {
            "Use single quotes"
        };
        let mut diagnostic =
            LintDiagnostic::warn(META.name, message, range.start, range.end).with_help(help);
        if let Some(fix) = quote_fix(source, range, expected, fix_message) {
            diagnostic = diagnostic.with_fix(fix);
        }
        diagnostic
    }
}

fn sfc_template_attribute_values(source: &str, content_start: Option<usize>) -> Vec<ValueRange> {
    let Some(content_start) = content_start.filter(|start| *start <= source.len()) else {
        return Vec::new();
    };
    let Some(tag_start) = source
        .get(..content_start)
        .and_then(|prefix| prefix.rfind("<template"))
    else {
        return Vec::new();
    };
    let bytes = source.as_bytes();
    let mut cursor = tag_start + "<template".len();
    let mut ranges = Vec::new();
    while cursor < content_start {
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if cursor >= content_start || matches!(bytes.get(cursor), Some(b'>') | Some(b'/')) {
            break;
        }
        while cursor < content_start
            && !bytes.get(cursor).is_some_and(|byte| {
                byte.is_ascii_whitespace() || matches!(byte, b'=' | b'>' | b'/')
            })
        {
            cursor += 1;
        }
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) != Some(&b'=') {
            continue;
        }
        cursor += 1;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let start = cursor;
        let quote = bytes
            .get(cursor)
            .copied()
            .filter(|byte| matches!(byte, b'\'' | b'"'));
        if let Some(delimiter) = quote {
            cursor += 1;
            while cursor < content_start && bytes.get(cursor) != Some(&delimiter) {
                cursor += 1;
            }
            if cursor < content_start {
                cursor += 1;
            }
        } else {
            while cursor < content_start
                && !bytes
                    .get(cursor)
                    .is_some_and(|byte| byte.is_ascii_whitespace() || matches!(byte, b'>' | b'/'))
            {
                cursor += 1;
            }
        }
        if start < cursor
            && let (Ok(start), Ok(end)) = (u32::try_from(start), u32::try_from(cursor))
        {
            ranges.push(ValueRange { start, end, quote });
        }
    }
    ranges
}

/// Whether the value already contains the quote it would have to be enclosed by.
fn holds(source: &str, range: ValueRange, quote: u8) -> bool {
    let (start, end) = range.inner();
    match (usize::try_from(start), usize::try_from(end)) {
        (Ok(start), Ok(end)) => source
            .as_bytes()
            .get(start..end)
            .is_some_and(|inner| inner.contains(&quote)),
        _ => false,
    }
}

/// Re-delimit the value, unless doing so would need the expected quote escaped.
fn quote_fix(
    source: &str,
    range: ValueRange,
    expected_quote: u8,
    message: &'static str,
) -> Option<Fix> {
    let (inner_start, inner_end) = range.inner();
    let inner = source
        .as_bytes()
        .get(usize::try_from(inner_start).ok()?..usize::try_from(inner_end).ok()?)?;
    if inner.contains(&expected_quote) {
        return None;
    }
    let replacement = if expected_quote == b'"' { "\"" } else { "'" };
    let edits = match range.quote {
        Some(_) => vec![
            TextEdit::replace(range.start, inner_start, replacement),
            TextEdit::replace(inner_end, range.end, replacement),
        ],
        None => {
            let text = core::str::from_utf8(inner).ok()?;
            vec![TextEdit::replace(
                range.start,
                range.end,
                cstr!("{replacement}{text}{replacement}"),
            )]
        }
    };
    Some(Fix::with_edits(message, edits))
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests;
