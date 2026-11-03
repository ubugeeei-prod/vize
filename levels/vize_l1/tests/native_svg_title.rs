//! Complete independent source-shape controls for SVG title and HTML RCDATA.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "independent complete authored parser corpus"
)]

use vize_l0::Allocator;
use vize_l1::markup::parse_component;
use vize_l1::markup::{SurfaceChild, check_fidelity};

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
            SurfaceChild::Interpolation(node) => values.push(format!("?{}", node.content.text)),
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
        let (tree, errors) = vize_l1::markup::parse(&allocator, source);
        assert!(errors.is_empty(), "{name}: generic parse errors");
        assert_eq!(check_fidelity(&tree), Ok(()), "{name}: generic fidelity");
        let mut projected = Vec::new();
        observe(&tree.children, &mut projected);
        assert_eq!(projected, expected, "{name}: complete generic projection");
        count += 1;
    }
    assert_eq!(count, 12, "all authored controls execute");
}

#[test]
fn generic_projection_retains_neutral_heads_entities_comments_and_errors() {
    let cases = [
        (
            "<div v-pre v-pre.foo :[keys['a.b']]=\"value\">{{ one }}<span>{{ two }}</span></div>",
            false,
            vec!["+div", "? one ", "+span", "? two ", "-span", "-div"],
        ),
        (
            "<div data-x='&NotEqualTilde;'>&NotEqualTilde;{{ value }}</div>",
            false,
            vec!["+div", "=&NotEqualTilde;", "? value ", "-div"],
        ),
        (
            "<div // retained comment\n :[key]=\"value\">{{ value }}</div>",
            true,
            vec!["+div", "? value ", "-div"],
        ),
    ];
    for (source, in_tag_comments, expected) in cases {
        let allocator = Allocator::new();
        let (tree, errors) = vize_l1::markup::parse_with_options(
            &allocator,
            source,
            vize_l1::markup::SurfaceParseOptions {
                experimental_in_tag_comments: in_tag_comments,
            },
        );
        assert!(errors.is_empty(), "{source}: {errors:?}");
        assert_eq!(check_fidelity(&tree), Ok(()), "{source}");
        let mut actual = Vec::new();
        observe(&tree.children, &mut actual);
        assert_eq!(actual, expected, "complete neutral source structure");
        let div = match &tree.children[0] {
            SurfaceChild::Element(div) => div,
            _ => panic!("authored div"),
        };
        assert!(!div.open.is_verbatim(), "generic heads never enable v-pre");
    }
    let source = "<div :[broken>{{ value }}</div>";
    let allocator = Allocator::new();
    let (tree, errors) = vize_l1::markup::parse(&allocator, source);
    assert_eq!(check_fidelity(&tree), Ok(()));
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].code,
        vize_l0::ErrorCode::MissingDynamicDirectiveArgumentEnd
    );
    assert_eq!(errors[0].offset, 13);
}

fn pinned_vue36_scope_controls(generic: bool) {
    let corpus = include_str!(
        "../../../tests/_fixtures/differential/compiler/vapor-svg-title-scope-8624/controls.tsv"
    );
    let mut snapshots = Vec::new();
    let mut expected = Vec::new();
    for row in corpus.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 3);
        let allocator = Allocator::new();
        let mut actual = Vec::new();
        if generic {
            let (tree, errors) = vize_l1::markup::parse(&allocator, fields[1]);
            assert!(errors.is_empty(), "{}: {errors:?}", fields[0]);
            assert_eq!(check_fidelity(&tree), Ok(()));
            observe(&tree.children, &mut actual);
        } else {
            let parsed = parse_component(&allocator, fields[1]).unwrap();
            assert!(
                parsed.errors.is_empty(),
                "{}: {:?}",
                fields[0],
                parsed.errors
            );
            assert!(parsed.unsupported.is_empty(), "{}", fields[0]);
            assert_eq!(check_fidelity(&parsed.tree), Ok(()));
            observe(&parsed.tree.children, &mut actual);
        }
        snapshots.push((fields[0], actual));
        expected.push((fields[0], fields[2].split('|').map(str::to_owned).collect()));
    }
    assert_eq!(snapshots.len(), 11, "all additional whole controls execute");
    assert_eq!(snapshots, expected, "complete pinned Vue 3.6 scope corpus");
}

#[test]
fn native_svg_title_matches_pinned_vue36_scope() {
    pinned_vue36_scope_controls(false);
}

#[test]
fn generic_svg_title_matches_pinned_vue36_scope() {
    pinned_vue36_scope_controls(true);
}
