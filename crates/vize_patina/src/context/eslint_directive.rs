//! ESLint suppression pragma parsing.

use vize_carton::String;

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
    const MARKERS: [(&str, EslintDisableKind); 4] = [
        (
            "eslint-disable-next-line",
            EslintDisableKind::DisableNextLine,
        ),
        ("eslint-disable-line", EslintDisableKind::DisableLine),
        ("eslint-disable", EslintDisableKind::Disable),
        ("eslint-enable", EslintDisableKind::Enable),
    ];

    for (marker, kind) in MARKERS {
        if let Some(index) = line.find(marker) {
            let rules = parse_eslint_rule_list(&line[index + marker.len()..]);
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
        .map(String::from)
        .collect()
}
