mod art;
pub(in crate::linter::engine) mod facade;
pub(super) mod pug;

use super::{
    SfcTemplateLintInput, TemplateAnalysis, TemplateRuleEnv, extract_template_fast, offset_result,
};
use crate::linter::config::{LintResult, Linter};
use crate::linter::sfc_scripts::{ScriptParses, SfcScripts, lint_scripts};
use crate::markup::L2Template;
use vize_l0::dialect::VueDialect;
use vize_l0::{Allocator, ToCompactString, profile};

impl Linter {
    pub(crate) fn lint_sfc_template_root<'a>(
        &self,
        input: SfcTemplateLintInput<'a>,
        scripts: &'a SfcScripts<'_>,
    ) -> (LintResult, LintResult) {
        let mut script_result = empty_lint_result(input.filename);
        let valid_template = !matches!(&input.analysis, TemplateAnalysis::Disabled);
        let lint = input.l2.lint_template().with_compatibility_root(|root| {
                self.lint_template_root_with_finish(
                    super::template::TemplateExecution {
                        allocator: input.allocator,
                        source: &input.template.content,
                        filename: input.filename,
                        root,
                        l2: input.l2,
                        env: TemplateRuleEnv {
                            sfc_descriptor: Some(input.descriptor),
                            sfc_scripts: Some(scripts),
                            art_script_analysis: None,
                            petite_document: None,
                            defer_exact_suppressions: true,
                            dialect: VueDialect::Vue,
                            facade_rules: facade::RULES,
                        },
                    },
                    input.analysis,
                    |ctx| {
                        super::super::script_rules::append_builtin_script_diagnostics_with_template_context(
                            self,
                            input.descriptor,
                            scripts,
                            &mut script_result,
                            valid_template.then_some((input.l2, input.template.loc.start as u32)),
                            Some(ctx),
                        );
                    },
                )
            });
        (lint, script_result)
    }

    fn lint_sfc_with_descriptor<'a>(
        &self,
        filename: &str,
        descriptor: &vize_atelier_sfc::SfcDescriptor<'a>,
        scripts: &SfcScripts<'_>,
        derived: bool,
    ) -> LintResult {
        let Some(template) = descriptor.template.as_ref() else {
            let mut result = empty_lint_result(filename);
            super::super::script_rules::append_builtin_script_diagnostics(
                self,
                descriptor,
                scripts,
                &mut result,
                None,
            );
            if super::super::css_rules::has_active_builtin_css_rules(self) {
                super::super::css_rules::append_builtin_css_diagnostics(
                    self,
                    descriptor,
                    &mut result,
                    None,
                );
            }
            return result;
        };

        let profiler = vize_l0::profiler::global_profiler();
        let template_profile = profiler
            .is_enabled()
            .then(|| profiler.global_span("patina.sfc.descriptor.template_lint"));
        let allocator =
            Allocator::with_capacity((template.content.len() * 4).max(self.initial_capacity));
        let l2 = profile!(
            "patina.sfc.descriptor.template_l1_parse_lower",
            L2Template::lower(&allocator, &template.content)
        );
        let has_fatal_parse_errors = self.has_fatal_template_parse_errors(&l2);
        let analysis = if !has_fatal_parse_errors
            && (self.has_active_semantic_template_rules() || self.has_resolved_macro_type_demand())
        {
            Some(profile!(
                "patina.sfc.descriptor.croquis",
                if descriptor.script_setup.is_some()
                    && self.registry.has_rule("vue/no-unused-components")
                    && self.is_rule_enabled("vue/no-unused-components")
                    && descriptor.script_setup.as_ref().is_some_and(|script| {
                        // Script-setup candidates for this rule are runtime
                        // imports whose module path ends in `.vue`. Without
                        // one, the ordinary analysis already owns every
                        // possible Options API registration and there are no
                        // authored component-import reads to retain.
                        script.content.contains(".vue")
                    })
                {
                    super::rule_sets::analyze_descriptor_for_component_reads(
                        scripts,
                        Some(&l2),
                        self.has_resolved_macro_type_demand().then_some(filename),
                        self.has_unused_bindings_demand(),
                        derived,
                    )
                } else {
                    super::rule_sets::analyze_descriptor_for_lint_with_template(
                        scripts,
                        Some(&l2),
                        self.has_resolved_macro_type_demand().then_some(filename),
                        self.has_unused_bindings_demand(),
                        derived,
                    )
                }
            ))
        } else {
            None
        };

        let mut parse_result =
            Self::l1_template_parse_lint_result(filename, &template.content, &l2);
        self.append_compatibility_template_parse_diagnostics(
            &mut parse_result,
            &template.content,
            &l2,
        );
        offset_result(&mut parse_result, template.loc.start as u32);
        self.append_style_parse_diagnostics(&mut parse_result, &allocator, &descriptor.styles);
        let (lint_result, scripts) = self.lint_sfc_template_root(
            SfcTemplateLintInput {
                filename,
                template,
                allocator: &allocator,
                l2: &l2,
                descriptor,
                analysis: if has_fatal_parse_errors {
                    TemplateAnalysis::Disabled
                } else if let Some(analysis) = analysis.as_ref() {
                    TemplateAnalysis::Precomputed(analysis)
                } else {
                    TemplateAnalysis::Lazy
                },
            },
            scripts,
        );

        let mut result = Self::merge_lint_results(parse_result, lint_result);
        result.error_count += scripts.error_count;
        result.warning_count += scripts.warning_count;
        result.diagnostics.extend(scripts.diagnostics);
        drop(template_profile);
        if super::super::css_rules::has_active_builtin_css_rules(self) {
            super::super::css_rules::append_builtin_css_diagnostics(
                self,
                descriptor,
                &mut result,
                (!has_fatal_parse_errors
                    && template.src.is_none()
                    && template.lang.as_deref().is_none_or(|lang| lang == "html"))
                .then_some(l2.surface()),
            );
        }
        result
    }

    /// Lint a full Vue SFC file.
    ///
    /// Uses ultra-fast template extraction optimized for linting.
    #[inline]
    pub fn lint_sfc(&self, source: &str, filename: &str) -> LintResult {
        // `<template lang="pug">` lints through its derived Vue template.
        let result = self
            .lint_pug_sfc(source, filename)
            .unwrap_or_else(|| self.lint_sfc_unrouted(source, filename));
        self.apply_diagnostic_policy(result)
    }

    /// [`Self::lint_sfc`] without the template-dialect selector.
    pub(crate) fn lint_sfc_unrouted(&self, source: &str, filename: &str) -> LintResult {
        self.lint_sfc_view(source, filename, false)
    }

    pub(crate) fn lint_sfc_view(&self, source: &str, filename: &str, derived: bool) -> LintResult {
        let shared_descriptor_result = if self.needs_sfc_descriptor_for_lint() {
            profile!(
                "patina.sfc.shared_parse_sfc",
                Some(super::super::script_rules::parse_sfc_for_lint(
                    source, filename
                ))
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
        let script_allocator = oxc_allocator::Allocator::default();
        let scripts =
            shared_descriptor.map(|descriptor| lint_scripts(&script_allocator, descriptor));
        let shared = shared_descriptor.zip(scripts.as_ref());

        let sfc_result = sfc_parse_result.unwrap_or_else(|| {
            profile!(
                "patina.sfc.level_rules",
                self.lint_sfc_level(
                    source,
                    filename,
                    shared.map(|(descriptor, scripts)| (descriptor, scripts as &dyn ScriptParses)),
                )
            )
        });

        #[cfg(not(target_arch = "wasm32"))]
        if super::super::native_type_aware::has_active_type_aware_rules(self) {
            let template_result = profile!(
                "patina.type_aware.lint_sfc_with_corsa",
                super::super::native_type_aware::lint_sfc_with_corsa_descriptor(
                    self,
                    source,
                    filename,
                    shared_descriptor,
                    derived,
                )
            );
            return self.append_sfc_document_rule_diagnostics(
                source,
                filename,
                Self::merge_lint_results(template_result, sfc_result),
                shared_descriptor,
            );
        }

        if super::super::script_rules::has_active_builtin_script_rules(self)
            || super::super::css_rules::has_active_builtin_css_rules(self)
            || self.compatibility_template_parse_enabled()
            || self.has_active_semantic_template_rules()
            || self.has_active_shared_sfc_descriptor_rules()
            || super::super::musea_rules::has_active_builtin_musea_rules(self)
        {
            let template_result = match shared {
                Some((descriptor, scripts)) => {
                    profile!("patina.sfc.descriptor_rules", {
                        self.lint_sfc_with_descriptor(filename, descriptor, scripts, derived)
                    })
                }
                None => self.fast_template_lint_or_empty(source, filename),
            };
            return self.append_sfc_document_rule_diagnostics(
                source,
                filename,
                Self::merge_lint_results(template_result, sfc_result),
                shared_descriptor,
            );
        }

        let result = match profile!(
            "patina.template.extract_fast",
            extract_template_fast(source)
        ) {
            Some((content, byte_offset)) => {
                let mut result = self.lint_sfc_template_source(&content, filename);
                offset_result(&mut result, byte_offset);
                Self::merge_lint_results(result, sfc_result)
            }
            None if sfc_result.has_diagnostics() => sfc_result,
            None => empty_lint_result(filename),
        };

        self.append_sfc_document_rule_diagnostics(source, filename, result, shared_descriptor)
    }

    /// Template body extracted from an SFC. [`facade::RULES`] run on the L2 facade.
    fn lint_sfc_template_source(&self, source: &str, filename: &str) -> LintResult {
        let capacity = (source.len() * 4).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);
        self.lint_template_with_allocator_config(
            &allocator,
            source,
            filename,
            true,
            true,
            TemplateRuleEnv {
                sfc_descriptor: None,
                sfc_scripts: None,
                art_script_analysis: None,
                petite_document: None,
                defer_exact_suppressions: true,
                dialect: VueDialect::Vue,
                facade_rules: facade::RULES,
            },
        )
    }

    fn fast_template_lint_or_empty(&self, source: &str, filename: &str) -> LintResult {
        match profile!(
            "patina.template.extract_fast",
            extract_template_fast(source)
        ) {
            Some((content, byte_offset)) => {
                let mut fallback = self.lint_sfc_template_source(&content, filename);
                offset_result(&mut fallback, byte_offset);
                fallback
            }
            None => empty_lint_result(filename),
        }
    }

    pub(super) fn append_sfc_document_rule_diagnostics(
        &self,
        source: &str,
        filename: &str,
        mut result: LintResult,
        descriptor: Option<&vize_atelier_sfc::SfcDescriptor<'_>>,
    ) -> LintResult {
        super::super::musea_rules::append_builtin_musea_diagnostics(
            self,
            source,
            filename,
            &mut result,
        );
        // `<variant>` markup is not an SFC `<template>`, so the template lane
        // never sees it. Gallery variants are real components.
        self.append_art_variant_template_diagnostics(source, filename, &mut result, descriptor);
        if source.contains("eslint-")
            || source.contains("oxlint-")
            || source.contains("vize-disable")
            || source.contains("vize-enable")
            || source.contains("@vize:")
        {
            let parsed = descriptor
                .is_none()
                .then(|| super::super::script_rules::parse_sfc_for_lint(source, filename));
            let descriptor =
                descriptor.or_else(|| parsed.as_ref().and_then(|result| result.as_ref().ok()));
            if let Some(suppressions) = descriptor.and_then(crate::context::SfcSuppression::new) {
                let changed = suppressions.retain(&mut result.diagnostics);
                let reported = self.can_report_unused_directives()
                    && suppressions.append_unused(&mut result.diagnostics, |rule| {
                        self.can_report_suppression_rule(rule)
                    });
                if changed || reported {
                    result.error_count = result
                        .diagnostics
                        .iter()
                        .filter(|diagnostic| diagnostic.severity == crate::Severity::Error)
                        .count();
                    result.warning_count = result
                        .diagnostics
                        .iter()
                        .filter(|diagnostic| diagnostic.severity == crate::Severity::Warning)
                        .count();
                }
            }
        }
        result
    }
}

pub(in crate::linter::engine) fn empty_lint_result(filename: &str) -> LintResult {
    LintResult {
        filename: filename.to_compact_string(),
        diagnostics: Vec::new(),
        error_count: 0,
        warning_count: 0,
    }
}
