//! Semantic JSX component calls for the plain-TypeScript check document.
//!
//! JSX lowering keeps a rich component/prop tree, but the original check
//! codegen retained only dynamic expressions. This module turns component
//! elements into type-only calls whose props parameter comes from the imported
//! SFC constructor or local function component contract.

use std::vec::Vec;

use vize_carton::{String as CompactString, ToCompactString};
use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode};

use crate::virtual_ts::{VizeMapping, VizeSubSpan};

use super::{JsxExpr, push_mapped_expr};

/// Type-only component invocation. The constructor branch consumes the
/// `$props` contract exported by generated SFC modules; the function branch
/// covers locally declared functional components. Unknown component shapes
/// remain a permissive fallback instead of introducing false positives.
pub(super) const HELPER: &str = crate::virtual_ts::JSX_COMPONENT_HELPER;

pub(super) struct JsxComponent {
    tag: JsxExpr,
    props: Vec<JsxComponentProp>,
}

impl JsxComponent {
    /// The tag expression, so a scoped slot under this component can type its
    /// parameter from the component's declared `$slots`.
    pub(super) fn tag(&self) -> &JsxExpr {
        &self.tag
    }
}

enum JsxComponentProp {
    Property {
        name: PropName,
        value: PropValue,
        source_start: u32,
        source_end: u32,
    },
    Spread {
        value: JsxExpr,
        source_start: u32,
        source_end: u32,
    },
}

enum PropName {
    Static {
        content: CompactString,
        start: u32,
        end: u32,
    },
    Computed(JsxExpr),
}

enum PropValue {
    Boolean,
    Expression(JsxExpr),
}

pub(super) fn collect(element: &ElementNode<'_>) -> Option<JsxComponent> {
    if element.tag_type != ElementType::Component {
        return None;
    }
    let tag = component_tag(element)?;
    let props = element
        .props
        .iter()
        .filter_map(|prop| component_prop(element, prop))
        .collect();
    Some(JsxComponent { tag, props })
}

/// Whether `component_prop` already preserves this prop. Props that do not
/// participate in the component contract continue through the general JSX
/// expression collector so directive/model expressions are never lost.
pub(super) fn captures_prop(element: &ElementNode<'_>, prop: &PropNode<'_>) -> bool {
    match prop {
        PropNode::Attribute(attribute) => !is_reserved_prop(attribute.name.as_str()),
        PropNode::Directive(directive) if directive.name.as_str() == "bind" => {
            if directive.exp.is_none() {
                return false;
            }
            match directive.arg.as_ref() {
                None => true,
                Some(arg) => match static_name(arg) {
                    Some(name) => {
                        is_dynamic_component_tag_prop(element, name) || !is_reserved_prop(name)
                    }
                    None => super::expr_of(arg).is_some(),
                },
            }
        }
        PropNode::Directive(_) => false,
    }
}

pub(super) fn render(
    out: &mut CompactString,
    mappings: &mut Vec<VizeMapping>,
    component: &JsxComponent,
) {
    out.push_str("__vize_jsx_component__(");
    push_mapped_expr(out, mappings, &component.tag);
    out.push_str(", ");
    let literal_start = out.len();
    out.push('{');
    for (index, prop) in component.props.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        render_prop(out, mappings, prop);
    }
    out.push('}');
    let literal_end = out.len();
    mappings.push(VizeMapping {
        gen_range: literal_start..literal_end,
        src_range: component.tag.start as usize..component.tag.end as usize,
        sub_spans: Vec::new(),
    });
    out.push(')');
}

fn render_prop(out: &mut CompactString, mappings: &mut Vec<VizeMapping>, prop: &JsxComponentProp) {
    match prop {
        JsxComponentProp::Property {
            name,
            value,
            source_start,
            source_end,
        } => {
            let entry_start = out.len();
            let (name_gen_range, name_src_range) = render_name(out, mappings, name);
            out.push_str(": ");
            match value {
                PropValue::Boolean => out.push_str("true"),
                PropValue::Expression(expression) => {
                    push_mapped_expr(out, mappings, expression);
                }
            }
            let entry_end = out.len();
            mappings.push(VizeMapping {
                gen_range: entry_start..entry_end,
                src_range: *source_start as usize..*source_end as usize,
                sub_spans: name_gen_range
                    .zip(name_src_range)
                    .map(|(gen_range, src_range)| VizeSubSpan {
                        gen_range,
                        src_range,
                    })
                    .into_iter()
                    .collect(),
            });
        }
        JsxComponentProp::Spread {
            value,
            source_start,
            source_end,
        } => {
            let entry_start = out.len();
            out.push_str("...__vize_jsx_component_spread__(");
            push_mapped_expr(out, mappings, value);
            out.push(')');
            let entry_end = out.len();
            mappings.push(VizeMapping {
                gen_range: entry_start..entry_end,
                src_range: *source_start as usize..*source_end as usize,
                sub_spans: Vec::new(),
            });
        }
    }
}

fn render_name(
    out: &mut CompactString,
    mappings: &mut Vec<VizeMapping>,
    name: &PropName,
) -> (
    Option<std::ops::Range<usize>>,
    Option<std::ops::Range<usize>>,
) {
    match name {
        PropName::Static {
            content,
            start,
            end,
        } => {
            let gen_start = out.len();
            out.push_str(&json_string(content));
            let gen_end = out.len();
            (
                Some(gen_start..gen_end),
                Some(*start as usize..*end as usize),
            )
        }
        PropName::Computed(expression) => {
            out.push('[');
            push_mapped_expr(out, mappings, expression);
            out.push(']');
            (None, None)
        }
    }
}

