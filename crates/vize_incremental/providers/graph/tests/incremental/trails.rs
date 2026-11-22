use super::*;
use std::ops::ControlFlow;
use vize_l3::query::{EdgePattern, GraphError, TwoHopMatch, TwoHopQuery};

type Row = (TwoHopMatch, Span, Span);

fn read_trails(db: &ResidentDatabase, file: SourceFile) -> Result<Vec<Row>, TemplateGraphError> {
    let graph = view(db, file).reach_batch(Lang::Js).map_err(Clone::clone)?;
    let step = EdgePattern {
        direction: Direction::Forward,
        kinds: EdgeKinds::ALL,
    };
    let query = TwoHopQuery {
        start: OpId::new(0),
        first: step,
        second: step,
    };
    let mut rows = Vec::new();
    let result = graph.visit_two_hop(query, |row| {
        rows.push((
            row,
            graph.authored_span(row.middle).unwrap(),
            graph.authored_span(row.end).unwrap(),
        ));
        ControlFlow::<()>::Continue(())
    })?;
    assert_eq!(result, ControlFlow::Continue(()));
    let certificate: Vec<_> = rows.iter().map(|(matched, _, _)| *matched).collect();
    assert_eq!(graph.check_two_hop(query, &certificate), Ok(()));
    if let Some((_, prefix)) = certificate.split_last() {
        assert!(matches!(
            graph.check_two_hop(query, prefix),
            Err(vize_l3::query::TrailError::Mismatch { at, .. }) if at == prefix.len()
        ));
    }
    assert_eq!(
        graph.authored_span(OpId::new(99)),
        Err(TemplateGraphError::Graph(GraphError::UnknownOp(OpId::new(
            99
        ))))
    );
    Ok(rows)
}

#[test]
fn two_edge_matches_follow_current_source_offsets_and_broken_then_repaired_templates() {
    let mut db = ResidentDatabase::default();
    let source = "<script>/*雪*/</script><template><p/><b/><i/><s/></template>";
    let file = db.open("Trail.vue", source);
    let original = read_trails(&db, file).unwrap();
    assert_eq!(original.len(), 1);
    assert_eq!(
        (original[0].1.slice(source), original[0].2.slice(source)),
        ("<b/>", "<i/>")
    );
    assert_eq!(executions(&db), (1, 0));
    let prefix = source.replace("雪", "人🌸");
    db.edit(file, &prefix);
    let shifted = read_trails(&db, file).unwrap();
    assert_eq!(executions(&db), (0, 0));
    assert_eq!(original[0].0, shifted[0].0);
    let delta = (prefix.len() - source.len()) as u32;
    for (old, new) in [(original[0].1, shifted[0].1), (original[0].2, shifted[0].2)] {
        assert_eq!(new, Span::new(old.start + delta, old.end + delta));
    }
    for (text, broken) in [
        (prefix.replace("<b/>", "<b>{{(}}</b>"), true),
        (prefix.replace("<b/>", "<b>ok</b>"), false),
    ] {
        db.edit(file, &text);
        let updated = read_trails(&db, file);
        assert_eq!(executions(&db), (1, 0));
        let fresh = ResidentDatabase::default();
        assert_eq!(updated, read_trails(&fresh, fresh.open("Trail.vue", &text)));
        if broken {
            assert!(matches!(
                updated,
                Err(TemplateGraphError::Incomplete { .. })
            ));
        } else {
            assert!(updated.is_ok());
        }
    }
    db.edit(file, source);
    assert_eq!(read_trails(&db, file).unwrap(), original);
}
