//! Lint execution engine.
//!
//! Contains the core linting methods: single-file template linting,
//! full SFC linting with template extraction, and batch file processing.

use crate::{
    context::LintContext,
    diagnostic::{LintDiagnostic, LintSummary, Severity},
    preset::LintPreset,
    visitor::LintVisitor,
};
use vize_armature::Parser;
use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor};
use vize_atelier_sfc::{SfcError, SfcParseOptions, parse_sfc};
use vize_carton::Allocator;
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_carton::profile;
use vize_croquis::{Analyzer, Croquis};
use vize_relief::{CompilerError, ErrorCode, ast::RootNode};

use super::config::{LintResult, Linter};

const TEMPLATE_PARSE_RULE: &str = "parser/template";
const SFC_PARSE_RULE: &str = "parser/sfc";
const INVALID_SELF_CLOSING_HTML_MESSAGE: &str =
    "Invalid self-closing syntax on non-void HTML element";

pub(crate) enum TemplateAnalysis<'a> {
    Disabled,
    Precomputed(&'a Croquis),
    Lazy,
}

pub(crate) struct SfcTemplateLintInput<'a> {
    pub filename: &'a str,
    pub template: &'a vize_atelier_sfc::SfcTemplateBlock<'a>,
    pub allocator: &'a Allocator,
    pub root: &'a RootNode<'a>,
    pub descriptor: Option<&'a vize_atelier_sfc::SfcDescriptor<'a>>,
    pub analysis: TemplateAnalysis<'a>,
}

fn template_parse_diagnostic(parse_error: &CompilerError, source_len: usize) -> LintDiagnostic {
    let (start, end) = template_parse_span(parse_error, source_len);
    if parse_error.is_recoverable() {
        LintDiagnostic::warn(TEMPLATE_PARSE_RULE, parse_error.message.clone(), start, end)
    } else {
        LintDiagnostic::error(TEMPLATE_PARSE_RULE, parse_error.message.clone(), start, end)
    }
}

fn should_report_template_parse_diagnostic(parse_error: &CompilerError) -> bool {
    // Standard mode rewrites invalid HTML self-closing syntax as a compatibility
    // warning. Lint surfaces parser errors, but should not turn this rewrite
    // notice into a project warning budget failure.
    !(parse_error.code == ErrorCode::ExtendPoint
        && parse_error
            .message
            .starts_with(INVALID_SELF_CLOSING_HTML_MESSAGE))
}

fn template_parse_span(parse_error: &CompilerError, source_len: usize) -> (u32, u32) {
    if source_len == 0 {
        return (0, 0);
    }

    let source_len = source_len as u32;
    let (raw_start, raw_end) = parse_error
        .loc
        .as_ref()
        .map(|loc| (loc.start.offset, loc.end.offset))
        .unwrap_or((0, 0));
    let start = raw_start.min(source_len.saturating_sub(1));
    let end = raw_end
        .max(raw_start.saturating_add(1))
        .max(start.saturating_add(1))
        .min(source_len);

    (start, end)
}

fn sfc_parse_span(parse_error: &SfcError, source_len: usize) -> (u32, u32) {
    if source_len == 0 {
        return (0, 0);
    }

    let source_len = source_len as u32;
    let (raw_start, raw_end) = parse_error
        .loc
        .as_ref()
        .map(|loc| (loc.start as u32, loc.end as u32))
        .unwrap_or((0, 0));
    let start = raw_start.min(source_len.saturating_sub(1));
    let end = raw_end
        .max(raw_start.saturating_add(1))
        .max(start.saturating_add(1))
        .min(source_len);

    (start, end)
}

const SEMANTIC_TEMPLATE_RULES: &[&str] = &[
    "vue/no-unused-vars",
    "vue/no-unused-components",
    "vue/require-component-registration",
    "vue/no-undefined-refs",
    "vue/no-mutating-props",
    "a11y/no-refer-to-non-existent-id",
];

const SHARED_SFC_DESCRIPTOR_RULES: &[&str] = &[
    "vue/sfc-element-order",
    "vue/require-scoped-style",
    "vue/single-style-block",
    "ecosystem/void-link-require-href",
    "ecosystem/void-link-valid-method",
];

pub(crate) fn analyze_descriptor_for_lint(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    template_ast: Option<&RootNode<'_>>,
) -> Croquis {
    analyze_sfc_descriptor(descriptor, template_ast, SfcCroquisOptions::for_lint())
}

