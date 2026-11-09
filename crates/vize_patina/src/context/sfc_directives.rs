//! Directive state whose offsets and line numbers address a complete SFC.

mod lexer;

use memchr::memchr_iter;
use vize_atelier_sfc::SfcDescriptor;
use vize_l0::directive::{
    DirectiveKind, DirectiveSeverity, parse_level_severity, parse_vize_directive,
};
use vize_l0::{Allocator, FxHashMap, FxHashSet};
use vize_l1::css::lang::StyleLanguage;

use super::eslint_directive::{
    EslintDisableDirective, EslintDisableKind, ExactSuppressionState, InlineSuppressionState,
    parse_eslint_disable_comment, parse_script_disable_comment,
};
use super::{DisabledRange, LintContext, TemplateCommentDirectives};
use lexer::{DirectiveLexer, StyleDirectiveLexer};

mod root_comments;

/// Plain modules use the same lexical comment admission as SFC scripts.
pub(super) fn script_suppressions(source: &str, filename: &str) -> InlineSuppressionState {
    let tsx = filename.ends_with(".tsx");
    let mut lexer = DirectiveLexer::new(tsx || filename.ends_with(".jsx"), tsx);
    let mut state = InlineSuppressionState::default();
    let mut offset = 0;
    for (line_number, raw_line) in (1u32..).zip(source.split_inclusive('\n')) {
        let line = raw_line.trim_end_matches(['\r', '\n']);
        let markers = lexer.scan_line(line, source.get(offset..).unwrap_or_default());
        if let Some(index) = markers
            .eslint
            .filter(|&index| marker_starts_comment_content(line, index))
            && let Some(directive) = line.get(index..).and_then(parse_script_disable_comment)
        {
            let text = line.get(index..).unwrap_or_default();
            let text = text.split_once("*/").map_or(text, |(comment, _)| comment);
            let marker = (!text.starts_with("vize-")).then_some((
                offset as u32 + index as u32,
                offset as u32 + index as u32 + text.trim_end().len() as u32,
            ));
            state.record_at(directive, line_number, marker);
        }
        offset += raw_line.len();
    }
    state
}

#[derive(Clone, Copy)]
enum BlockDomain<'a> {
    Script { jsx: bool, tsx: bool },
    Style(Option<&'a str>),
}

#[derive(Default)]
pub(super) struct SfcDirectiveState {
    blocks: Vec<BlockDirectiveState>,
    /// File-level script regions in complete authored source order.
    file_regions: InlineSuppressionState,
    /// Pragmas in comments written between the blocks.
    root: Option<InlineSuppressionState>,
    template_exact: Option<TemplateExactSuppressions>,
    line_offsets: Vec<u32>,
}

struct TemplateExactSuppressions {
    start: u32,
    end: u32,
    state: ExactSuppressionState,
}

#[derive(Default)]
struct BlockDirectiveState {
    start: u32,
    end: u32,
    script: bool,
    inline_suppressions: InlineSuppressionState,
    /// `eslint-disable` / `eslint-enable` block comments of a script block.
    /// ESLint reads them as file-level directives, so a region left open
    /// here still covers a template that follows the block.
    file_regions: InlineSuppressionState,
    ignored_regions: Vec<DisabledRange>,
    expected_error_lines: FxHashSet<u32>,
    severity_overrides: FxHashMap<u32, DirectiveSeverity>,
}

