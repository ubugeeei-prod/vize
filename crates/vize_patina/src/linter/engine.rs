//! Lint execution engine.
//!
//! Contains the core linting methods: single-file template linting,
//! full SFC linting with template extraction, and batch file processing.

use crate::{context::LintContext, diagnostic::LintSummary, visitor::LintVisitor};
use vize_armature::Parser;
use vize_carton::Allocator;
use vize_carton::String;
use vize_carton::ToCompactString;

use super::config::{LintResult, Linter};

impl Linter {
    fn lint_sfc_level(&self, source: &str, filename: &str) -> LintResult {
        let capacity = (source.len() * 2).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);
        let mut ctx = LintContext::with_locale(&allocator, source, filename, self.locale);
        ctx.set_enabled_rules(self.enabled_rules.clone());
        ctx.set_help_level(self.help_level);

        for rule in self.registry.rules() {
            ctx.current_rule = rule.meta().name;
            rule.run_on_sfc(&mut ctx);
        }

        let error_count = ctx.error_count();
        let warning_count = ctx.warning_count();
        let diagnostics = ctx.into_diagnostics();

        LintResult {
            filename: filename.to_compact_string(),
            diagnostics,
            error_count,
            warning_count,
        }
    }

    fn merge_lint_results(
        mut template_result: LintResult,
        mut sfc_result: LintResult,
    ) -> LintResult {
        if sfc_result.diagnostics.is_empty() {
            return template_result;
        }

        if template_result.diagnostics.is_empty() {
            return sfc_result;
        }

        template_result.error_count += sfc_result.error_count;
        template_result.warning_count += sfc_result.warning_count;
        template_result
            .diagnostics
            .append(&mut sfc_result.diagnostics);
        template_result
            .diagnostics
            .sort_unstable_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
        template_result
    }

    /// Lint a Vue template source.
    #[inline]
    pub fn lint_template(&self, source: &str, filename: &str) -> LintResult {
        // Create allocator sized for source (rough heuristic: 4x source size)
        let capacity = (source.len() * 4).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);

        self.lint_template_with_allocator(&allocator, source, filename)
    }

    /// Lint a Vue template with a provided allocator (for reuse).
    pub fn lint_template_with_allocator(
        &self,
        allocator: &Allocator,
        source: &str,
        filename: &str,
    ) -> LintResult {
        // Parse the template
        let parser = Parser::new(allocator.as_bump(), source);
        let (root, _parse_errors) = parser.parse();

        // Create lint context with locale, help level, and enabled rules filter
        let mut ctx = LintContext::with_locale(allocator, source, filename, self.locale);
        ctx.set_enabled_rules(self.enabled_rules.clone());
        ctx.set_help_level(self.help_level);

        // Run visitor with all rules (filtering happens in context)
        let mut visitor = LintVisitor::new(&mut ctx, self.registry.rules());
        visitor.visit_root(&root);

        // Collect results (error/warning counts are cached)
        let error_count = ctx.error_count();
        let warning_count = ctx.warning_count();
        let diagnostics = ctx.into_diagnostics();

        LintResult {
            filename: filename.to_compact_string(),
            diagnostics,
            error_count,
            warning_count,
        }
    }

    /// Lint multiple files and aggregate results.
    pub fn lint_files(&self, files: &[(String, String)]) -> (Vec<LintResult>, LintSummary) {
        let mut results = Vec::with_capacity(files.len());
        let mut summary = LintSummary::default();

        // Reuse allocator across files for better memory efficiency
        let mut allocator = Allocator::with_capacity(self.initial_capacity);

        for (filename, source) in files {
            let result = self.lint_template_with_allocator(&allocator, source, filename);
            summary.error_count += result.error_count;
            summary.warning_count += result.warning_count;
            results.push(result);

            // Reset allocator for next file
            allocator.reset();
        }

        summary.file_count = files.len();
        (results, summary)
    }

    /// Lint a full Vue SFC file.
    ///
    /// Uses ultra-fast template extraction optimized for linting.
    #[inline]
    pub fn lint_sfc(&self, source: &str, filename: &str) -> LintResult {
        let sfc_result = self.lint_sfc_level(source, filename);

        #[cfg(not(target_arch = "wasm32"))]
        if super::native_type_aware::has_active_type_aware_rules(self) {
            let template_result =
                super::native_type_aware::lint_sfc_with_tsgo(self, source, filename);
            return Self::merge_lint_results(template_result, sfc_result);
        }

        if super::script_rules::has_active_builtin_script_rules(self) {
            let template_result = match super::script_rules::parse_sfc_for_lint(source, filename) {
                Ok(descriptor) => {
                    super::script_rules::lint_with_descriptor(self, filename, &descriptor)
                }
                Err(_) => {
                    if let Some((content, byte_offset)) = extract_template_fast(source) {
                        let mut fallback = self.lint_template(&content, filename);
                        if byte_offset > 0 {
                            for diag in &mut fallback.diagnostics {
                                diag.start += byte_offset;
                                diag.end += byte_offset;
                                for label in &mut diag.labels {
                                    label.start += byte_offset;
                                    label.end += byte_offset;
                                }
                            }
                        }
                        fallback
                    } else {
                        LintResult {
                            filename: filename.to_compact_string(),
                            diagnostics: Vec::new(),
                            error_count: 0,
                            warning_count: 0,
                        }
                    }
                }
            };
            return Self::merge_lint_results(template_result, sfc_result);
        }

        // Fast template extraction using memchr
        let (content, byte_offset) = match extract_template_fast(source) {
            Some(r) => r,
            None => {
                if sfc_result.has_diagnostics() {
                    return sfc_result;
                }
                return LintResult {
                    filename: filename.to_compact_string(),
                    diagnostics: Vec::new(),
                    error_count: 0,
                    warning_count: 0,
                };
            }
        };

        let mut result = self.lint_template(&content, filename);

        // Adjust byte offsets in diagnostics to match original file positions
        if byte_offset > 0 {
            for diag in &mut result.diagnostics {
                diag.start += byte_offset;
                diag.end += byte_offset;
                for label in &mut diag.labels {
                    label.start += byte_offset;
                    label.end += byte_offset;
                }
            }
        }

        Self::merge_lint_results(result, sfc_result)
    }
}