impl Linter {
    fn template_rule_count_for_source(
        &self,
        template_source: &str,
        sfc_source: Option<&str>,
    ) -> usize {
        if !matches!(self.preset, Some(LintPreset::Ecosystem))
            || self.enabled_rules.is_some()
            || !self.disabled_rules.is_empty()
            || source_may_contain_ecosystem_template_rule(template_source, sfc_source)
        {
            return self.registry.rules().len();
        }

        self.registry
            .rules()
            .len()
            .saturating_sub(crate::rules::ecosystem::TEMPLATE_RULE_COUNT)
    }

    fn lint_sfc_level<'a>(
        &self,
        source: &'a str,
        filename: &str,
        shared_descriptor: Option<&'a vize_atelier_sfc::SfcDescriptor<'a>>,
    ) -> LintResult {
        let capacity = (source.len() * 2).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);
        let mut ctx = LintContext::with_locale(&allocator, source, filename, self.locale);
        ctx.set_enabled_rules(self.enabled_rules.clone());
        ctx.set_help_level(self.help_level);

        // SFC-level rules are uncommon but expensive when each one reparses the
        // file. Reuse the descriptor produced by the main lint pipeline whenever
        // available, and only parse lazily when a caller enters this path without
        // a shared descriptor.
        let owned_descriptor;
        let shared_descriptor = if !self.has_active_shared_sfc_descriptor_rules() {
            None
        } else if let Some(descriptor) = shared_descriptor {
            Some(descriptor)
        } else {
            owned_descriptor = profile!(
                "patina.sfc.level_rules.parse_sfc",
                parse_sfc(
                    source,
                    SfcParseOptions {
                        filename: filename.into(),
                        ..Default::default()
                    },
                )
                .ok()
            );
            owned_descriptor.as_ref()
        };

        if let Some(descriptor) = shared_descriptor {
            ctx.set_sfc_descriptor(descriptor);
        }

        profile!("patina.sfc.rules.run_on_sfc", {
            for (rule, rule_name) in self
                .registry
                .rules()
                .iter()
                .zip(self.rule_names().iter().copied())
            {
                ctx.current_rule = rule_name;
                rule.run_on_sfc(&mut ctx);
            }
        });

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

    pub(crate) fn merge_lint_results(
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

    pub(crate) fn offset_result(result: &mut LintResult, byte_offset: u32) {
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

    pub(crate) fn template_parse_lint_result(
        filename: &str,
        source_len: usize,
        parse_errors: &[CompilerError],
    ) -> LintResult {
        let mut diagnostics = Vec::with_capacity(parse_errors.len());
        let mut error_count = 0;
        let mut warning_count = 0;

        for parse_error in parse_errors {
            if !should_report_template_parse_diagnostic(parse_error) {
                continue;
            }
            let diagnostic = template_parse_diagnostic(parse_error, source_len);
            match diagnostic.severity {
                Severity::Error => error_count += 1,
                Severity::Warning => warning_count += 1,
            }
            diagnostics.push(diagnostic);
        }

        LintResult {
            filename: filename.to_compact_string(),
            diagnostics,
            error_count,
            warning_count,
        }
    }

    pub(crate) fn has_fatal_template_parse_errors(parse_errors: &[CompilerError]) -> bool {
        parse_errors.iter().any(|error| !error.is_recoverable())
    }

    fn sfc_parse_lint_result(
        filename: &str,
        source_len: usize,
        parse_error: &SfcError,
    ) -> LintResult {
        let (start, end) = sfc_parse_span(parse_error, source_len);
        LintResult {
            filename: filename.to_compact_string(),
            diagnostics: vec![LintDiagnostic::error(
                SFC_PARSE_RULE,
                parse_error.message.clone(),
                start,
                end,
            )],
            error_count: 1,
            warning_count: 0,
        }
    }

    fn has_active_semantic_template_rules(&self) -> bool {
        SEMANTIC_TEMPLATE_RULES
            .iter()
            .copied()
            .any(|rule_name| self.registry.has_rule(rule_name) && self.is_rule_enabled(rule_name))
    }

    fn has_active_shared_sfc_descriptor_rules(&self) -> bool {
        SHARED_SFC_DESCRIPTOR_RULES
            .iter()
            .copied()
            .any(|rule_name| self.registry.has_rule(rule_name) && self.is_rule_enabled(rule_name))
    }

    fn needs_sfc_descriptor_for_lint(&self) -> bool {
        // This gate decides whether the outer SFC lint path should pay the parse
        // cost up front. Keep every consumer that can reuse descriptor metadata
        // listed here; otherwise a rule may quietly fall back to its own parse and
        // reintroduce per-rule work on large files.
        self.has_active_shared_sfc_descriptor_rules()
            || super::script_rules::has_active_builtin_script_rules(self)
            || self.has_active_semantic_template_rules()
            || {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    super::native_type_aware::has_active_type_aware_rules(self)
                }
                #[cfg(target_arch = "wasm32")]
                {
                    false
                }
            }
    }

    fn run_template_rules<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        root: &RootNode<'a>,
        sfc_descriptor: Option<&'a vize_atelier_sfc::SfcDescriptor<'a>>,
        analysis: Option<&'a Croquis>,
    ) -> LintResult {
        let mut ctx = LintContext::with_locale(allocator, source, filename, self.locale);
        ctx.set_enabled_rules(self.enabled_rules.clone());
        ctx.set_config_disabled_rules(self.disabled_rules.clone());
        ctx.set_help_level(self.help_level);
        if let Some(descriptor) = sfc_descriptor {
            ctx.set_sfc_descriptor(descriptor);
        }
        #[cfg(not(target_arch = "wasm32"))]
        #[cfg(not(target_arch = "wasm32"))]
        let has_analysis = analysis.is_some();
        if let Some(analysis) = analysis {
            ctx.set_analysis(analysis);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if has_analysis && super::native_type_aware::has_active_type_aware_rules(self) {
            ctx.set_analysis_excluded_rules(super::native_type_aware::TYPE_AWARE_RULES);
        }

        let rule_count = self.template_rule_count_for_source(
            source,
            sfc_descriptor.map(|descriptor| descriptor.source.as_ref()),
        );
        let mut visitor = LintVisitor::new(
            &mut ctx,
            &self.registry.rules()[..rule_count],
            &self.rule_names()[..rule_count],
            self.registry.has_exit_element_rules(),
        );
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
        sfc_descriptor: Option<&'a vize_atelier_sfc::SfcDescriptor<'a>>,
        analysis: TemplateAnalysis<'a>,
    ) -> LintResult {
        if matches!(analysis, TemplateAnalysis::Disabled)
            || !self.has_active_semantic_template_rules()
        {
            return self.run_template_rules(
                allocator,
                source,
                filename,
                root,
                sfc_descriptor,
                None,
            );
        }
        let owned_analysis;
        let analysis = match analysis {
            TemplateAnalysis::Disabled => unreachable!(),
            TemplateAnalysis::Precomputed(analysis) => analysis,
            TemplateAnalysis::Lazy => {
                owned_analysis = profile!("patina.template.croquis", {
                    let mut analyzer = Analyzer::for_lint();
                    analyzer.analyze_template(root);
                    analyzer.finish()
                });
                &owned_analysis
            }
        };

        self.run_template_rules(
            allocator,
            source,
            filename,
            root,
            sfc_descriptor,
            Some(analysis),
        )
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
        self.lint_template_with_allocator_config(allocator, source, filename, true, true)
    }

    #[cfg(test)]
    pub(crate) fn lint_template_rules_only(&self, source: &str, filename: &str) -> LintResult {
        let capacity = (source.len() * 4).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);

        self.lint_template_with_allocator_config(&allocator, source, filename, false, true)
    }

    fn lint_template_with_allocator_config(
        &self,
        allocator: &Allocator,
        source: &str,
        filename: &str,
        report_parse_errors: bool,
        gate_semantic_on_fatal_parse: bool,
    ) -> LintResult {
        // Parse the template
        let parser = Parser::new(allocator.as_bump(), source);
        let (root, parse_errors) = profile!("patina.template.parse", parser.parse());
        let has_fatal_parse_errors = Self::has_fatal_template_parse_errors(&parse_errors);

        let parse_result = Self::template_parse_lint_result(filename, source.len(), &parse_errors);
        let lint_result = self.lint_template_root(
            allocator,
            source,
            filename,
            &root,
            None,
            if gate_semantic_on_fatal_parse && has_fatal_parse_errors {
                TemplateAnalysis::Disabled
            } else {
                TemplateAnalysis::Lazy
            },
        );

        if report_parse_errors {
            Self::merge_lint_results(parse_result, lint_result)
        } else {
            lint_result
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

    pub(crate) fn lint_sfc_template_root<'a>(&self, input: SfcTemplateLintInput<'a>) -> LintResult {
        let mut result = self.lint_template_root(
            input.allocator,
            &input.template.content,
            input.filename,
            input.root,
            input.descriptor,
            input.analysis,
        );
        Self::offset_result(&mut result, input.template.loc.start as u32);
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
        let (root, parse_errors) = profile!("patina.sfc.descriptor.template_parse", parser.parse());
        let has_fatal_parse_errors = Self::has_fatal_template_parse_errors(&parse_errors);

        let analysis = if !has_fatal_parse_errors && self.has_active_semantic_template_rules() {
            Some(profile!(
                "patina.sfc.descriptor.croquis",
                analyze_descriptor_for_lint(descriptor, Some(&root))
            ))
        } else {
            None
        };

        let mut parse_result =
            Self::template_parse_lint_result(filename, template.content.len(), &parse_errors);
        Self::offset_result(&mut parse_result, template.loc.start as u32);
        let lint_result = self.lint_sfc_template_root(SfcTemplateLintInput {
            filename,
            template,
            allocator: &allocator,
            root: &root,
            descriptor: Some(descriptor),
            analysis: if has_fatal_parse_errors {
                TemplateAnalysis::Disabled
            } else if let Some(analysis) = analysis.as_ref() {
                TemplateAnalysis::Precomputed(analysis)
            } else {
                TemplateAnalysis::Lazy
            },
        });

        Self::merge_lint_results(parse_result, lint_result)
    }

    /// Lint a full Vue SFC file.
    ///
    /// Uses ultra-fast template extraction optimized for linting.
    #[inline]
    pub fn lint_sfc(&self, source: &str, filename: &str) -> LintResult {
        let shared_descriptor_result = if self.needs_sfc_descriptor_for_lint() {
            profile!(
                "patina.sfc.shared_parse_sfc",
                Some(super::script_rules::parse_sfc_for_lint(source, filename))
            )
        } else {
            None
        };
        let sfc_parse_result = shared_descriptor_result
            .as_ref()
            .and_then(|result| result.as_ref().err())
            .map(|parse_error| Self::sfc_parse_lint_result(filename, source.len(), parse_error));
        let shared_descriptor = shared_descriptor_result
            .as_ref()
            .and_then(|result| result.as_ref().ok());

        let sfc_result = sfc_parse_result.unwrap_or_else(|| {
            profile!(
                "patina.sfc.level_rules",
                self.lint_sfc_level(source, filename, shared_descriptor)
            )
        });

        #[cfg(not(target_arch = "wasm32"))]
        if super::native_type_aware::has_active_type_aware_rules(self) {
            let template_result = profile!(
                "patina.type_aware.lint_sfc_with_corsa",
                super::native_type_aware::lint_sfc_with_corsa_descriptor(
                    self,
                    source,
                    filename,
                    shared_descriptor,
                )
            );
            return Self::merge_lint_results(template_result, sfc_result);
        }

        if super::script_rules::has_active_builtin_script_rules(self)
            || self.has_active_semantic_template_rules()
            || self.has_active_shared_sfc_descriptor_rules()
        {
            let template_result = match shared_descriptor {
                Some(descriptor) => {
                    profile!("patina.sfc.descriptor_rules", {
                        super::script_rules::lint_with_descriptor(self, filename, descriptor)
                    })
                }
                None => {
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

    /// Lint a standalone HTML document that may use Vue from a CDN.
    #[inline]
    pub fn lint_standalone_html(&self, source: &str, filename: &str) -> LintResult {
        let capacity = (source.len() * 4).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);
        let mut result =
            self.lint_template_with_allocator_config(&allocator, source, filename, false, false);

        if super::script_rules::has_active_builtin_script_rules(self) {
            super::script_rules::append_builtin_script_diagnostics_from_html(
                self,
                source,
                &mut result,
            );
            result
                .diagnostics
                .sort_unstable_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
        }

        result
    }
}

fn source_may_contain_ecosystem_template_rule(
    template_source: &str,
    sfc_source: Option<&str>,
) -> bool {
    let template_bytes = template_source.as_bytes();
    if template_may_contain_ecosystem_element(template_bytes, sfc_source) {
        return true;
    }

    sfc_source.is_some_and(|source| source.contains("<i18n"))
        && template_may_call_i18n(template_bytes)
}

fn template_may_contain_ecosystem_element(bytes: &[u8], sfc_source: Option<&str>) -> bool {
    let imports_void_vue = sfc_source.is_some_and(|source| source.contains("@void/vue"));
    let mut cursor = 0;
    while let Some(relative) = memchr::memchr(b'<', &bytes[cursor..]) {
        let tag_start = cursor + relative;
        let Some((tag_name, name_end)) = tag_name_at(bytes, tag_start) else {
            cursor = tag_start + 1;
            continue;
        };

        if matches!(
            tag_name,
            b"RouterLink" | b"router-link" | b"NuxtLink" | b"nuxt-link"
        ) {
            return true;
        }
        if tag_name == b"Link" && imports_void_vue {
            return true;
        }

        let Some(tag_end) = find_tag_end(bytes, name_end) else {
            return false;
        };
        if tag_name.eq_ignore_ascii_case(b"a")
            && static_internal_href_may_exist(&bytes[name_end..tag_end])
        {
            return true;
        }

        cursor = tag_end + 1;
    }
    false
}

fn template_may_call_i18n(bytes: &[u8]) -> bool {
    memchr::memmem::find(bytes, b"$t(").is_some()
        || memchr::memmem::find(bytes, b"$te(").is_some()
        || memchr::memmem::find(bytes, b"$tm(").is_some()
        || memchr::memmem::find(bytes, b"t(").is_some()
        || memchr::memmem::find(bytes, b"te(").is_some()
        || memchr::memmem::find(bytes, b"tm(").is_some()
}

fn static_internal_href_may_exist(bytes: &[u8]) -> bool {
    let mut search_start = 0;
    while let Some(relative) = memchr::memmem::find(&bytes[search_start..], b"href") {
        let href_start = search_start + relative;
        search_start = href_start + "href".len();

        if href_start > 0 && is_identifier_byte(bytes[href_start - 1]) {
            continue;
        }
        if previous_non_whitespace(bytes, href_start)
            .is_some_and(|byte| matches!(byte, b':' | b'-' | b'.' | b'@'))
        {
            continue;
        }

        let mut cursor = skip_ascii_whitespace(bytes, search_start);
        if bytes.get(cursor) != Some(&b'=') {
            continue;
        }
        cursor = skip_ascii_whitespace(bytes, cursor + 1);
        if !matches!(bytes.get(cursor), Some(b'\'' | b'"')) {
            continue;
        }
        if bytes.get(cursor + 1) == Some(&b'/') && bytes.get(cursor + 2) != Some(&b'/') {
            return true;
        }
    }
    false
}

fn previous_non_whitespace(bytes: &[u8], before: usize) -> Option<u8> {
    bytes[..before]
        .iter()
        .rev()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
}

fn skip_ascii_whitespace(bytes: &[u8], mut cursor: usize) -> usize {
    while bytes
        .get(cursor)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        cursor += 1;
    }
    cursor
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}

/// Ultra-fast template extraction using memchr for SIMD-accelerated search.
#[inline]
pub(crate) fn extract_template_fast(source: &str) -> Option<(String, u32)> {
    let bytes = source.as_bytes();

    let (_, content_start) = find_template_block_start(bytes)?;

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
        if tag_name_at(bytes, next_lt)
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case(b"template"))
        {
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
        } else if closing_tag_name_at(bytes, next_lt)
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case(b"template"))
        {
            depth -= 1;
            if depth == 0 {
                let content = std::str::from_utf8(&bytes[content_start..next_lt]).ok()?;
                return Some((content.to_compact_string(), content_start as u32));
            }
            pos = find_tag_end(bytes, next_lt).map_or(next_lt + 11, |gt| gt + 1);
        } else {
            pos = next_lt + 1;
        }
    }

    None
}