impl SfcDirectiveState {
    pub(super) fn scan_if_present(descriptor: &SfcDescriptor<'_>) -> Option<Self> {
        let has_markers = descriptor
            .script
            .iter()
            .chain(descriptor.script_setup.iter())
            .map(|block| block.content.as_ref())
            .chain(descriptor.styles.iter().map(|block| block.content.as_ref()))
            .any(has_directive_marker);
        let source = descriptor.source.as_ref();
        let has_root_comments = source.contains("<!--") && has_directive_marker(source);
        if !has_markers && !has_root_comments {
            return None;
        }

        let mut state = Self {
            blocks: Vec::new(),
            file_regions: InlineSuppressionState::default(),
            root: None,
            template_exact: None,
            line_offsets: std::iter::once(0)
                .chain(memchr_iter(b'\n', source.as_bytes()).map(|offset| (offset + 1) as u32))
                .collect(),
        };
        let mut blocks = descriptor
            .script
            .iter()
            .chain(descriptor.script_setup.iter())
            .map(|block| {
                let (jsx, tsx) = match block.lang.as_deref() {
                    Some("tsx") => (true, true),
                    Some("jsx") => (true, false),
                    _ => (false, false),
                };
                (
                    block.loc.start,
                    block.loc.end,
                    block.content.as_ref(),
                    BlockDomain::Script { jsx, tsx },
                )
            })
            .chain(descriptor.styles.iter().map(|block| {
                (
                    block.loc.start,
                    block.loc.end,
                    block.content.as_ref(),
                    BlockDomain::Style(block.lang.as_deref()),
                )
            }))
            .collect::<Vec<_>>();
        blocks.sort_unstable_by_key(|(start, _, _, _)| *start);
        for (start, end, content, domain) in blocks {
            if !has_directive_marker(content) {
                continue;
            }
            let first_line = state.offset_to_line(start as u32);
            let mut block = BlockDirectiveState {
                start: start as u32,
                end: end as u32,
                script: matches!(domain, BlockDomain::Script { .. }),
                ..BlockDirectiveState::default()
            };
            match domain {
                BlockDomain::Script { jsx, tsx } => {
                    let mut lexer = DirectiveLexer::new(jsx, tsx);
                    block.scan(content, first_line, |line, remaining| {
                        lexer.scan_line(line, remaining)
                    });
                }
                BlockDomain::Style(lang) => {
                    let allow_line_comments = StyleLanguage::from_lang(lang)
                        .is_some_and(StyleLanguage::has_line_comments);
                    let mut lexer = StyleDirectiveLexer::new(allow_line_comments);
                    block.scan(content, first_line, |line, _| lexer.scan_line(line));
                }
            }
            state.file_regions.append(&mut block.file_regions);
            state.blocks.push(block);
        }
        if has_root_comments {
            let blocks = descriptor
                .template
                .iter()
                .map(|block| &block.loc)
                .chain(descriptor.script.iter().map(|block| &block.loc))
                .chain(descriptor.script_setup.iter().map(|block| &block.loc))
                .chain(descriptor.styles.iter().map(|block| &block.loc))
                .chain(descriptor.custom_blocks.iter().map(|block| &block.loc))
                // Descriptors built without tag offsets only know the content.
                .map(|loc| match loc.tag_end > loc.end {
                    true => (loc.tag_start, loc.tag_end),
                    false => (loc.start, loc.end),
                })
                .collect();
            let root =
                root_comments::scan(source, blocks, |offset| state.offset_to_line(offset as u32));
            state.root = root;
        }
        state.template_exact = descriptor.template.as_ref().and_then(|template| {
            if template.src.is_some()
                || !template.lang.as_deref().is_none_or(|lang| lang == "html")
                || !template.content.contains("@vize:forget")
            {
                return None;
            }
            let allocator = Allocator::with_capacity((template.content.len() * 4).max(64 * 1024));
            let (surface, _, _) =
                vize_l1::markup::parse_with_authored_diagnostics(&allocator, &template.content);
            let mut context = LintContext::new(&allocator, &template.content, "<sfc-directives>");
            TemplateCommentDirectives::new(&surface).prescan(&mut context);
            let mut exact = context.exact_suppressions;
            exact.offset_markers(template.loc.start as u32);
            Some(TemplateExactSuppressions {
                start: template.loc.start as u32,
                end: template.loc.end as u32,
                state: exact,
            })
        });
        Some(state)
    }

    /// Named project rules share the lexical admission used for normal SFC diagnostics.
    pub(super) fn unused_named(
        &self,
        is_active: impl Fn(&str) -> bool,
    ) -> Vec<super::eslint_directive::UnusedDirective> {
        self.unused(is_active)
            .into_iter()
            .filter(|directive| directive.rule.is_some())
            .collect()
    }

    /// Accepted script/style/root pragmas that suppressed no final diagnostic.
    pub(super) fn unused(
        &self,
        is_active: impl Fn(&str) -> bool,
    ) -> Vec<super::eslint_directive::UnusedDirective> {
        let mut unused = Vec::new();
        for block in &self.blocks {
            block.inline_suppressions.import_usage(&self.file_regions);
            unused.extend(block.inline_suppressions.unused(&is_active));
        }
        if let Some(root) = &self.root {
            unused.extend(root.unused(&is_active));
        }
        if let Some(template) = &self.template_exact {
            unused.extend(template.state.unused(&is_active));
        }
        unused.sort_unstable_by_key(|directive| (directive.start, directive.end));
        unused.dedup_by(|left, right| {
            left.start == right.start
                && left.end == right.end
                && left.rule == right.rule
                && left.source == right.source
        });
        unused
    }

    /// Blanket pragmas that suppressed a diagnostic after the ordinary SFC
    /// pass finished. Project diagnostics use this to retire the matching
    /// early unused-directive report without affecting peer comments.
    pub(super) fn used_blanket_markers(&self) -> Vec<(u32, u32)> {
        let mut used = self
            .blocks
            .iter()
            .flat_map(|block| block.inline_suppressions.used_blanket_markers())
            .chain(self.file_regions.used_blanket_markers())
            .collect::<Vec<_>>();
        if let Some(root) = &self.root {
            used.extend(root.used_blanket_markers());
        }
        used.sort_unstable();
        used.dedup();
        used
    }

    pub(super) fn offset_to_line(&self, offset: u32) -> u32 {
        match self.line_offsets.binary_search(&offset) {
            Ok(line) => (line + 1) as u32,
            Err(line) => line as u32,
        }
    }

