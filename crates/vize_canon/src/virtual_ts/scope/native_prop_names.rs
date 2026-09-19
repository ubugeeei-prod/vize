//! Strict native attribute names are checked against the installed Vue types.

use crate::virtual_ts::VizeMapping;
use vize_carton::{String, append, is_native_tag};
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

pub(super) fn emit(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    root: &RootNode<'_>,
    offset: u32,
) {
    for child in &root.children {
        visit(ts, mappings, child, offset);
    }
}

fn visit(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    child: &TemplateChildNode<'_>,
    offset: u32,
) {
    let TemplateChildNode::Element(element) = child else {
        return;
    };
    let fragment = element.tag == "template" && element.props.iter().any(|prop| {
        matches!(prop,
            PropNode::Directive(d) if matches!(d.name, "for" | "if" | "else-if" | "else" | "slot")
        )
    });
    if is_native_tag(element.tag) && !fragment {
        for prop in &element.props {
            let (name, location) = match prop {
                PropNode::Attribute(attribute) => (attribute.name, &attribute.name_loc),
                PropNode::Directive(directive) if directive.name == "bind" => {
                    let Some(ExpressionNode::Simple(argument)) = &directive.arg else {
                        continue;
                    };
                    if !argument.is_static {
                        continue;
                    }
                    (argument.content, &argument.loc)
                }
                _ => continue,
            };
            // Vue accepts user data attributes independently of DOM typings.
            if name.starts_with("data-") {
                continue;
            }
            let tag = serde_json::to_string(element.tag).expect("string serialization");
            let key = serde_json::to_string(name).expect("string serialization");
            let start = ts.len();
            append!(*ts, "  const __vize_native_key_{}", location.span.start);
            let name_end = ts.len();
            append!(
                *ts,
                ": unknown extends __VizeNativeElement<{tag}> ? unknown : {key} extends keyof __VizeNativeElement<{tag}> ? unknown : __VizeComponentAttrCamel<{key}> extends keyof __VizeNativeElement<{tag}> ? unknown : never = {key};\n"
            );
            mappings.push(VizeMapping {
                gen_range: start + 8..name_end,
                src_range: (offset + location.span.start) as usize
                    ..(offset + location.span.end) as usize,
                sub_spans: Vec::new(),
            });
            append!(*ts, "  void __vize_native_key_{};\n", location.span.start);
        }
    }
    for child in &element.children {
        visit(ts, mappings, child, offset);
    }
}
