//! Lint execution engine.
//!
//! Contains the core linting methods: single-file template linting,
//! full SFC linting with template extraction, and batch file processing.

use crate::{context::LintContext, diagnostic::LintSummary, visitor::LintVisitor};
use vize_armature::Parser;
use vize_carton::profile;
use vize_carton::Allocator;
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_croquis::{Analyzer, AnalyzerOptions, Croquis};
use vize_relief::ast::RootNode;

use super::config::{LintResult, Linter};

const SEMANTIC_TEMPLATE_RULES: &[&str] = &[
    "vue/no-unused-vars",
    "vue/no-undefined-refs",
    "vue/no-mutating-props",
    "a11y/no-refer-to-non-existent-id",
];

pub(crate) fn analyze_descriptor_for_lint(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
) -> Croquis {
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::for_lint());

    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        let generic = script_setup
            .attrs
            .get("generic")
            .map(|value| value.as_ref());
        analyzer.analyze_script_setup_with_generic(script_setup.content.as_ref(), generic);
    } else if let Some(script) = descriptor.script.as_ref() {
        analyzer.analyze_script_plain(script.content.as_ref());
    }

    if let Some(root) = template_ast {
        analyzer.analyze_template(root);
    }

    analyzer.finish()
}

impl Linter {
    fn lint_sfc_level(&self, source: &str, filename: &str) -> LintResult {
        let capacity = (source.len() * 2).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);
        let mut ctx = LintContext::with_locale(&allocator, source, filename, self.locale);
        ctx.set_enabled_rules(self.enabled_rules.clone());
        ctx.set_help_level(self.help_level);

        for rule in self.registry.rules() {
            ctx.current_rule = rule.meta().name;
            profile!("patina.sfc.rule.run_on_sfc", rule.run_on_sfc(&mut ctx));
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

    fn offset_result(result: &mut LintResult, byte_offset: u32) {
        if byte_offset == 0 {
            return;
        }

        for diag in &mut result.diagnostics {
            diag.start += byte_offset;
            diag.end += byte_offset;
            for label in &mut diag.labels {
                label.start += byte_offset;
                label.end += byte_offset;
            }
        }
    }

    fn has_active_semantic_template_rules(&self) -> bool {
        SEMANTIC_TEMPLATE_RULES
            .iter()
            .copied()
            .any(|rule_name| self.registry.has_rule(rule_name) && self.is_rule_enabled(rule_name))
    }

    fn run_template_rules<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        root: &RootNode<'a>,
        analysis: Option<&'a Croquis>,
    ) -> LintResult {
        let mut ctx = LintContext::with_locale(allocator, source, filename, self.locale);
        ctx.set_enabled_rules(self.enabled_rules.clone());
        ctx.set_help_level(self.help_level);
        let has_analysis = analysis.is_some();
        if let Some(analysis) = analysis {
            ctx.set_analysis(analysis);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if has_analysis && super::native_type_aware::has_active_type_aware_rules(self) {
            ctx.set_analysis_excluded_rules(super::native_type_aware::TYPE_AWARE_RULES);
        }

        let mut visitor = LintVisitor::new(&mut ctx, self.registry.rules());
        profile!("patina.template.visit", visitor.visit_root(root));

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

    fn lint_template_root<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        root: &RootNode<'a>,
        analysis: Option<&'a Croquis>,
    ) -> LintResult {
        if !self.has_active_semantic_template_rules() {
            return self.run_template_rules(allocator, source, filename, root, None);
        }
        let owned_analysis;
        let analysis = if let Some(analysis) = analysis {
            analysis
        } else {
            owned_analysis = profile!("patina.template.croquis", {
                let mut analyzer = Analyzer::with_options(AnalyzerOptions::for_lint());
                analyzer.analyze_template(root);
                analyzer.finish()
            });
            &owned_analysis
        };

        self.run_template_rules(allocator, source, filename, root, Some(analysis))
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
        let (root, _parse_errors) = profile!("patina.template.parse", parser.parse());

        self.lint_template_root(allocator, source, filename, &root, None)
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

    pub(crate) fn lint_sfc_template_root<'a>(
        &self,
        filename: &str,
        template_content: &'a str,
        template_offset: u32,
        allocator: &'a Allocator,
        root: &RootNode<'a>,
        analysis: Option<&'a Croquis>,
    ) -> LintResult {
        let mut result =
            self.lint_template_root(allocator, template_content, filename, root, analysis);
        Self::offset_result(&mut result, template_offset);
        result
    }

    pub(crate) fn lint_sfc_template_with_descriptor<'a>(
        &self,
        filename: &str,
        descriptor: &vize_atelier_sfc::SfcDescriptor<'a>,
    ) -> LintResult {
        let Some(template) = descriptor.template.as_ref() else {
            return LintResult {
                filename: filename.to_compact_string(),
                diagnostics: Vec::new(),
                error_count: 0,
                warning_count: 0,
            };
        };

        let allocator =
            Allocator::with_capacity((template.content.len() * 4).max(self.initial_capacity));
        let parser = Parser::new(allocator.as_bump(), &template.content);
        let (root, _parse_errors) =
            profile!("patina.sfc.descriptor.template_parse", parser.parse());

        let analysis = if self.has_active_semantic_template_rules() {
            Some(profile!(
                "patina.sfc.descriptor.croquis",
                analyze_descriptor_for_lint(descriptor, Some(&root))
            ))
        } else {
            None
        };

        self.lint_sfc_template_root(
            filename,
            &template.content,
            template.loc.start as u32,
            &allocator,
            &root,
            analysis.as_ref(),
        )
    }

    /// Lint a full Vue SFC file.
    ///
    /// Uses ultra-fast template extraction optimized for linting.
    #[inline]
    pub fn lint_sfc(&self, source: &str, filename: &str) -> LintResult {
        let sfc_result = profile!(
            "patina.sfc.level_rules",
            self.lint_sfc_level(source, filename)
        );

        #[cfg(not(target_arch = "wasm32"))]
        if super::native_type_aware::has_active_type_aware_rules(self) {
            let template_result = profile!(
                "patina.type_aware.lint_sfc_with_corsa",
                super::native_type_aware::lint_sfc_with_corsa(self, source, filename)
            );
            return Self::merge_lint_results(template_result, sfc_result);
        }

        if super::script_rules::has_active_builtin_script_rules(self)
            || self.has_active_semantic_template_rules()
        {
            let template_result = match profile!(
                "patina.sfc.parse_for_script_rules",
                super::script_rules::parse_sfc_for_lint(source, filename)
            ) {
                Ok(descriptor) => {
                    profile!("patina.sfc.descriptor_rules", {
                        super::script_rules::lint_with_descriptor(self, filename, &descriptor)
                    })
                }
                Err(_) => {
                    if let Some((content, byte_offset)) = profile!(
                        "patina.template.extract_fast",
                        extract_template_fast(source)
                    ) {
                        let mut fallback = self.lint_template(&content, filename);
                        Self::offset_result(&mut fallback, byte_offset);
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
        let (content, byte_offset) = match profile!(
            "patina.template.extract_fast",
            extract_template_fast(source)
        ) {
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
        Self::offset_result(&mut result, byte_offset);

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
