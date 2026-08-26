use vize_carton::{CompactString, FxHashMap, FxHashSet};
use vize_croquis::{
    Croquis, ScopeId, TemplateExpression, TemplateExpressionKind,
    croquis::{PassedProp, SpreadProp},
};
use vize_relief::{ElementNode, ExpressionNode, PropNode, RootNode, TemplateChildNode};

use super::SlotOutlet;

pub(super) fn collect_slot_outlets_by_scope(
    summary: &Croquis,
    root: Option<&RootNode<'_>>,
) -> FxHashMap<u32, Vec<SlotOutlet>> {
    let mut outlets = Vec::new();
    let Some(root) = root else {
        return FxHashMap::default();
    };
    for child in &root.children {
        collect_child_outlets(summary, child, &mut outlets, root.source);
    }

    let mut by_scope: FxHashMap<u32, Vec<SlotOutlet>> = FxHashMap::default();
    for outlet in outlets {
        by_scope.entry(outlet.scope_id).or_default().push(outlet);
    }
    by_scope
}

/// Authored v-bind ranges covered by already collected outlets. The v-bind
/// expressions are indexed by start offset once, so each outlet binding costs a
/// binary search instead of a full scan of every template expression.
pub(super) fn slot_outlet_expression_ranges(
    summary: &Croquis,
    by_scope: &FxHashMap<u32, Vec<SlotOutlet>>,
) -> FxHashSet<(u32, u32)> {
    let mut v_binds: Vec<&TemplateExpression> = summary
        .template_expressions
        .iter()
        .filter(|expr| expr.kind == TemplateExpressionKind::VBind)
        .collect();
    v_binds.sort_unstable_by_key(|expr| expr.start);
    let nested_v_bind = |start: u32, end: u32, content: &str| {
        let from = v_binds.partition_point(|expr| expr.start < start);
        v_binds[from..]
            .iter()
            .take_while(|expr| expr.start <= end)
            .find(|expr| expr.end <= end && expr.content.as_str().trim() == content)
            .map(|expr| (expr.start, expr.end))
    };

    let mut ranges = FxHashSet::default();
    for outlet in by_scope.values().flatten() {
        for prop in &outlet.props {
            if prop.is_dynamic
                && let Some(value) = prop.value.as_ref()
                && let Some(range) = nested_v_bind(prop.start, prop.end, value.as_str().trim())
            {
                ranges.insert(range);
            }
        }
        for spread in &outlet.spread_props {
            if let Some(range) =
                nested_v_bind(spread.start, spread.end, spread.expression.as_str().trim())
            {
                ranges.insert(range);
            }
        }
    }
    ranges
}

fn collect_child_outlets(
    summary: &Croquis,
    child: &TemplateChildNode<'_>,
    outlets: &mut Vec<SlotOutlet>,
    source: &str,
) {
    match child {
        TemplateChildNode::Element(element) => {
            collect_element_outlets(summary, element, outlets, source)
        }
        TemplateChildNode::If(node) => {
            for branch in &node.branches {
                for child in &branch.children {
                    collect_child_outlets(summary, child, outlets, source);
                }
            }
        }
        TemplateChildNode::IfBranch(branch) => {
            for child in &branch.children {
                collect_child_outlets(summary, child, outlets, source);
            }
        }
        TemplateChildNode::For(node) => {
            for child in &node.children {
                collect_child_outlets(summary, child, outlets, source);
            }
        }
        _ => {}
    }
}

fn collect_element_outlets(
    summary: &Croquis,
    element: &ElementNode<'_>,
    outlets: &mut Vec<SlotOutlet>,
    source: &str,
) {
    if element.tag == "slot"
        && let Some(outlet) = slot_outlet(summary, element, source)
    {
        outlets.push(outlet);
    }
    for child in &element.children {
        collect_child_outlets(summary, child, outlets, source);
    }
}

