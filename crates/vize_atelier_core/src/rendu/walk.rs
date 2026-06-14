//! Allocation-free Rendu views over Relief trees.

use crate::{
    ForNode, IfBranchNode, RootNode, TemplateChildNode,
    rendu::{RenduExprRef, RenduOp, RenduSpan},
};

/// Cheap summary collected while walking a Relief tree as Rendu operations.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct RenduWalkSummary {
    pub operations: usize,
    pub max_depth: usize,
}

/// Walk a transformed Relief root as borrowed Rendu operations.
///
/// This does not allocate and does not persist an IR. It gives render, inspect,
/// and profiling lanes a common view that can later be replaced by an arena
/// plate where that proves useful.
pub fn walk_rendu_ops<'a>(root: &'a RootNode<'a>, mut visit: impl FnMut(RenduOp<'a>)) {
    for child in root.children.iter() {
        walk_child(child, &mut visit);
    }
}

/// Collect a cheap operation/depth summary without building a Rendu block.
pub fn summarize_rendu_ops(root: &RootNode<'_>) -> RenduWalkSummary {
    let mut summary = RenduWalkSummary::default();
    walk_rendu_ops(root, |_| {
        summary.operations += 1;
    });
    summary.max_depth = max_rendu_depth(root);
    summary
}

fn max_rendu_depth(root: &RootNode<'_>) -> usize {
    root.children
        .iter()
        .map(|child| child_depth(child, 0))
        .max()
        .unwrap_or(0)
}

fn walk_child<'a>(child: &'a TemplateChildNode<'a>, visit: &mut impl FnMut(RenduOp<'a>)) {
    visit(RenduOp::from_template_child(child));
    match child {
        TemplateChildNode::Element(element) => {
            // Props attach to the nearest preceding Element op, emitted before
            // the element's children so the run is unambiguous.
            for prop in element.props.iter() {
                visit(RenduOp::from_prop(prop));
            }
            for child in element.children.iter() {
                walk_child(child, visit);
            }
        }
        TemplateChildNode::If(node) => {
            for branch in node.branches.iter() {
                walk_if_branch(branch, visit);
            }
        }
        TemplateChildNode::IfBranch(branch) => {
            walk_if_branch(branch, visit);
        }
        TemplateChildNode::For(node) => {
            walk_for(node, visit);
        }
        TemplateChildNode::Text(_)
        | TemplateChildNode::Comment(_)
        | TemplateChildNode::Interpolation(_)
        | TemplateChildNode::TextCall(_)
        | TemplateChildNode::CompoundExpression(_)
        | TemplateChildNode::Hoisted(_) => {}
    }
}

fn walk_if_branch<'a>(branch: &'a IfBranchNode<'a>, visit: &mut impl FnMut(RenduOp<'a>)) {
    visit(RenduOp::IfBranch {
        condition: branch.condition.as_ref().map(RenduExprRef::from_expression),
        span: RenduSpan::from_location(&branch.loc),
    });
    for child in branch.children.iter() {
        walk_child(child, visit);
    }
}

fn walk_for<'a>(node: &'a ForNode<'a>, visit: &mut impl FnMut(RenduOp<'a>)) {
    for child in node.children.iter() {
        walk_child(child, visit);
    }
}

