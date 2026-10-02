//! Cases whose imports differ from their structural spelling order.

use vize_l0::{Allocator, Box};
use vize_l2::op::{CommentOp, Op, Region};

use super::{bind, element, interpolation, region, span, text};

pub(super) fn build<'a>(arena: &'a Allocator, id: &str, source: &'a str) -> Option<Region<'a>> {
    let root = match id {
        "text-before-element" => region(
            arena,
            [element(
                arena,
                source,
                source,
                "div",
                [],
                [],
                [
                    text(arena, source, "hi"),
                    element(arena, source, "<span/>", "span", [], [], []),
                ],
            )],
        ),
        "class-with-children" => region(
            arena,
            [element(
                arena,
                source,
                source,
                "div",
                [],
                [bind(
                    arena,
                    source,
                    ":class=\"classes\"",
                    "class",
                    "classes",
                )],
                [element(
                    arena,
                    source,
                    "<span>{{msg}}</span>",
                    "span",
                    [],
                    [],
                    [interpolation(arena, source, "msg")],
                )],
            )],
        ),
        "style-before-class" => region(
            arena,
            [element(
                arena,
                source,
                source,
                "div",
                [],
                [
                    bind(arena, source, ":style=\"styles\"", "style", "styles"),
                    bind(arena, source, ":class=\"classes\"", "class", "classes"),
                ],
                [],
            )],
        ),
        "comment-root-fragment" => region(
            arena,
            [
                element(
                    arena,
                    source,
                    "<p>A</p>",
                    "p",
                    [],
                    [],
                    [text(arena, source, "A")],
                ),
                Op::Comment(Box::new_in(
                    CommentOp {
                        content: "ok",
                        span: span(source, "<!--ok-->"),
                    },
                    &arena,
                )),
            ],
        ),
        _ => return None,
    };
    Some(root)
}
