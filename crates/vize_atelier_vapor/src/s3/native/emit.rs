//! Projection of the admitted native payload to the shared emitter IR.
//! The legacy template AST is neither an input nor a source of payloads.

mod control;
mod spans;
mod text;

use vize_atelier_core::{
    RootNode, SimpleExpressionNode, SourceLocation, codegen::spanned::SpannedText,
};
use vize_carton::{Allocator, Box, FxHashMap, Vec, ensure_sufficient_stack};

use super::{AuthoredSpan, Content, NativeArtifact};
use crate::generate::spans::{TemplateSpans, VaporSourceSpans};
use crate::ir::{
    BlockIRNode, ChildRefIRNode, EventModifiers, IREffect, IRProp, OperationNode, RootIRNode,
    SetEventIRNode, SetPropIRNode,
};
use spans::{argument_offset, tag_offset, value_offset};

pub(super) fn emit<'a>(
    artifact: NativeArtifact<'a>,
    allocator: &'a Allocator,
    source: &'a str,
    scope_id: Option<&str>,
    spans: bool,
) -> (RootIRNode<'a>, Option<VaporSourceSpans>) {
    let mut ir = RootIRNode {
        node: RootNode::new(allocator, ""),
        source,
        template: Default::default(),
        template_index_map: Default::default(),
        root_template_indexes: Vec::new_in(&allocator),
        component: Vec::new_in(&allocator),
        directive: Vec::new_in(&allocator),
        block: BlockIRNode::new(allocator),
        has_template_ref: false,
        has_deferred_v_show: false,
        templates: Vec::new_in(&allocator),
        element_template_map: Default::default(),
        standalone_text_elements: Default::default(),
    };
    // A node is materialized when it or a descendant in the same template is
    // updated, listened to, or anchors a control-flow insertion.
    let mut dynamic = std::vec![false; artifact.nodes.len()];
    for (index, node) in artifact.nodes.iter().enumerate().rev() {
        dynamic[index] = !node.bindings.is_empty()
            || matches!(
                node.content,
                Content::Text { dynamic: true, .. } | Content::If { .. } | Content::For(_)
            )
            || node.children.iter().any(|child| dynamic[*child]);
    }
    let root = artifact.root;
    let mut emitter = Emitter {
        allocator,
        artifact,
        dynamic,
        ir: &mut ir,
        next_id: 0,
        scope_id,
        source: spans.then_some(source),
        template_spans: TemplateSpans::default(),
        units: FxHashMap::default(),
        else_units: FxHashMap::default(),
    };
    let block = emitter.block(root);
    let spans = spans.then(|| {
        VaporSourceSpans::native(
            std::mem::take(&mut emitter.template_spans),
            std::mem::take(&mut emitter.units),
            std::mem::take(&mut emitter.else_units),
        )
    });
    ir.block = block;
    (ir, spans)
}

struct Emitter<'a, 'b> {
    allocator: &'a Allocator,
    artifact: NativeArtifact<'a>,
    dynamic: std::vec::Vec<bool>,
    ir: &'b mut RootIRNode<'a>,
    next_id: usize,
    scope_id: Option<&'b str>,
    /// The authored source, only for map-requesting compiles.
    source: Option<&'a str>,
    /// Anchors of each emitted template, by template index (P3-9).
    template_spans: TemplateSpans,
    /// Condition / loop-source start -> authored start of its element.
    units: FxHashMap<u32, u32>,
    /// Condition start -> authored start of the following `v-else` element.
    else_units: FxHashMap<u32, u32>,
}