fn find_template_block_start(bytes: &[u8]) -> Option<(usize, usize)> {
    let mut pos = 0;

    while pos < bytes.len() {
        let next_lt = match memchr::memchr(b'<', &bytes[pos..]) {
            Some(offset) => pos + offset,
            None => return None,
        };

        if bytes[next_lt..].starts_with(b"<!--") {
            pos = memchr::memmem::find(&bytes[next_lt + 4..], b"-->")
                .map_or(next_lt + 4, |offset| next_lt + 4 + offset + 3);
            continue;
        }

        let Some((tag_name, _)) = tag_name_at(bytes, next_lt) else {
            pos = next_lt + 1;
            continue;
        };

        let tag_end = find_tag_end(bytes, next_lt)?;
        if tag_name.eq_ignore_ascii_case(b"template") {
            return Some((next_lt, tag_end + 1));
        }

        if tag_end > next_lt && bytes[tag_end - 1] == b'/' {
            pos = tag_end + 1;
            continue;
        }

        pos = find_closing_tag(bytes, tag_name, tag_end + 1)
            .and_then(|close_idx| find_tag_end(bytes, close_idx))
            .map_or(tag_end + 1, |close_end| close_end + 1);
    }

    None
}

fn find_tag_end(bytes: &[u8], start: usize) -> Option<usize> {
    memchr::memchr(b'>', &bytes[start..]).map(|offset| start + offset)
}

