//! Edge-case coverage for JSX/TSX lowering without partial string matching.

mod common;

use common::{as_attribute, as_directive, as_element, lower_one, lower_one_tsx, simple_content};
use vize_carton::Bump;
use vize_relief::{ElementType, ExpressionNode, TemplateChildNode};

fn expr_text<'a>(expr: &'a ExpressionNode<'a>) -> &'a str {
    simple_content(expr)
}

fn element_tag<'a>(child: &'a TemplateChildNode<'a>) -> &'a str {
    as_element(child).tag.as_str()
}

#[test]
fn fragment_preserves_mixed_child_order_exactly() {
    let bump = Bump::new();
    let root = lower_one(
        &bump,
        r#"const View = () => (
  <>
    <header id="top">Title</header>
    {subtitle}
    <Widget.Panel active data-id="panel" />
  </>
);"#,
    );

    assert_eq!(root.children.len(), 3);

    let header = as_element(&root.children[0]);
    assert_eq!(header.tag.as_str(), "header");
    assert_eq!(header.tag_type, ElementType::Element);
    assert_eq!(header.props.len(), 1);
    let id = as_attribute(&header.props[0]);
    assert_eq!(id.name.as_str(), "id");
    assert_eq!(id.value.as_ref().unwrap().content.as_str(), "top");

    match &root.children[1] {
        TemplateChildNode::Interpolation(interpolation) => {
            assert_eq!(expr_text(&interpolation.content), "subtitle");
        }
        other => panic!("expected interpolation, got {:?}", other.node_type()),
    }

    let panel = as_element(&root.children[2]);
    assert_eq!(panel.tag.as_str(), "Widget.Panel");
    assert_eq!(panel.tag_type, ElementType::Component);
    assert_eq!(panel.props.len(), 2);
    assert_eq!(as_attribute(&panel.props[0]).name.as_str(), "active");
    assert_eq!(as_attribute(&panel.props[1]).name.as_str(), "data-id");
    assert_eq!(
        as_attribute(&panel.props[1])
            .value
            .as_ref()
            .unwrap()
            .content
            .as_str(),
        "panel"
    );
}

#[test]
fn tsx_generic_component_map_lowers_aliases_and_key_exactly() {
    let bump = Bump::new();
    let root = lower_one_tsx(
        &bump,
        r#"const Select = <T extends string>({ options }: { options: T[] }) => (
  <ul>{options.map((option, index) => <li key={option}>{option + index}</li>)}</ul>
);"#,
    );

    let ul = as_element(&root.children[0]);
    assert_eq!(ul.tag.as_str(), "ul");
    assert_eq!(ul.children.len(), 1);

    let for_node = match &ul.children[0] {
        TemplateChildNode::For(for_node) => for_node,
        other => panic!("expected v-for, got {:?}", other.node_type()),
    };
    assert_eq!(expr_text(&for_node.source), "options");
    assert_eq!(
        expr_text(for_node.value_alias.as_ref().expect("value alias")),
        "option"
    );
    assert_eq!(
        expr_text(for_node.key_alias.as_ref().expect("key alias")),
        "index"
    );
    assert!(for_node.object_index_alias.is_none());
    assert_eq!(for_node.children.len(), 1);

    let li = as_element(&for_node.children[0]);
    assert_eq!(li.tag.as_str(), "li");
    assert_eq!(li.props.len(), 1);
    let key = as_directive(&li.props[0]);
    assert_eq!(key.name.as_str(), "bind");
    assert_eq!(expr_text(key.arg.as_ref().expect("key arg")), "key");
    assert_eq!(expr_text(key.exp.as_ref().expect("key exp")), "option");

    match &li.children[0] {
        TemplateChildNode::Interpolation(interpolation) => {
            assert_eq!(expr_text(&interpolation.content), "option + index");
        }
        other => panic!("expected interpolation, got {:?}", other.node_type()),
    }
}

#[test]
fn nested_ternary_records_each_branch_condition_and_child() {
    let bump = Bump::new();
    let root = lower_one(
        &bump,
        r#"const Badge = () => (
  <div>{state === "idle" ? <p>Idle</p> : state === "busy" ? <em>Busy</em> : <strong>Done</strong>}</div>
);"#,
    );

    let div = as_element(&root.children[0]);
    assert_eq!(div.children.len(), 1);

    let if_node = match &div.children[0] {
        TemplateChildNode::If(if_node) => if_node,
        other => panic!("expected if node, got {:?}", other.node_type()),
    };
    assert_eq!(if_node.branches.len(), 3);
    assert_eq!(
        expr_text(if_node.branches[0].condition.as_ref().expect("condition")),
        r#"state === "idle""#
    );
    assert_eq!(element_tag(&if_node.branches[0].children[0]), "p");
    assert_eq!(
        expr_text(if_node.branches[1].condition.as_ref().expect("condition")),
        r#"state === "busy""#
    );
    assert_eq!(element_tag(&if_node.branches[1].children[0]), "em");
    assert!(if_node.branches[2].condition.is_none());
    assert_eq!(element_tag(&if_node.branches[2].children[0]), "strong");
}

#[test]
fn directive_arguments_modifiers_and_plain_attrs_are_kept_separate() {
    let bump = Bump::new();
    let root = lower_one(
        &bump,
        r#"const Form = () => <input id="email" v-model={model.email} v-focus:lazy={focusOptions} />;"#,
    );
    let input = as_element(&root.children[0]);
    assert_eq!(input.tag.as_str(), "input");
    assert_eq!(input.props.len(), 3);

    let id = as_attribute(&input.props[0]);
    assert_eq!(id.name.as_str(), "id");
    assert_eq!(id.value.as_ref().unwrap().content.as_str(), "email");

    let model = as_directive(&input.props[1]);
    assert_eq!(model.name.as_str(), "model");
    assert!(model.arg.is_none());
    assert_eq!(
        expr_text(model.exp.as_ref().expect("model exp")),
        "model.email"
    );

    let focus = as_directive(&input.props[2]);
    assert_eq!(focus.name.as_str(), "focus");
    assert_eq!(expr_text(focus.arg.as_ref().expect("focus arg")), "lazy");
    assert_eq!(
        expr_text(focus.exp.as_ref().expect("focus exp")),
        "focusOptions"
    );
}

#[test]
fn scoped_style_extraction_removes_style_child_and_keeps_interpolations() {
    let bump = Bump::new();
    let src = r#"const Themed = ({ color, gap }: { color: string; gap: number }) => (
  <>
    <section class="box">content</section>
    <style scoped>{`
      .box {
        color: ${color};
        gap: ${gap}px;
      }
    `}</style>
  </>
);"#;
    let out = vize_atelier_jsx::lower_source(&bump, src, vize_atelier_jsx::JsxLang::Tsx);
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.roots.len(), 1);

    let root = &out.roots[0];
    assert_eq!(root.root.children.len(), 1);
    assert_eq!(element_tag(&root.root.children[0]), "section");
    assert_eq!(
        root.scoped_style_exprs
            .iter()
            .map(|expr| expr.content.as_str())
            .collect::<std::vec::Vec<_>>(),
        vec!["color", "gap"]
    );
    assert_eq!(
        root.scoped_style_exprs
            .iter()
            .map(|expr| &src[expr.start as usize..expr.end as usize])
            .collect::<std::vec::Vec<_>>(),
        vec!["color", "gap"]
    );
}
