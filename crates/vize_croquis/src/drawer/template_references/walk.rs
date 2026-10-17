use super::{Capture, MAX_DEPTH, TemplateReferenceGapKind};
use vize_armature::parser::TemplateSyntaxRecord;
use vize_l0::Span;
use vize_relief::{ElementNode, ExpressionNode, IfBranchNode, PropNode, TemplateChildNode};

impl Capture<'_, '_> {
    pub(super) fn walk(&mut self, children: &[TemplateChildNode<'_>], depth: usize) {
        if depth > MAX_DEPTH {
            self.traversal_refused = true;
            self.gap(
                Span::new(0, self.source.len() as u32),
                TemplateReferenceGapKind::TraversalLimit,
            );
            return;
        }
        for child in children {
            if self.traversal_refused {
                break;
            }
            if !self.spend(1, child_span(child)) {
                break;
            }
            match child {
                TemplateChildNode::Element(element) => self.element(element, depth),
                TemplateChildNode::Interpolation(node) => self.expression(&node.content),
                TemplateChildNode::If(node) => {
                    for branch in &node.branches {
                        if !self.spend(1, branch.loc.span) {
                            break;
                        }
                        self.branch(branch, depth);
                    }
                }
                TemplateChildNode::IfBranch(branch) => self.branch(branch, depth),
                TemplateChildNode::For(node) => {
                    let old_len = self.locals.len();
                    let old_unknown = self.unknown_scope;
                    let syntax = self.syntax;
                    let record = syntax.records_for(self.source).and_then(|records| {
                        records
                            .iter()
                            .find(|record| self.is_original_for_collection(record, &node.source))
                    });
                    if let Some(record) = record {
                        self.for_scope(record);
                    } else {
                        self.gap(node.loc.span, TemplateReferenceGapKind::MissingSyntax);
                        self.unknown_scope = true;
                    }
                    self.walk(&node.children, depth + 1);
                    self.locals.truncate(old_len);
                    self.unknown_scope = old_unknown;
                }
                TemplateChildNode::Text(_) | TemplateChildNode::Comment(_) => {}
                _ => self.gap(child_span(child), TemplateReferenceGapKind::UnsupportedNode),
            }
        }
    }

    fn branch(&mut self, branch: &IfBranchNode<'_>, depth: usize) {
        if let Some(condition) = &branch.condition {
            self.expression(condition);
        }
        if let Some(PropNode::Directive(directive)) = &branch.user_key {
            if let Some(argument) = &directive.arg {
                self.dynamic_argument(argument);
            }
            if let Some(value) = &directive.exp {
                self.expression(value);
            }
        }
        self.walk(&branch.children, depth + 1);
    }

    fn element(&mut self, element: &ElementNode<'_>, depth: usize) {
        if !self.spend(element.props.len(), element.loc.span) {
            return;
        }
        if element.props.iter().any(|prop| {
            matches!(prop,
                PropNode::Attribute(attribute) if attribute.name.strip_prefix("v-pre")
                    .is_some_and(|tail| tail.is_empty() || tail.starts_with([':', '.']))
            )
        }) {
            return;
        }
        let old_len = self.locals.len();
        let old_unknown = self.unknown_scope;
        // Vue 3 evaluates conditions before same-element aliases exist.
        for prop in &element.props {
            if let PropNode::Directive(directive) = prop
                && matches!(directive.name, "if" | "else-if")
                && let Some(value) = &directive.exp
            {
                self.expression(value);
            }
        }
        for prop in &element.props {
            if let PropNode::Directive(directive) = prop
                && directive.name == "for"
                && let Some(value) = &directive.exp
            {
                let record = self.record(value);
                if let Some(record) = record {
                    self.for_scope(record);
                } else {
                    self.gap(value.loc().span, TemplateReferenceGapKind::MissingSyntax);
                    self.unknown_scope = true;
                }
            }
        }
        // Slot names and ordinary attrs execute outside the slot props. They
        // can read a same-element loop alias, which was introduced above.
        for prop in &element.props {
            let PropNode::Directive(directive) = prop else {
                continue;
            };
            if let Some(argument) = &directive.arg {
                self.dynamic_argument(argument);
            }
            if !matches!(directive.name, "for" | "if" | "else-if" | "slot")
                && let Some(value) = &directive.exp
            {
                self.expression(value);
            }
        }
        for prop in &element.props {
            let record: Option<&TemplateSyntaxRecord<'_>> = match prop {
                PropNode::Directive(directive) if directive.name == "slot" => {
                    let Some(value) = &directive.exp else {
                        continue;
                    };
                    self.record(value)
                }
                PropNode::Attribute(attribute)
                    if self.syntax.legacy_slot_bindings_enabled()
                        && (attribute.name == "slot-scope"
                            || (attribute.name == "scope" && element.tag == "template")) =>
                {
                    let Some(value) = &attribute.value else {
                        continue;
                    };
                    self.syntax.syntax_for(
                        self.source,
                        oxc_span::Span::new(value.loc.span.start, value.loc.span.end),
                        value.content,
                    )
                }
                _ => continue,
            };
            if let Some(record) = record {
                self.slot_scope(record);
            } else {
                self.gap(element.loc.span, TemplateReferenceGapKind::MissingSyntax);
                self.unknown_scope = true;
            }
        }
        self.walk(&element.children, depth + 1);
        self.locals.truncate(old_len);
        self.unknown_scope = old_unknown;
    }

    fn dynamic_argument(&mut self, argument: &ExpressionNode<'_>) {
        if matches!(argument, ExpressionNode::Simple(node) if node.is_static) {
            return;
        }
        self.expression(argument);
    }
}

fn child_span(child: &TemplateChildNode<'_>) -> Span {
    match child {
        TemplateChildNode::Element(node) => node.loc.span,
        TemplateChildNode::Text(node) => node.loc.span,
        TemplateChildNode::Comment(node) => node.loc.span,
        TemplateChildNode::Interpolation(node) => node.loc.span,
        TemplateChildNode::If(node) => node.loc.span,
        TemplateChildNode::IfBranch(node) => node.loc.span,
        TemplateChildNode::For(node) => node.loc.span,
        TemplateChildNode::TextCall(node) => node.loc.span,
        TemplateChildNode::CompoundExpression(node) => node.loc.span,
        TemplateChildNode::Hoisted(_) => Span::new(0, 0),
    }
}
