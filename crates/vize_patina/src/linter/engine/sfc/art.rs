//! Template rules for Musea `<variant>` markup inside `<art>`.

use vize_armature::Parser;
use vize_atelier_sfc::SfcDescriptor;
use vize_croquis::Croquis;
use vize_l0::{Allocator, SourceRoot};

use super::super::{TemplateAnalysis, TemplateRuleEnv, offset_result};
use super::empty_lint_result;
use crate::diagnostic::LintDiagnostic;
use crate::linter::config::{LintResult, Linter};
use crate::linter::engine::tag_scan::{find_closing_tag, find_start_tag_end};

mod analysis;

impl Linter {
    pub(super) fn append_art_variant_template_diagnostics(
        &self,
        source: &str,
        filename: &str,
        result: &mut LintResult,
        shared_descriptor: Option<&SfcDescriptor<'_>>,
    ) {
        if !filename.ends_with(".art.vue") || !source_has_block(source, b"<art") {
            return;
        }
        let parsed;
        let descriptor = if let Some(descriptor) = shared_descriptor {
            descriptor
        } else {
            parsed = match crate::linter::script_rules::parse_sfc_for_lint(source, filename) {
                Ok(descriptor) => descriptor,
                Err(_) => return,
            };
            &parsed
        };
        let source = descriptor.source.as_ref();
        let mut summary = self
            .has_active_semantic_template_rules()
            .then(|| analysis::script_summary(self, descriptor));
        let mut extra = empty_lint_result(filename);
        for (start, end) in art_variant_ranges(source, descriptor) {
            let Some(inner) = source.get(start..end) else {
                continue;
            };
            let one = self.lint_art_variant(inner, filename, descriptor, summary.as_mut());
            extra = Self::merge_lint_results(extra, one);
        }
        if self.has_unused_bindings_demand()
            && let Some(summary) = summary.as_ref()
        {
            let one = analysis::report_unused(self, filename, descriptor, summary);
            extra = Self::merge_lint_results(extra, one);
        }
        if extra.diagnostics.is_empty() {
            return;
        }
        let current = std::mem::replace(result, empty_lint_result(filename));
        *result = Self::merge_lint_results(current, extra);
    }

    fn lint_art_variant(
        &self,
        source: &str,
        filename: &str,
        descriptor: &SfcDescriptor<'_>,
        summary: Option<&mut Croquis>,
    ) -> LintResult {
        let offset = SourceRoot::new(descriptor.source.as_ref())
            .ok()
            .and_then(|root| root.whole_block().offset_of(source));
        let Some(offset) = offset else {
            let mut refused = empty_lint_result(filename);
            refused.diagnostics.push(LintDiagnostic::error(
                "parser/sfc",
                "Art variant cannot be located in the original SFC source",
                0,
                0,
            ));
            refused.error_count = 1;
            return refused;
        };
        let allocator = Allocator::with_capacity((source.len() * 4).max(self.initial_capacity));
        let (root, errors) = Parser::new(&allocator, source).parse();
        let fatal = Self::has_fatal_template_parse_errors(&errors);
        let analysis = summary
            .as_deref()
            .filter(|_| !fatal)
            .map(|script| analysis::variant(&root, script, self.has_unused_bindings_demand()));
        let mut parsed = Self::template_parse_lint_result(filename, source.len(), &errors);
        offset_result(&mut parsed, offset);
        let linted = self.lint_template_root(
            &allocator,
            source,
            filename,
            &root,
            if fatal {
                TemplateAnalysis::Disabled
            } else if let Some(analysis) = analysis.as_ref() {
                TemplateAnalysis::Precomputed(analysis)
            } else {
                TemplateAnalysis::Lazy
            },
            TemplateRuleEnv {
                sfc_descriptor: Some(descriptor),
                art_script_analysis: summary.as_deref().filter(|_| !fatal),
                dialect: vize_l0::dialect::VueDialect::Vue,
                facade_rules: super::facade::RULES,
            },
        );
        if self.has_unused_bindings_demand()
            && let Some(summary) = summary
        {
            if let Some(analysis) = analysis {
                summary
                    .unused_bindings
                    .retain(|name| analysis.unused_bindings.contains(name));
            } else {
                // An invalid variant may contain reads we cannot establish.
                summary.unused_bindings.clear();
            }
        }
        Self::merge_lint_results(parsed, linted)
    }
}

