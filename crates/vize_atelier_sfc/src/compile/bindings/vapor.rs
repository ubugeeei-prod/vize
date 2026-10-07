//! Match Vapor's separate render scope to the SFC assembler's actual hoists.

use std::borrow::Cow;

use oxc_ast::ast::{BindingPattern, Program, Statement};

use crate::{
    script::hoistable_literal_name,
    types::{BindingMetadata, BindingType},
};

/// Keep enums and module declarations lexical, but read unhoisted setup
/// literals from the setup-state proxy. Preserve the public binding metadata.
pub(in crate::compile) fn for_vapor<'b>(
    bindings: &'b BindingMetadata,
    program: &Option<Program<'_>>,
) -> Cow<'b, BindingMetadata> {
    let mut result = Cow::Borrowed(bindings);
    let Some(program) = program else {
        return result;
    };
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        if hoistable_literal_name(statement).is_some() {
            continue;
        }
        for declarator in &declaration.declarations {
            if let BindingPattern::BindingIdentifier(identifier) = &declarator.id
                && bindings.bindings.get(identifier.name.as_str())
                    == Some(&BindingType::LiteralConst)
                && let Some(binding) = result.to_mut().bindings.get_mut(identifier.name.as_str())
            {
                *binding = BindingType::SetupConst;
            }
        }
    }
    result
}