fn component_tag(element: &ElementNode<'_>) -> Option<JsxExpr> {
    if element.tag.as_str() == "component" {
        return element.props.iter().find_map(|prop| {
            let PropNode::Directive(directive) = prop else {
                return None;
            };
            (directive.name.as_str() == "bind"
                && directive.arg.as_ref().and_then(static_name) == Some("is"))
            .then(|| directive.exp.as_ref().and_then(super::expr_of))
            .flatten()
        });
    }

    let start = element.loc.span.start.saturating_add(1);
    Some(JsxExpr {
        content: element.tag.as_str().to_compact_string(),
        start,
        end: start.saturating_add(element.tag.len() as u32),
    })
}

fn component_prop(element: &ElementNode<'_>, prop: &PropNode<'_>) -> Option<JsxComponentProp> {
    match prop {
        PropNode::Attribute(attribute) if !is_reserved_prop(attribute.name.as_str()) => {
            let value = match attribute.value.as_ref() {
                Some(value) => PropValue::Expression(JsxExpr {
                    content: json_string(value.content.as_str()),
                    start: value.loc.span.start,
                    end: value.loc.span.end,
                }),
                None => PropValue::Boolean,
            };
            Some(JsxComponentProp::Property {
                name: PropName::Static {
                    content: canonical_prop_name(attribute.name.as_str()),
                    start: attribute.name_loc.span.start,
                    end: attribute.name_loc.span.end,
                },
                value,
                source_start: attribute.loc.span.start,
                source_end: attribute.loc.span.end,
            })
        }
        PropNode::Attribute(_) => None,
        PropNode::Directive(directive) if directive.name.as_str() == "bind" => {
            let value = directive.exp.as_ref().and_then(super::expr_of)?;
            let Some(arg) = directive.arg.as_ref() else {
                return Some(JsxComponentProp::Spread {
                    value,
                    source_start: directive.loc.span.start,
                    source_end: directive.loc.span.end,
                });
            };
            let name = match arg {
                ExpressionNode::Simple(simple) if simple.is_static => {
                    if is_dynamic_component_tag_prop(element, simple.content.as_str())
                        || is_reserved_prop(simple.content.as_str())
                    {
                        return None;
                    }
                    PropName::Static {
                        content: canonical_prop_name(simple.content.as_str()),
                        start: simple.loc.span.start,
                        end: simple.loc.span.end,
                    }
                }
                _ => PropName::Computed(super::expr_of(arg)?),
            };
            Some(JsxComponentProp::Property {
                name,
                value: PropValue::Expression(value),
                source_start: directive.loc.span.start,
                source_end: directive.loc.span.end,
            })
        }
        PropNode::Directive(_) => None,
    }
}

fn static_name<'e>(expression: &'e ExpressionNode<'_>) -> Option<&'e str> {
    match expression {
        ExpressionNode::Simple(simple) if simple.is_static => Some(simple.content.as_str()),
        ExpressionNode::Simple(_) | ExpressionNode::Compound(_) => None,
    }
}

fn is_dynamic_component_tag_prop(element: &ElementNode<'_>, name: &str) -> bool {
    element.tag.as_str() == "component" && name == "is"
}

fn is_reserved_prop(name: &str) -> bool {
    matches!(name, "key" | "ref")
}

/// JSX accepts Vue's kebab aliases, while the generated SFC exposes its raw
/// required contract under camelCase keys. Canonicalizing the authored key lets
/// the helper intersect `$props` with that raw contract without making both
/// spellings optional (which would silently lose required props).
///
/// # Documented divergence from `vue-tsc`
///
/// TypeScript's own JSX checking has no notion of Vue's kebab aliasing, and it
/// exempts hyphenated attribute names from excess-property checks because they
/// are not valid identifiers. So for `<Widget foo-bar="ok"/>` against
/// `defineProps<{ fooBar: string }>()`, `vue-tsc` reports
/// `TS2322: Type '{ "foo-bar": string; }' is not assignable to …` (the required
/// `fooBar` is missing) while vize accepts it, and for an *unknown* hyphenated
/// attribute (`what-ever="x"`) `vue-tsc` is silent while vize reports the
/// canonicalized key. Both follow from accepting kebab aliases, which is
/// required behaviour for this path (#4042); recovering `vue-tsc`'s exact
/// answers would mean giving up either kebab acceptance or required-prop
/// enforcement, because a type admitting both spellings of a required prop
/// cannot keep either one required.
fn canonical_prop_name(name: &str) -> CompactString {
    if name.starts_with("data-") || name.starts_with("aria-") {
        return name.to_compact_string();
    }
    let mut out = CompactString::default();
    let mut uppercase_next = false;
    for ch in name.chars() {
        if ch == '-' {
            uppercase_next = true;
        } else if uppercase_next {
            out.extend(ch.to_uppercase());
            uppercase_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn json_string(value: &str) -> CompactString {
    serde_json::to_string(value)
        .expect("serializing a Rust string to JSON cannot fail")
        .to_compact_string()
}