    pub(super) fn is_disabled_at(&self, rule_name: &str, line: u32, offset: u32) -> bool {
        self.template_exact.as_ref().is_some_and(|template| {
            template.start <= offset
                && offset < template.end
                && template
                    .state
                    .is_disabled_at(rule_name, offset - template.start)
        }) || self
            .block_at(offset)
            .is_some_and(|block| block.is_disabled_at(rule_name, line))
            || self.file_regions.is_disabled_at(rule_name, line)
            || self
                .root
                .as_ref()
                .is_some_and(|root| root.is_disabled_at(rule_name, line))
    }

    pub(super) fn is_expected_at(&self, line: u32, offset: u32) -> bool {
        self.block_at(offset)
            .is_some_and(|block| block.expected_error_lines.contains(&line))
    }

    pub(super) fn severity_at(&self, line: u32, offset: u32) -> Option<DirectiveSeverity> {
        self.block_at(offset)
            .and_then(|block| block.severity_overrides.get(&line).copied())
    }

    fn block_at(&self, offset: u32) -> Option<&BlockDirectiveState> {
        self.blocks
            .iter()
            .find(|block| block.start <= offset && offset < block.end)
    }
}

impl BlockDirectiveState {
    fn scan<F>(&mut self, source: &str, first_line: u32, mut scan_line: F)
    where
        F: FnMut(&str, &str) -> lexer::CommentMarkers,
    {
        let mut offset = 0;
        for (line_number, raw_line) in (first_line..).zip(source.split_inclusive('\n')) {
            let line = raw_line
                .strip_suffix('\n')
                .map_or(raw_line, |line| line.strip_suffix('\r').unwrap_or(line));
            let markers = scan_line(line, source.get(offset..).unwrap_or_default());
            self.scan_eslint_directive(
                line,
                line_number,
                self.start + offset as u32,
                markers.eslint,
            );
            self.scan_vize_directive(line, line_number, markers.vize);
            offset += raw_line.len();
        }
    }

    fn is_disabled_at(&self, rule_name: &str, line: u32) -> bool {
        self.ignored_regions
            .iter()
            .any(|range| range_contains(range, line))
            || self.inline_suppressions.is_disabled_at(rule_name, line)
    }

    fn scan_eslint_directive(
        &mut self,
        line: &str,
        line_number: u32,
        line_start: u32,
        index: Option<usize>,
    ) {
        let Some(index) = index else {
            return;
        };
        if !marker_starts_comment_content(line, index) {
            return;
        }
        let parser = if self.script {
            parse_script_disable_comment
        } else {
            parse_eslint_disable_comment
        };
        let Some(directive) = line.get(index..).and_then(parser) else {
            return;
        };
        let text = line.get(index..).unwrap_or_default();
        let text = text.split_once("*/").map_or(text, |(comment, _)| comment);
        // Vize aliases suppress script findings without joining the existing
        // ESLint/Oxlint unused-directive reporting contract.
        let marker = (!text.starts_with("vize-")).then_some((
            line_start + index as u32,
            line_start + index as u32 + text.trim_end().len() as u32,
        ));
        // ESLint accepts region pragmas only in block comments; in a line
        // comment they are not directives and must not reach the template.
        let before = line.get(..index).unwrap_or_default();
        let in_line_comment = match (before.rfind("//"), before.rfind("/*")) {
            (Some(line_comment), Some(block_comment)) => line_comment > block_comment,
            (line_comment, _) => line_comment.is_some(),
        };
        if self.script
            && !in_line_comment
            && matches!(
                directive.kind,
                EslintDisableKind::Disable | EslintDisableKind::Enable
            )
        {
            let region = EslintDisableDirective {
                kind: directive.kind,
                rules: directive.rules.clone(),
            };
            self.file_regions.record_at(region, line_number, marker);
        }
        self.inline_suppressions
            .record_at(directive, line_number, marker);
    }

    fn scan_vize_directive(&mut self, line: &str, line_number: u32, index: Option<usize>) {
        let Some(index) = index else {
            return;
        };
        let Some(from_marker) = line.get(index..) else {
            return;
        };
        let content = from_marker
            .split_once("*/")
            .map_or(from_marker, |(content, _)| content);
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
}

fn has_directive_marker(source: &str) -> bool {
    source.contains("eslint-")
        || source.contains("oxlint-")
        || source.contains("vize-disable")
        || source.contains("vize-enable")
        || source.contains("@vize:")
}

fn marker_starts_comment_content(line: &str, index: usize) -> bool {
    let Some(before) = line.get(..index) else {
        return false;
    };
    let opener_end = [
        before.rfind("//").map(|at| at + 2),
        before.rfind("/*").map(|at| at + 2),
    ]
    .into_iter()
    .flatten()
    .max();
    let prefix = opener_end
        .and_then(|start| before.get(start..))
        .unwrap_or(before)
        .trim();
    prefix.is_empty() || prefix == "*"
}

fn range_contains(range: &DisabledRange, line: u32) -> bool {
    line >= range.start_line && range.end_line.is_none_or(|end| line <= end)
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
