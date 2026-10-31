//! Native shorthand object bindings also refer to the source property.

use oxc_ast::AstKind;
use oxc_ast::ast::{BindingPattern, Program, Statement};
use oxc_ast_visit::Visit;

/// Rename's initial private surface deliberately excludes every nested scope,
/// property/member identity, and shorthand transaction. References retain their
/// original, independent top-level pattern policy below.
pub(super) fn rename_hazard(program: &Program<'_>) -> bool {
    let mut visitor = RenameHazard(false);
    visitor.visit_program(program);
    visitor.0
}

struct RenameHazard(bool);

impl<'a> Visit<'a> for RenameHazard {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::Function(_)
            | AstKind::ArrowFunctionExpression(_)
            | AstKind::Class(_)
            | AstKind::ComputedMemberExpression(_)
            | AstKind::StaticMemberExpression(_)
            | AstKind::PrivateFieldExpression(_) => self.0 = true,
            AstKind::ObjectProperty(property) => self.0 |= property.shorthand || property.computed,
            AstKind::BindingProperty(property) => self.0 |= property.shorthand || property.computed,
            _ => {}
        }
    }
}

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
