use super::extract_retained_slot_prop_bindings;
use crate::drawer::{Drawer, DrawerOptions};
use crate::scope::ScopeKind;
use vize_carton::{Allocator, CompactString};
use vize_relief::{ExpressionNode, PropNode, RootNode, SimpleExpressionNode, TemplateChildNode};

fn slot_expression<'n, 'a>(root: &'n RootNode<'a>) -> &'n SimpleExpressionNode<'a> {
    let TemplateChildNode::Element(element) = &root.children[0] else {
        panic!("the original component must be retained");
    };
    let PropNode::Directive(directive) = &element.props[0] else {
        panic!("the original slot directive must be retained");
    };
    let Some(ExpressionNode::Simple(expression)) = directive.exp.as_ref() else {
        panic!("the original slot value must be retained");
    };
    expression
}

#[test]
fn retained_slot_declarations_keep_nested_defaults_rest_types_and_authored_byte_offsets() {
    for (pattern, names) in [
        (
            "  { item: { id }, list: [first = fallback(outer, argument), , ...tail], ...rest }: SlotProps",
            vec!["id", "first", "tail", "rest"],
        ),
        (
            "プロパティ: SlotProps, index = fallback(outer, argument), ...残り: Item[]",
            vec!["プロパティ", "index", "残り"],
        ),
        ("\t props /* trailing */ ", vec!["props"]),
    ] {
        let allocator = Allocator::new();
        let template = format!("<Panel v-slot=\"{pattern}\" />");
        let (root, errors) = vize_armature::parse(&allocator, &template);
        assert!(errors.is_empty(), "{pattern}: {errors:?}");
        let expression = slot_expression(&root);
        let retained = expression.js_ast.expect("slot parameters must be retained");
        assert_eq!(retained.raw, pattern);
        assert!(
            retained.as_expression().is_none(),
            "slot role must remain distinct"
        );
        let actual = extract_retained_slot_prop_bindings(expression)
            .expect("the whole original slot parameter goal must parse");
        let expected: Vec<_> = names
            .iter()
            .map(|name| (CompactString::new(name), pattern.find(name).unwrap() as u32))
            .collect();
        assert_eq!(actual.as_slice(), expected, "{pattern}");
    }
}

#[test]
fn retained_slot_errors_and_guard_refusals_never_reparse_the_header() {
    for pattern in ["value + other", "{ value: }", "value, ...rest, next"] {
        let allocator = Allocator::new();
        let template = format!("<Panel v-slot=\"{pattern}\" />");
        let (root, _) = vize_armature::parse(&allocator, &template);
        let expression = slot_expression(&root);
        let parameters = expression
            .js_ast
            .expect("a parser error retains its original slot goal")
            .as_slot_parameters()
            .expect("the role remains slot parameters");
        assert!(parameters.is_err(), "{pattern}");
        assert!(
            extract_retained_slot_prop_bindings(expression).is_none(),
            "{pattern}"
        );
    }

    let allocator = Allocator::new();
    let pattern = format!("{}value{}", "[".repeat(512), "]".repeat(512));
    let template = format!("<Panel v-slot=\"{pattern}\" />");
    let (root, _) = vize_armature::parse(&allocator, &template);
    let expression = slot_expression(&root);
    let retained = expression
        .js_ast
        .expect("guard refusal preserves the slot role to prevent a retry");
    assert!(retained.as_expression().is_none());
    assert!(
        retained
            .as_slot_parameters()
            .expect("the refused role remains slot parameters")
            .expect_err("guard refusal never enters the parameter parser")
            .is_empty()
    );
    assert!(extract_retained_slot_prop_bindings(expression).is_none());
}

#[test]
fn retained_slot_binding_reads_refuse_stale_missing_and_expression_role_payloads() {
    let allocator = Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, "<Panel v-slot=\"{ value }\" />");
    assert!(errors.is_empty());
    let original = slot_expression(&root);
    let mut stale = SimpleExpressionNode::from_node(original);
    stale.content = "{ other }";
    assert!(extract_retained_slot_prop_bindings(&stale).is_none());
    let mut missing = SimpleExpressionNode::from_node(original);
    missing.js_ast = None;
    assert!(extract_retained_slot_prop_bindings(&missing).is_none());

    let (ordinary, errors) = vize_armature::parse(&allocator, "<Panel :value=\"props\" />");
    assert!(errors.is_empty());
    let ordinary = slot_expression(&ordinary);
    assert!(ordinary.js_ast.unwrap().as_expression().is_some());
    assert!(extract_retained_slot_prop_bindings(ordinary).is_none());
}

#[test]
fn slot_scope_and_component_usage_share_retained_declarations_and_refuse_missing_payloads() {
    const TEMPLATE: &str =
        "<Panel v-slot=\"{ ラベル: value = defaultValue, ...rest }: Props\">{{ value }}</Panel>";
    for retained in [true, false] {
        let allocator = Allocator::new();
        let (mut root, errors) = vize_armature::parse(&allocator, TEMPLATE);
        assert!(errors.is_empty());
        if !retained {
            let TemplateChildNode::Element(element) = &mut root.children[0] else {
                panic!("original component");
            };
            let PropNode::Directive(directive) = &mut element.props[0] else {
                panic!("original slot");
            };
            let Some(ExpressionNode::Simple(expression)) = directive.exp.as_mut() else {
                panic!("original slot value");
            };
            expression.js_ast = None;
        }
        let mut drawer = Drawer::with_options(DrawerOptions::full());
        drawer.draw_template(&root);
        let croquis = drawer.finish();
        let scope = croquis
            .scopes
            .iter()
            .find(|scope| scope.kind == ScopeKind::VSlot)
            .expect("the original slot still owns a lexical scope");
        let component = croquis
            .component_usages
            .iter()
            .find(|usage| usage.name == "Panel")
            .expect("the original component usage is recorded");
        let slot = &component.slots[0];
        if retained {
            assert_eq!(
                slot.scope_vars.as_slice(),
                [CompactString::new("value"), CompactString::new("rest")]
            );
            for name in ["value", "rest"] {
                assert_eq!(
                    scope.get_binding(name).unwrap().declaration_offset,
                    TEMPLATE.find(name).unwrap() as u32
                );
            }
            assert!(
                scope.get_binding("defaultValue").is_none(),
                "initializers are reads, not declarations"
            );
        } else {
            assert!(
                slot.scope_vars.is_empty(),
                "usage must not reconstruct bindings from raw text"
            );
            assert!(
                scope.get_binding("value").is_none(),
                "scope must not reconstruct bindings from raw text"
            );
            assert!(scope.get_binding("rest").is_none());
        }
    }
}
