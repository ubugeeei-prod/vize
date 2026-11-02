//! Consume exact original-Program entry identity without changing the indexing expression.
use oxc_ast::ast::Expression;
use vize_croquis::{Croquis, croquis::ComponentUsage};
use vize_relief::{ExpressionNode, RootNode};

pub(super) fn value<'a>(
    summary: &'a Croquis,
    root: &RootNode<'_>,
    usage: &ComponentUsage,
) -> Option<&'a str> {
    let ExpressionNode::Simple(expression) = super::is_expression_node(root, usage.start)? else {
        return None;
    };
    let js = expression.js_ast?.as_expression()?;
    if js.raw != expression.content {
        return None;
    }
    let Expression::ComputedMemberExpression(member) = js.ast else {
        return None;
    };
    let (Expression::Identifier(object), Expression::Identifier(_key)) =
        (&member.object, &member.expression)
    else {
        return None;
    };
    let value = vize_croquis::facts::components::homogeneous_index_value(
        summary,
        object.name.as_str(),
        usage.scope_id,
    )?;
    // The new alias must not make its own annotation self-referential.
    (value != usage.name.as_str()).then_some(value)
}

#[cfg(test)]
mod tests {
    use vize_armature::parse;
    use vize_carton::{Allocator, cstr};
    use vize_croquis::{Drawer, DrawerOptions};

    #[test]
    fn exact_import_annotation_refuses_self_capture() {
        let allocator = Allocator::new();
        let template = r#"<component :is="map[foo]" />"#;
        let (root, errors) = parse(&allocator, template);
        assert!(errors.is_empty());
        for (import, expected) in [("Child", Some("Child")), ("__vize_dynamic_is_0", None)] {
            let script = cstr!(
                "import {import} from './Child.vue'; const map = {{ a: {import}, b: {import} }}; const foo = 'a' as K;"
            );
            let mut drawer = Drawer::with_options(DrawerOptions::full());
            drawer.draw_script_setup_with_generic(script.as_str(), Some("K extends 'a' | 'b'"));
            drawer.draw_template(&root);
            let summary = drawer.finish();
            assert_eq!(summary.component_usages.len(), 1);
            assert_eq!(
                super::value(&summary, &root, &summary.component_usages[0]),
                expected
            );
        }
    }
}
