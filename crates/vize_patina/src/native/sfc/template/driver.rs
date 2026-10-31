//! Freeze registered capabilities before owning and traversing the original SFC.

use super::{NativeSfcTemplateLintContext, NativeSfcTemplateOwner, NativeSfcTemplateRule};
use crate::native::sfc::NativeSfcLintRefusal as Refusal;
use crate::native::template::{NativeTemplateAttributeProfile, NativeTemplateTextProfile};
use crate::{LintResult, Linter, Severity};
use vize_l0::{Allocator, String, config::VueVersion};

pub(in crate::native::sfc) fn lint<'a>(
    linter: &Linter,
    allocator: &'a Allocator,
    source: &'a str,
    filename: &str,
) -> Result<LintResult, Refusal> {
    let callbacks = callbacks(linter)?;
    let attributes = callbacks
        .iter()
        .map(|entry| entry.rule.attribute_profile())
        .reduce(NativeTemplateAttributeProfile::intersect)
        .unwrap_or_default();
    let text = callbacks
        .iter()
        .map(|entry| entry.rule.text_profile())
        .reduce(NativeTemplateTextProfile::intersect)
        .unwrap_or_default();
    let owner = NativeSfcTemplateOwner::parse_in(allocator, source)?;
    let mut context = NativeSfcTemplateLintContext::new(linter, &owner, filename);
    for entry in &callbacks {
        context.current_rule = entry.name;
        context.default_severity = entry.severity;
        entry
            .rule
            .run_on_template(&mut context, &owner)
            .map_err(Refusal::Template)?;
    }
    crate::native::admission::children_with_text_profile(
        owner.template().component(),
        owner.template().children(),
        attributes,
        text,
        &mut |element| {
            for entry in &callbacks {
                context.current_rule = entry.name;
                context.default_severity = entry.severity;
                entry.rule.run_on_element(&mut context, element)?;
            }
            Ok(())
        },
    )
    .map_err(Refusal::Template)?;
    Ok(context.finish())
}

fn callbacks(linter: &Linter) -> Result<Vec<Callback<'_>>, Refusal> {
    if let Some(requested) = linter.requested_vue_version
        && requested != VueVersion::V3
    {
        return Err(Refusal::UnsupportedVueVersion { requested });
    }
    if linter.requested_vapor_mode == Some(true) || linter.vapor_mode {
        return Err(Refusal::UnsupportedVaporMode { requested: true });
    }
    if linter.type_aware_enabled {
        return Err(Refusal::UnsupportedTypeAwareMode);
    }
    if let Some(requested) = &linter.enabled_rules {
        let mut names: Vec<_> = requested
            .iter()
            .filter(|name| linter.is_rule_enabled(name))
            .collect();
        names.sort();
        for name in names {
            if !linter.registry.has_rule(name) {
                return Err(Refusal::UnprovidedRule { rule: name.clone() });
            }
        }
    }
    for &name in linter
        .script_rules
        .iter()
        .chain(linter.css_rules)
        .chain(linter.musea_rules)
    {
        if linter.is_rule_enabled(name) {
            return Err(Refusal::UnprovidedRule {
                rule: String::new(name),
            });
        }
    }
    linter
        .registry
        .rules()
        .iter()
        .filter(|rule| linter.is_rule_enabled(rule.meta().name))
        .map(|instance| {
            let meta = instance.meta();
            let rule =
                instance
                    .as_native_sfc_template_rule()
                    .ok_or_else(|| Refusal::UnprovidedRule {
                        rule: String::new(meta.name),
                    })?;
            Ok(Callback {
                name: meta.name,
                rule,
                severity: meta.default_severity,
            })
        })
        .collect()
}

struct Callback<'o> {
    name: &'static str,
    rule: &'o dyn NativeSfcTemplateRule,
    severity: Severity,
}
