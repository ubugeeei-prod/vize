use super::*;

#[test]
fn edits_on_either_half_of_a_crlf_replay_at_client_line_boundaries() {
    for source in [
        "a\r\nb",
        "🪴\r\n雪\rX\n",
        "\r\r\n\r\n",
        "\r\n",
        "\u{2028}A\rB\u{2029}",
    ] {
        let store = DocumentStore::new();
        let uri = open(&store, "file:///line-endings.vue", source);
        let snapshot = SourceSnapshotCache::default()
            .capture(&store, &uri)
            .unwrap();
        let frame = snapshot.versioned_source().unwrap();
        let offsets: Vec<_> = (0..=source.len())
            .filter(|at| source.is_char_boundary(*at))
            .collect();
        for (i, &a) in offsets.iter().enumerate() {
            for (j, &b) in offsets.iter().enumerate().skip(i) {
                let selected = frame
                    .select_edits([(Span::new(a as u32, b as u32), "🪴")])
                    .unwrap()
                    .unwrap();
                let edit = selected.as_edit_set();
                let wire = snapshot
                    .workspace_edit_for_fixes(&store, request(&uri), edit)
                    .unwrap();
                assert_eq!(
                    apply_wire(source, &wire, &uri, 7),
                    snapshot.apply_edits(&store, edit).unwrap().as_str()
                );
                for &c in offsets.iter().skip(j) {
                    for replacement in ["", "x", "\r\n🪴"] {
                        let selected = frame
                            .select_edits([
                                (Span::new(a as u32, b as u32), replacement),
                                (Span::new(b as u32, b as u32), "1"),
                                (Span::new(b as u32, b as u32), "2"),
                                (Span::new(b as u32, c as u32), replacement),
                            ])
                            .unwrap()
                            .unwrap();
                        let edits = selected.as_edit_set();
                        let wire = snapshot
                            .workspace_edit_for_fixes(&store, request(&uri), edits)
                            .unwrap();
                        let expected = snapshot.apply_edits(&store, edits).unwrap();
                        assert_eq!(
                            apply_wire(source, &wire, &uri, 7),
                            expected.as_str(),
                            "{source:?} {a} {b} {c}"
                        );
                    }
                }
            }
        }
        if source == "a\r\nb" {
            let selected = frame
                .select_edits([(Span::new(1, 2), "")])
                .unwrap()
                .unwrap();
            let edits = selected.as_edit_set();
            let old = snapshot.workspace_edit(&store, edits).unwrap();
            let fixed = snapshot
                .workspace_edit_for_fixes(&store, request(&uri), edits)
                .unwrap();
            assert_eq!(apply_wire(source, &old, &uri, 7), source);
            assert_eq!(apply_wire(source, &fixed, &uri, 7), "a\nb");
        }
    }
}