impl<'a> Emitter<'a, '_> {
    /// One render block: an element instantiated from its own template, or a
    /// control-flow operation at the template root.
    fn block(&mut self, root: usize) -> BlockIRNode<'a> {
        ensure_sufficient_stack(|| {
            let mut block = BlockIRNode::new(self.allocator);
            let id = if matches!(self.artifact.nodes[root].content, Content::Element { .. }) {
                let id = self.id();
                let mut template = SpannedText::default();
                self.element(root, Some(id), &mut template, &mut block);
                // Nested blocks registered their templates first.
                let index = self.ir.templates.len();
                self.ir.element_template_map.insert(id, index);
                self.ir
                    .templates
                    .push(self.allocator.alloc_str(template.as_str()));
                if self.source.is_some() {
                    self.template_spans
                        .insert(index, template.anchors().to_vec());
                }
                id
            } else {
                self.control(root, None, &mut block)
            };
            block.returns.push(id);
            block
        })
    }

    fn element(
        &mut self,
        index: usize,
        id: Option<usize>,
        template: &mut SpannedText,
        block: &mut BlockIRNode<'a>,
    ) {
        ensure_sufficient_stack(|| self.element_inner(index, id, template, block));
    }

    fn element_inner(
        &mut self,
        index: usize,
        id: Option<usize>,
        template: &mut SpannedText,
        block: &mut BlockIRNode<'a>,
    ) {
        let Content::Element {
            tag,
            tag_span,
            ref attributes,
        } = self.artifact.nodes[index].content
        else {
            unreachable!("element payload checked by the caller")
        };
        template.push_str("<");
        // S3 keeps element and attribute spans; the tokens inside them are
        // located by the HTML syntax of that authored text (P3-9).
        self.mark(template, self.token(tag_span, |raw| tag_offset(raw, tag)));
        template.push_str(tag);
        for (name, value, span) in attributes {
            template.push_str(" ");
            self.mark(
                template,
                self.token(*span, |raw| raw.starts_with(name).then_some(0)),
            );
            template.push_str(name);
            if let Some(value) = value {
                template.push_str("=\"");
                self.mark(
                    template,
                    self.token(*span, |raw| value_offset(raw, name, value)),
                );
                escape(template, value);
                template.push_str("\"");
            }
        }
        if let Some(scope_id) = self.scope_id {
            template.push_str(" ");
            template.push_str(scope_id);
        }
        template.push_str(">");
        for binding_index in 0..self.artifact.nodes[index].bindings.len() {
            let binding = &self.artifact.nodes[index].bindings[binding_index];
            let element = id.expect("binding target is materialized");
            let [name_span, value_span] = binding.spans;
            let value_span = self.trimmed(value_span);
            let name_span = self.token(name_span, |raw| argument_offset(raw, binding.name));
            let key = self.expression(binding.name, true, name_span);
            if binding.event {
                let modifiers = EventModifiers::from_names(
                    self.allocator,
                    Some(binding.name),
                    binding.modifiers.iter().copied(),
                );
                let name = modifiers.event_name(binding.name);
                let delegate = modifiers.can_delegate(name);
                let key = self.expression(name, true, None);
                let value = Some(self.expression(binding.value, false, Some(value_span)));
                block
                    .operation
                    .push(OperationNode::SetEvent(SetEventIRNode {
                        element,
                        key,
                        value,
                        modifiers,
                        delegate,
                        effect: false,
                    }));
            } else {
                let values = self.values(binding.value, value_span);
                self.effect(
                    OperationNode::SetProp(SetPropIRNode {
                        element,
                        tag,
                        camel: false,
                        prop_modifier: false,
                        prop: IRProp {
                            key,
                            values,
                            is_component: false,
                        },
                    }),
                    block,
                );
            }
        }
        self.children(index, id, template, block);
        if !vize_carton::is_void_tag(tag) {
            template.push_str("</");
            template.push_str(tag);
            template.push_str(">");
        }
    }

    fn children(
        &mut self,
        index: usize,
        parent: Option<usize>,
        template: &mut SpannedText,
        block: &mut BlockIRNode<'a>,
    ) {
        let only_child = self.artifact.nodes[index].children.len() == 1;
        let mut cursor = 0;
        let mut offset = 0;
        while cursor < self.artifact.nodes[index].children.len() {
            let child = self.artifact.nodes[index].children[cursor];
            match self.artifact.nodes[child].content {
                Content::Text { .. } => {
                    cursor = self.text_run(index, cursor, parent, offset, template, block);
                }
                Content::Element { .. } => {
                    let id = self.dynamic[child].then(|| {
                        self.child(
                            parent.expect("dynamic ancestry is materialized"),
                            offset,
                            block,
                        )
                    });
                    self.element(child, id, template, block);
                    cursor += 1;
                }
                Content::If { .. } | Content::For(_) => {
                    // The placeholder is the authored insertion position; the
                    // control block inserts before it.
                    template.push_str("<!---->");
                    let parent = parent.expect("control-flow parent is materialized");
                    let anchor = self.child(parent, offset, block);
                    self.control(child, Some((parent, anchor, only_child)), block);
                    cursor += 1;
                }
            }
            offset += 1;
        }
    }

    fn id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn child(&mut self, parent_id: usize, offset: usize, block: &mut BlockIRNode<'a>) -> usize {
        let child_id = self.id();
        block
            .operation
            .push(OperationNode::ChildRef(ChildRefIRNode {
                child_id,
                parent_id,
                offset,
            }));
        child_id
    }

    fn expression(
        &self,
        value: &'a str,
        is_static: bool,
        span: Option<AuthoredSpan>,
    ) -> Box<'a, SimpleExpressionNode<'a>> {
        // Map-requesting compiles keep the payload's authored span.
        let loc = span
            .filter(|_| self.source.is_some())
            .map_or(SourceLocation::STUB, |(start, end)| {
                SourceLocation::new(start, end)
            });
        Box::new_in(
            SimpleExpressionNode::new(value, is_static, loc),
            &self.allocator,
        )
    }

    fn values(
        &self,
        value: &'a str,
        span: AuthoredSpan,
    ) -> Vec<'a, Box<'a, SimpleExpressionNode<'a>>> {
        let mut values = Vec::new_in(&self.allocator);
        values.push(self.expression(value, false, Some(span)));
        values
    }

    fn effect(&mut self, op: OperationNode<'a>, block: &mut BlockIRNode<'a>) {
        let mut operations = Vec::new_in(&self.allocator);
        operations.push(op);
        block.effect.push(IREffect { operations });
    }
}

fn escape(output: &mut SpannedText, value: &str) {
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            ch => output.push_str(ch.encode_utf8(&mut [0; 4])),
        }
    }
}
