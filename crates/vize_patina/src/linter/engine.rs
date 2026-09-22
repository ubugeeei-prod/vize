//! Lint execution engine.
//!
//! Contains the core linting methods: single-file template linting,
//! full SFC linting with template extraction, and batch file processing.
//!
//! - [`parse_diagnostics`]: parser-error to lint-diagnostic translation
//! - [`template_extract`]: ultra-fast `<template>` block extraction
//! - [`ecosystem_hint`]: source heuristics for ecosystem template rules
//! - [`tag_scan`]: shared byte-oriented tag scanning primitives
//! - [`rule_sets`]: rule-name sets gating shared analysis work

mod ecosystem_hint;
mod jsx;
mod offset;
mod parse_diagnostics;
mod rule_sets;
mod script;
mod sfc;
mod tag_scan;
mod template_extract;

pub(crate) use template_extract::extract_template_fast;

use crate::{context::LintContext, diagnostic::LintSummary, preset::LintPreset};
use vize_armature::Parser;
use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_croquis::{Croquis, Drawer};
use vize_relief::RootNode;
use vize_s0::Allocator;
use vize_s0::String;
use vize_s0::ToCompactString;
use vize_s0::dialect::{VueDialect, standalone_html_dialect};
use vize_s0::profile;

use super::config::{LintResult, Linter};

use ecosystem_hint::source_may_contain_ecosystem_template_rule;
pub(crate) use offset::offset_result;
use rule_sets::{SEMANTIC_TEMPLATE_RULES, SHARED_SFC_DESCRIPTOR_RULES};

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

/// Document-level inputs shared by the template-rule passes.
///
/// Bundles the optional SFC descriptor with the resolved [`VueDialect`] so the
/// rule context can gate dialect-specific rules (e.g. petite-vue keyless
/// `v-for`) without growing the already-wide pass signatures.
#[derive(Clone, Copy)]
pub(crate) struct TemplateRuleEnv<'a> {
    pub sfc_descriptor: Option<&'a vize_atelier_sfc::SfcDescriptor<'a>>,
    pub dialect: VueDialect,
    /// Markup rule `lint_sfc` runs on the S2 facade instead of the visitor.
    /// `None` on the raw-template and standalone-HTML lanes.
    pub facade_rule: Option<&'static str>,
}

