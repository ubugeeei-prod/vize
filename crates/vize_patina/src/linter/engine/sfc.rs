use super::{
    SfcTemplateLintInput, TemplateAnalysis, TemplateRuleEnv, analyze_descriptor_for_lint,
    extract_template_fast, offset_result,
};
use crate::linter::config::{LintResult, Linter};
use vize_armature::Parser;
use vize_s0::dialect::VueDialect;
use vize_s0::{Allocator, ToCompactString, profile};

impl Linter {
    pub(crate) fn lint_sfc_template_root<'a>(&self, input: SfcTemplateLintInput<'a>) -> LintResult {
        self.lint_template_root(
            input.allocator,
            &input.template.content,
            input.filename,
            input.root,
            input.analysis,
            TemplateRuleEnv {
                sfc_descriptor: input.descriptor,
                dialect: VueDialect::Vue,
            },
        )
    }

    pub(crate) fn lint_sfc_template_with_descriptor<'a>(
        &self,
        filename: &str,
        descriptor: &vize_atelier_sfc::SfcDescriptor<'a>,
    ) -> LintResult {
        let Some(template) = descriptor.template.as_ref() else {
            return empty_lint_result(filename);
        };

        let allocator =
            Allocator::with_capacity((template.content.len() * 4).max(self.initial_capacity));
        let parser = Parser::new(&allocator, &template.content);
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
        offset_result(&mut parse_result, template.loc.start as u32);
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

        let sfc_result = sfc_parse_result.unwrap_or_else(|| {
            profile!(
                "patina.sfc.level_rules",
                self.lint_sfc_level(source, filename, shared_descriptor)
            )
        });

        #[cfg(not(target_arch = "wasm32"))]
        if super::super::native_type_aware::has_active_type_aware_rules(self) {
            let mut template_result = profile!(
                "patina.type_aware.lint_sfc_with_corsa",
                super::super::native_type_aware::lint_sfc_with_corsa_descriptor(
                    self,
                    source,
                    filename,
                    shared_descriptor,
                )
            );
            if super::super::css_rules::has_active_builtin_css_rules(self)
                && let Some(descriptor) = shared_descriptor
            {
                super::super::css_rules::append_builtin_css_diagnostics(
                    self,
                    descriptor,
                    &mut template_result,
                );
            }
            return self.append_sfc_document_rule_diagnostics(
                source,
                filename,
                Self::merge_lint_results(template_result, sfc_result),
            );
        }

        if super::super::script_rules::has_active_builtin_script_rules(self)
            || super::super::css_rules::has_active_builtin_css_rules(self)
            || self.has_active_semantic_template_rules()
            || self.has_active_shared_sfc_descriptor_rules()
            || super::super::musea_rules::has_active_builtin_musea_rules(self)
        {
            let template_result = match shared_descriptor {
                Some(descriptor) => {
                    profile!("patina.sfc.descriptor_rules", {
                        let mut result = super::super::script_rules::lint_with_descriptor(
                            self, filename, descriptor,
                        );
                        if super::super::css_rules::has_active_builtin_css_rules(self) {
                            super::super::css_rules::append_builtin_css_diagnostics(
                                self,
                                descriptor,
                                &mut result,
                            );
                        }
                        result
                    })
                }
                None => self.fast_template_lint_or_empty(source, filename),
            };
            return self.append_sfc_document_rule_diagnostics(
                source,
                filename,
                Self::merge_lint_results(template_result, sfc_result),
            );
        }

        let result = match profile!(
            "patina.template.extract_fast",
            extract_template_fast(source)
        ) {
            Some((content, byte_offset)) => {
                let mut result = self.lint_template(&content, filename);
                offset_result(&mut result, byte_offset);
                Self::merge_lint_results(result, sfc_result)
            }
            None if sfc_result.has_diagnostics() => sfc_result,
            None => empty_lint_result(filename),
        };

        self.append_sfc_document_rule_diagnostics(source, filename, result)
    }

    fn fast_template_lint_or_empty(&self, source: &str, filename: &str) -> LintResult {
        match profile!(
            "patina.template.extract_fast",
            extract_template_fast(source)
        ) {
            Some((content, byte_offset)) => {
                let mut fallback = self.lint_template(&content, filename);
                offset_result(&mut fallback, byte_offset);
                fallback
            }
            None => empty_lint_result(filename),
        }
    }

    fn append_sfc_document_rule_diagnostics(
        &self,
        source: &str,
        filename: &str,
        mut result: LintResult,
    ) -> LintResult {
        super::super::musea_rules::append_builtin_musea_diagnostics(
            self,
            source,
            filename,
            &mut result,
        );
        result
    }
}

fn empty_lint_result(filename: &str) -> LintResult {
    LintResult {
        filename: filename.to_compact_string(),
        diagnostics: Vec::new(),
        error_count: 0,
        warning_count: 0,
    }
}
