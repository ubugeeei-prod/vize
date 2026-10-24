//! Preserve the complete native lowering contract for the recorded input.

use core::fmt::Write;
use sha2::{Digest, Sha256};
use vize_l0::dump::{Dump, Mode};
use vize_l0::side_table::SideTable;
use vize_l0::{Allocator, SourceRoot};
use vize_l1_to_l2::lower;
use vize_l2::dump::Page;
use vize_l2::verify::{Rigor, verify, verify_table};

#[test]
fn recorded_attribute_input_retains_all_lowering_and_roundtrip_laws() {
    let source = include_str!("../../../tests/fuzz/regressions/l1_lowering/issue-8628.input");
    assert_eq!(source.len(), 1516, "keep the complete original input");
    let mut source_digest = vize_l0::String::default();
    for byte in Sha256::digest(source.as_bytes()).iter() {
        write!(source_digest, "{byte:02x}").expect("source digest formats");
    }
    assert_eq!(
        source_digest.as_str(),
        "6f40100935c784e26c9f7c149c7a66055d0f86ddab3eadb2f215555b0ad2e371",
        "keep every byte of the authenticated original input",
    );
    let allocator = Allocator::new();
    let (tree, errors) = vize_l1::markup::parse(&allocator, source);
    let lowered = lower(&allocator, &tree, &errors);
    let authored = SourceRoot::new(source).expect("retained input is u32-addressable UTF-8");
    for diagnostic in &lowered.diagnostics {
        assert!(authored.contains_span(diagnostic.span));
    }
    for record in &lowered.provenance {
        assert!(authored.contains_span(record.span));
    }
    let page = Page::of(&lowered.root.ops);
    assert_eq!(u64::from(lowered.op_count), page.op_count());
    assert_eq!(verify(&page, Rigor::Canonical), vec![]);
    assert_eq!(verify_table(&page, &lowered.scopes), vec![]);
    assert_eq!(verify_table(&page, &lowered.texts), vec![]);
    assert_eq!(verify_table(&page, &lowered.wrappers), vec![]);
    assert_eq!(verify_table(&page, &lowered.for_wrappers), vec![]);
    let provenance_ids: SideTable<()> = lowered
        .provenance
        .iter()
        .filter_map(|record| record.node.map(|node| (node, ())))
        .collect();
    assert_eq!(verify_table(&page, &provenance_ids), vec![]);
    let printed = page.print_to_string(Mode::Full);
    let parsed = Page::parse(printed.as_str()).expect("complete canonical page parses");
    assert_eq!(parsed, page);
    assert_eq!(parsed.print_to_string(Mode::Full), printed);
}

#[test]
fn recorded_newline_component_name_retains_all_lowering_laws() {
    let source = include_str!("../../../tests/fuzz/regressions/l1_lowering/issue-10400.input");
    assert_eq!(source.len(), 187);
    let mut digest = vize_l0::String::default();
    for byte in Sha256::digest(source.as_bytes()).iter() {
        write!(digest, "{byte:02x}").unwrap();
    }
    assert_eq!(
        digest.as_str(),
        "95aa7f63dc04bc8b3f30cad75337ff3637c48e0092a70c61c8547aa0df14d60b"
    );
    for name in [
        "Foo Bar",
        "Foo\nBar",
        "Foo\r\nBar",
        "Foo\tBar",
        "\"Quoted\"",
        "日本🧭",
        "@11562:11601n",
        "ns=svg",
    ] {
        for quote in ['\"', '\''] {
            let input = format!("<button is={quote}vue:{name}{quote} />");
            crate::support::assert_sound(&input, name);
        }
    }
    crate::support::assert_sound(source, "issue-10400");
}
