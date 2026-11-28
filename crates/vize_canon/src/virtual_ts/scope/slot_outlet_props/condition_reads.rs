//! The declared slots each `v-if` branch renders through a `<slot>` outlet.

use vize_relief::{
    ElementNode, ExpressionNode, IfBranchNode, PropNode, RootNode, TemplateChildNode,
};

use crate::virtual_ts::expressions::SlotConditionReads;

pub(super) fn collect_slot_condition_reads(root: &RootNode<'_>) -> SlotConditionReads {
    let mut walk = ConditionWalk {
        source: root.source,
        conditions: Vec::new(),
        reads: SlotConditionReads::default(),
    };
    for child in &root.children {
        walk.child(child);
    }
    walk.reads
}

struct ConditionWalk<'a> {
    source: &'a str,
    /// The authored `v-if` / `v-else-if` conditions around the current node.
    conditions: Vec<((u32, u32), &'a str)>,
    reads: SlotConditionReads,
}

impl<'a> ConditionWalk<'a> {
    fn child(&mut self, child: &TemplateChildNode<'a>) {
        match child {
            TemplateChildNode::Element(element) => self.element(element),
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    self.branch(branch);
                }
            }
            TemplateChildNode::IfBranch(branch) => self.branch(branch),
            TemplateChildNode::For(node) => {
                for child in &node.children {
                    self.child(child);
                }
            }
            _ => {}
        }
    }

    fn branch(&mut self, branch: &IfBranchNode<'a>) {
        let opened = self.open(branch.condition.as_ref());
        for child in &branch.children {
            self.child(child);
        }
        self.close(opened);
    }

    fn element(&mut self, element: &ElementNode<'a>) {
        let condition = element.props.iter().find_map(|prop| match prop {
            PropNode::Directive(directive)
                if directive.name == "if" || directive.name == "else-if" =>
            {
                directive.exp.as_ref()
            }
            _ => None,
        });
        let opened = self.open(condition);
        if element.tag == "slot"
            && let Some(name) = static_slot_name(element)
        {
            for &(range, condition) in &self.conditions {
                self.reads.record(range, condition, name);
            }
        }
        for child in &element.children {
            self.child(child);
        }
        self.close(opened);
    }

    fn open(&mut self, condition: Option<&ExpressionNode<'a>>) -> bool {
        let Some(condition) = condition else {
            return false;
        };
        let content = match condition {
            ExpressionNode::Simple(simple) => simple.content,
            ExpressionNode::Compound(compound) => compound.loc.span.slice(self.source),
        };
        let span = condition.loc().span;
        self.conditions.push(((span.start, span.end), content));
        true
    }

    fn close(&mut self, opened: bool) {
        if opened {
            self.conditions.pop();
        }
    }
}

/// The name a `<slot>` renders when it is static: its `name` attribute, or
/// `default` without one. A bound `:name` renders a runtime slot.
fn static_slot_name<'a>(element: &ElementNode<'a>) -> Option<&'a str> {
    let mut name = "default";
    for prop in &element.props {
        match prop {
            PropNode::Attribute(attr) if attr.name == "name" => {
                name = attr.value.as_ref()?.content;
            }
            PropNode::Directive(directive)
                if directive.name == "bind"
                    && directive.arg.as_ref().is_none_or(|arg| match arg {
                        ExpressionNode::Simple(simple) => simple.content == "name",
                        ExpressionNode::Compound(_) => true,
                    }) =>
            {
                return None;
            }
            _ => {}
        }
    }
    Some(name)
}