/// Ultra-fast template extraction using memchr for SIMD-accelerated search.
#[inline]
pub(crate) fn extract_template_fast(source: &str) -> Option<(String, u32)> {
    let bytes = source.as_bytes();

    // Find <template using memchr (SIMD accelerated)
    let start_pattern = b"<template";

    // Find first <template occurrence
    let start_idx = memchr::memmem::find(bytes, start_pattern)?;

    // Find > after <template (end of opening tag)
    let tag_end = memchr::memchr(b'>', &bytes[start_idx..])? + start_idx;
    let content_start = tag_end + 1;

    // Find matching </template> - handle nesting with simple depth tracking
    let mut depth = 1u32;
    let mut pos = content_start;

    while pos < bytes.len() && depth > 0 {
        // Find next < character
        let next_lt = match memchr::memchr(b'<', &bytes[pos..]) {
            Some(p) => pos + p,
            None => break,
        };

        // Check if it's <template or </template
        if bytes.len() > next_lt + 9 && &bytes[next_lt..next_lt + 9] == b"<template" {
            // Check if self-closing
            if let Some(gt) = memchr::memchr(b'>', &bytes[next_lt..]) {
                let tag_end_pos = next_lt + gt;
                if tag_end_pos > 0 && bytes[tag_end_pos - 1] != b'/' {
                    depth += 1;
                }
                pos = tag_end_pos + 1;
            } else {
                pos = next_lt + 9;
            }
        } else if bytes.len() > next_lt + 11 && &bytes[next_lt..next_lt + 11] == b"</template>" {
            depth -= 1;
            if depth == 0 {
                let content = std::str::from_utf8(&bytes[content_start..next_lt]).ok()?;
                return Some((content.to_compact_string(), content_start as u32));
            }
            pos = next_lt + 11;
        } else {
            pos = next_lt + 1;
        }
    }

    None
}