fn art_variant_ranges(source: &str, descriptor: &SfcDescriptor<'_>) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    for block in &descriptor.custom_blocks {
        if block.block_type.as_ref() != "art" {
            continue;
        }
        let Some(body) = source.get(block.loc.start..block.loc.end) else {
            continue;
        };
        let origin = block.loc.start;
        for (start, end) in variant_inner_ranges(body) {
            ranges.push((origin + start, origin + end));
        }
    }
    ranges
}

/// Inner markup of each non-empty `<variant>`, as ranges within `art_content`.
fn variant_inner_ranges(art_content: &str) -> Vec<(usize, usize)> {
    let bytes = art_content.as_bytes();
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = bytes
        .get(cursor..)
        .and_then(|rest| memchr::memmem::find(rest, b"<variant"))
    {
        let start = cursor + relative;
        let after_name = start + "<variant".len();
        if !is_block_tag_boundary(bytes.get(after_name).copied()) {
            cursor = after_name;
            continue;
        }
        let Some(tag_end) = find_start_tag_end(bytes, start) else {
            break;
        };
        if is_self_closing(bytes, tag_end) {
            cursor = tag_end + 1;
            continue;
        }
        let inner_start = tag_end + 1;
        let Some(close) = find_closing_tag(bytes, b"variant", inner_start) else {
            break;
        };
        if art_content
            .get(inner_start..close)
            .is_some_and(|inner| !inner.trim().is_empty())
        {
            ranges.push((inner_start, close));
        }
        cursor = find_start_tag_end(bytes, close)
            .map(|end| end + 1)
            .unwrap_or(close + "</variant>".len());
    }
    ranges
}

fn source_has_block(source: &str, tag: &[u8]) -> bool {
    let bytes = source.as_bytes();
    let mut cursor = 0;
    while let Some(relative) = bytes
        .get(cursor..)
        .and_then(|rest| memchr::memmem::find(rest, tag))
    {
        let start = cursor + relative;
        let next = bytes.get(start + tag.len()).copied();
        if is_block_tag_boundary(next) {
            return true;
        }
        cursor = start + tag.len();
    }
    false
}

fn is_block_tag_boundary(next: Option<u8>) -> bool {
    matches!(
        next,
        Some(b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/') | None
    )
}

fn is_self_closing(bytes: &[u8], tag_end: usize) -> bool {
    let mut index = tag_end;
    while index > 0 {
        index -= 1;
        match bytes.get(index) {
            Some(b' ' | b'\t' | b'\n' | b'\r') => continue,
            Some(b'/') => return true,
            _ => return false,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RuleRegistry;

    #[test]
    fn original_fragment_admission_refuses_equivalent_foreign_buffers_before_parse() {
        let source = "<art><variant name='One'><div/></variant></art>";
        let descriptor =
            crate::linter::script_rules::parse_sfc_for_lint(source, "Gallery.art.vue").unwrap();
        let original = descriptor.source.as_ref();
        let start = original.find("<div/>").unwrap();
        let fragment = &original[start..start + "<div/>".len()];
        let linter = Linter::with_registry(RuleRegistry::new());
        let clean = empty_lint_result("Gallery.art.vue");
        let actual = linter.lint_art_variant(fragment, "Gallery.art.vue", &descriptor, None);
        assert_eq!(format!("{actual:#?}"), format!("{clean:#?}"));
        let foreign = fragment.to_owned();
        assert_eq!(foreign, fragment);
        let mut refused = empty_lint_result("Gallery.art.vue");
        refused.diagnostics.push(LintDiagnostic::error(
            "parser/sfc",
            "Art variant cannot be located in the original SFC source",
            0,
            0,
        ));
        refused.error_count = 1;
        let actual = linter.lint_art_variant(&foreign, "Gallery.art.vue", &descriptor, None);
        assert_eq!(format!("{actual:#?}"), format!("{refused:#?}"));
    }
}
