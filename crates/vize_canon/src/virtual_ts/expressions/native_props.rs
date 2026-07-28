//! Native Vue prop discovery, type assertions, and authored-range mappings.

use super::super::types::{VizeMapping, VizeSubSpan};
use vize_carton::{CompactString, FxHashMap, String, append, is_native_tag};
use vize_croquis::TemplateExpression;
use vize_relief::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, RootNode, TemplateChildNode,
};

pub(crate) type NativePropBindings = FxHashMap<(u32, u32), NativePropBinding>;

pub(crate) struct NativePropBinding {
    tag: CompactString,
    name: CompactString,
    name_start: u32,
    name_end: u32,
}

pub(crate) fn collect_native_prop_bindings(
    root: Option<&RootNode<'_>>,
    enabled: bool,
) -> NativePropBindings {
    let mut bindings = NativePropBindings::default();
    let Some(root) = root.filter(|_| enabled) else {
        return bindings;
    };
    for child in &root.children {
        collect_child_bindings(child, &mut bindings);
    }
    bindings
}

pub(crate) fn generate_native_prop_helpers(ts: &mut String, bindings: &NativePropBindings) {
    if bindings.is_empty() {
        return;
    }
    ts.push_str(
        "  type __VizeNativeElement<Elements, Tag extends PropertyKey> = Tag extends keyof Elements ? Elements[Tag] : unknown;\n",
    );
    ts.push_str(
        "  type __VizeNativeElementProp<Element, Prop extends PropertyKey> = Prop extends keyof Element ? Element[Prop] : unknown;\n",
    );
}

fn collect_child_bindings(child: &TemplateChildNode<'_>, bindings: &mut NativePropBindings) {
    match child {
        TemplateChildNode::Element(element) => collect_element_bindings(element, bindings),
        TemplateChildNode::If(node) => {
            for branch in &node.branches {
                for child in &branch.children {
                    collect_child_bindings(child, bindings);
                }
            }
        }
        TemplateChildNode::IfBranch(branch) => {
            for child in &branch.children {
                collect_child_bindings(child, bindings);
            }
        }
        TemplateChildNode::For(node) => {
            for child in &node.children {
                collect_child_bindings(child, bindings);
            }
        }
        _ => {}
    }
}

fn collect_element_bindings(element: &ElementNode<'_>, bindings: &mut NativePropBindings) {
    if is_native_tag(element.tag.as_str()) {
        for prop in &element.props {
            let PropNode::Directive(directive) = prop else {
                continue;
            };
            if let Some((range, binding)) = native_prop_binding(element, directive) {
                bindings.insert(range, binding);
            }
        }
    }
    for child in &element.children {
        collect_child_bindings(child, bindings);
    }
}

fn native_prop_binding(
    element: &ElementNode<'_>,
    directive: &DirectiveNode<'_>,
) -> Option<((u32, u32), NativePropBinding)> {
    if directive.name != "bind" {
        return None;
    }
    let ExpressionNode::Simple(argument) = directive.arg.as_ref()? else {
        return None;
    };
    if !argument.is_static || !is_booleanish_native_prop(argument.content.as_str()) {
        return None;
    }
    let expression = directive.exp.as_ref()?;
    let expression_location = expression.loc();
    Some((
        (
            expression_location.start.offset,
            expression_location.end.offset,
        ),
        NativePropBinding {
            tag: element.tag.clone(),
            name: argument.content.clone(),
            name_start: argument.loc.start.offset,
            name_end: argument.loc.end.offset,
        },
    ))
}

fn is_booleanish_native_prop(name: &str) -> bool {
    matches!(
        name,
        "allowfullscreen"
            | "async"
            | "autofocus"
            | "autoplay"
            | "capture"
            | "checked"
            | "contenteditable"
            | "controls"
            | "default"
            | "defer"
            | "disabled"
            | "draggable"
            | "formnovalidate"
            | "hidden"
            | "indeterminate"
            | "inert"
            | "ismap"
            | "itemscope"
            | "loop"
            | "multiple"
            | "muted"
            | "nomodule"
            | "novalidate"
            | "open"
            | "playsinline"
            | "readonly"
            | "required"
            | "reversed"
            | "selected"
            | "spellcheck"
    )
}

pub(super) fn generate_native_prop_statement(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expr: &TemplateExpression,
    native_prop: &NativePropBinding,
    generated_expression: &str,
    template_offset: u32,
    indent: &str,
) {
    let name_src_start = (template_offset + native_prop.name_start) as usize;
    let name_src_end = (template_offset + native_prop.name_end) as usize;
    let value_src_start = (template_offset + expr.start) as usize;
    let value_src_end = (template_offset + expr.end) as usize;
    let gen_stmt_start = ts.len();

    append!(*ts, "{indent}const ");
    let check_name_start = ts.len();
    append!(*ts, "__vize_native_prop_check_{}", expr.start);
    let check_name_end = ts.len();
    ts.push_str(": __VizeNativeElementProp<__VizeNativeElement<");
    ts.push_str("import('vue').NativeElements, ");
    push_ts_string_literal(ts, native_prop.tag.as_str());
    ts.push_str(">, ");
    push_ts_string_literal(ts, native_prop.name.as_str());
    ts.push_str("> = (");
    let value_gen_start = ts.len();
    ts.push_str(generated_expression);
    let value_gen_end = ts.len();
    ts.push_str(");\n");
    let gen_stmt_end = ts.len();
    append!(
        *ts,
        "{indent}void __vize_native_prop_check_{}; // VBind\n",
        expr.start
    );

    mappings.push(VizeMapping {
        gen_range: gen_stmt_start..gen_stmt_end,
        src_range: name_src_start..value_src_end,
        sub_spans: vec![
            VizeSubSpan {
                gen_range: check_name_start..check_name_end,
                src_range: name_src_start..name_src_end,
            },
            VizeSubSpan {
                gen_range: value_gen_start..value_gen_end,
                src_range: value_src_start..value_src_end,
            },
        ],
    });
}

fn push_ts_string_literal(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            _ => output.push(character),
        }
    }
    output.push('"');
}
