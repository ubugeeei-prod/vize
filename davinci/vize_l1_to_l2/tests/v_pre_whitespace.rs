//! Removed whitespace retains source provenance; literal and active text stay distinct.

mod support;

use support::{assert_sound, assert_transformed_sound, with_lowered};
use vize_l0::{Span, String};
use vize_l2::dump::{Element, Op, Text};
use vize_l2::op::Namespace;
use vize_l2::provenance::ProvenanceRecord;

#[test]
fn v_pre_drops_only_the_authored_planned_gaps() {
    let source = "<div v-pre>\n<i/>\n<b/>\n</div>";
    with_lowered(source, |lowered, page| {
        let element = |tag: &str, start, end, children| {
            Op::Element(Element {
                tag: String::from(tag),
                namespace: Namespace::Html,
                attributes: vec![],
                bindings: vec![],
                children,
                span: Span::new(start, end),
            })
        };
        assert_eq!(
            page.ops,
            vec![element(
                "div",
                0,
                28,
                vec![element("i", 12, 16, vec![]), element("b", 17, 21, vec![])],
            )]
        );
        assert!(lowered.diagnostics.is_empty());
        assert_eq!(lowered.op_count, 3);
        let drops: Vec<_> = lowered
            .provenance
            .iter()
            .filter(|record| record.rule == "condense.drop-whitespace")
            .cloned()
            .collect();
        assert_eq!(
            drops,
            [11, 16, 21].map(|start| ProvenanceRecord {
                rule: String::from("condense.drop-whitespace"),
                node: None,
                before: String::from("\n"),
                after: String::default(),
                span: Span::new(start, start + 1),
            })
        );
    });
    assert_sound(source, "v-pre gaps");
    assert_transformed_sound(source, "v-pre gaps");
}

#[test]
fn inline_spaces_and_unicode_whitespace_remain_authored_text() {
    for (source, gap) in [
        ("<div v-pre><i/> <b/></div>", " "),
        ("<div v-pre><i/>\u{a0}<b/></div>", "\u{a0}"),
        ("<div v-pre><i/>\u{2003}<b/></div>", "\u{2003}"),
    ] {
        with_lowered(source, |lowered, page| {
            let Op::Element(element) = &page.ops[0] else {
                panic!("expected element");
            };
            assert_eq!(element.children.len(), 3);
            assert_eq!(
                element.children[1],
                Op::Text(Text {
                    content: String::from(gap),
                    span: Span::new(15, 15 + gap.len() as u32),
                })
            );
            assert!(lowered.diagnostics.is_empty());
            assert!(
                !lowered
                    .provenance
                    .iter()
                    .any(|record| record.rule == "condense.drop-whitespace")
            );
        });
        assert_sound(source, "v-pre retained gap");
        assert_transformed_sound(source, "v-pre retained gap");
    }
}

#[test]
fn pre_preserves_newlines_and_v_pre_does_not_freeze_its_sibling() {
    for (source, expected) in [
        (
            "<pre v-pre>x\n  {{ literal }}\n y</pre>",
            "x\n  {{ literal }}\n y",
        ),
        (
            "<div v-pre> before {{ literal }} after </div>",
            " before {{ literal }} after ",
        ),
        ("<div v-pre>\n  {{ literal }}\n</div>", " {{ literal }} "),
    ] {
        with_lowered(source, |lowered, page| {
            let Op::Element(element) = &page.ops[0] else {
                panic!("expected element");
            };
            assert_eq!(element.children.len(), 1);
            assert!(matches!(&element.children[0], Op::Text(text) if text.content == expected));
            assert!(lowered.diagnostics.is_empty());
        });
        assert_sound(source, "v-pre literal run");
        assert_transformed_sound(source, "v-pre literal run");
    }
    let source = "<div v-pre>{{ literal }}</div><p>{{ active }}</p>";
    with_lowered(source, |lowered, page| {
        assert_eq!(page.ops.len(), 2);
        let (Op::Element(frozen), Op::Element(active)) = (&page.ops[0], &page.ops[1]) else {
            panic!("expected both elements");
        };
        assert!(
            matches!(&frozen.children[..], [Op::Text(text)] if text.content == "{{ literal }}")
        );
        assert!(matches!(&active.children[..], [Op::Interpolation(_)]));
        assert!(lowered.diagnostics.is_empty());
    });
    assert_sound(source, "v-pre active sibling");
    assert_transformed_sound(source, "v-pre active sibling");
}