fn find_closing_tag(bytes: &[u8], tag_name: &[u8], from: usize) -> Option<usize> {
    let mut pos = from;

    while pos < bytes.len() {
        let next_lt = match memchr::memmem::find(&bytes[pos..], b"</") {
            Some(offset) => pos + offset,
            None => return None,
        };

        if closing_tag_name_at(bytes, next_lt)
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case(tag_name))
        {
            return Some(next_lt);
        }

        pos = next_lt + 2;
    }

    None
}

fn tag_name_at(bytes: &[u8], lt_idx: usize) -> Option<(&[u8], usize)> {
    if bytes.get(lt_idx) != Some(&b'<') {
        return None;
    }

    let name_start = lt_idx + 1;
    match bytes.get(name_start) {
        Some(b'!' | b'/' | b'?') | None => return None,
        _ => {}
    }

    read_tag_name(bytes, name_start)
}

fn closing_tag_name_at(bytes: &[u8], lt_idx: usize) -> Option<(&[u8], usize)> {
    if bytes.get(lt_idx) != Some(&b'<') || bytes.get(lt_idx + 1) != Some(&b'/') {
        return None;
    }

    read_tag_name(bytes, lt_idx + 2)
}

fn read_tag_name(bytes: &[u8], name_start: usize) -> Option<(&[u8], usize)> {
    let mut name_end = name_start;
    while bytes
        .get(name_end)
        .is_some_and(|byte| is_tag_name_byte(*byte))
    {
        name_end += 1;
    }

    if name_end == name_start || !is_tag_boundary(bytes, name_end) {
        return None;
    }

    Some((&bytes[name_start..name_end], name_end))
}

