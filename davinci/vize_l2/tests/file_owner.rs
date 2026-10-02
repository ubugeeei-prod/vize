use vize_l0::{Allocator, Span, id::NodeId};
use vize_l2::artifact::ArtifactError;
use vize_l2::file::FileBuilder;

#[test]
fn file_owner_seals_the_actual_checked_canonical_tree() {
    let arena = Allocator::default();
    let source = "é<!--x-->";
    let mut builder = FileBuilder::new(&arena, source).unwrap();
    assert_eq!(builder.text("é", Span::new(0, 2)).unwrap(), NodeId::FIRST);
    assert_eq!(builder.comment("x", Span::new(2, 10)).unwrap().index(), 1);
    let file = builder.finish().unwrap();
    assert!(core::ptr::eq(file.artifact().source(), source));
    assert_eq!(file.artifact().node_count(), 2);
    assert_eq!(file.scopes().len(), 1);
    assert_eq!(file.scopes()[0].span, Span::new(0, 10));
    assert!(file.units().is_empty());
    assert_eq!(file.bindings().count(), 0);
    assert!(file.references().is_empty());
    assert!(file.expression(NodeId::FIRST).is_none());
}

#[test]
fn refused_factory_sites_do_not_mint_or_replace_file_nodes() {
    let arena = Allocator::default();
    let mut builder = FileBuilder::new(&arena, "é").unwrap();
    assert!(matches!(
        builder.text("", Span::new(1, 2)),
        Err(ArtifactError::InvalidSpan { .. })
    ));
    assert_eq!(builder.text("é", Span::new(0, 2)).unwrap(), NodeId::FIRST);
    assert_eq!(builder.finish().unwrap().artifact().node_count(), 1);
}

#[test]
fn empty_file_has_no_fabricated_binding_or_expression_entries() {
    let arena = Allocator::default();
    let file = FileBuilder::new(&arena, "").unwrap().finish().unwrap();
    assert_eq!(file.artifact().node_count(), 0);
    assert!(file.imports().is_empty());
    assert!(file.exports().is_empty());
    assert!(file.issues().is_empty());
    assert!(file.expression(NodeId::FIRST).is_none());
}
