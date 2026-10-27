//! Source-owned directives for diagnostics produced after a file lint pass.

use memchr::memchr_iter;
use vize_armature::Parser;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l0::directive::DirectiveSeverity;
use vize_l0::{Allocator, FxHashMap, FxHashSet, cstr};

use super::eslint_directive::{InlineSuppressionState, UnusedDirective};
use super::{DisabledRange, LintContext, SfcDirectiveState};
use crate::LintDiagnostic;
use crate::visitor::LintVisitor;

const UNUSED_RULE: &str = "vue/comment-directive";

/// Suppression and usage for project diagnostics with authored SFC ranges.
/// Callers apply configuration and source admission before querying this state.
pub struct ProjectLintDirectives {
    sfc: Option<SfcDirectiveState>,
    template: Option<TemplateDirectives>,
    source_len: u32,
    line_offsets: Vec<u32>,
}

struct TemplateDirectives {
    start: u32,
    end: u32,
    first_line: u32,
    suppressions: InlineSuppressionState,
    disabled_all: Vec<DisabledRange>,
    expected_error_lines: FxHashSet<u32>,
    severity_overrides: FxHashMap<u32, DirectiveSeverity>,
}

impl ProjectLintDirectives {
    /// Inspect the same comment grammar as the ordinary SFC lint pass.
    /// Strings, attribute values, custom blocks, and external templates cannot
    /// manufacture directives.
    pub fn from_sfc(source: &str, filename: &str) -> Option<Self> {
        if !source.contains("eslint-") && !source.contains("oxlint-") && !source.contains("@vize:")
        {
            return None;
        }
        let descriptor = parse_sfc(
            source,
            SfcParseOptions {
                filename: filename.into(),
                ..Default::default()
            },
        )
        .ok()?;
        let sfc = SfcDirectiveState::scan_if_present(&descriptor);
        let template = descriptor.template.as_ref().and_then(|template| {
            if template.src.is_some()
                || !template.lang.as_deref().is_none_or(|lang| lang == "html")
                || (!template.content.contains("eslint-")
                    && !template.content.contains("oxlint-")
                    && !template.content.contains("@vize:"))
            {
                return None;
            }
            let allocator = Allocator::with_capacity((template.content.len() * 4).max(64 * 1024));
            let (root, _) = Parser::new(&allocator, &template.content).parse();
            let mut context = LintContext::new(&allocator, &template.content, filename);
            LintVisitor::new(&mut context, &[], &[], false).prescan_suppression_directives(&root);
            let mut suppressions = context.inline_suppressions;
            suppressions.offset_markers(template.loc.start as u32);
            Some(TemplateDirectives {
                start: template.loc.start as u32,
                end: template.loc.end as u32,
                first_line: source_line(source, template.loc.start as u32),
                suppressions,
                disabled_all: context.disabled_all,
                expected_error_lines: context.expected_error_lines,
                severity_overrides: context.severity_overrides,
            })
        });
        Some(Self {
            sfc,
            template,
            source_len: source.len() as u32,
            line_offsets: std::iter::once(0)
                .chain(memchr_iter(b'\n', source.as_bytes()).map(|offset| (offset + 1) as u32))
                .collect(),
        })
    }

    /// Whether a known rule is disabled at its mapped authored source range.
    pub fn is_disabled_at(&self, rule: &str, start: u32) -> bool {
        if start >= self.source_len {
            return false;
        }
        let line = match self.line_offsets.binary_search(&start) {
            Ok(line) => (line + 1) as u32,
            Err(line) => line as u32,
        };
        self.sfc.as_ref().is_some_and(|state| {
            state.is_disabled_at(rule, line, start) || state.is_expected_at(line, start)
        }) || self.template.as_ref().is_some_and(|template| {
            template.start <= start && start < template.end && {
                let line = line - template.first_line + 1;
                template.disabled_all.iter().any(|range| {
                    line >= range.start_line && range.end_line.is_none_or(|end| line <= end)
                }) || template.suppressions.is_disabled_at(rule, line)
                    || template.expected_error_lines.contains(&line)
            }
        })
    }

    /// Apply the ordinary directive ordering after host rule configuration.
    /// The caller retains its diagnostic identity and authored range.
    pub fn admit(&self, rule: &str, diagnostic: &mut LintDiagnostic) -> bool {
        if self.is_disabled_at(rule, diagnostic.start) {
            return false;
        }
        let line = match self.line_offsets.binary_search(&diagnostic.start) {
            Ok(line) => (line + 1) as u32,
            Err(line) => line as u32,
        };
        let severity = self
            .sfc
            .as_ref()
            .and_then(|state| state.severity_at(line, diagnostic.start))
            .or_else(|| {
                self.template
                    .as_ref()
                    .filter(|template| {
                        template.start <= diagnostic.start && diagnostic.start < template.end
                    })
                    .and_then(|template| {
                        template
                            .severity_overrides
                            .get(&(line - template.first_line + 1))
                            .copied()
                    })
            });
        match severity {
            Some(DirectiveSeverity::Off) => false,
            Some(DirectiveSeverity::Warn) => {
                diagnostic.severity = crate::Severity::Warning;
                true
            }
            Some(DirectiveSeverity::Error) => {
                diagnostic.severity = crate::Severity::Error;
                true
            }
            None => true,
        }
    }

    /// Only registered, enabled project rule names participate in this pass.
    /// Ordinary rule names and foreign linter names retain their own accounting.
    pub fn unused_named(&self, is_active: impl Fn(&str) -> bool) -> Vec<LintDiagnostic> {
        let mut unused = self
            .sfc
            .as_ref()
            .map_or_else(Vec::new, |state| state.unused_named(&is_active));
        if let Some(template) = &self.template {
            unused.extend(
                template
                    .suppressions
                    .unused(&is_active)
                    .into_iter()
                    .filter(|directive| directive.rule.is_some()),
            );
        }
        unused.sort_unstable_by_key(|directive| (directive.start, directive.end));
        unused.dedup_by(|a, b| a.start == b.start && a.end == b.end && a.rule == b.rule);
        unused.into_iter().filter_map(unused_diagnostic).collect()
    }

    /// The ordinary template pass finishes before project diagnostics exist.
    /// Retire its blanket unused report only when this exact comment suppressed
    /// an admitted project diagnostic. Listed ordinary-rule reports are kept.
    pub fn reconcile_template_unused(&self, diagnostics: &mut Vec<LintDiagnostic>) {
        let Some(template) = &self.template else {
            return;
        };
        let used = template.suppressions.used_blanket_markers();
        diagnostics.retain(|diagnostic| {
            diagnostic.rule_name != UNUSED_RULE
                || !used.contains(&(diagnostic.start, diagnostic.end))
        });
    }
}

fn source_line(source: &str, offset: u32) -> u32 {
    source
        .get(..offset as usize)
        .unwrap_or(source)
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count() as u32
        + 1
}

fn unused_diagnostic(directive: UnusedDirective) -> Option<LintDiagnostic> {
    let rule = directive.rule?;
    Some(LintDiagnostic::error(
        UNUSED_RULE,
        cstr!("Unused eslint-disable directive (no problems were reported from '{rule}')."),
        directive.start,
        directive.end,
    ))
}

#[cfg(test)]
mod tests;
