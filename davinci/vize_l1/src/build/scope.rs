//! Structural recovery shared by node construction and live scope consumers.

use vize_l0::{Allocator, Namespace, Vec, is_html_tag, is_math_ml_tag, is_svg_tag};

/// A caller's open-element frame; recovery never owns or copies its payload.
pub(crate) trait Frame<'a> {
    fn tag(&self) -> &'a str;
    fn namespace(&self) -> Namespace;
}

struct Closed<'a> {
    tag: &'a str,
    depth: usize,
}

/// Holds only the redundant-end-tag evidence already needed by the builder.
pub(crate) struct Recovery<'a> {
    closed: Vec<'a, Closed<'a>>,
    repair_interactive: bool,
}

impl<'a> Recovery<'a> {
    pub(crate) fn new(allocator: &'a Allocator, repair_interactive: bool) -> Self {
        Self {
            closed: Vec::new_in(&allocator),
            repair_interactive,
        }
    }

    /// Compute the namespace and first implicitly closed depth. The caller
    /// pops its own frames, preserving node payloads without a second stack.
    pub(crate) fn open<F: Frame<'a>>(
        &mut self,
        stack: &[F],
        tag: &str,
        self_closing: bool,
    ) -> (Namespace, Option<usize>) {
        let ns = namespace(stack, tag);
        let depth =
            (!self_closing && self.repair_interactive && ns == Namespace::Html && interactive(tag))
                .then(|| {
                    stack.iter().rposition(|frame| {
                        frame.namespace() == Namespace::Html
                            && interactive(frame.tag())
                            && frame.tag().eq_ignore_ascii_case(tag)
                    })
                })
                .flatten();
        if let Some(depth) = depth {
            for (depth, frame) in stack.iter().enumerate().skip(depth) {
                self.closed.push(Closed {
                    tag: frame.tag(),
                    depth,
                });
            }
        }
        (ns, depth)
    }

    /// Matching an ancestor also ends descendants; stray and redundant
    /// implicitly closed tags leave the caller's live scope intact.
    pub(crate) fn close<F: Frame<'a>>(&mut self, stack: &[F], tag: &str) -> Option<usize> {
        let matched = stack
            .iter()
            .rposition(|frame| frame.tag().eq_ignore_ascii_case(tag));
        if self.closed.last().is_some_and(|closed| {
            closed.tag.eq_ignore_ascii_case(tag) && matched.is_none_or(|depth| depth < closed.depth)
        }) {
            self.closed.pop();
            return None;
        }
        matched
    }
}

fn namespace<'a, F: Frame<'a>>(stack: &[F], tag: &str) -> Namespace {
    if is_svg_tag(tag) {
        Namespace::Svg
    } else if is_math_ml_tag(tag) {
        Namespace::MathMl
    } else {
        stack
            .last()
            .map_or(Namespace::Html, |frame| match frame.namespace() {
                Namespace::Svg if matches!(frame.tag(), "foreignObject" | "desc" | "title") => {
                    Namespace::Html
                }
                Namespace::MathMl if matches!(frame.tag(), "mi" | "mo" | "mn" | "ms" | "mtext") => {
                    Namespace::Html
                }
                other => other,
            })
    }
}

fn interactive(tag: &str) -> bool {
    is_html_tag(tag) && (tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("button"))
}
