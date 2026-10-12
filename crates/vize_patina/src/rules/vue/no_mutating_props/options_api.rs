//! The configured rule's ordinary-script callback; no separate rule registration.

mod component;
mod mutations;
mod targets;
#[cfg(test)]
mod tests;

use super::{META, NoMutatingPropsOptions};
use crate::rules::script::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use oxc_ast::ast::{ExportDefaultDeclarationKind, Program, Statement};

// Trait metadata projects the canonical rule metadata; it is never registered.
static SCRIPT_META: ScriptRuleMeta = ScriptRuleMeta {
    name: META.name,
    description: META.description,
    default_severity: META.default_severity,
};

#[derive(Default)]
pub(super) struct OptionsApiScript(NoMutatingPropsOptions);

impl From<NoMutatingPropsOptions> for OptionsApiScript {
    fn from(options: NoMutatingPropsOptions) -> Self {
        Self(options)
    }
}

impl std::ops::Deref for OptionsApiScript {
    type Target = NoMutatingPropsOptions;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ScriptRule for OptionsApiScript {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &SCRIPT_META
    }

    fn uses_ast(&self) -> bool {
        true
    }

    fn runs_on_script_setup(&self) -> bool {
        false
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        if !sfc.is_sfc || sfc.is_script_setup {
            return;
        }
        let previous = result.diagnostics.len();
        for statement in &program.body {
            let Statement::ExportDefaultDeclaration(export) = statement else {
                continue;
            };
            let ExportDefaultDeclarationKind::ObjectExpression(options) = &export.declaration
            else {
                continue;
            };
            let props = component::prop_names(options);
            if props.is_empty() {
                continue;
            }
            let allow_object_assign = !mutations::has_object_binding(program);
            component::visit_instance_functions(options, |function| {
                mutations::check(
                    function,
                    &props,
                    self.shallow_only,
                    allow_object_assign,
                    offset,
                    result,
                );
            });
        }
        if let Some(diagnostics) = result.diagnostics.get_mut(previous..) {
            diagnostics.sort_unstable_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
        }
    }
}
