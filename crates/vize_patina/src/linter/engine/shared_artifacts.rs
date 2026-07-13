//! Linting from Atlas-owned SFC artifacts.

use vize_carton::{
    Allocator, ToCompactString, dialect::VueDialect, dialect::standalone_html_dialect, profile,
};
use vize_croquis::Croquis;
use vize_relief::{CompilerError, ReliefSnapshot};

use super::{SfcTemplateLintInput, TemplateAnalysis, analyze_descriptor_for_lint, offset_result};
use crate::linter::config::{LintResult, Linter};
use crate::markup::{MarkupContext, MarkupDocument};

struct SharedTemplateOptions {
    report_parse_errors: bool,
    gate_semantic_on_fatal_parse: bool,
    dialect: VueDialect,
}

impl SharedTemplateOptions {
    const VUE_TEMPLATE: Self = Self {
        report_parse_errors: true,
        gate_semantic_on_fatal_parse: true,
        dialect: VueDialect::Vue,
    };

    fn standalone_html(source: &str) -> Self {
        Self {
            report_parse_errors: false,
            gate_semantic_on_fatal_parse: false,
            dialect: standalone_html_dialect(None, source),
        }
    }
}

impl Linter {
    /// Lint a raw Vue template from Atlas-owned Relief syntax.
    ///
    /// The syntax snapshot is materialized into this call's arena without
    /// parsing the source again. Parse diagnostics remain identical to the
    /// direct template API.
    pub fn lint_template_with_shared_artifacts(
        &self,
        source: &str,
        filename: &str,
        syntax: &ReliefSnapshot,
        parse_errors: &[CompilerError],
    ) -> LintResult {
        self.lint_template_with_shared_products(source, filename, syntax, parse_errors, None)
    }

    /// Lint a raw Vue template from Atlas-owned syntax and semantics.
    pub fn lint_template_with_shared_products(
        &self,
        source: &str,
        filename: &str,
        syntax: &ReliefSnapshot,
        parse_errors: &[CompilerError],
        analysis: Option<&Croquis>,
    ) -> LintResult {
        self.lint_shared_template(
            source,
            filename,
            syntax,
            parse_errors,
            SharedTemplateOptions::VUE_TEMPLATE,
            analysis,
        )
    }

    /// Lint standalone HTML from Atlas-owned Relief syntax.
    ///
    /// Standalone HTML intentionally suppresses parser diagnostics and keeps
    /// semantic rules enabled for recoverable browser-shaped documents, just
    /// like [`Linter::lint_standalone_html`].
    pub fn lint_standalone_html_with_shared_artifacts(
        &self,
        source: &str,
        filename: &str,
        syntax: &ReliefSnapshot,
        parse_errors: &[CompilerError],
    ) -> LintResult {
        self.lint_standalone_html_with_shared_products(source, filename, syntax, parse_errors, None)
    }

