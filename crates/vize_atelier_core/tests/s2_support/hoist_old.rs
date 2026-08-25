//! The legacy half of the hoist-decision projection ([`super::hoist`]):
//! decisions read from the *mutated* hoist-armed run, the taint
//! detectors, the two template-level pre-scans, and the legacy tree's
//! shape projection (the pairing contract's old half).
//!
//! The decisions are never replayed on this side: the comparator runs
//! the shipped `hoist_static` itself (a second parse + transform with
//! `hoist_static: true`) and reads what it actually did —
//! `TemplateChildNode::Hoisted` for a whole-vnode hoist,
//! `hoisted_props_index` for a props hoist. What *is* computed here is
//! replay **control**: `get_static_type` (the shipped predicate,
//! exported) tells the walk which arm the shipped driver took, so the
//! comparison's descent always mirrors reality rather than a model.

use vize_atelier_core::codegen::is_constant_simple_expression;
use vize_atelier_core::{
    ElementNode, ElementType, ExpressionNode, PropNode, SimpleExpressionNode, TemplateChildNode,
};
use vize_carton::{Allocator, Span, camelize, is_native_tag};
use vize_ricalco::pass::hoist::constant_for_hoist;
use vize_s2::expr::ExprRef;

use super::surface_old_help::is_excluded_builtin;

/// One legacy hoisting decision, as the mutated tree records it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Untouched.
    None,
    /// `hoisted_props_index` was set: the props object hoisted.
    Props,
    /// The child became `TemplateChildNode::Hoisted`.
    Whole,
}

/// Read the decision off one mutated child position.
pub fn decision_of(child: &TemplateChildNode<'_>) -> Decision {
    match child {
        TemplateChildNode::Hoisted(_) => Decision::Whole,
        TemplateChildNode::Element(el) if el.hoisted_props_index.is_some() => Decision::Props,
        _ => Decision::None,
    }
}

/// The shipped `has_directives`: any directive prop at all.
pub fn has_directives(el: &ElementNode<'_>) -> bool {
    el.props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(_)))
}

/// A directive the S2 lowering still defers (`defer.*`), so the S2
/// facts cannot see it: the excluded-builtin set plus the
/// dynamic-argument `v-model`.
pub fn carries_deferred_builtin(el: &ElementNode<'_>) -> bool {
    el.props.iter().any(|prop| match prop {
        PropNode::Directive(dir) => {
            is_excluded_builtin(dir.name)
                || (dir.name == "model"
                    && match dir.arg.as_ref() {
                        Some(ExpressionNode::Simple(arg)) => !arg.is_static,
                        Some(ExpressionNode::Compound(_)) => true,
                        None => false,
                    })
        }
        PropNode::Attribute(_) => false,
    })
}

/// A direct comment child — the legacy lattice refuses staticness (and
/// the nested-child class) across it, and S2 carries no comment ops.
pub fn has_comment_child(el: &ElementNode<'_>) -> bool {
    el.children
        .iter()
        .any(|child| matches!(child, TemplateChildNode::Comment(_)))
}

/// The two template-level pre-scan verdicts (run over the *unmutated*
/// run-1 tree, so legacy-whole-hoisted subtrees are still visible).
#[derive(Debug, Default, Clone, Copy)]
pub struct TemplateScan {
    /// An owner the two lanes classify differently (legacy element,
    /// S2 component: a non-native tag the legacy default options leave
    /// as an element) — levels and the vnodes flag both diverge, in
    /// both directions, so the whole template is a counted class.
    pub classifier: bool,
    /// A hoist-shape bind whose value the shipped classifier calls
    /// constant and the S2 rule (deliberately weaker) does not — the
    /// legacy lane may hoist more, whole subtrees included, so the
    /// whole template is a counted class.
    pub consts: bool,
}

