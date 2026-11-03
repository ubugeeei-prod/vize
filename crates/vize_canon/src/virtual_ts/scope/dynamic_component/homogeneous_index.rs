//! Consume exact original-Program entry identity without changing the indexing expression.
use oxc_ast::ast::Expression;
use vize_croquis::{Croquis, croquis::ComponentUsage};
use vize_relief::{ExpressionNode, RootNode};

pub(super) fn value(
    summary: &Croquis,
    root: &RootNode<'_>,
    usage: &ComponentUsage,
) -> Option<vize_carton::String> {
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
    let property = vize_croquis::facts::components::homogeneous_index_property(
        summary,
        object.name.as_str(),
        usage.scope_id,
    )?;
    let mut annotation = vize_carton::cstr!("(typeof {})[", object.name);
    crate::virtual_ts::helpers::push_ts_string_literal(&mut annotation, property);
    annotation.push(']');
    Some(annotation)
}

#[cfg(test)]
mod tests {
    use vize_armature::parse;
    use vize_carton::{Allocator, cstr};
    use vize_croquis::{Drawer, DrawerOptions};

    // An object spelled like the alias is never admitted, so the annotation
    // cannot name the alias it declares.
    #[test]
    fn exact_import_annotation_refuses_self_capture() {
        let allocator = Allocator::new();
        for (object, expected) in [
            ("map", Some(Some("(typeof map)[\"a\"]"))),
            ("__vize_dynamic_is_0", None),
        ] {
            let template = cstr!("<component :is=\"{object}[foo]\"></component>");
            let (root, errors) = parse(&allocator, template.as_str());
            assert!(errors.is_empty());
            let script = cstr!(
                "import Child from './Child.vue'; const {object} = {{ a: Child, b: Child }}; const foo = 'a' as K;"
            );
            let mut drawer = Drawer::with_options(DrawerOptions::full());
            drawer.draw_script_setup_with_generic(script.as_str(), Some("K extends 'a' | 'b'"));
            drawer.draw_template(&root);
            let summary = drawer.finish();
            assert_eq!(
                summary
                    .component_usages
                    .first()
                    .map(|usage| super::value(&summary, &root, usage)),
                expected.map(|value| value.map(vize_carton::String::from))
            );
        }
    }
}
