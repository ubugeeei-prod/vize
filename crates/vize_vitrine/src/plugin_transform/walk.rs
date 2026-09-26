//! Page-order numbering includes attached bindings, exactly as L2 lowering.
use super::schema::{Attribute as WireAttribute, Node};
use vize_l2::op::{Attribute, Namespace, Op, Region};

pub(super) fn visit<'a>(
    region: &mut Region<'a>,
    next: &mut u32,
    visitor: &mut impl FnMut(u32, &mut Op<'a>),
) {
    vize_l0::ensure_sufficient_stack(|| visit_guarded(region, next, visitor));
}

fn visit_guarded<'a>(
    region: &mut Region<'a>,
    next: &mut u32,
    visitor: &mut impl FnMut(u32, &mut Op<'a>),
) {
    for op in region.ops.iter_mut() {
        let id = *next;
        *next += 1;
        visitor(id, op);
        match op {
            Op::Element(owner) => {
                *next += owner.bindings.len() as u32;
                visit(&mut owner.children, next, visitor);
            }
            Op::Component(owner) => {
                *next += owner.bindings.len() as u32;
                visit(&mut owner.children, next, visitor);
            }
            Op::Slot(owner) => {
                *next += owner.bindings.len() as u32;
                visit(&mut owner.fallback, next, visitor);
            }
            Op::If(owner) => {
                for branch in owner.branches.iter_mut() {
                    visit(&mut branch.region, next, visitor);
                }
            }
            Op::For(owner) => visit(&mut owner.region, next, visitor),
            Op::Text(_) | Op::Interpolation(_) | Op::Comment(_) => {}
        }
    }
}

pub(super) fn nodes(region: &mut Region<'_>) -> Vec<Node> {
    let mut nodes = Vec::new();
    visit(region, &mut 0, &mut |id, op| {
        if let Op::Element(owner) = op {
            let namespace = match owner.namespace {
                Namespace::Html => "html",
                Namespace::Svg => "svg",
                Namespace::MathMl => "mathml",
            };
            nodes.push(Node {
                id,
                kind: "ui.element",
                tag: owner.tag.to_owned(),
                namespace,
                attrs: owner.attributes.iter().map(attribute).collect(),
            });
        }
    });
    nodes
}

fn attribute(attr: &Attribute<'_>) -> WireAttribute {
    WireAttribute {
        name: attr.name.to_owned(),
        value: attr
            .value
            .map(|value| vize_l1_to_l2::emit::decode_html_attribute_entities(value).to_string()),
    }
}
