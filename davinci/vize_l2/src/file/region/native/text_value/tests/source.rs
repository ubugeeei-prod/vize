use super::*;
use vize_l1::embed::{DecodeSegmentKind as Segment, SourceError};

#[test]
fn eight_original_root_values_keep_whole_source_maps_and_actual_single_event() -> Test {
    let cases = [
        ("&amp;lt;", "&lt;", Some(2)),
        (
            "&lt;script&gt;&amp;&lt;/script&gt;",
            "<script>&</script>",
            Some(7),
        ),
        ("&#123;&#123;x&#125;&#125;", "{{x}}", Some(5)),
        ("雪&#x1F338;&acE;", "雪🌸∾̳", Some(3)),
        ("&unknown;", "&unknown;", None),
        ("&amp=1&timesX", "&=1×X", Some(4)),
        ("root-text", "root-text", None),
        ("quote'\"\\", "quote'\"\\", None),
    ];
    for (raw, expected, atoms) in cases {
        let arena = Allocator::default();
        let source = alloc::format!("<template>{raw}</template>");
        reset_observations();
        let original = completed(&arena, &source)?;
        same(observations(), 1)?;
        let file = original.file().ok_or("complete File")?;
        check(file.is_complete() && original.view().is_ok())?;
        same(file.artifact().node_count(), 1)?;
        let actual = text(file)?;
        let view = file
            .native_text_value_for(NodeId::FIRST, actual)
            .ok_or("actual Text join")?;
        check(core::ptr::eq(view.file(), file) && core::ptr::eq(view.text(), actual))?;
        let value = view.observation().ok_or("whole preparation")?;
        let prepared = value.source();
        same(value.raw_text(), raw)?;
        same(prepared.text(), expected)?;
        check(core::ptr::eq(prepared.authored_root(), source.as_str()))?;
        check(core::ptr::eq(actual.content, prepared.text()))?;
        same(actual.span, Span::new(10, 10 + raw.len() as u32))?;
        same(prepared.span(), actual.span)?;
        same(prepared.decode_map().map(|map| map.segments().len()), atoms)?;
        let child = original
            .selected()
            .children()
            .next()
            .ok_or("original event retained")?;
        let joined = value
            .admitted_for(original.selected(), child)
            .ok_or("same original L1 join")?;
        same(joined.child().ordinal(), 0)?;
        check(joined.child().parent_element().is_none())?;
        vize_l1::check_fidelity(&original.selected().component().carrier().tree)
            .map_err(|_| "full original bytes")?;
        if prepared.decode_map().is_none() {
            check(core::ptr::eq(prepared.text(), value.raw_text()))?;
        }
        same(
            prepared
                .authored_span(Span::new(0, expected.len() as u32))
                .map_err(|_| "full projection")?,
            actual.span,
        )?;
    }
    Ok(())
}

#[test]
fn one_pass_and_unicode_map_atoms_keep_exact_partial_entity_refusal() -> Test {
    let arena = Allocator::default();
    let source = "<template>雪&#x1F338;&acE;</template>";
    let original = completed(&arena, source)?;
    let file = original.file().ok_or("File")?;
    let value = file
        .native_text_value_for(NodeId::FIRST, text(file)?)
        .ok_or("Text")?
        .observation()
        .ok_or("source")?;
    let source = value.source();
    same(source.text().len(), 12)?;
    same(source.text().encode_utf16().count(), 5)?;
    let atoms = source.decode_map().ok_or("whole map")?.segments();
    let expected = [
        (Span::new(0, 3), Span::new(10, 13), Segment::Identity),
        (Span::new(3, 7), Span::new(13, 22), Segment::Entity),
        (Span::new(7, 12), Span::new(22, 27), Segment::Entity),
    ];
    same(atoms.len(), expected.len())?;
    for (atom, (decoded, authored, kind)) in atoms.iter().zip(expected) {
        same(atom.decoded(), decoded)?;
        same(atom.authored(), authored)?;
        same(atom.kind(), kind)?;
        same(
            source
                .authored_span(decoded)
                .map_err(|_| "atom projection")?,
            authored,
        )?;
    }
    same(
        source.authored_span(Span::new(7, 10)),
        Err(SourceError::PartialEntityBoundary),
    )?;
    same(
        source
            .authored_covering_span(Span::new(7, 10))
            .map_err(|_| "cover")?,
        Span::new(22, 27),
    )?;
    let one = completed(&arena, "<template>&amp;lt;</template>")?;
    let file = one.file().ok_or("one-pass File")?;
    let value = file.native_text_values()[0]
        .observation()
        .ok_or("one-pass value")?;
    same(value.source().text(), "&lt;")?;
    let atoms = value
        .source()
        .decode_map()
        .ok_or("one-pass map")?
        .segments();
    same(atoms[0].authored(), Span::new(10, 15))?;
    same(atoms[1].authored(), Span::new(15, 18))?;
    Ok(())
}

#[test]
fn ordinary_generic_text_factory_keeps_its_original_layout_and_node_output() -> Test {
    same(core::mem::size_of::<TextOp<'_>>(), 24)?;
    let arena = Allocator::default();
    let mut builder = crate::artifact::Builder::new(&arena, "raw").map_err(|_| "Builder")?;
    same(
        builder
            .text("raw", Span::new(0, 3))
            .map_err(|_| "ordinary leaf")?,
        NodeId::FIRST,
    )?;
    let artifact = builder.finish().map_err(|_| "finish")?;
    same(artifact.node_count(), 1)?;
    let [Op::Text(actual)] = artifact.root().ops.as_slice() else {
        return Err("ordinary leaf shape");
    };
    same(actual.content, "raw")?;
    Ok(())
}
