//! Every L1 element of a surface tree by its `<tag` offset, built once per
//! view.
//!
//! The facade locates the L1 element of an L2 op from the tree root with a
//! binary search per children level ([`super::surface::siblings_with_pre`]),
//! and every element wrap does it twice. In a tree whose child lists are a
//! strict partition (each list ordered by start, each element's children
//! after its `<tag` token and before its next sibling), that descent reaches
//! exactly one element per `<tag` offset, and a page-order list of elements is
//! ordered by that offset. One pass records what the descent learns on its way
//! down, so a lookup is one binary search. A tree that is not such a partition
//! gets no index and keeps the descent.

use super::L2Markup;
use super::binding::is_v_pre;
use super::surface::{
    child_start, element_at, element_with_v_pre_at, offset_in, siblings_with_pre,
};
use vize_l0::Allocator;
use vize_l1::markup::{Element, SurfaceChild, SurfaceTree};

impl<'a> L2Markup<'a> {
    /// Read `v-pre` scopes from `authored`, the tree as written.
    pub(super) fn with_authored(mut self, authored: &'a SurfaceTree<'a>) -> Self {
        self.authored_index = match (self.surface, self.allocator) {
            (Some(surface), _) if std::ptr::eq(surface, authored) => self.surface_index,
            (_, Some(allocator)) => build(allocator, authored),
            _ => None,
        };
        self.authored = Some(authored);
        self
    }

    /// The L1 element of [`surface`](Self::surface) whose `<tag` token starts
    /// at `start`.
    pub(in crate::markup) fn surface_element_at(&self, start: u32) -> Option<&'a Element<'a>> {
        match self.surface_index {
            Some(index) => find(index, start).map(|entry| entry.element),
            None => element_at(self.surface?, start),
        }
    }

    /// The surface sibling list holding the element at `start`, and whether a
    /// `<pre>` encloses it.
    pub(in crate::markup) fn surface_siblings_at(
        &self,
        start: u32,
    ) -> Option<(&'a [SurfaceChild<'a>], bool)> {
        match self.surface_index {
            Some(index) => find(index, start).map(|entry| (entry.siblings, entry.in_pre)),
            None => siblings_with_pre(self.surface?, start),
        }
    }

    /// Whether the element at `start` sits in a `v-pre` scope as authored.
    pub(in crate::markup) fn frozen_at(&self, start: u32) -> bool {
        let (tree, index) = match self.authored {
            Some(authored) => (Some(authored), self.authored_index),
            None => (self.surface, self.surface_index),
        };
        match index {
            Some(index) => find(index, start).is_some_and(|entry| entry.frozen),
            None => tree
                .and_then(|tree| element_with_v_pre_at(tree, start))
                .is_some_and(|(_, frozen)| frozen),
        }
    }
}

/// One element and what the root descent to it reads.
#[derive(Clone, Copy)]
pub(in crate::markup) struct IndexedElement<'a> {
    start: u32,
    pub(in crate::markup) element: &'a Element<'a>,
    /// The child list holding the element.
    pub(in crate::markup) siblings: &'a [SurfaceChild<'a>],
    /// A strict ancestor is a `<pre>`.
    pub(in crate::markup) in_pre: bool,
    /// The element or an ancestor carries `v-pre`.
    pub(in crate::markup) frozen: bool,
}

struct Level<'a> {
    children: &'a [SurfaceChild<'a>],
    next: usize,
    /// Exclusive lower and upper bounds of every child's start.
    after: Option<u32>,
    before: Option<u32>,
    in_pre: bool,
    frozen: bool,
}

