//! Authored and normalized surfaces share bytes and diagnostics.

use vize_l0::{Allocator, CompactString};
use vize_l1::{SurfaceChild, SurfaceTree, check_fidelity, parse, parse_with_authored};

fn paths(tree: &SurfaceTree<'_>) -> Vec<(CompactString, usize)> {
    fn walk(children: &[SurfaceChild<'_>], depth: usize, rows: &mut Vec<(CompactString, usize)>) {
        for child in children {
            if let SurfaceChild::Element(element) = child {
                rows.push((element.tag().into(), depth));
                walk(&element.children, depth + 1, rows);
            }
        }
    }
    let mut rows = Vec::new();
    walk(&tree.children, 0, &mut rows);
    rows
}

#[test]
fn nested_interactive_tags_keep_the_authored_parent_relation() {
    for tag in ["a", "button"] {
        let source = [
            "<",
            tag,
            "><span><",
            tag,
            ">日本語😀</",
            tag,
            "></span></",
            tag,
            ">",
        ]
        .concat();
        let allocator = Allocator::new();
        let (normal, authored, errors) = parse_with_authored(&allocator, &source);
        assert!(errors.is_empty());
        assert!(check_fidelity(&normal).is_ok());
        let authored = authored.expect("must retain the authored tree");
        assert!(check_fidelity(&authored).is_ok());
        assert_eq!(
            paths(&normal),
            [(tag.into(), 0), ("span".into(), 1), (tag.into(), 0)]
        );
        assert_eq!(
            paths(&authored),
            [(tag.into(), 0), ("span".into(), 1), (tag.into(), 2)]
        );
    }
}

#[test]
fn ordinary_and_foreign_namespace_sources_reuse_the_existing_surface() {
    for source in [
        "<p><div>authored</div></p>",
        "<table><tr><td>row</td></tr></table>",
        "<svg><a><a>foreign</a></a></svg>",
        "<math><button><button>foreign</button></button></math>",
        "<Comp><template #footer><button>go</button></template></Comp>",
    ] {
        let allocator = Allocator::new();
        let (normal, authored, errors) = parse_with_authored(&allocator, source);
        let (reference, reference_errors) = parse(&allocator, source);
        assert!(authored.is_none(), "{source}");
        assert_eq!(
            vize_l0::cstr!("{normal:?}"),
            vize_l0::cstr!("{reference:?}")
        );
        assert_eq!(
            vize_l0::cstr!("{errors:?}"),
            vize_l0::cstr!("{reference_errors:?}")
        );
    }
}

#[test]
fn malformed_authored_projections_preserve_the_same_tokenizer_diagnostics() {
    for source in [
        "<a><a broken= >{{ }}<span",
        "<button><button x='unterminated",
        "<a><span><a>nested</a></span></a></a>",
    ] {
        let allocator = Allocator::new();
        let (normal, authored, errors) = parse_with_authored(&allocator, source);
        let (reference, reference_errors) = parse(&allocator, source);
        assert_eq!(
            vize_l0::cstr!("{normal:?}"),
            vize_l0::cstr!("{reference:?}"),
            "{source}"
        );
        assert!(check_fidelity(&normal).is_ok());
        if let Some(authored) = authored {
            assert!(check_fidelity(&authored).is_ok(), "{source}");
        }
        assert_eq!(
            vize_l0::cstr!("{errors:?}"),
            vize_l0::cstr!("{reference_errors:?}")
        );
    }
}
