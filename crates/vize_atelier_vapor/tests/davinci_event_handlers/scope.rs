//! Check class/decorator binding ownership in the emitted syntax.

use oxc_ast::ast::{Class, ClassElement, Expression, Statement};
use oxc_ast_visit::{Visit, walk};

#[test]
fn vdom_class_decorators_keep_the_enclosing_binding() {
    check("vdom", "context");
}

#[test]
fn vapor_class_decorators_keep_the_enclosing_binding() {
    check("vapor", "prefix");
}

fn check(backend: &str, mode: &str) {
    let code = super::compile(
        backend,
        mode,
        r#"<button @click="() => save([(@C class C { read() { return C } }), C])"></button>"#,
    );
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &code, oxc_span::SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{code}");
    let mut visitor = ClassBindings { classes: 0 };
    visitor.visit_program(&parsed.program);
    assert_eq!(visitor.classes, 1, "must inspect the emitted class");
}

struct ClassBindings {
    classes: usize,
}

impl<'a> Visit<'a> for ClassBindings {
    fn visit_class(&mut self, class: &Class<'a>) {
        self.classes += 1;
        assert_eq!(class.decorators.len(), 1);
        let Expression::StaticMemberExpression(reference) = &class.decorators[0].expression else {
            panic!("class decorator must resolve the enclosing context binding");
        };
        assert_eq!(reference.property.name, "C");
        let Expression::Identifier(context) = &reference.object else {
            panic!("decorator binding must be on the context");
        };
        assert_eq!(context.name, "_ctx");
        let ClassElement::MethodDefinition(method) = &class.body.body[0] else {
            panic!("expected the authored method");
        };
        let Statement::ReturnStatement(return_) =
            &method.value.body.as_ref().unwrap().statements[0]
        else {
            panic!("expected the authored return");
        };
        let Some(Expression::Identifier(self_name)) = &return_.argument else {
            panic!("class method must retain the local self binding");
        };
        assert_eq!(self_name.name, "C");
        walk::walk_class(self, class);
    }
}