/// The page-order element index of `tree`, or `None` when its child lists
/// are not a strict partition of the source.
pub(in crate::markup) fn build<'a>(
    allocator: &'a Allocator,
    tree: &'a SurfaceTree<'a>,
) -> Option<&'a [IndexedElement<'a>]> {
    let source = tree.source;
    let mut entries = std::vec::Vec::new();
    let mut levels = vec![Level {
        children: &tree.children,
        next: 0,
        after: None,
        before: None,
        in_pre: false,
        frozen: false,
    }];
    while let Some(level) = levels.last_mut() {
        let Some(child) = level.children.get(level.next) else {
            levels.pop();
            continue;
        };
        let index = level.next;
        level.next += 1;
        let start = child_start(source, child);
        let next_start = level
            .children
            .get(index + 1)
            .map(|next| child_start(source, next));
        if level.after.is_some_and(|after| start <= after)
            || level.before.is_some_and(|before| start >= before)
            || next_start.is_some_and(|next| next <= start)
        {
            return None;
        }
        let SurfaceChild::Element(element) = child else {
            continue;
        };
        debug_assert_eq!(start, offset_in(source, element.open.lt_name.text));
        let frozen = level.frozen || element.open.attrs.iter().any(is_v_pre);
        entries.push(IndexedElement {
            start,
            element,
            siblings: level.children,
            in_pre: level.in_pre,
            frozen,
        });
        let child_level = Level {
            children: &element.children,
            next: 0,
            after: Some(start),
            before: next_start.or(level.before),
            in_pre: level.in_pre || element.tag() == "pre",
            frozen,
        };
        levels.push(child_level);
    }
    Some(allocator.as_oxc().alloc_slice_copy(&entries))
}

/// The indexed element whose `<tag` token starts at `start`.
#[inline]
pub(in crate::markup) fn find<'i, 'a>(
    index: &'i [IndexedElement<'a>],
    start: u32,
) -> Option<&'i IndexedElement<'a>> {
    let found = index
        .binary_search_by_key(&start, |entry| entry.start)
        .ok()?;
    index.get(found)
}

#[cfg(test)]
mod tests {
    use super::super::surface::{element_at, element_with_v_pre_at, siblings_with_pre};
    use crate::markup::LintL2Template;
    use vize_l0::Allocator;
    use vize_l1::markup::{Element, SurfaceChild};

    fn element_ptr(element: Option<&Element<'_>>) -> Option<*const ()> {
        element.map(|element| std::ptr::from_ref(element).cast())
    }

    fn siblings_ptr(
        found: Option<(&[SurfaceChild<'_>], bool)>,
    ) -> Option<(*const (), usize, bool)> {
        found.map(|(siblings, in_pre)| (siblings.as_ptr().cast(), siblings.len(), in_pre))
    }

    /// Every offset answers exactly as the root descent does.
    fn assert_matches_descent(source: &str) -> bool {
        let allocator = Allocator::new();
        let lowered = LintL2Template::lower(&allocator, source);
        let markup = lowered.markup();
        let surface = &lowered.tree;
        let authored = lowered.authored.as_ref().unwrap_or(surface);
        for start in 0..=source.len() as u32 + 1 {
            assert_eq!(
                element_ptr(markup.surface_element_at(start)),
                element_ptr(element_at(surface, start)),
                "{source:?} element at {start}"
            );
            assert_eq!(
                siblings_ptr(markup.surface_siblings_at(start)),
                siblings_ptr(siblings_with_pre(surface, start)),
                "{source:?} siblings at {start}"
            );
            assert_eq!(
                markup.frozen_at(start),
                element_with_v_pre_at(authored, start).is_some_and(|(_, frozen)| frozen),
                "{source:?} v-pre at {start}"
            );
        }
        markup.surface_index.is_some()
    }

    #[test]
    fn indexed_lookups_match_the_root_descent() {
        for source in [
            "<div><pre><span v-if=\"a\">x</span><b v-else>y</b>\n<i>z</i></pre>\
             <p v-pre><i>{{ raw }}</i><em :x=\"y\">t</em></p>\
             <ul><li v-for=\"i in n\" :key=\"i\">{{ i }}</li></ul></div>",
            "<!-- c --><section><template v-if=\"x\"><a/></template>\
             <template v-else-if=\"y\"><b/></template><!-- gap --><c v-else/></section>\
             text<br><br><pre><pre><code>{{ q }}</code></pre></pre>",
            "<div v-pre><span><b>{{ a }}</b></span></div><div><span v-pre>{{ b }}</span><i/></div>",
        ] {
            assert!(assert_matches_descent(source), "{source:?} is a partition");
        }
    }

    #[test]
    fn trees_the_parser_reshapes_still_answer_as_the_descent() {
        // Tree construction may move or close nodes away from source order;
        // an index or its absence must never change an answer.
        for source in [
            "<table><tr><td>a</td></tr><div>b</div></table>",
            "<p><div>x</div></p><ul><li>a<li>b</ul>",
            "<template><div><span></div></span></template>",
        ] {
            assert_matches_descent(source);
        }
    }
}
