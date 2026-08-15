//! JSON projection for the WASM template AST API.

use vize_atelier_core::{PropNode, RootNode, TemplateChildNode};

pub(super) fn build_ast_json(root: &RootNode<'_>) -> serde_json::Value {
    fn build_children(children: &[TemplateChildNode<'_>]) -> Vec<serde_json::Value> {
        children.iter().map(build_child_json).collect()
    }

    fn build_child_json(child: &TemplateChildNode<'_>) -> serde_json::Value {
        match child {
            TemplateChildNode::Element(element) => {
                let props: Vec<serde_json::Value> = element
                    .props
                    .iter()
                    .map(|prop| match prop {
                        PropNode::Attribute(attribute) => serde_json::json!({
                            "type": "ATTRIBUTE",
                            "name": attribute.name,
                            "value": attribute.value.as_ref().map(|value| value.content),
                        }),
                        PropNode::Directive(directive) => serde_json::json!({
                            "type": "DIRECTIVE",
                            "name": directive.name,
                            "arg": directive.arg.as_ref().map(|argument| match argument {
                                vize_atelier_core::ExpressionNode::Simple(expression) => expression.content.to_string(),
                                _ => "<compound>".to_string(),
                            }),
                            "exp": directive.exp.as_ref().map(|expression| match expression {
                                vize_atelier_core::ExpressionNode::Simple(expression) => expression.content.to_string(),
                                _ => "<compound>".to_string(),
                            }),
                            "modifiers": directive.modifiers.iter().map(|modifier| modifier.content).collect::<Vec<_>>(),
                        }),
                    })
                    .collect();

                serde_json::json!({
                    "type": "ELEMENT",
                    "tag": element.tag,
                    "tagType": format!("{:?}", element.tag_type),
                    "props": props,
                    "children": build_children(&element.children),
                    "isSelfClosing": element.is_self_closing,
                })
            }
            TemplateChildNode::Text(text) => serde_json::json!({
                "type": "TEXT",
                "content": text.content,
            }),
            TemplateChildNode::Comment(comment) => serde_json::json!({
                "type": "COMMENT",
                "content": comment.content,
            }),
            TemplateChildNode::Interpolation(interpolation) => serde_json::json!({
                "type": "INTERPOLATION",
                "content": match &interpolation.content {
                    vize_atelier_core::ExpressionNode::Simple(expression) => expression.content,
                    _ => "<compound>",
                }
            }),
            _ => serde_json::json!({
                "type": "UNKNOWN"
            }),
        }
    }

    let children = build_children(&root.children);

    serde_json::json!({
        "type": "ROOT",
        "children": children,
        "helpers": root.helpers.iter().map(|helper| helper.name()).collect::<Vec<_>>(),
        "components": root.components.iter().map(|component| component).collect::<Vec<_>>(),
        "directives": root.directives.iter().map(|directive| directive).collect::<Vec<_>>(),
    })
}
