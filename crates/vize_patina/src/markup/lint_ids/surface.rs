//! Normal-surface projection of the historical ID collector's branch facts.

use super::LintStaticId;
use crate::markup::l2::surface::offset_in;
use crate::markup::lint_headers::attribute_range;
use crate::markup::surface_source::{
    DecodedSource, element_verbatim, has_directive, static_attribute_value,
};
use vize_l0::ToCompactString;
use vize_l1::markup::{Element, SurfaceChild, SurfaceTree};

pub(super) fn collect<'a>(tree: &'a SurfaceTree<'a>, ids: &mut Vec<LintStaticId>) {
    walk(tree, &tree.children, false, &mut Vec::new(), ids);
}

/// The compatibility tree omits ignored headers and flattens their children
/// into the same sibling list, including when finding a conditional successor.
struct Children<'a> {
    current: std::slice::Iter<'a, SurfaceChild<'a>>,
    parents: Vec<std::slice::Iter<'a, SurfaceChild<'a>>>,
}

impl<'a> Children<'a> {
    fn new(children: &'a [SurfaceChild<'a>]) -> Self {
        Self {
            current: children.iter(),
            parents: Vec::new(),
        }
    }
}

impl<'a> Iterator for Children<'a> {
    type Item = &'a SurfaceChild<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current.next() {
                Some(SurfaceChild::Element(element)) if element.open.is_ignored_start() => {
                    self.parents.push(std::mem::replace(
                        &mut self.current,
                        element.children.iter(),
                    ));
                }
                Some(
                    SurfaceChild::Cdata(_)
                    | SurfaceChild::ProcessingInstruction(_)
                    | SurfaceChild::Unexpected(_),
                ) => {}
                Some(child) => return Some(child),
                None => self.current = self.parents.pop()?,
            }
        }
    }
}

fn walk<'a>(
    tree: &'a SurfaceTree<'a>,
    children: &'a [SurfaceChild<'a>],
    frozen: bool,
    path: &mut Vec<(u32, usize)>,
    ids: &mut Vec<LintStaticId>,
) {
    let mut children = Children::new(children).peekable();
    while let Some(child) = children.next() {
        let SurfaceChild::Element(element) = child else {
            continue;
        };
        let element_frozen = element_verbatim(element, frozen);
        if !has_directive(element, "if", element_frozen) {
            visit(tree, element, element_frozen, path, ids);
            continue;
        }
        let start = offset_in(tree.source, element.open.lt_name.text);
        let mut index = 0;
        path.push((start, index));
        visit(tree, element, element_frozen, path, ids);
        path.pop();

        loop {
            while children.peek().is_some_and(|child| gap(tree, child)) {
                children.next();
            }
            let Some(SurfaceChild::Element(element)) = children.peek().copied() else {
                break;
            };
            let element_frozen = element_verbatim(element, frozen);
            // The original collector tests any directive, with else taking
            // precedence even when another structural spelling comes first.
            let is_else = has_directive(element, "else", element_frozen);
            if !is_else && !has_directive(element, "else-if", element_frozen) {
                break;
            }
            children.next();
            index += 1;
            path.push((start, index));
            visit(tree, element, element_frozen, path, ids);
            path.pop();
            if is_else {
                break;
            }
        }
    }
}

fn visit<'a>(
    tree: &'a SurfaceTree<'a>,
    element: &'a Element<'a>,
    frozen: bool,
    path: &mut Vec<(u32, usize)>,
    ids: &mut Vec<LintStaticId>,
) {
    vize_l0::ensure_sufficient_stack(|| {
        for attribute in &element.open.attrs {
            if attribute.name.text == "id"
                && let Some(value) = static_attribute_value(tree, attribute)
            {
                ids.push(LintStaticId {
                    value: value.text().to_compact_string(),
                    range: attribute_range(tree.source, attribute),
                    branches: path.clone(),
                });
            }
        }
        walk(tree, &element.children, frozen, path, ids);
    });
}

fn gap<'a>(tree: &'a SurfaceTree<'a>, child: &'a SurfaceChild<'a>) -> bool {
    match child {
        SurfaceChild::Comment(_) => true,
        SurfaceChild::Text(token) => DecodedSource::from_text_token(tree.source, token)
            .is_some_and(|text| text.text().trim().is_empty()),
        _ => false,
    }
}