    /// Lint standalone HTML from Atlas-owned syntax and semantics.
    pub fn lint_standalone_html_with_shared_products(
        &self,
        source: &str,
        filename: &str,
        syntax: &ReliefSnapshot,
        parse_errors: &[CompilerError],
        analysis: Option<&Croquis>,
    ) -> LintResult {
        let mut result = self.lint_shared_template(
            source,
            filename,
            syntax,
            parse_errors,
            SharedTemplateOptions::standalone_html(source),
            analysis,
        );
        if crate::linter::script_rules::has_active_builtin_script_rules(self) {
            crate::linter::script_rules::append_builtin_script_diagnostics_from_html(
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

    fn lint_shared_template(
        &self,
        source: &str,
        filename: &str,
        syntax: &ReliefSnapshot,
        parse_errors: &[CompilerError],
        options: SharedTemplateOptions,
        analysis: Option<&Croquis>,
    ) -> LintResult {
        let allocator = Allocator::with_capacity((source.len() * 4).max(self.initial_capacity));
        let root = syntax.materialize(allocator.as_bump());
        let has_fatal_parse_errors = Self::has_fatal_template_parse_errors(parse_errors);
        let lint_result = self.lint_template_root(
            &allocator,
            source,
            filename,
            &root,
            if options.gate_semantic_on_fatal_parse && has_fatal_parse_errors {
                TemplateAnalysis::Disabled
            } else if let Some(analysis) = analysis {
                TemplateAnalysis::Precomputed(analysis)
            } else {
                TemplateAnalysis::Lazy
            },
            super::TemplateRuleEnv {
                sfc_descriptor: None,
                dialect: options.dialect,
            },
        );
        if options.report_parse_errors {
            Self::merge_lint_results(
                Self::template_parse_lint_result(filename, source.len(), parse_errors),
                lint_result,
            )
        } else {
            lint_result
        }
    }

    /// Lint JSX/TSX from Atlas's owned syntax and Croquis products.
    ///
    /// Only rules with an explicit markup projection run on JSX. Rules without
    /// one deliberately no-op: production lint never reparses or lowers JSX
    /// through Relief to emulate template-only rule shapes.
    pub fn lint_jsx_with_shared_artifacts(
        &self,
        source: &str,
        filename: &str,
        syntax: &vize_atelier_jsx::JsxSyntaxSnapshot,
        analysis: Option<&Croquis>,
    ) -> LintResult {
        let allocator = Allocator::with_capacity((source.len() * 4).max(self.initial_capacity));
        let mut context =
            crate::context::LintContext::with_locale(&allocator, source, filename, self.locale);
        context.set_enabled_rules(self.enabled_rules.clone());
        context.set_config_disabled_rules(self.disabled_rules.clone());
        context.set_config_rule_severities(self.severity_overrides.clone());
        context.set_help_level(self.help_level);
        if let Some(analysis) = analysis {
            context.set_analysis(analysis);
        }

        let mut document = MarkupDocument::from_jsx_snapshot(syntax);
        if let Some(analysis) = analysis {
            document = document.with_analysis(analysis);
        }
        profile!("patina.jsx.atlas.visit", {
            let mut markup = MarkupContext::new(&mut context, &document);
            for rule in self.registry.rules() {
                if let Some(rule) = rule.as_markup_rule() {
                    document.visit_with(rule, &mut markup);
                }
            }
        });

        let result = LintResult {
            filename: filename.to_compact_string(),
            error_count: context.error_count(),
            warning_count: context.warning_count(),
            diagnostics: context.into_diagnostics(),
        };
        Self::merge_lint_results(
            Self::jsx_diagnostics_lint_result(filename, &syntax.diagnostics),
            result,
        )
    }

    /// Preserve legacy malformed-container diagnostics without reparsing the
    /// SFC container after Atlas has cached its structured parse error.
    pub(crate) fn lint_sfc_with_shared_parse_error(
        &self,
        source: &str,
        filename: &str,
        parse_error: &vize_atelier_sfc::SfcError,
    ) -> LintResult {
        let sfc_result = if self.needs_sfc_descriptor_for_lint() {
            Self::sfc_parse_lint_result(filename, source.len(), parse_error)
        } else {
            self.lint_sfc_level(source, filename, None)
        };
        let Some((content, byte_offset)) = super::extract_template_fast(source) else {
            return sfc_result;
        };
        let mut template_result = self.lint_template(&content, filename);
        offset_result(&mut template_result, byte_offset);
        Self::merge_lint_results(template_result, sfc_result)
    }

    /// Lint an SFC from Atlas-owned parse and semantic products.
    ///
    /// The Relief snapshot is materialized into this call's arena view without
    /// parsing template text. A caller that also supplies the complete Croquis
    /// document avoids semantic analysis as well. Script and CSS rule passes
    /// continue to consume the shared descriptor.
    pub fn lint_sfc_with_shared_artifacts<'a>(
        &self,
        source: &str,
        filename: &str,
        descriptor: &'a vize_atelier_sfc::SfcDescriptor<'a>,
        template_syntax: Option<(&ReliefSnapshot, &[CompilerError])>,
        analysis: Option<&Croquis>,
    ) -> LintResult {
        let sfc_result = profile!(
            "patina.sfc.shared.level_rules",
            self.lint_sfc_level(source, filename, Some(descriptor))
        );

        #[cfg(not(target_arch = "wasm32"))]
        if crate::linter::native_type_aware::has_active_type_aware_rules(self) {
            let mut typed = crate::linter::native_type_aware::lint_sfc_with_corsa_artifacts(
                self,
                source,
                filename,
                descriptor,
                template_syntax,
                analysis,
            );
            if crate::linter::css_rules::has_active_builtin_css_rules(self) {
                crate::linter::css_rules::append_builtin_css_diagnostics(
                    self, descriptor, &mut typed,
                );
            }
            return Self::merge_lint_results(typed, sfc_result);
        }

        let mut result = LintResult {
            filename: filename.to_compact_string(),
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
        };
        if let (Some(template), Some((snapshot, parse_errors))) =
            (descriptor.template.as_ref(), template_syntax)
        {
            let allocator =
                Allocator::with_capacity((template.content.len() * 4).max(self.initial_capacity));
            let root = snapshot.materialize(allocator.as_bump());
            let has_fatal_parse_errors = Self::has_fatal_template_parse_errors(parse_errors);
            let owned_analysis;
            let analysis = if has_fatal_parse_errors || !self.has_active_semantic_template_rules() {
                None
            } else if let Some(analysis) = analysis {
                Some(analysis)
            } else {
                owned_analysis = profile!(
                    "patina.sfc.shared.croquis_fallback",
                    analyze_descriptor_for_lint(descriptor, Some(&root))
                );
                Some(&owned_analysis)
            };
            let mut parse_result =
                Self::template_parse_lint_result(filename, template.content.len(), parse_errors);
            offset_result(&mut parse_result, template.loc.start as u32);
            let template_result = self.lint_sfc_template_root(SfcTemplateLintInput {
                filename,
                template,
                allocator: &allocator,
                root: &root,
                descriptor: Some(descriptor),
                analysis: if has_fatal_parse_errors {
                    TemplateAnalysis::Disabled
                } else if let Some(analysis) = analysis {
                    TemplateAnalysis::Precomputed(analysis)
                } else {
                    TemplateAnalysis::Lazy
                },
            });
            result = Self::merge_lint_results(parse_result, template_result);
        }

        crate::linter::script_rules::append_builtin_script_diagnostics(
            self,
            descriptor,
            &mut result,
        );
        if crate::linter::css_rules::has_active_builtin_css_rules(self) {
            crate::linter::css_rules::append_builtin_css_diagnostics(self, descriptor, &mut result);
        }
        Self::merge_lint_results(result, sfc_result)
    }
}
