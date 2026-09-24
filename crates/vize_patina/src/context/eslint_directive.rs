//! ESLint and Oxlint suppression pragma parsing.

use vize_s0::String;

#[derive(Clone, Copy)]
pub(super) enum EslintDisableKind {
    DisableNextLine,
    DisableLine,
    Disable,
    Enable,
}

pub(super) struct EslintDisableDirective {
    pub(super) kind: EslintDisableKind,
    pub(super) rules: Vec<String>,
}

pub(super) fn parse_eslint_disable_comment(line: &str) -> Option<EslintDisableDirective> {
    const MARKERS: [(&str, EslintDisableKind); 8] = [
        (
            "eslint-disable-next-line",
            EslintDisableKind::DisableNextLine,
        ),
        (
            "oxlint-disable-next-line",
            EslintDisableKind::DisableNextLine,
        ),
        ("eslint-disable-line", EslintDisableKind::DisableLine),
        ("oxlint-disable-line", EslintDisableKind::DisableLine),
        ("eslint-disable", EslintDisableKind::Disable),
        ("oxlint-disable", EslintDisableKind::Disable),
        ("eslint-enable", EslintDisableKind::Enable),
        ("oxlint-enable", EslintDisableKind::Enable),
    ];

    for (marker, kind) in MARKERS {
        if let Some(index) = line.find(marker) {
            let rules =
                parse_eslint_rule_list(line.get(index + marker.len()..).unwrap_or_default());
            return Some(EslintDisableDirective { kind, rules });
        }
    }

    None
}

fn parse_eslint_rule_list(raw: &str) -> Vec<String> {
    let raw = raw
        .split_once("--")
        .map_or(raw, |(before_reason, _)| before_reason)
        .replace("*/", " ")
        .replace("-->", " ");

    raw.split(|char: char| char == ',' || char.is_ascii_whitespace())
        .map(str::trim)
        .filter(|rule| !rule.is_empty())
        .map(|rule| {
            rule.trim_matches(|char: char| {
                matches!(char, '"' | '\'' | '[' | ']' | '{' | '}' | '(' | ')' | ';')
            })
        })
        .filter(|rule| !rule.is_empty())
        // Oxlint's plugin rule IDs include the plugin prefix. The standalone
        // linter uses the same IDs without that prefix.
        .map(|rule| {
            rule.strip_prefix("vize/")
                .filter(|name| name.contains('/'))
                .unwrap_or(rule)
        })
        .map(String::from)
        .collect()
}