impl<'a> TemplateRuleEnv<'a> {
    const fn relief(dialect: VueDialect) -> Self {
        Self {
            sfc_descriptor: None,
            dialect,
            facade_rule: None,
        }
    }
}

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
        ctx.set_config_disabled_rules(self.disabled_rules.clone());
        ctx.set_config_rule_severities(self.severity_overrides.clone());
        ctx.set_help_level(self.help_level);

        // SFC-level rules are uncommon but expensive when each one reparses the file.
        // Reuse the descriptor produced by the main lint pipeline whenever available,
        // and only parse lazily when a caller enters this path without one.
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
            || super::css_rules::has_active_builtin_css_rules(self)
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
        analysis: Option<&'a Croquis>,
        env: TemplateRuleEnv<'a>,
    ) -> LintResult {
        let mut ctx = LintContext::with_locale(allocator, source, filename, self.locale);
        ctx.set_enabled_rules(self.enabled_rules.clone());
        ctx.set_config_disabled_rules(self.disabled_rules.clone());
        ctx.set_config_rule_severities(self.severity_overrides.clone());
        ctx.set_help_level(self.help_level);
        ctx.set_dialect(env.dialect);
        if let Some(descriptor) = env.sfc_descriptor {
            ctx.set_sfc_template_descriptor(descriptor);
        }
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
            env.sfc_descriptor
                .map(|descriptor| descriptor.source.as_ref()),
        );
        sfc::facade::dispatch_template_rules(
            self,
            &mut ctx,
            sfc::facade::Dispatch {
                allocator,
                source,
                root,
                analysis,
                rule: env.facade_rule,
                rule_count,
            },
        );

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
        analysis: TemplateAnalysis<'a>,
        env: TemplateRuleEnv<'a>,
    ) -> LintResult {
        if matches!(analysis, TemplateAnalysis::Disabled)
            || !self.has_active_semantic_template_rules()
        {
            return self.run_template_rules(allocator, source, filename, root, None, env);
        }
        let owned_analysis;
        let analysis = match analysis {
            TemplateAnalysis::Disabled => unreachable!(),
            TemplateAnalysis::Precomputed(analysis) => analysis,
            TemplateAnalysis::Lazy => {
                owned_analysis = profile!("patina.template.croquis", {
                    let mut analyzer = Drawer::for_lint();
                    analyzer.analyze_template(root);
                    analyzer.finish()
                });
                &owned_analysis
            }
        };

        self.run_template_rules(allocator, source, filename, root, Some(analysis), env)
    }

    /// Lint a Vue template source.
    #[inline]
    pub fn lint_template(&self, source: &str, filename: &str) -> LintResult {
        // Create allocator sized for source (rough heuristic: 4x source size)
        let capacity = (source.len() * 4).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);

        self.lint_template_with_allocator(&allocator, source, filename)
    }

    /// Lint JSX/TSX source.
    ///
    /// Markup rules ([`Rule::as_markup_rule`](crate::rule::Rule::as_markup_rule))
    /// run over the S2 facade, the P2-16 projection of each render root.
    /// [`Rule::jsx_needs_lowering`](crate::rule::Rule::jsx_needs_lowering) only
    /// partitions that walk so each rule runs once. A root the projection
    /// refuses keeps the lowered Relief document. Rules with no markup entry
    /// point use that lowering as the legacy fallback. A directive with no JSX
    /// analogue (e.g. `v-html`) never matches, the documented no-op.
    pub fn lint_jsx(
        &self,
        source: &str,
        filename: &str,
        lang: vize_atelier_jsx::JsxLang,
    ) -> LintResult {
        let capacity = (source.len() * 4).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);

        // Partition the active rules into three disjoint groups so each rule runs
        // exactly once (no double-report). Both markup groups walk the S2
        // projection; `jsx_needs_lowering` only selects which pass owns the rule.
        //
        //   ir       — markup-capable, no list/branch shape required.
        //   lowered  — markup-capable and `jsx_needs_lowering` (ui.for / ui.if).
        //   legacy   — no markup entry point, served by the lowering fallback.
        let rules = self.registry.rules();
        let mut any_ir = false;
        let mut any_lowered_markup = false;
        let legacy_keep_mask: Vec<bool> = rules
            .iter()
            .map(|rule| match rule.as_markup_rule() {
                Some(_) if rule.jsx_needs_lowering() => {
                    any_lowered_markup = true;
                    false
                }
                Some(_) => {
                    any_ir = true;
                    false
                }
                None => true,
            })
            .collect();
        let any_legacy = legacy_keep_mask.iter().any(|keep| *keep);
        let needs_s2 = any_ir || any_lowered_markup;

        let oxc_allocator = oxc_allocator::Allocator::default();
        let parsed = profile!(
            "patina.jsx.parse",
            vize_atelier_jsx::parse_module(&oxc_allocator, source, lang)
        );
        let mut result = Self::jsx_diagnostics_lint_result(filename, &parsed.diagnostics);

        if needs_s2 || any_legacy {
            let lowered = profile!(
                "patina.jsx.lower",
                vize_atelier_jsx::lower_source(&allocator, &oxc_allocator, source, lang)
            );
            let analysis = (any_ir && self.jsx_ir_needs_analysis()).then_some(&lowered.analysis);
            // Lowering diagnostics are a superset of the parse. Merge them only
            // when this lane lowered before P4-7b (a list-shaped markup rule or
            // a legacy rule). An element-shaped rule now lowers only to build
            // the S2 view; its parse diagnostics stay the OXC set.
            if any_lowered_markup || any_legacy {
                let mut lower_diags =
                    Self::jsx_diagnostics_lint_result(filename, &lowered.diagnostics);
                Self::dedupe_against(&mut lower_diags, &result);
                result = Self::merge_lint_results(result, lower_diags);
            }

            for lowered_root in &lowered.roots {
                if let Ok(projected) = lowered_root.s2.as_ref() {
                    let markup = crate::markup::S2Markup::from_projected_root(projected);
                    if any_ir {
                        let ir_result =
                            self.lint_jsx_over_ir(&allocator, source, filename, &markup, analysis);
                        result = Self::merge_lint_results(result, ir_result);
                    }
                    if any_lowered_markup {
                        let lowered_markup =
                            self.lint_jsx_lowered_markup_s2(&allocator, source, filename, &markup);
                        result = Self::merge_lint_results(result, lowered_markup);
                    }
                } else {
                    // The P2-16 projection refused this root (an unsupported
                    // element, a comment, a compound expression). Keep the
                    // lowered Relief document so the rule still runs.
                    if any_ir {
                        let ir_result = self.lint_jsx_over_ir_refused(
                            &allocator,
                            source,
                            filename,
                            &lowered_root.root,
                            analysis,
                        );
                        result = Self::merge_lint_results(result, ir_result);
                    }
                    if any_lowered_markup {
                        let lowered_markup = self.lint_jsx_lowered_markup_root(
                            &allocator,
                            source,
                            filename,
                            &lowered_root.root,
                        );
                        result = Self::merge_lint_results(result, lowered_markup);
                    }
                }
                if any_legacy {
                    let legacy = self.lint_jsx_fallback_root(
                        &allocator,
                        source,
                        filename,
                        &lowered_root.root,
                        &legacy_keep_mask,
                    );
                    result = Self::merge_lint_results(result, legacy);
                }
            }
        }

        result
    }

    /// Lint a Vue template with a provided allocator (for reuse).
    pub fn lint_template_with_allocator(
        &self,
        allocator: &Allocator,
        source: &str,
        filename: &str,
    ) -> LintResult {
        self.lint_template_with_allocator_config(
            allocator,
            source,
            filename,
            true,
            true,
            TemplateRuleEnv::relief(VueDialect::Vue),
        )
    }

    #[cfg(test)]
    pub(crate) fn lint_template_rules_only(&self, source: &str, filename: &str) -> LintResult {
        let capacity = (source.len() * 4).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);

        self.lint_template_with_allocator_config(
            &allocator,
            source,
            filename,
            false,
            true,
            TemplateRuleEnv::relief(VueDialect::Vue),
        )
    }

    fn lint_template_with_allocator_config(
        &self,
        allocator: &Allocator,
        source: &str,
        filename: &str,
        report_parse_errors: bool,
        gate_semantic_on_fatal_parse: bool,
        env: TemplateRuleEnv<'_>,
    ) -> LintResult {
        // Parse the template
        let parser = Parser::new(allocator, source);
        let (root, parse_errors) = profile!("patina.template.parse", parser.parse());
        let has_fatal_parse_errors = Self::has_fatal_template_parse_errors(&parse_errors);

        let parse_result = Self::template_parse_lint_result(filename, source.len(), &parse_errors);
        let lint_result = self.lint_template_root(
            allocator,
            source,
            filename,
            &root,
            if gate_semantic_on_fatal_parse && has_fatal_parse_errors {
                TemplateAnalysis::Disabled
            } else {
                TemplateAnalysis::Lazy
            },
            env,
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

    /// Lint a standalone HTML document that may use Vue from a CDN.
    #[inline]
    pub fn lint_standalone_html(&self, source: &str, filename: &str) -> LintResult {
        let capacity = (source.len() * 4).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);
        // Resolve the document dialect so dialect-specific rules (e.g.
        // require-v-for-key, which petite-vue does not require) can gate
        // themselves on petite-vue documents.
        let dialect = standalone_html_dialect(None, source);
        let mut result = self.lint_template_with_allocator_config(
            &allocator,
            source,
            filename,
            false,
            false,
            TemplateRuleEnv::relief(dialect),
        );

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
