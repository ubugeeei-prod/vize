//! The DOM type of every possible single template root.

use vize_carton::{CompactString, FxHashSet, String, append, cstr, is_native_tag};
use vize_croquis::Croquis;
use vize_relief::{ElementNode, Namespace, PropNode, RootNode, TemplateChildNode};

use crate::virtual_ts::{
    component_reference::component_binding_reference,
    helpers::push_ts_string_literal,
    types::{VirtualTsCheckOptions, VirtualTsOptions},
};

pub(super) fn emit_setup_type(
    ts: &mut String,
    summary: &Croquis,
    options: &VirtualTsOptions,
    type_only_names: &FxHashSet<CompactString>,
    root: Option<&RootNode<'_>>,
    checks: VirtualTsCheckOptions,
) -> bool {
    if !checks.infer_component_dollar_el && !checks.infer_template_dollar_el {
        return false;
    }
    let roots = root.and_then(|root| collect_roots(root.children.as_slice()));
    let mut types = Vec::new();
    for element in roots.iter().flatten() {
        let ty = if is_native_tag(element.tag) {
            let map = match element.ns {
                Namespace::Svg => "SVGElementTagNameMap",
                Namespace::MathMl => "MathMLElementTagNameMap",
                _ => "HTMLElementTagNameMap",
            };
            let mut tag = String::default();
            push_ts_string_literal(&mut tag, element.tag);
            cstr!("__VizeRootNativeElement<{tag}, {map}>")
        } else {
            let reference =
                component_binding_reference(summary, options, type_only_names, element.tag);
            cstr!("__VizeRootComponentEl<typeof {reference}>")
        };
        if !types.contains(&ty) {
            types.push(ty);
        }
    }
    let inferred = !types.is_empty();
    if inferred {
        ts.push_str("  type __VizeRootNativeElement<T extends PropertyKey, M> = T extends keyof M ? M[T] : Element;\n");
        ts.push_str("  type __VizeRootComponentEl<C> = C extends abstract new (...args: any[]) => infer I ? I extends { $el: infer E } ? E : any : C extends (props: any, ctx: any, expose: (exposed: infer E) => any, ...args: any[]) => any ? E extends { $el: infer D } ? D : any : any;\n");
        let ty = types
            .iter()
            .map(|ty| cstr!("({ty})"))
            .collect::<Vec<_>>()
            .join(" | ");
        append!(*ts, "  type __VizeRootEl = {ty};\n");
    } else {
        ts.push_str("  type __VizeRootEl = any;\n");
    }
    if checks.infer_component_dollar_el && inferred {
        ts.push_str("  const __vize_root_el = {} as __VizeRootEl;\n");
        return true;
    }
    false
}

/// Mirrors Vue's root selection without interpreting the truth of a branch.
/// A fragment in any branch invalidates the single-root contract for the SFC.
fn collect_roots<'a>(children: &'a [TemplateChildNode<'a>]) -> Option<Vec<&'a ElementNode<'a>>> {
    let children: Vec<_> = children
        .iter()
        .filter(|child| match child {
            TemplateChildNode::Comment(_) => false,
            TemplateChildNode::Text(text) => !text.content.trim().is_empty(),
            _ => true,
        })
        .collect();
    if children.is_empty() {
        return Some(Vec::new());
    }
    if children.len() > 1 && !is_raw_if_chain(&children) {
        return None;
    }
    let mut roots = Vec::new();
    for child in children {
        match child {
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    roots.extend(collect_roots(branch.children.as_slice())?);
                }
            }
            TemplateChildNode::Element(element) => {
                if has_directive(element, "for") {
                    continue;
                }
                if element.tag == "template"
                    || matches!(
                        element.tag,
                        "Transition"
                            | "transition"
                            | "KeepAlive"
                            | "keep-alive"
                            | "Teleport"
                            | "teleport"
                            | "Suspense"
                            | "suspense"
                    )
                {
                    roots.extend(collect_roots(element.children.as_slice())?);
                } else if element.tag != "slot" {
                    roots.push(&**element);
                }
            }
            _ => {}
        }
    }
    Some(roots)
}

fn is_raw_if_chain(children: &[&TemplateChildNode<'_>]) -> bool {
    let mut ended = false;
    children.iter().enumerate().all(|(index, child)| {
        let TemplateChildNode::Element(element) = child else {
            return false;
        };
        if index == 0 {
            return has_directive(element, "if");
        }
        if ended {
            return false;
        }
        ended = has_directive(element, "else");
        ended || has_directive(element, "else-if")
    })
}

fn has_directive(element: &ElementNode<'_>, name: &str) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(directive) if directive.name == name))
}
