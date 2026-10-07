//! Native shorthand object bindings also refer to the source property.

use oxc_ast::ast::{BindingPattern, Program, Statement};

pub(super) fn property_linked(program: &Program<'_>, name: &str) -> bool {
    program.body.iter().any(|statement| {
        let Statement::VariableDeclaration(declaration) = statement else {
            return false;
        };
        declaration
            .declarations
            .iter()
            .any(|declarator| pattern(&declarator.id, name))
    })
}

fn pattern(binding: &BindingPattern<'_>, name: &str) -> bool {
    match binding {
        BindingPattern::ObjectPattern(object) => {
            object.properties.iter().any(|property| {
                (property.shorthand && identifier(&property.value, name))
                    || pattern(&property.value, name)
            }) || object
                .rest
                .as_ref()
                .is_some_and(|rest| identifier(&rest.argument, name))
        }
        BindingPattern::ArrayPattern(array) => array
            .elements
            .iter()
            .flatten()
            .any(|element| pattern(element, name)),
        BindingPattern::AssignmentPattern(assignment) => pattern(&assignment.left, name),
        BindingPattern::BindingIdentifier(_) => false,
    }
}

fn identifier(binding: &BindingPattern<'_>, name: &str) -> bool {
    match binding {
        BindingPattern::BindingIdentifier(id) => id.name == name,
        BindingPattern::AssignmentPattern(assignment) => identifier(&assignment.left, name),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::property_linked;
    use oxc_span::SourceType;
    use vize_l0::Allocator;

    #[test]
    fn only_shorthand_object_binding_names_have_project_property_identity() {
        let source = "const local = 1; const { show, label: aliased, fallback = 1, outer: { nested } } = value; const [{ arrayNested }] = list; const { ...rest } = value; function inner({ parameter }: Shape) { return parameter; } // const { fake } = value";
        let allocator = Allocator::default();
        let parsed = vize_croquis::script_parser::parse_program_for_analysis(
            &allocator,
            source,
            SourceType::ts(),
        );
        assert!(!parsed.panicked);
        for (name, expected) in [
            ("local", false),
            ("show", true),
            ("aliased", false),
            ("fallback", true),
            ("nested", true),
            ("arrayNested", true),
            ("rest", true),
            ("parameter", false),
            ("fake", false),
        ] {
            assert_eq!(property_linked(&parsed.program, name), expected, "{name}");
        }
    }
}
