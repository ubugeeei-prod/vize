//! Freeze actual configured instances, then own one original SFC traversal.

use super::{
    NativeSfcLintContext, NativeSfcLintOwner, NativeSfcLintRefusal as Refusal, NativeSfcSetupRule,
};
use crate::{LintResult, Linter, Severity};
use vize_l0::{Allocator, String, config::VueVersion};

impl Linter {
    /// Explicit native original-SFC entry. Every enabled unprovided instance
    /// refuses; this never changes the ordinary/default public product path.
    pub fn lint_native_sfc(&self, source: &str, filename: &str) -> Result<LintResult, Refusal> {
        let allocator = Allocator::with_capacity((source.len() * 4).max(self.initial_capacity));
        self.lint_native_sfc_with_allocator(&allocator, source, filename)
    }

    pub fn lint_native_sfc_with_allocator<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &str,
    ) -> Result<LintResult, Refusal> {
        let callbacks = self.native_sfc_callbacks()?;
        let owner = NativeSfcLintOwner::parse_in(allocator, source)?;
        let setup = owner.setup()?;
        let mut context = NativeSfcLintContext::new(self, &owner, filename);
        for callback in callbacks {
            context.current_rule = callback.name;
            context.default_severity = callback.severity;
            callback.rule.run_on_setup(&mut context, &setup)?;
        }
        // Pending setup findings confer no parser-output/whole-template credit.
        // Reuse the same original child/header walk, with no element callbacks
        // and the unchanged strict StaticOnly profile. Any late error discards
        // the complete context, including earlier successful callback output.
        super::super::admission::children(
            owner.template().component(),
            owner.template().children(),
            super::super::template::NativeTemplateAttributeProfile::StaticOnly,
            &mut |_| Ok(()),
        )
        .map_err(Refusal::Template)?;
        Ok(context.finish())
    }

    fn native_sfc_callbacks(&self) -> Result<Vec<Callback<'_>>, Refusal> {
        if let Some(requested) = self.requested_vue_version
            && requested != VueVersion::V3
        {
            return Err(Refusal::UnsupportedVueVersion { requested });
        }
        if self.requested_vapor_mode == Some(true) || self.vapor_mode {
            return Err(Refusal::UnsupportedVaporMode { requested: true });
        }
        if self.type_aware_enabled {
            return Err(Refusal::UnsupportedTypeAwareMode);
        }
        // Disabled precedence stays exact. A catalog name cannot substitute for
        // an actual enabled instance in its original configured registry.
        if let Some(requested) = &self.enabled_rules {
            let mut names: Vec<_> = requested
                .iter()
                .filter(|name| self.is_rule_enabled(name))
                .collect();
            names.sort();
            for name in names {
                if !self.script_rules.contains(&name.as_str()) {
                    return Err(Refusal::UnprovidedRule { rule: name.clone() });
                }
            }
        }
        for rule in self.registry.rules() {
            if self.is_rule_enabled(rule.meta().name) {
                return Err(Refusal::UnprovidedRule {
                    rule: String::new(rule.meta().name),
                });
            }
        }
        for name in self.css_rules.iter().chain(self.musea_rules) {
            if self.is_rule_enabled(name) {
                return Err(Refusal::UnprovidedRule {
                    rule: String::new(name),
                });
            }
        }
        let mut callbacks = Vec::new();
        for &name in self.script_rules {
            if !self.is_rule_enabled(name) {
                continue;
            }
            let missing = || Refusal::UnprovidedRule {
                rule: String::new(name),
            };
            let instance =
                crate::linter::script_rules::native::configured(self, name).ok_or_else(missing)?;
            let rule = instance.as_native_sfc_setup_rule().ok_or_else(missing)?;
            if !instance.runs_on_script_setup() {
                return Err(missing());
            }
            if crate::linter::script_rules::native::requires_invocation_policy(instance) {
                return Err(Refusal::UnprovidedInvocationPolicy {
                    rule: String::new(name),
                });
            }
            callbacks.push(Callback {
                name,
                rule,
                severity: instance.meta().default_severity,
            });
        }
        Ok(callbacks)
    }
}

struct Callback<'o> {
    name: &'static str,
    rule: &'o dyn NativeSfcSetupRule,
    severity: Severity,
}