/// Pre-scan every element of the template (branches and regions
/// included).
pub fn scan_template(children: &[TemplateChildNode<'_>], scan: &mut TemplateScan) {
    for child in children {
        match child {
            TemplateChildNode::Element(el) => {
                if el.tag_type == ElementType::Element && !is_native_tag(el.tag) {
                    scan.classifier = true;
                }
                if el.props.iter().any(const_rule_diverges) {
                    scan.consts = true;
                }
                scan_template(&el.children, scan);
            }
            TemplateChildNode::If(node) => {
                for branch in node.branches.iter() {
                    scan_template(&branch.children, scan);
                }
            }
            TemplateChildNode::IfBranch(branch) => scan_template(&branch.children, scan),
            TemplateChildNode::For(node) => scan_template(&node.children, scan),
            _ => {}
        }
    }
}

/// Whether one prop is a hoist-shape `v-bind` on which the shipped
/// classifier and the S2 rule disagree. The shape gates mirror
/// `hoistable_static_bind_parts`; the two classifiers then run on the
/// same authored text — the shipped one directly (the exported
/// oracle), the S2 one through the same admission rule the lowering
/// uses (`ExprRef::parse_js_in`) and the pass's own
/// [`constant_for_hoist`].
fn const_rule_diverges(prop: &PropNode<'_>) -> bool {
    let PropNode::Directive(dir) = prop else {
        return false;
    };
    if dir.name != "bind" {
        return false;
    }
    let Some(ExpressionNode::Simple(arg)) = dir.arg.as_ref() else {
        return false;
    };
    if !arg.is_static {
        return false;
    }
    let mut has_camel = false;
    let mut has_prop = false;
    let mut has_attr = false;
    for modifier in dir.modifiers.iter() {
        match modifier.content {
            "camel" => has_camel = true,
            "prop" => has_prop = true,
            "attr" => has_attr = true,
            _ => return false,
        }
    }
    let key = if has_camel {
        camelize(arg.content)
    } else if has_prop {
        prefixed('.', arg.content)
    } else if has_attr {
        prefixed('^', arg.content)
    } else {
        vize_carton::String::from(arg.content)
    };
    if matches!(key.as_str(), "ref" | "class") {
        return false;
    }
    let Some(ExpressionNode::Simple(exp)) = dir.exp.as_ref() else {
        return false;
    };
    legacy_constant(exp) != s2_constant(exp.content)
}

fn prefixed(prefix: char, name: &str) -> vize_carton::String {
    let mut out = vize_carton::String::with_capacity(1 + name.len());
    out.push(prefix);
    out.push_str(name);
    out
}

/// The shipped verdict, from the shipped classifier itself.
fn legacy_constant(exp: &SimpleExpressionNode<'_>) -> bool {
    is_constant_simple_expression(exp, None)
}

/// The S2 verdict, through the one shared admission rule.
fn s2_constant(content: &str) -> bool {
    let arena = Allocator::new();
    let expr = ExprRef::parse_js_in(
        &arena,
        content,
        Span::new(0, u32::try_from(content.len()).unwrap_or(u32::MAX)),
    );
    constant_for_hoist(&expr)
}

/// The legacy tree's shape projection — the pairing contract: owner
/// kinds and structural nesting, wrapper templates unwrapped exactly as
/// the S2 lowering unwraps them. Compared byte-for-byte against
/// [`super::hoist::shape_of_s2`]; a mismatch is the S1-scope
/// tree-construction class (`tree_templates`), counted, never walked.
pub fn shape_of(children: &[TemplateChildNode<'_>], out: &mut vize_carton::String) {
    for child in children {
        match child {
            TemplateChildNode::Element(el) => {
                out.push(if el.tag == "slot" {
                    's'
                } else if el.tag_type == ElementType::Component {
                    'c'
                } else {
                    'e'
                });
                out.push('(');
                shape_of(&el.children, out);
                out.push(')');
            }
            TemplateChildNode::If(node) => {
                out.push('i');
                for branch in node.branches.iter() {
                    out.push('[');
                    match &branch.children[..] {
                        [TemplateChildNode::Element(el)] if branch.is_template_if => {
                            shape_of(&el.children, out);
                        }
                        _ => shape_of(&branch.children, out),
                    }
                    out.push(']');
                }
            }
            TemplateChildNode::IfBranch(branch) => shape_of(&branch.children, out),
            TemplateChildNode::For(node) => {
                out.push('f');
                out.push('(');
                match &node.children[..] {
                    [TemplateChildNode::Element(el)] if el.tag == "template" => {
                        shape_of(&el.children, out);
                    }
                    _ => shape_of(&node.children, out),
                }
                out.push(')');
            }
            _ => {}
        }
    }
}
