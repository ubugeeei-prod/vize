//! Compare the complete existing operation and provenance pages before admitting new operands.
use super::*;
use vize_l0::Span;
use vize_l2::dump::{Page, ProvenancePage};

#[derive(Default, Debug)]
struct Counts {
    files: usize,
    invalid: usize,
    admitted_before: usize,
    admitted_after: usize,
    changed_files: usize,
    new_operands: usize,
    unresolved: usize,
    new_prop_sites: usize,
}

fn compare(source: &str, file: &str, counts: &mut Counts) {
    let a = Allocator::new();
    let b = Allocator::new();
    let old = observe(&a, source);
    let new = shorthand(&b, source, VueVersion::V3);
    let admitted_before = old.admitted().is_some();
    let admitted_after = new.admitted().is_some();
    assert!(!admitted_before || admitted_after, "{file}");
    let pairs = (old.template(), new.template());
    counts.admitted_before += usize::from(admitted_before);
    counts.admitted_after += usize::from(admitted_after);
    let mut added = Vec::new();
    let mut unresolved = Vec::new();
    if let (Some(before), Some(after)) = pairs {
        assert_eq!(before.block().span(), after.block().span(), "{file}");
        let mut original = before.embeds().iter().peekable();
        for retained in after.embeds() {
            let syntax = retained.syntax.source();
            if let Some(prior) = original.next_if(|r| r.syntax.source().span() == syntax.span()) {
                assert_eq!(prior.syntax.source().text(), syntax.text(), "{file}");
                assert_eq!(prior.node.is_some(), retained.node.is_some(), "{file}");
                continue;
            }
            let Some(oxc_ast::ast::Expression::Identifier(id)) = retained.syntax.expression()
            else {
                panic!("unexpected additional syntax in {file}")
            };
            assert_eq!(id.name.as_str(), syntax.text(), "{file}");
            assert!(
                core::ptr::eq(syntax.text(), syntax.span().slice(source)),
                "{file}"
            );
            let hole = before
                .holes()
                .iter()
                .find(|hole| {
                    hole.kind == NativeHoleKind::Directive
                        && hole.span.end == syntax.span().end
                        && hole.span.start < syntax.span().start
                        && matches!(
                            Span::new(hole.span.start, syntax.span().start).slice(source),
                            ":" | "v-bind:"
                        )
                })
                .unwrap_or_else(|| {
                    panic!("no original directive for {:?} in {file}", syntax.span())
                });
            if retained.node.is_some() {
                added.push(*hole);
            } else {
                assert!(!admitted_after, "{file}");
                let refused = after
                    .holes()
                    .iter()
                    .find(|r| {
                        r.span == hole.span && matches!(r.kind, NativeHoleKind::Construction(_))
                    })
                    .unwrap_or_else(|| panic!("missing construction refusal in {file}"));
                unresolved.push((*hole, *refused));
            }
        }
        assert!(original.next().is_none(), "{file}");
        let unchanged: Vec<_> = before
            .holes()
            .iter()
            .copied()
            .filter(|h| !added.contains(h))
            .map(|h| {
                unresolved
                    .iter()
                    .find(|(old, _)| *old == h)
                    .map_or(h, |(_, new)| *new)
            })
            .collect();
        assert_eq!(unchanged, after.holes(), "{file}");
        let diagnostics: Vec<_> = before
            .diagnostics()
            .iter()
            .filter(|d| !added.iter().any(|h| h.span == d.span))
            .collect();
        assert_eq!(
            diagnostics,
            after.diagnostics().iter().collect::<Vec<_>>(),
            "{file}"
        );
    } else {
        assert_eq!(pairs.0.is_some(), pairs.1.is_some(), "{file}");
    }
    if added.is_empty() && unresolved.is_empty() {
        assert_eq!(old.issues(), new.issues(), "{file}");
        assert_eq!(admitted_before, admitted_after, "{file}");
        assert_eq!(old.file().is_some(), new.file().is_some(), "{file}");
        if let (Some(lhs), Some(rhs)) = (old.file(), new.file()) {
            let left = lhs.l2ir().artifact();
            let right = rhs.l2ir().artifact();
            assert_eq!(
                Page::of(&left.root().ops),
                Page::of(&right.root().ops),
                "{file}"
            );
            assert_eq!(
                ProvenancePage::of(left.provenance()),
                ProvenancePage::of(right.provenance()),
                "{file}"
            );
        }
    }
    if !added.is_empty()
        && let Ok(imports) = new.setup_component_imports()
    {
        match component_props::compare(&imports) {
            Ok((sites, _, _)) => counts.new_prop_sites += sites,
            Err(vize_l2::query::ComponentPropIssue::Binding(_)) => {}
            Err(issue) => panic!("{file}: {issue:?}"),
        }
    }
    if !added.is_empty() || !unresolved.is_empty() {
        eprintln!(
            "shorthand_file {file} operations={} unresolved={}",
            added.len(),
            unresolved.len()
        );
        counts.changed_files += 1;
    }
    counts.new_operands += added.len();
    counts.unresolved += unresolved.len();
}

#[test]
#[ignore = "compare both file owners across every original input"]
fn original_files() {
    let manifest =
        std::fs::read_to_string(std::env::var_os("VIZE_EXPRESSION_QUERY_MANIFEST").unwrap())
            .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut counts = Counts::default();
    for file in manifest.split_terminator('\0') {
        counts.files += 1;
        let bytes = std::fs::read(root.join(file)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            counts.invalid += 1;
            continue;
        };
        compare(source, file, &mut counts);
    }
    assert!(counts.files > 0 && counts.admitted_before > 0);
    eprintln!("authored_shorthand {counts:?}");
}
