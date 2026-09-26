use vize_davinci::dump::{Dump, Mode};
use vize_l3::dump::Page;
use vize_l3::op::OpKind;

const GRAPH: &str = "\
[l3-dump-v2]
phase=built

[l3-dump-v2.regions]
id=0 parent=- owner=- span=0:1

[l3-dump-v2.ops]
id=0 kind=l3.set-text region=0 effect=- span=0:1

[l3-dump-v2.edges]

[l3-dump-v2.effects]

";

#[test]
fn canonical_graph_protocol_roundtrips_without_changing_rows() {
    let graph = Page::parse(GRAPH).expect("canonical graph parses");
    assert_eq!(graph.ops.len(), 1);
    assert_eq!(graph.ops[0].kind, OpKind::SetText);
    let printed = graph.print_to_string(Mode::Full);
    assert_eq!(Page::parse(printed.as_str()).unwrap(), graph);
    assert!(printed.as_str().starts_with("[l3-dump-v2]\n"));
}

#[test]
fn old_graph_protocol_is_rejected() {
    // Historical input is a negative fixture, never an accepted compatibility alias.
    assert!(Page::parse(&GRAPH.replace("l3-dump-v2", "s3-folio")).is_err());
}

#[test]
fn old_and_unknown_opcode_vocabularies_are_rejected() {
    for kind in ["impeto.set-text", "l3.unknown", "l2.set-text"] {
        assert_eq!(OpKind::from_mnemonic(kind), None);
        assert!(Page::parse(&GRAPH.replace("l3.set-text", kind)).is_err());
    }
}
