use super::*;
use oxc_ast::ast::{Expression, Statement};

#[test]
fn actual_constructor_tagged_and_dynamic_import_invocations_cannot_bypass_no_observer() {
    let arena = Allocator::default();
    for (source, family) in [
        (
            "<script setup>/* original */ function Factory() {} let value = new Factory();</script>",
            0,
        ),
        (
            "<script setup>/* original */ function tag() { return 1; } let value = tag`text`;</script>",
            1,
        ),
        (
            "<script setup>/* original */ let value = import('dep');</script>",
            2,
        ),
    ] {
        let observed = Observed::new(&arena, source).unwrap();
        let original = observed.syntax.program().unwrap();
        let Statement::VariableDeclaration(variable) = original.body.last().unwrap() else {
            panic!("variable");
        };
        let [declaration] = variable.declarations.as_slice() else {
            panic!("declaration");
        };
        let expression = declaration.init.as_ref().unwrap();
        assert!(matches!(
            (family, expression),
            (0, Expression::NewExpression(_))
                | (1, Expression::TaggedTemplateExpression(_))
                | (2, Expression::ImportExpression(_))
        ));
        let file = observed.file(&arena).unwrap();
        assert!(file.is_complete(), "neutral original facts");
        assert_eq!(
            kind(observed.view(&file).unwrap()).unwrap(),
            ExposureIssueKind::ScriptCall
        );
        assert!(
            file.bindings()
                .any(|binding| binding.declaration().unwrap().name == "value")
        );
        assert!(core::ptr::eq(observed.syntax.program().unwrap(), original));
        assert_eq!(observed.syntax.comments().count(), 1);
        assert!(core::ptr::eq(file.artifact().source(), source));
    }
}