fn is_tag_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.')
}

fn is_tag_boundary(bytes: &[u8], idx: usize) -> bool {
    matches!(
        bytes.get(idx),
        None | Some(b'>' | b'/' | b' ' | b'\n' | b'\r' | b'\t' | b'\x0c')
    )
}

#[cfg(test)]
mod tests {
    use super::{extract_template_fast, source_may_contain_ecosystem_template_rule};

    fn extract(source: &str) -> Option<vize_carton::String> {
        extract_template_fast(source).map(|(content, _)| content)
    }

    #[test]
    fn extract_template_fast_skips_template_prefix_custom_blocks() {
        let source = "<template-card></template-card><template><div /></template>";

        assert_eq!(extract(source).as_deref(), Some("<div />"));
    }

    #[test]
    fn extract_template_fast_skips_template_strings_in_script_blocks() {
        let source =
            "<script>const tag = '<template></template>';</script><template><span /></template>";

        assert_eq!(extract(source).as_deref(), Some("<span />"));
    }

    #[test]
    fn extract_template_fast_handles_nested_template_tags() {
        let source = "<template><template #default><slot /></template></template>";

        assert_eq!(
            extract(source).as_deref(),
            Some("<template #default><slot /></template>")
        );
    }

    #[test]
    fn ecosystem_template_hint_detects_static_internal_href() {
        assert!(source_may_contain_ecosystem_template_rule(
            r#"<a href = "/docs">Docs</a>"#,
            None
        ));
    }

    #[test]
    fn ecosystem_template_hint_ignores_bound_href_with_script_path_strings() {
        let template = r#"<a :href="link.url">Docs</a>"#;
        let sfc = r#"<template><a :href="link.url">Docs</a></template>
<script setup>
const links = [{ url: '/docs' }]
</script>"#;
        assert!(!source_may_contain_ecosystem_template_rule(
            template,
            Some(sfc)
        ));
    }
}
