//! The hook-trace recorder: a [`MarkupRule`] that writes down everything a
//! rule can observe — every hook with its spans, names, values and
//! modifiers, and the element-level queries (children, attributes, opening
//! items, ancestry) evaluated at the moment the hook fires.

use crate::ir::ByteRange;
use crate::markup::{
    MarkupBinding, MarkupConditional, MarkupContext, MarkupDirective, MarkupDocument,
    MarkupElement, MarkupList, MarkupNode, MarkupRule, MarkupText,
};
use std::cell::RefCell;
use vize_s0::{String, append};

/// Records the full hook trace of one document, one line per event.
#[derive(Default)]
pub struct TraceRecorder {
    lines: RefCell<std::vec::Vec<String>>,
}

impl TraceRecorder {
    /// The recorded lines, in hook order.
    pub fn into_lines(self) -> std::vec::Vec<String> {
        self.lines.into_inner()
    }

    fn push(&self, line: String) {
        self.lines.borrow_mut().push(line);
    }
}

fn range(out: &mut String, range: ByteRange) {
    append!(*out, "{}..{}", range.start, range.end);
}

fn modifier(out: &mut String, modifier: &str) {
    if !out.ends_with('[') {
        out.push(',');
    }
    out.push_str(modifier);
}

/// One child node, as `walk_children` presents it.
pub(super) fn node(out: &mut String, node: MarkupNode<'_>) {
    match node {
        MarkupNode::Element(element) => {
            append!(*out, "E<{}>", element.tag());
            range(out, element.range());
        }
        MarkupNode::Text(text) => {
            append!(*out, "T{:?}", text.content());
            range(out, text.range());
        }
        MarkupNode::Comment(at) => {
            out.push('C');
            range(out, at);
        }
        MarkupNode::Interpolation(at) => {
            out.push('I');
            range(out, at);
        }
        MarkupNode::If(at) => {
            out.push_str("If");
            range(out, at);
        }
        MarkupNode::For(at) => {
            out.push_str("For");
            range(out, at);
        }
        MarkupNode::Other(at) => {
            out.push('O');
            range(out, at);
        }
    }
}

/// The element and every query a rule can ask of it.
pub(super) fn element(out: &mut String, element: &MarkupElement<'_>) {
    append!(
        *out,
        "<{}> kind={:?} component={} ",
        element.tag(),
        element.kind(),
        element.is_component()
    );
    range(out, element.range());
    out.push_str(" children=[");
    element.walk_children(&mut |child| {
        node(out, child);
        out.push(' ');
    });
    out.push_str("] attrs=[");
    element.walk_attributes(&mut |attr| {
        append!(*out, "{}={:?}@", attr.name(), attr.value());
        range(out, attr.range());
        append!(*out, " dyn={} ", attr.is_dynamic());
    });
    out.push_str("] opening=[");
    element.walk_opening_item_ranges(&mut |at| {
        range(out, at);
        out.push(' ');
    });
    append!(
        *out,
        "] key={} text={:?}",
        element.has_key_binding(),
        element.direct_text_content().as_str()
    );
}

pub(super) fn binding(out: &mut String, binding: &MarkupBinding<'_>) {
    append!(
        *out,
        "binding kind={:?} arg={:?} dyn={} static={:?} expr={:?} key={} ",
        binding.kind(),
        binding.arg_name(),
        binding.is_dynamic(),
        binding.static_value(),
        binding.expression(),
        binding.is_key()
    );
    range(out, binding.range());
    out.push_str(" mods=[");
    binding.walk_modifiers(&mut |name| modifier(out, name));
    out.push(']');
}

pub(super) fn directive(out: &mut String, directive: &MarkupDirective<'_>) {
    append!(
        *out,
        "directive name={} kind={:?} arg={:?} ",
        directive.name(),
        directive.kind(),
        directive.arg_name()
    );
    range(out, directive.range());
    out.push_str(" mods=[");
    directive.walk_modifiers(&mut |name| modifier(out, name));
    out.push(']');
}

impl MarkupRule for TraceRecorder {
    fn name(&self) -> &'static str {
        "davinci/markup-trace"
    }

    fn enter_document(&self, _ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        let out = RefCell::new(String::from("document tree=["));
        document.walk_tree(
            &mut |entered| {
                let mut out = out.borrow_mut();
                append!(*out, "+{}@", entered.tag());
                range(&mut out, entered.range());
                out.push(' ');
            },
            &mut |exited| {
                append!(*out.borrow_mut(), "-{} ", exited.tag());
            },
        );
        let mut out = out.into_inner();
        out.push(']');
        self.push(out);
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, entered: &MarkupElement<'a>) {
        let mut out = String::from("enter ");
        element(&mut out, entered);
        out.push_str(" ancestors=[");
        for ancestor in ctx.ancestor_elements() {
            append!(out, "{}@", ancestor.tag());
            range(&mut out, ancestor.range());
            out.push(' ');
        }
        out.push(']');
        self.push(out);
    }

    fn exit_element<'a>(&self, _ctx: &mut MarkupContext<'_, 'a>, exited: &MarkupElement<'a>) {
        let mut out = String::default();
        append!(out, "exit <{}> ", exited.tag());
        range(&mut out, exited.range());
        self.push(out);
    }

    fn enter_binding<'a>(
        &self,
        _ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        entered: &MarkupBinding<'a>,
    ) {
        let mut out = String::default();
        binding(&mut out, entered);
        self.push(out);
    }

    fn enter_directive<'a>(
        &self,
        _ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        entered: &MarkupDirective<'a>,
    ) {
        let mut out = String::default();
        directive(&mut out, entered);
        self.push(out);
    }

    fn enter_conditional<'a>(
        &self,
        _ctx: &mut MarkupContext<'_, 'a>,
        conditional: &MarkupConditional<'a>,
    ) {
        let mut out = String::default();
        append!(
            out,
            "conditional branches={} else={} ",
            conditional.branch_count(),
            conditional.has_else()
        );
        range(&mut out, conditional.range());
        self.push(out);
    }

    fn enter_list<'a>(&self, _ctx: &mut MarkupContext<'_, 'a>, list: &MarkupList<'a>) {
        let mut out = String::default();
        append!(
            out,
            "list source={:?} alias={:?} ",
            list.source_expression(),
            list.value_alias()
        );
        range(&mut out, list.range());
        out.push_str(" repeats=[");
        list.walk_elements(&mut |repeated| {
            append!(out, "{}@", repeated.tag());
            range(&mut out, repeated.range());
            out.push(' ');
        });
        out.push(']');
        self.push(out);
    }

    fn enter_text<'a>(&self, _ctx: &mut MarkupContext<'_, 'a>, text: &MarkupText<'a>) {
        let mut out = String::default();
        append!(
            out,
            "text {:?} significant={} ",
            text.content(),
            text.is_significant()
        );
        range(&mut out, text.range());
        self.push(out);
    }

    fn enter_interpolation(&self, _ctx: &mut MarkupContext<'_, '_>, at: ByteRange) {
        let mut out = String::from("interpolation ");
        range(&mut out, at);
        self.push(out);
    }
}
