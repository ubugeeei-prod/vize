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

#[derive(Default)]
pub(super) struct InlineSuppressionState {
    events: Vec<InlineSuppressionEvent>,
}

struct InlineSuppressionEvent {
    start_line: u32,
    end_line: Option<u32>,
    enabled: bool,
    rules: Vec<String>,
}

impl InlineSuppressionState {
    pub(super) fn record(&mut self, directive: EslintDisableDirective, line: u32) {
        let (start_line, end_line, enabled) = match directive.kind {
            EslintDisableKind::DisableNextLine => (line + 1, Some(line + 1), false),
            EslintDisableKind::DisableLine => (line, Some(line), false),
            EslintDisableKind::Disable => (line, None, false),
            EslintDisableKind::Enable => (line, None, true),
        };
        self.events.push(InlineSuppressionEvent {
            start_line,
            end_line,
            enabled,
            rules: directive.rules,
        });
    }

    pub(super) fn is_disabled_at(&self, rule_name: &str, line: u32) -> bool {
        self.events
            .iter()
            .rev()
            .find(|event| {
                line >= event.start_line
                    && event.end_line.is_none_or(|end| line <= end)
                    && (event.rules.is_empty() || event.rules.iter().any(|rule| rule == rule_name))
            })
            .is_some_and(|event| !event.enabled)
    }
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
