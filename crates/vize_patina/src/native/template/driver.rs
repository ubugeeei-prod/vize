//! Actual configured registry dispatch and once-parsed bare ownership.

use crate::{LintResult, Linter};
use vize_l0::{Allocator, SourceRoot, String, config::VueVersion};
use vize_l1::markup::NativeLintComponent;

use super::{NativeTemplateLintContext, NativeTemplateLintRefusal, NativeTemplateRule, admission};

impl Linter {
    /// Lint the exact original bare template through genuine native callbacks.
    /// Every enabled unprovided rule or unsupported input context refuses the
    /// entire result. The ordinary public/default lint route stays unchanged.
    pub fn lint_native_template(
        &self,
        source: &str,
        filename: &str,
    ) -> Result<LintResult, NativeTemplateLintRefusal> {
        let allocator = Allocator::with_capacity((source.len() * 4).max(self.initial_capacity));
        self.lint_native_template_with_allocator(&allocator, source, filename)
    }

    /// Reuse a caller's real arena without dropping original source custody.
    pub fn lint_native_template_with_allocator<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &str,
    ) -> Result<LintResult, NativeTemplateLintRefusal> {
        let callbacks = self.native_template_callbacks()?;
        let block = SourceRoot::new(source)
            .map_err(NativeTemplateLintRefusal::Source)?
            .whole_block();
        let root = NativeLintComponent::parse_in(allocator, block)
            .map_err(NativeTemplateLintRefusal::Parse)?;
        admission::component(root.component())?;
        let mut context = NativeTemplateLintContext::new(self, &root, filename);
        for (name, callback) in callbacks {
            context.current_rule = name;
            callback.run_on_template(&mut context, &root)?;
        }
        // Root findings are pending. A late refusal discards them wholesale.
        // There is no suppression pre-scan, second tree/header walk or reparse.
        admission::children(root.component(), root.children())?;
        Ok(context.finish())
    }

    fn native_template_callbacks(
        &self,
    ) -> Result<Vec<(&'static str, &dyn NativeTemplateRule)>, NativeTemplateLintRefusal> {
        if let Some(requested) = self.requested_vue_version
            && requested != VueVersion::V3
        {
            return Err(NativeTemplateLintRefusal::UnsupportedVueVersion { requested });
        }
        if self.requested_vapor_mode == Some(true) {
            return Err(NativeTemplateLintRefusal::UnsupportedVaporMode { requested: true });
        }
        // Preserve disabled precedence and refuse requested names that never
        // became a registry callback, including unknown/external rule names.
        if let Some(requested) = &self.enabled_rules {
            let mut names: Vec<_> = requested
                .iter()
                .filter(|name| self.is_rule_enabled(name))
                .collect();
            names.sort();
            for name in names {
                if !self.registry.has_rule(name) {
                    return Err(NativeTemplateLintRefusal::UnprovidedRule { rule: name.clone() });
                }
            }
        }
        for name in self
            .script_rules
            .iter()
            .chain(self.css_rules)
            .chain(self.musea_rules)
        {
            if self.is_rule_enabled(name) {
                return Err(NativeTemplateLintRefusal::UnprovidedRule {
                    rule: String::new(name),
                });
            }
        }
        let mut callbacks = Vec::new();
        for rule in self.registry.rules() {
            let name = rule.meta().name;
            if !self.is_rule_enabled(name) {
                continue;
            }
            let callback = rule.as_native_template_rule().ok_or_else(|| {
                NativeTemplateLintRefusal::UnprovidedRule {
                    rule: String::new(name),
                }
            })?;
            callbacks.push((name, callback));
        }
        Ok(callbacks)
    }
}