fn slot_outlet(summary: &Croquis, element: &ElementNode<'_>, source: &str) -> Option<SlotOutlet> {
    let mut name = CompactString::const_new("default");
    let mut name_is_dynamic = false;
    let mut name_source_range = None;
    let mut props = Vec::new();
    let mut spread_props = Vec::new();
    let mut scope = None;

    for prop in &element.props {
        match prop {
            PropNode::Attribute(attr) => {
                if attr.name == "name" {
                    if let Some(value) = attr.value.as_ref() {
                        name = value.content.into();
                        name_source_range = Some(value.loc.span.start..value.loc.span.end);
                    }
                    continue;
                }
                props.push(PassedProp {
                    name: attr.name.into(),
                    name_is_dynamic: false,
                    value: attr.value.as_ref().map(|value| value.content.into()),
                    start: attr.loc.span.start,
                    end: attr.loc.span.end,
                    is_dynamic: false,
                });
            }
            PropNode::Directive(directive) if directive.name == "bind" => {
                if let Some(ref arg) = directive.arg {
                    let (prop_name, prop_name_is_dynamic) = directive_argument(arg, source);
                    let value = directive
                        .exp
                        .as_ref()
                        .map(|exp| CompactString::new(expression_content(exp, source)))
                        .or_else(|| Some(prop_name.clone()));
                    if prop_name == "name" && !prop_name_is_dynamic {
                        name_is_dynamic = true;
                        name_source_range = None;
                        record_expression_scope(summary, directive.exp.as_ref(), &mut scope);
                        continue;
                    }
                    if prop_name_is_dynamic {
                        continue;
                    }
                    record_expression_scope(summary, directive.exp.as_ref(), &mut scope);
                    props.push(PassedProp {
                        name: prop_name,
                        name_is_dynamic: false,
                        value,
                        start: directive.loc.span.start,
                        end: directive.loc.span.end,
                        is_dynamic: true,
                    });
                } else if let Some(ref exp) = directive.exp {
                    record_expression_scope(summary, Some(exp), &mut scope);
                    spread_props.push(SpreadProp {
                        expression: CompactString::new(expression_content(exp, source)),
                        start: directive.loc.span.start,
                        end: directive.loc.span.end,
                    });
                }
            }
            PropNode::Directive(_) => {}
        }
    }

    if props.is_empty() && spread_props.is_empty() {
        return None;
    }
    let (scope_id, vif_guard) = scope.unwrap_or((ScopeId::ROOT.as_u32(), None));
    Some(SlotOutlet {
        scope_id,
        name,
        name_is_dynamic,
        name_source_range,
        start: element.loc.span.start,
        vif_guard,
        props,
        spread_props,
    })
}

fn directive_argument(arg: &ExpressionNode<'_>, source: &str) -> (CompactString, bool) {
    match arg {
        ExpressionNode::Simple(simple) => (simple.content.into(), !simple.is_static),
        ExpressionNode::Compound(compound) => {
            (CompactString::new(compound.loc.span.slice(source)), true)
        }
    }
}

fn record_expression_scope(
    summary: &Croquis,
    exp: Option<&ExpressionNode<'_>>,
    scope: &mut Option<(u32, Option<CompactString>)>,
) {
    if scope.is_some() {
        return;
    }
    let Some(exp) = exp else {
        return;
    };
    let loc = exp.loc();
    let Some(expr) = template_expression(summary, loc.span.start, loc.span.end) else {
        return;
    };
    *scope = Some((expr.scope_id.as_u32(), expr.vif_guard.clone()));
}

fn template_expression(summary: &Croquis, start: u32, end: u32) -> Option<&TemplateExpression> {
    summary.template_expressions.iter().find(|expr| {
        expr.kind == TemplateExpressionKind::VBind && expr.start == start && expr.end == end
    })
}

fn expression_content<'a>(exp: &'a ExpressionNode<'_>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(simple) => simple.content,
        ExpressionNode::Compound(compound) => compound.loc.span.slice(source),
    }
}
