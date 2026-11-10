//! The configured rule's ordinary-script callback; no separate rule registration.

mod component;
mod mutations;
mod targets;
#[cfg(test)]
mod tests;

use super::{META, NoMutatingPropsOptions};
use crate::rules::script::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use oxc_ast::ast::{
    ExportDefaultDeclarationKind, Expression, ImportDeclarationSpecifier, ObjectExpression,
    Program, Statement,
};

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
        for options in component_options(program) {
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
            if let Some(setup) = component::setup_function(options) {
                for mutation in
                    super::script_mutations::find_setup_parameter_mutations(_source, program, setup)
                {
                    if self.shallow_only
                        && matches!(
                            mutation.kind,
                            super::script_mutations::ScriptPropMutationKind::Deep
                        )
                    {
                        continue;
                    }
                    result.add_diagnostic(
                        crate::diagnostic::LintDiagnostic::error(
                            META.name,
                            vize_l0::cstr!(
                                "Unexpected mutation of prop '{}' in <script>",
                                mutation.target
                            ),
                            offset as u32 + mutation.span.start,
                            offset as u32 + mutation.span.end,
                        )
                        .with_help(
                            "Use a local ref or emit an event instead of mutating props directly",
                        ),
                    );
                }
            }
        }
        if let Some(diagnostics) = result.diagnostics.get_mut(previous..) {
            diagnostics.sort_unstable_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
        }
    }
}

fn component_options<'a>(program: &'a Program<'a>) -> Vec<&'a ObjectExpression<'a>> {
    let mut options = Vec::new();
    for statement in &program.body {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        match &export.declaration {
            ExportDefaultDeclarationKind::ObjectExpression(object) => options.push(object.as_ref()),
            ExportDefaultDeclarationKind::CallExpression(call)
                if imported_define_component(program, &call.callee) =>
            {
                if let Some(Expression::ObjectExpression(object)) = call
                    .arguments
                    .first()
                    .and_then(|argument| argument.as_expression())
                {
                    options.push(object.as_ref());
                }
            }
            _ => {}
        }
    }
    options
}

fn imported_define_component(program: &Program<'_>, callee: &Expression<'_>) -> bool {
    let Expression::Identifier(callee) = callee.get_inner_expression() else {
        return false;
    };
    program.body.iter().any(|statement| {
        let Statement::ImportDeclaration(import) = statement else {
            return false;
        };
        import.source.value == "vue"
            && import.specifiers.iter().flatten().any(|specifier| {
                matches!(specifier, ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if specifier.local.name == callee.name
                        && specifier.imported.name() == "defineComponent")
            })
    })
}
