//! Directive state whose offsets and line numbers address a complete SFC.

use memchr::memchr_iter;
use vize_carton::directive::{
    DirectiveKind, DirectiveSeverity, parse_level_severity, parse_vize_directive,
};
use vize_carton::{CompactString, FxHashMap, FxHashSet, String};

use super::DisabledRange;
use super::eslint_directive::{EslintDisableKind, parse_eslint_disable_comment};

#[derive(Clone, Copy, PartialEq, Eq)]
enum LexicalContext {
    Code,
    Interpolation(u32),
    SingleQuote,
    DoubleQuote,
    Template,
    BlockComment,
    HtmlComment,
    LineComment,
}

struct DirectiveLexer {
    stack: Vec<LexicalContext>,
}

impl Default for DirectiveLexer {
    fn default() -> Self {
        Self {
            stack: vec![LexicalContext::Code],
        }
    }
}

#[derive(Default)]
struct CommentMarkers {
    eslint: Option<usize>,
    vize: Option<usize>,
}

#[derive(Default)]
pub(super) struct SfcDirectiveState {
    disabled_all: Vec<DisabledRange>,
    disabled_rules: FxHashMap<CompactString, Vec<DisabledRange>>,
    ignored_regions: Vec<DisabledRange>,
    expected_error_lines: FxHashSet<u32>,
    severity_overrides: FxHashMap<u32, DirectiveSeverity>,
    line_offsets: Vec<u32>,
}

impl SfcDirectiveState {
    pub(super) fn scan_if_present(source: &str) -> Option<Self> {
        if !source.contains("eslint-") && !source.contains("@vize:") {
            return None;
        }

        let mut state = Self {
            line_offsets: std::iter::once(0)
                .chain(memchr_iter(b'\n', source.as_bytes()).map(|offset| (offset + 1) as u32))
                .collect(),
            ..Self::default()
        };
        let mut lexer = DirectiveLexer::default();
        for (line_number, line) in (1u32..).zip(source.lines()) {
            let markers = lexer.scan_line(line);
            state.scan_eslint_directive(line, line_number, markers.eslint);
            state.scan_vize_directive(line, line_number, markers.vize);
        }
        Some(state)
    }

    pub(super) fn offset_to_line(&self, offset: u32) -> u32 {
        match self.line_offsets.binary_search(&offset) {
            Ok(line) => (line + 1) as u32,
            Err(line) => line as u32,
        }
    }

    pub(super) fn is_disabled_at(&self, rule_name: &str, line: u32) -> bool {
        self.disabled_all
            .iter()
            .any(|range| range_contains(range, line))
            || self
                .ignored_regions
                .iter()
                .any(|range| range_contains(range, line))
            || self
                .disabled_rules
                .get(rule_name)
                .is_some_and(|ranges| ranges.iter().any(|range| range_contains(range, line)))
    }

    pub(super) fn is_expected_at(&self, line: u32) -> bool {
        self.expected_error_lines.contains(&line)
    }

    pub(super) fn severity_at(&self, line: u32) -> Option<DirectiveSeverity> {
        self.severity_overrides.get(&line).copied()
    }

    fn scan_eslint_directive(&mut self, line: &str, line_number: u32, index: Option<usize>) {
        let Some(index) = index else {
            return;
        };
        let Some(directive) = parse_eslint_disable_comment(&line[index..]) else {
            return;
        };
        match directive.kind {
            EslintDisableKind::DisableNextLine => {
                self.disable_rules(directive.rules, line_number + 1, Some(line_number + 1))
            }
            EslintDisableKind::DisableLine => {
                self.disable_rules(directive.rules, line_number, Some(line_number));
            }
            EslintDisableKind::Disable => {
                self.disable_rules(directive.rules, line_number, None);
            }
            EslintDisableKind::Enable => self.enable_rules(directive.rules, line_number),
        }
    }

    fn scan_vize_directive(&mut self, line: &str, line_number: u32, index: Option<usize>) {
        let Some(index) = index else {
            return;
        };
        let content = line[index..]
            .split_once("-->")
            .map_or(&line[index..], |(content, _)| content);
        let content = content
            .split_once("*/")
            .map_or(content, |(content, _)| content);
        let Some(directive) = parse_vize_directive(content, line_number, 0) else {
            return;
        };
        match directive.kind {
            DirectiveKind::Expected => {
                self.expected_error_lines.insert(line_number + 1);
            }
            DirectiveKind::Level => {
                if let Some(severity) = parse_level_severity(&directive.payload) {
                    self.severity_overrides.insert(line_number + 1, severity);
                }
            }
            DirectiveKind::IgnoreStart => self.ignored_regions.push(DisabledRange {
                start_line: line_number,
                end_line: None,
            }),
            DirectiveKind::IgnoreEnd => close_last_range(&mut self.ignored_regions, line_number),
            _ => {}
        }
    }

