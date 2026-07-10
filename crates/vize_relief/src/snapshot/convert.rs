use crate::{
    ElementNode, ForNode, IfBranchNode, IfNode, RootNode, SourceLocation, TemplateChildNode,
    TextCallNode,
};

use super::{
    ReliefSnapshot, ReliefSnapshotNode, ReliefSnapshotNodeId, SnapshotElement, SnapshotFor,
    SnapshotHoisted, SnapshotIf, SnapshotIfBranch, SnapshotImport, SnapshotTextCall, copy,
};

impl ReliefSnapshot {
    /// Copy an arena-allocated Relief root into an owned cache product.
    pub fn from_root(root: &RootNode<'_>) -> Self {
        let mut builder = SnapshotBuilder::default();
        let children = root
            .children
            .iter()
            .map(|child| builder.push_node(child))
            .collect();
        let nodes = builder.nodes;
        Self {
            source: root.source.clone(),
            location: root.loc.clone(),
            nodes,
            children,
            comments: root.comments.iter().map(copy::comment).collect(),
            helpers: root.helpers.iter().copied().collect(),
            components: root.components.iter().cloned().collect(),
            directives: root.directives.iter().cloned().collect(),
            #[cfg(feature = "_legacy")]
            filters: root.filters.iter().cloned().collect(),
            imports: root
                .imports
                .iter()
                .map(|item| SnapshotImport {
                    expression: copy::simple_expression(&item.exp),
                    path: item.path.clone(),
                })
                .collect(),
            temps: root.temps,
            transformed: root.transformed,
        }
    }
}

impl<'a> From<&RootNode<'a>> for ReliefSnapshot {
    fn from(root: &RootNode<'a>) -> Self {
        Self::from_root(root)
    }
}

#[derive(Default)]
struct SnapshotBuilder {
    nodes: Vec<ReliefSnapshotNode>,
}

impl SnapshotBuilder {
    fn push_node(&mut self, node: &TemplateChildNode<'_>) -> ReliefSnapshotNodeId {
        match node {
            TemplateChildNode::Element(node) => self.push_element(node),
            TemplateChildNode::Text(node) => {
                self.reserve(ReliefSnapshotNode::Text(copy::text(node)))
            }
            TemplateChildNode::Comment(node) => {
                self.reserve(ReliefSnapshotNode::Comment(copy::comment(node)))
            }
            TemplateChildNode::Interpolation(node) => {
                self.reserve(ReliefSnapshotNode::Interpolation(copy::interpolation(node)))
            }
            TemplateChildNode::If(node) => self.push_if(node),
            TemplateChildNode::IfBranch(node) => self.push_if_branch(node),
            TemplateChildNode::For(node) => self.push_for(node),
            TemplateChildNode::TextCall(node) => self.push_text_call(node),
            TemplateChildNode::CompoundExpression(node) => self.reserve(
                ReliefSnapshotNode::CompoundExpression(copy::compound_expression(node)),
            ),
            TemplateChildNode::Hoisted(index) => {
                self.reserve(ReliefSnapshotNode::Hoisted(SnapshotHoisted {
                    index: *index,
                    location: SourceLocation::STUB,
                }))
            }
        }
    }

    fn push_element(&mut self, node: &ElementNode<'_>) -> ReliefSnapshotNodeId {
        let id = self.reserve(ReliefSnapshotNode::Element(SnapshotElement {
            namespace: node.ns,
            tag: node.tag.clone(),
            tag_type: node.tag_type,
            props: node.props.iter().map(copy::property).collect(),
            children: Vec::new(),
            is_self_closing: node.is_self_closing,
            location: node.loc.clone(),
            inner_location: node.inner_loc.clone(),
            hoisted_props_index: node.hoisted_props_index,
        }));
        let children = node
            .children
            .iter()
            .map(|child| self.push_node(child))
            .collect();
        self.set_children(id, children);
        id
    }

    fn push_if(&mut self, node: &IfNode<'_>) -> ReliefSnapshotNodeId {
        let id = self.reserve(ReliefSnapshotNode::If(SnapshotIf {
            branches: Vec::new(),
            location: node.loc.clone(),
        }));
        let branches = node
            .branches
            .iter()
            .map(|branch| self.push_if_branch(branch))
            .collect();
        self.set_children(id, branches);
        id
    }

    fn push_if_branch(&mut self, node: &IfBranchNode<'_>) -> ReliefSnapshotNodeId {
        let id = self.reserve(ReliefSnapshotNode::IfBranch(SnapshotIfBranch {
            condition: node.condition.as_ref().map(copy::expression),
            children: Vec::new(),
            user_key: node.user_key.as_ref().map(copy::property),
            is_template_if: node.is_template_if,
            location: node.loc.clone(),
        }));
        let children = node
            .children
            .iter()
            .map(|child| self.push_node(child))
            .collect();
        self.set_children(id, children);
        id
    }

    fn push_for(&mut self, node: &ForNode<'_>) -> ReliefSnapshotNodeId {
        let id = self.reserve(ReliefSnapshotNode::For(Box::new(SnapshotFor {
            source: copy::expression(&node.source),
            value_alias: node.value_alias.as_ref().map(copy::expression),
            key_alias: node.key_alias.as_ref().map(copy::expression),
            object_index_alias: node.object_index_alias.as_ref().map(copy::expression),
            parse_result: copy::for_parse_result(&node.parse_result),
            children: Vec::new(),
            location: node.loc.clone(),
        })));
        let children = node
            .children
            .iter()
            .map(|child| self.push_node(child))
            .collect();
        self.set_children(id, children);
        id
    }

    fn push_text_call(&mut self, node: &TextCallNode<'_>) -> ReliefSnapshotNodeId {
        self.reserve(ReliefSnapshotNode::TextCall(SnapshotTextCall {
            content: copy::text_call_content(&node.content),
            location: node.loc.clone(),
        }))
    }

    fn reserve(&mut self, node: ReliefSnapshotNode) -> ReliefSnapshotNodeId {
        let id = ReliefSnapshotNodeId::from_index(self.nodes.len());
        self.nodes.push(node);
        id
    }

    fn set_children(&mut self, id: ReliefSnapshotNodeId, children: Vec<ReliefSnapshotNodeId>) {
        match &mut self.nodes[id.index()] {
            ReliefSnapshotNode::Element(node) => node.children = children,
            ReliefSnapshotNode::If(node) => node.branches = children,
            ReliefSnapshotNode::IfBranch(node) => node.children = children,
            ReliefSnapshotNode::For(node) => node.children = children,
            ReliefSnapshotNode::Text(_)
            | ReliefSnapshotNode::Comment(_)
            | ReliefSnapshotNode::Interpolation(_)
            | ReliefSnapshotNode::TextCall(_)
            | ReliefSnapshotNode::CompoundExpression(_)
            | ReliefSnapshotNode::Hoisted(_) => {
                unreachable!("leaf snapshot nodes cannot own syntax children")
            }
        }
    }
}