fn child_depth(child: &TemplateChildNode<'_>, depth: usize) -> usize {
    match child {
        TemplateChildNode::Element(element) => element
            .children
            .iter()
            .map(|child| child_depth(child, depth + 1))
            .max()
            .unwrap_or(depth),
        TemplateChildNode::If(node) => node
            .branches
            .iter()
            .map(|branch| {
                branch
                    .children
                    .iter()
                    .map(|child| child_depth(child, depth + 2))
                    .max()
                    .unwrap_or(depth + 1)
            })
            .max()
            .unwrap_or(depth),
        TemplateChildNode::IfBranch(branch) => branch
            .children
            .iter()
            .map(|child| child_depth(child, depth + 1))
            .max()
            .unwrap_or(depth),
        TemplateChildNode::For(node) => node
            .children
            .iter()
            .map(|child| child_depth(child, depth + 1))
            .max()
            .unwrap_or(depth),
        TemplateChildNode::Text(_)
        | TemplateChildNode::Comment(_)
        | TemplateChildNode::Interpolation(_)
        | TemplateChildNode::TextCall(_)
        | TemplateChildNode::CompoundExpression(_)
        | TemplateChildNode::Hoisted(_) => depth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Allocator, TransformOptions, lane::transform, parse, rendu::RenduElementKind};

    #[test]
    fn walks_nested_relief_nodes_as_rendu_operations() {
        let allocator = Allocator::default();
        let bump = allocator.as_bump();
        let (root, errors) = parse(bump, "<div><span>{{ msg }}</span><!--ok--></div>");
        assert!(errors.is_empty());

        let mut ops = std::vec::Vec::new();
        walk_rendu_ops(&root, |op| ops.push(op));

        assert!(matches!(
            ops[0],
            RenduOp::Element {
                tag: "div",
                kind: RenduElementKind::Element,
                ..
            }
        ));
        assert!(matches!(
            ops[1],
            RenduOp::Element {
                tag: "span",
                kind: RenduElementKind::Element,
                ..
            }
        ));
        assert!(matches!(ops[2], RenduOp::Interpolation { .. }));
        assert!(matches!(ops[3], RenduOp::Comment { content: "ok", .. }));

        let summary = summarize_rendu_ops(&root);
        assert_eq!(summary.operations, 4);
        assert_eq!(summary.max_depth, 2);
    }

    #[test]
    fn element_props_surface_as_attribute_and_directive_ops() {
        let allocator = Allocator::default();
        let bump = allocator.as_bump();
        let (root, errors) = parse(
            bump,
            "<button id=\"b\" :class=\"cls\" @click=\"onClick\">X</button>",
        );
        assert!(errors.is_empty());

        let mut ops = std::vec::Vec::new();
        walk_rendu_ops(&root, |op| ops.push(op));

        // Element op first, then its props in source order, then the text child.
        assert!(matches!(ops[0], RenduOp::Element { tag: "button", .. }));
        assert!(matches!(
            ops[1],
            RenduOp::Attribute {
                name: "id",
                value: Some("b"),
                ..
            }
        ));
        assert!(matches!(ops[2], RenduOp::Directive { name: "bind", .. }));
        assert!(matches!(ops[3], RenduOp::Directive { name: "on", .. }));
        assert!(matches!(ops[4], RenduOp::Text { content: "X", .. }));

        // The bind directive carries its arg ("class") and expression ("cls").
        if let RenduOp::Directive { arg, exp, .. } = ops[2] {
            assert_eq!(arg.map(RenduExprRef::text), Some("class"));
            assert_eq!(exp.map(RenduExprRef::text), Some("cls"));
        }
    }

    #[test]
    fn for_walk_preserves_value_key_and_index_aliases_from_parsed_source() {
        let allocator = Allocator::default();
        let bump = allocator.as_bump();
        let (mut root, errors) = parse(
            bump,
            "<li v-for=\"(item, key, index) in items\">{{ item }}</li>",
        );
        assert!(errors.is_empty());
        let diagnostics = transform(bump, &mut root, TransformOptions::default(), None);
        assert!(diagnostics.is_empty());

        let mut for_aliases = None;
        walk_rendu_ops(&root, |op| {
            if let RenduOp::For {
                source,
                value,
                key,
                index,
                ..
            } = op
            {
                for_aliases = Some((
                    source.text(),
                    value.map(RenduExprRef::text),
                    key.map(RenduExprRef::text),
                    index.map(RenduExprRef::text),
                ));
            }
        });

        assert_eq!(
            for_aliases,
            Some(("items", Some("item"), Some("key"), Some("index"))),
            "v-for aliases should survive parse -> transform -> Rendu walk"
        );
    }

    #[test]
    fn walks_transformed_control_flow_as_structured_rendu() {
        let allocator = Allocator::default();
        let bump = allocator.as_bump();
        let (mut root, errors) = parse(bump, "<div v-if=\"ok\">A</div><p v-else>B</p>");
        assert!(errors.is_empty());
        let diagnostics = transform(bump, &mut root, TransformOptions::default(), None);
        assert!(diagnostics.is_empty());

        let mut has_if = false;
        let mut branches = 0;
        let mut elements = 0;
        walk_rendu_ops(&root, |op| match op {
            RenduOp::If { .. } => has_if = true,
            RenduOp::IfBranch { .. } => branches += 1,
            RenduOp::Element {
                kind: RenduElementKind::Element,
                ..
            } => elements += 1,
            _ => {}
        });

        assert!(has_if);
        assert_eq!(branches, 2);
        assert_eq!(elements, 2);
    }
}
