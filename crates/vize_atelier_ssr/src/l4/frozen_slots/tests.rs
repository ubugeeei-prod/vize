//! Authored before the source-bound empty literal-slot exception.

use super::{FrozenSlotFacts, FrozenTemplate};
use crate::l4::{LegacyReason, SsrL4Selection, differential_tests, l2_input};
use crate::options::SsrCompilerOptions;
use vize_atelier_core::parser::Parser;
use vize_l0::{Allocator, Box, Span, Vec};
use vize_l2::op::{ElementOp, Namespace, Op, Region};

fn lower<'a>(arena: &'a Allocator, source: &'a str) -> vize_l1_to_l2::lower::Lowered<'a> {
    let (tree, errors) = vize_l1::parse(arena, source);
    assert!(errors.is_empty(), "{errors:?}");
    vize_l1_to_l2::lower(arena, &tree, &errors)
}

#[test]
fn frozen_template_rejects_foreign_actual_root_and_caller_before_mutation() {
    for case in 0..3 {
        let arena = Allocator::new();
        let source_a = "<slot v-pre></slot>".to_owned();
        let source_b = if case == 2 {
            source_a.clone()
        } else {
            "<slot></slot>".to_owned()
        };
        let (root_a, errors, frozen) = Parser::new(&arena, &source_a).parse_with_frozen_elements();
        assert!(errors.is_empty(), "{errors:?}");
        let (root_b, errors) = Parser::new(&arena, &source_b).parse();
        assert!(errors.is_empty(), "{errors:?}");
        let (root, caller) = if case == 1 {
            (&root_a, source_b.as_str())
        } else {
            (&root_b, source_a.as_str())
        };
        let before = vize_l0::cstr!("{root:?}");
        assert_eq!(
            FrozenTemplate::new(root, caller, &frozen).err(),
            Some("Frozen literal slot provenance belongs to a different template source.")
        );
        assert_eq!(vize_l0::cstr!("{root:?}"), before, "case {case}");
    }
}

#[test]
fn capability_is_bound_to_the_actual_lowered_region_and_source() {
    let arena = Allocator::new();
    let source = "<slot v-pre></slot>".to_owned();
    let copy = source.clone();
    let (root, errors, frozen) = Parser::new(&arena, &source).parse_with_frozen_elements();
    assert!(errors.is_empty(), "{errors:?}");
    let template = FrozenTemplate::new(&root, &source, &frozen).expect("original parser bundle");
    let owned = lower(&arena, &source);
    let equivalent = lower(&arena, &source);
    let facts = FrozenSlotFacts::new(template, &source, &owned.root).expect("owned lowering");
    assert!(facts.validate(&source, &owned.root).is_ok());
    for (caller, region) in [
        (source.as_str(), &equivalent.root),
        (copy.as_str(), &owned.root),
    ] {
        let before = vize_l0::cstr!("{region:?}");
        assert_eq!(
            facts.validate(caller, region),
            Err("Frozen literal slot facts belong to a different L2 artifact.")
        );
        assert_eq!(vize_l0::cstr!("{region:?}"), before);
    }
    assert_eq!(
        FrozenSlotFacts::new(template, &copy, &owned.root).err(),
        Some("Frozen literal slot provenance belongs to a different template source.")
    );
    let Op::Element(slot) = &owned.root.ops[0] else {
        panic!("frozen literal is ElementOp")
    };
    assert!(facts.permits(slot));
}

#[test]
fn direct_l2_cannot_forge_literal_slot_custody_from_offsets_or_tag_text() {
    let arena = &Allocator::new();
    let source = "<slot v-pre></slot>";
    let detached = "slot".to_owned();
    for tag in [&source[1..5], detached.as_str()] {
        let element = ElementOp {
            tag,
            namespace: Namespace::Html,
            attributes: Vec::new_in(&arena),
            bindings: Vec::new_in(&arena),
            children: Region {
                ops: Vec::new_in(&arena),
            },
            span: Span::new(0, source.len() as u32),
        };
        let mut root = Region {
            ops: Vec::new_in(&arena),
        };
        root.ops.push(Op::Element(Box::new_in(element, &arena)));
        let options = SsrCompilerOptions::default();
        assert!(matches!(
            l2_input::select_l2_lane(&arena, source, &root, &options),
            SsrL4Selection::Legacy(LegacyReason::Element)
        ));
        assert!(crate::compile_l2_to_ssr(&arena, source, &root, &options).is_none());
    }
    let normal = lower(&arena, "<slot></slot>");
    assert!(matches!(&normal.root.ops[0], Op::Slot(_)));
    assert!(matches!(
        l2_input::select_l2_lane(
            &arena,
            "<slot></slot>",
            &normal.root,
            &SsrCompilerOptions::default()
        ),
        SsrL4Selection::Emitted(_)
    ));
}

#[test]
fn original_empty_literal_and_normal_outlet_admit_while_nonempty_literal_refuses() {
    for (name, source, admitted) in [
        ("own-empty", "<slot v-pre></slot>", true),
        ("inherited-empty", "<div v-pre><slot></slot></div>", true),
        ("normal-outlet", "<slot></slot>", true),
        (
            "original-refused",
            "<slot v-pre>{{ not }} an interpolation</slot>",
            false,
        ),
    ] {
        for (set, options, experimental) in differential_tests::option_sets() {
            let selection = differential_tests::selection(source, &options, &experimental);
            if admitted {
                assert!(
                    matches!(selection, SsrL4Selection::Emitted(_)),
                    "{name}/{set}: {selection:?}"
                );
            } else {
                assert!(
                    matches!(selection, SsrL4Selection::Legacy(LegacyReason::Element)),
                    "{name}/{set}: {selection:?}"
                );
            }
            differential_tests::assert_parity(source, &options, &experimental, name);
        }
    }
}
