//! script/no-deprecated-destroyed-lifecycle
//!
//! Disallow the Vue 2 `destroyed` and `beforeDestroy` Options API hooks.
//!
//! Vue 3 renamed these hooks to `unmounted` and `beforeUnmount`. This rule
//! mirrors eslint-plugin-vue's `vue/no-deprecated-destroyed-lifecycle`,
//! including its safe fixes for methods, properties, static computed keys,
//! and shorthand properties.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use oxc_ast::ast::{ObjectProperty, ObjectPropertyKind, Program, PropertyKey};
use oxc_span::GetSpan;
use vize_carton::{CompactString, cstr};
use vize_croquis::script_parser::collect_options_object;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-deprecated-destroyed-lifecycle",
    description: "Disallow deprecated destroyed and beforeDestroy lifecycle hooks",
    default_severity: Severity::Error,
};

const HOOKS: [DeprecatedHook; 2] = [
    DeprecatedHook {
        old_name: "destroyed",
        new_name: "unmounted",
        message: "The `destroyed` lifecycle hook is deprecated. Use `unmounted` instead.",
    },
    DeprecatedHook {
        old_name: "beforeDestroy",
        new_name: "beforeUnmount",
        message: "The `beforeDestroy` lifecycle hook is deprecated. Use `beforeUnmount` instead.",
    },
];

struct DeprecatedHook {
    old_name: &'static str,
    new_name: &'static str,
    message: &'static str,
}

/// Disallow Vue 2's renamed destroyed lifecycle hooks.
pub struct NoDeprecatedDestroyedLifecycle;

impl ScriptRule for NoDeprecatedDestroyedLifecycle {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn runs_on_script_setup(&self) -> bool {
        false
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let Some(options) = collect_options_object(program) else {
            return;
        };

        // Match upstream's two independent `findProperty` calls: at most the
        // first occurrence of each deprecated hook is reported.
        for hook in &HOOKS {
            let Some(property) = options.properties.iter().find_map(|candidate| {
                let ObjectPropertyKind::ObjectProperty(property) = candidate else {
                    return None;
                };
                property_has_static_name(property, hook.old_name).then_some(property.as_ref())
            }) else {
                continue;
            };
            report(property, hook, source, offset, result);
        }
    }
}

fn property_has_static_name(property: &ObjectProperty<'_>, expected: &str) -> bool {
    if property.computed && !is_static_computed_key(&property.key) {
        return false;
    }
    property.key.is_specific_static_name(expected)
}

fn is_static_computed_key(key: &PropertyKey<'_>) -> bool {
    match key {
        PropertyKey::StringLiteral(_) => true,
        PropertyKey::TemplateLiteral(template) => template.expressions.is_empty(),
        _ => false,
    }
}

fn report(
    property: &ObjectProperty<'_>,
    hook: &DeprecatedHook,
    source: &str,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let span = property.key.span();
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    let mut diagnostic = LintDiagnostic::error(META.name, hook.message, start, end)
        .with_label(
            cstr!("`{}` is a Vue 2 lifecycle hook", hook.old_name),
            start,
            end,
        )
        .with_help(cstr!("Rename this hook to `{}` for Vue 3.", hook.new_name));

    if let Some(edit) = replacement_edit(property, hook.new_name, source, offset) {
        diagnostic = diagnostic.with_fix(Fix::new(
            cstr!("Rename `{}` to `{}`", hook.old_name, hook.new_name),
            edit,
        ));
    }
    result.add_diagnostic(diagnostic);
}

fn replacement_edit(
    property: &ObjectProperty<'_>,
    replacement: &'static str,
    source: &str,
    offset: usize,
) -> Option<TextEdit> {
    let span = property.key.span();
    let (start, end, new_text): (u32, u32, CompactString) = if property.computed {
        let key = source.get(span.start as usize..span.end as usize)?;
        let delimiter = key.as_bytes().first().copied()?;
        if key.len() < 2
            || key.as_bytes().last().copied() != Some(delimiter)
            || !matches!(delimiter, b'\'' | b'"' | b'`')
        {
            return None;
        }
        (
            span.start + 1,
            span.end - 1,
            CompactString::new(replacement),
        )
    } else if property.shorthand {
        (span.start, span.start, cstr!("{}:", replacement))
    } else {
        (span.start, span.end, CompactString::new(replacement))
    };

    let offset = offset as u32;
    Some(TextEdit::replace(offset + start, offset + end, new_text))
}

#[cfg(test)]
mod tests;