    fn disable_rules(&mut self, rules: Vec<String>, start_line: u32, end_line: Option<u32>) {
        if rules.is_empty() {
            self.disabled_all.push(DisabledRange {
                start_line,
                end_line,
            });
            return;
        }
        for rule in rules {
            self.disabled_rules
                .entry(CompactString::new(rule.as_str()))
                .or_default()
                .push(DisabledRange {
                    start_line,
                    end_line,
                });
        }
    }

    fn enable_rules(&mut self, rules: Vec<String>, line: u32) {
        if rules.is_empty() {
            close_ranges(&mut self.disabled_all, line);
            for ranges in self.disabled_rules.values_mut() {
                close_ranges(ranges, line);
            }
            return;
        }
        for rule in rules {
            if let Some(ranges) = self.disabled_rules.get_mut(rule.as_str()) {
                close_ranges(ranges, line);
            }
        }
    }
}

fn range_contains(range: &DisabledRange, line: u32) -> bool {
    line >= range.start_line && range.end_line.is_none_or(|end| line <= end)
}

fn close_ranges(ranges: &mut [DisabledRange], line: u32) {
    for range in ranges {
        if range.end_line.is_none() {
            range.end_line = Some(line);
        }
    }
}

fn close_last_range(ranges: &mut [DisabledRange], line: u32) {
    if let Some(range) = ranges
        .iter_mut()
        .rev()
        .find(|range| range.end_line.is_none())
    {
        range.end_line = Some(line);
    }
}

impl DirectiveLexer {
    fn scan_line(&mut self, line: &str) -> CommentMarkers {
        let bytes = line.as_bytes();
        let mut markers = CommentMarkers::default();
        let mut index = 0;
        while index < bytes.len() {
            let context = *self.stack.last().expect("lexer always has a root context");
            if matches!(
                context,
                LexicalContext::BlockComment
                    | LexicalContext::HtmlComment
                    | LexicalContext::LineComment
            ) {
                if markers.eslint.is_none() && bytes[index..].starts_with(b"eslint-") {
                    markers.eslint = Some(index);
                }
                if markers.vize.is_none() && bytes[index..].starts_with(b"@vize:") {
                    markers.vize = Some(index);
                }
            }

            let current = bytes[index];
            let next = bytes.get(index + 1).copied();
            match context {
                LexicalContext::Code | LexicalContext::Interpolation(_) => match (current, next) {
                    (b'/', Some(b'/')) => {
                        self.stack.push(LexicalContext::LineComment);
                        index += 1;
                    }
                    (b'/', Some(b'*')) => {
                        self.stack.push(LexicalContext::BlockComment);
                        index += 1;
                    }
                    (b'<', Some(b'!'))
                        if bytes
                            .get(index..index + 4)
                            .is_some_and(|slice| slice == b"<!--") =>
                    {
                        self.stack.push(LexicalContext::HtmlComment);
                        index += 3;
                    }
                    (b'\'', _) => self.stack.push(LexicalContext::SingleQuote),
                    (b'"', _) => self.stack.push(LexicalContext::DoubleQuote),
                    (b'`', _) => self.stack.push(LexicalContext::Template),
                    (b'{', _) => {
                        if let Some(LexicalContext::Interpolation(depth)) = self.stack.last_mut() {
                            *depth += 1;
                        }
                    }
                    (b'}', _) => {
                        if let Some(LexicalContext::Interpolation(depth)) = self.stack.last_mut() {
                            if *depth == 0 {
                                self.stack.pop();
                            } else {
                                *depth -= 1;
                            }
                        }
                    }
                    _ => {}
                },
                LexicalContext::SingleQuote => match (current, next) {
                    (b'\\', Some(_)) => index += 1,
                    (b'\'', _) => {
                        self.stack.pop();
                    }
                    _ => {}
                },
                LexicalContext::DoubleQuote => match (current, next) {
                    (b'\\', Some(_)) => index += 1,
                    (b'"', _) => {
                        self.stack.pop();
                    }
                    _ => {}
                },
                LexicalContext::Template => match (current, next) {
                    (b'\\', Some(_)) => index += 1,
                    (b'`', _) => {
                        self.stack.pop();
                    }
                    (b'$', Some(b'{')) => {
                        self.stack.push(LexicalContext::Interpolation(0));
                        index += 1;
                    }
                    _ => {}
                },
                LexicalContext::BlockComment => {
                    if (current, next) == (b'*', Some(b'/')) {
                        self.stack.pop();
                        index += 1;
                    }
                }
                LexicalContext::HtmlComment => {
                    if bytes
                        .get(index..index + 3)
                        .is_some_and(|slice| slice == b"-->")
                    {
                        self.stack.pop();
                        index += 2;
                    }
                }
                LexicalContext::LineComment => {}
            }
            index += 1;
        }

        if matches!(self.stack.last(), Some(LexicalContext::LineComment)) {
            self.stack.pop();
        }
        while matches!(
            self.stack.last(),
            Some(LexicalContext::SingleQuote | LexicalContext::DoubleQuote)
        ) {
            self.stack.pop();
        }
        markers
    }
}
