//! Narrow authored-source recovery for malformed or generator-shaped v-for.

use super::{META, NoUnusedVars, authored_element_is_empty, authored_identifier_reference_present};
use crate::context::LintContext;
use vize_l0::cstr;

pub(super) fn report_unparenthesized_v_for(ctx: &mut LintContext<'_>, rule: &NoUnusedVars) {
    let source = ctx.source;
    let bytes = source.as_bytes();
    let mut search_from = 0;
    while let Some(relative) = source
        .get(search_from..)
        .and_then(|rest| rest.find("v-for"))
    {
        let directive_start = search_from + relative;
        search_from = directive_start + "v-for".len();
        let mut cursor = search_from;
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
        let Some(quote @ (b'\'' | b'"')) = bytes.get(cursor).copied() else {
            continue;
        };
        let value_start = cursor + 1;
        let Some(value_end_relative) = source
            .get(value_start..)
            .and_then(|value| value.find(quote as char))
        else {
            continue;
        };
        let value_end = value_start + value_end_relative;
        let Some(value) = source.get(value_start..value_end) else {
            continue;
        };
        let Some((aliases, _)) = value.split_once(" in ") else {
            continue;
        };
        if aliases.trim_start().starts_with('(') || !aliases.contains(',') {
            continue;
        }
        let Some(open) = source
            .get(..directive_start)
            .and_then(|prefix| prefix.rfind('<'))
        else {
            continue;
        };
        let opening_before = source.get(open..value_start).unwrap_or_default();
        let element_end = authored_element_end(source, open, value_end);
        let authored_after = source.get(value_end..element_end).unwrap_or_default();
        report_aliases(
            ctx,
            rule,
            aliases,
            value_start,
            value_end,
            opening_before,
            authored_after,
            false,
        );
    }
}

fn authored_element_end(source: &str, open: usize, after: usize) -> usize {
    let Some(tag_start) = source.get(open + 1..) else {
        return after;
    };
    let tag_len = tag_start
        .bytes()
        .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        .count();
    let Some(tag) = tag_start.get(..tag_len) else {
        return after;
    };
    let closing = cstr!("</{tag}");
    source
        .get(after..)
        .and_then(|rest| rest.find(closing.as_str()))
        .and_then(|relative| {
            let closing_start = after + relative;
            source
                .get(closing_start..)
                .and_then(|rest| rest.find('>'))
                .map(|end| closing_start + end + 1)
        })
        .unwrap_or(after)
}

pub(super) fn report_empty_v_for(ctx: &mut LintContext<'_>, rule: &NoUnusedVars) {
    let source = ctx.source;
    let bytes = source.as_bytes();
    let mut search_from = 0;

    while let Some(relative) = source
        .get(search_from..)
        .and_then(|rest| rest.find("v-for"))
    {
        let directive_start = search_from + relative;
        search_from = directive_start + "v-for".len();
        if directive_start > 0
            && bytes
                .get(directive_start - 1)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        {
            continue;
        }

        let Some(open) = source
            .get(..directive_start)
            .and_then(|prefix| prefix.rfind('<'))
        else {
            continue;
        };
        let Some(close_relative) = source
            .get(directive_start..)
            .and_then(|rest| rest.find('>'))
        else {
            continue;
        };
        let close = directive_start + close_relative;
        if !authored_element_is_empty(source, open, close) {
            continue;
        }

        let mut cursor = search_from;
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
        let Some(quote @ (b'\'' | b'"')) = bytes.get(cursor).copied() else {
            continue;
        };
        let value_start = cursor + 1;
        let Some(value_end_relative) = source
            .get(value_start..close)
            .and_then(|value| value.find(quote as char))
        else {
            continue;
        };
        let value_end = value_start + value_end_relative;
        let Some(value) = source.get(value_start..value_end) else {
            continue;
        };
        let Some((aliases, rhs)) = value.split_once(" in ") else {
            continue;
        };
        if !rhs.contains(" in ") || !(rhs.contains('\'') || rhs.contains('"')) {
            continue;
        }

        let opening_before = source.get(open..directive_start).unwrap_or_default();
        let opening_after = source.get(value_end + 1..=close).unwrap_or_default();
        let aliases = aliases.trim();
        let aliases = aliases
            .strip_prefix('(')
            .and_then(|aliases| aliases.strip_suffix(')'))
            .unwrap_or(aliases);
        report_aliases(
            ctx,
            rule,
            aliases,
            value_start,
            value_end,
            opening_before,
            opening_after,
            true,
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the shared recovery reporter retains each authored range and alias fact explicitly"
)]
fn report_aliases(
    ctx: &mut LintContext<'_>,
    rule: &NoUnusedVars,
    aliases: &str,
    value_start: usize,
    value_end: usize,
    opening_before: &str,
    opening_after: &str,
    require_identifiers: bool,
) {
    let source = ctx.source;
    let mut alias_search = value_start;
    for (index, alias) in aliases.split(',').map(str::trim).enumerate() {
        if alias.is_empty()
            || (require_identifiers
                && !alias
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')))
            || rule.should_ignore(alias)
            || authored_identifier_reference_present(opening_before, alias)
            || authored_identifier_reference_present(opening_after, alias)
        {
            continue;
        }
        let Some(relative_start) = source
            .get(alias_search..value_end)
            .and_then(|remaining| remaining.find(alias))
        else {
            continue;
        };
        let start = alias_search + relative_start;
        alias_search = start + alias.len();
        if ctx
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == META.name && diagnostic.start == start as u32)
        {
            continue;
        }
        let (message, help) = if index == 0 {
            (
                cstr!("Variable '{}' is defined by v-for but never used", alias),
                "If the variable is intentionally unused, prefix it with underscore: _item",
            )
        } else if index == 1 {
            (
                cstr!(
                    "Key variable '{}' is defined by v-for but never used",
                    alias
                ),
                "Consider removing the key variable or prefix it with underscore: _key",
            )
        } else {
            (
                cstr!(
                    "Index variable '{}' is defined by v-for but never used",
                    alias
                ),
                "Consider removing the index variable or prefix it with underscore: _index",
            )
        };
        ctx.report(
            crate::diagnostic::LintDiagnostic::warn(
                ctx.current_rule,
                message,
                start as u32,
                (start + alias.len()) as u32,
            )
            .with_help(help),
        );
    }
}
