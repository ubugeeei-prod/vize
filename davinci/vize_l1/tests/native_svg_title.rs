//! Complete independent source-shape controls for SVG title and HTML RCDATA.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "independent complete authored parser corpus"
)]

use vize_l0::Allocator;
use vize_l1::markup::parse_component;
use vize_l1::{SurfaceChild, check_fidelity};

fn observe(children: &[SurfaceChild<'_>], values: &mut Vec<String>) {
    for child in children {
        match child {
            SurfaceChild::Element(node) => {
                let name = node.open.lt_name.text.strip_prefix('<').unwrap();
                values.push(format!("+{name}"));
                observe(&node.children, values);
                values.push(format!("-{name}"));
            }
            SurfaceChild::Text(node) => values.push(format!("={}", node.text)),
            _ => panic!("unexpected authored parser structure"),
        }
    }
}

#[test]
fn native_svg_title_children_preserve_complete_html_rcdata_controls() {
    let corpus =
        include_str!("../../../tests/_fixtures/differential/compiler/vapor-svg-title-8624.tsv");
    let mut count = 0;
    for row in corpus.lines() {
        let mut fields = row.split('\t');
        let name = fields.next().unwrap();
        let source = fields.next().unwrap();
        let expected: Vec<_> = fields.next().unwrap().split('|').collect();
        assert!(fields.next().is_none(), "{name}: exact corpus row");
        let allocator = Allocator::new();
        let parsed = parse_component(&allocator, source).unwrap();
        assert!(parsed.errors.is_empty(), "{name}: {:?}", parsed.errors);
        assert!(parsed.unsupported.is_empty(), "{name}");
        assert_eq!(check_fidelity(&parsed.tree), Ok(()), "{name}");
        let mut actual = Vec::new();
        observe(&parsed.tree.children, &mut actual);
        assert_eq!(actual, expected, "{name}: entire ordered structure");
        count += 1;
    }
    assert_eq!(count, 11, "all authored controls execute");
}
