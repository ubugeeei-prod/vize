//! Registration facts that remain available after a fatal template recovery.
//!
//! A malformed template must not erase an independently valid Options API
//! `components` declaration. The ordinary Croquis path owns the complete
//! registration model; this narrow AST fallback is only for template rules
//! that still run on the recovered tree when that shared analysis is withheld.

use crate::context::LintContext;
use oxc_ast::ast::{
    Argument, ExportDefaultDeclarationKind, Expression, ObjectExpression, ObjectPropertyKind,
    Program, PropertyKey, Statement,
};
use vize_croquis::naming::to_pascal_case;

pub(super) fn options_api_registers(ctx: &LintContext<'_>, tag: &str) -> bool {
    ctx.sfc_scripts()
        .and_then(|scripts| scripts.script())
        .is_some_and(|parsed| program_registers(&parsed.program, tag))
}

fn program_registers(program: &Program<'_>, tag: &str) -> bool {
    program.body.iter().any(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return false;
        };
        options_object(&export.declaration)
            .and_then(components_object)
            .is_some_and(|components| {
                components.properties.iter().any(|property| {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        return false;
                    };
                    // eslint-plugin-vue compares the PascalCase form of the
                    // authored tag with the registration key itself. Preserve
                    // that asymmetry: a lower-camel shorthand such as
                    // `{ svgIcon }` does not make `<svg-icon>` a target of the
                    // default registered-components-only policy.
                    static_key_name(&property.key).is_some_and(|name| to_pascal_case(tag) == name)
                })
            })
    })
}

fn options_object<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call)
            if is_component_options_callee(&call.callee) =>
        {
            call.arguments.iter().find_map(|argument| match argument {
                Argument::ObjectExpression(object) => Some(object.as_ref()),
                _ => None,
            })
        }
        _ => None,
    }
}

fn is_component_options_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(identifier) => matches!(
            identifier.name.as_str(),
            "defineComponent" | "_defineComponent" | "extend"
        ),
        Expression::StaticMemberExpression(member) => match member.property.name.as_str() {
            "defineComponent" | "_defineComponent" => true,
            "extend" => matches!(
                &member.object,
                Expression::Identifier(identifier)
                    if matches!(identifier.name.as_str(), "Vue" | "_Vue")
            ),
            _ => false,
        },
        _ => false,
    }
}

fn components_object<'a>(options: &'a ObjectExpression<'a>) -> Option<&'a ObjectExpression<'a>> {
    options.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if static_key_name(&property.key) != Some("components") {
            return None;
        }
        match &property.value {
            Expression::ObjectExpression(object) => Some(object.as_ref()),
            _ => None,
        }
    })
}

fn static_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    if let PropertyKey::TemplateLiteral(template) = key {
        return template
            .expressions
            .is_empty()
            .then(|| template.quasis.first())
            .flatten()
            .and_then(|quasi| quasi.value.cooked.as_ref().map(|value| value.as_str()));
    }
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::program_registers;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    #[test]
    fn reads_direct_and_recognized_wrapped_options_api_registrations() {
        for source in [
            "export default { components: { Popover, 'algolia-search': Search } }",
            "export default defineComponent({ components: { Popover } })",
        ] {
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
            assert!(program_registers(&parsed.program, "popover"));
        }
    }

    #[test]
    fn ignores_unrecognized_options_wrappers() {
        let allocator = Allocator::default();
        let parsed = Parser::new(
            &allocator,
            "export default create({ components: { Popover } })",
            SourceType::ts(),
        )
        .parse();
        assert!(!program_registers(&parsed.program, "popover"));
    }

    #[test]
    fn preserves_authored_registration_key_casing() {
        let allocator = Allocator::default();
        let parsed = Parser::new(
            &allocator,
            "export default { components: { svgIcon, Popover } }",
            SourceType::ts(),
        )
        .parse();
        assert!(!program_registers(&parsed.program, "svg-icon"));
        assert!(program_registers(&parsed.program, "popover"));
    }
}
