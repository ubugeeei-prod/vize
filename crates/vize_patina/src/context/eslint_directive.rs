//! ESLint and Oxlint suppression pragma parsing.

use std::cell::{Cell, RefCell};

use vize_l0::String;

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
    /// Byte range of the pragma marker, for template comments only.
    marker: Option<(u32, u32)>,
    /// Whether this pragma suppressed any diagnostic.
    used: Cell<bool>,
    /// Listed rules whose diagnostics this pragma suppressed.
    used_rules: RefCell<Vec<String>>,
}

/// A disable pragma that suppressed nothing (`--report-unused-disable-directives`).
pub(crate) struct UnusedDirective {
    pub(crate) start: u32,
    pub(crate) end: u32,
    /// The listed rule that reported nothing; `None` for a pragma without rules.
    pub(crate) rule: Option<String>,
}

impl InlineSuppressionState {
    pub(super) fn record(&mut self, directive: EslintDisableDirective, line: u32) {
        self.record_at(directive, line, None);
    }

    pub(super) fn record_at(
        &mut self,
        directive: EslintDisableDirective,
        line: u32,
        marker: Option<(u32, u32)>,
    ) {
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
            marker,
            used: Cell::new(false),
            used_rules: RefCell::new(Vec::new()),
        });
    }

    pub(super) fn is_disabled_at(&self, rule_name: &str, line: u32) -> bool {
        let Some(event) = self.events.iter().rev().find(|event| {
            line >= event.start_line
                && event.end_line.is_none_or(|end| line <= end)
                && (event.rules.is_empty() || event.rules.iter().any(|rule| rule == rule_name))
        }) else {
            return false;
        };
        if event.enabled {
            return false;
        }
        event.used.set(true);
        let mut used_rules = event.used_rules.borrow_mut();
        if !used_rules.iter().any(|rule| rule == rule_name) {
            used_rules.push(String::from(rule_name));
        }
        true
    }

    /// Located disable pragmas that suppressed nothing. A listed rule counts
    /// only when `is_active` says this run could have reported it.
    pub(super) fn unused(&self, is_active: impl Fn(&str) -> bool) -> Vec<UnusedDirective> {
        let mut unused = Vec::new();
        for event in &self.events {
            let Some((start, end)) = event.marker.filter(|_| !event.enabled) else {
                continue;
            };
            if event.rules.is_empty() {
                if !event.used.get() {
                    unused.push(UnusedDirective {
                        start,
                        end,
                        rule: None,
                    });
                }
                continue;
            }
            let used_rules = event.used_rules.borrow();
            for rule in &event.rules {
                if is_active(rule) && !used_rules.contains(rule) {
                    unused.push(UnusedDirective {
                        start,
                        end,
                        rule: Some(rule.clone()),
                    });
                }
            }
        }
        unused
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
        // These registered script rules share ESLint's identity. Keep the
        // compatibility mapping narrow; unrelated Vue IDs still match exactly.
        .map(|rule| match rule {
            "vue/no-reserved-keys" => "script/no-reserved-keys",
            "vue/no-dupe-keys" => "script/no-dupe-keys",
            "vue/no-side-effects-in-computed-properties" => {
                "script/no-side-effects-in-computed-properties"
            }
            "vue/no-deprecated-dollar-listeners-api" => "script/no-deprecated-dollar-listeners-api",
            "vue/no-deprecated-dollar-scopedslots-api" => {
                "script/no-deprecated-dollar-scopedslots-api"
            }
            "vue/no-use-computed-property-like-method" => {
                "script/no-use-computed-property-like-method"
            }
            "vue/require-valid-default-prop" => "script/require-valid-default-prop",
            rule => rule,
        })
        .map(String::from)
        .collect()
}
