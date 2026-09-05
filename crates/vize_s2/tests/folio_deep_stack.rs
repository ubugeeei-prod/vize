//! Deep S2 folio mirroring and printing stay heap-grown on small stacks.

#![allow(clippy::disallowed_macros)]

use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::{Allocator, Box, Span, Vec as ArenaVec};
use vize_s2::folio::S2Folio;
use vize_s2::op::{ElementOp, Namespace, Op, Region, TextOp};

const DEPTH: usize = 1_100;
const SMALL_STACK: usize = 256 * 1024;

fn nested_tree<'a>(allocator: &'a Allocator) -> ArenaVec<'a, Op<'a>> {
    let mut ops = ArenaVec::from_iter_in(
        [Op::Text(Box::new_in(
            TextOp {
                content: "x",
                span: Span::new(0, 1),
            },
            &allocator,
        ))],
        &allocator,
    );

    for depth in 0..DEPTH {
        let start = depth as u32;
        let element = Op::Element(Box::new_in(
            ElementOp {
                tag: "div",
                namespace: Namespace::Html,
                attributes: ArenaVec::new_in(&allocator),
                bindings: ArenaVec::new_in(&allocator),
                children: Region { ops },
                span: Span::new(start, start + 1),
            },
            &allocator,
        ));
        ops = ArenaVec::from_iter_in([element], &allocator);
    }

    ops
}

#[test]
fn deep_folio_mirror_count_and_print_survive_small_stack() {
    std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(|| {
            let allocator = Allocator::default();
            let ops = nested_tree(&allocator);
            let folio = S2Folio::of(&ops);

            assert_eq!(folio.op_count(), DEPTH as u64 + 1);
            let printed = folio.print_to_string(FolioMode::Display);
            let expected_tail = format!("{}ui.text \"x\"\n\n", "  ".repeat(DEPTH));
            assert_eq!(printed.lines().nth(1), Some("ops=1101"));
            assert_eq!(
                printed.get(printed.len() - expected_tail.len()..),
                Some(expected_tail.as_str())
            );
            std::mem::forget(folio);
        })
        .expect("spawn worker thread")
        .join()
        .expect("worker thread finished");
}
