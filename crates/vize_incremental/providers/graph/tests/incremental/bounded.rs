use crate::*;
use std::ops::ControlFlow;
use vize_l3::query::{BoundedTrailQuery, EdgePattern, TrailVisit};

type Sites = Vec<(Vec<u32>, OpId, Span)>;

fn read(
    db: &ResidentDatabase,
    file: SourceFile,
    budget: usize,
) -> Result<(TrailVisit<()>, Sites), TemplateGraphError> {
    let batch = view(db, file).reach_batch(Lang::Js).map_err(Clone::clone)?;
    let q = BoundedTrailQuery {
        start: OpId::new(0),
        step: EdgePattern {
            direction: Direction::Forward,
            kinds: EdgeKinds::ALL,
        },
        min_hops: 0,
        max_hops: 4,
    };
    let mut sites = Vec::new();
    let status = batch.visit_bounded_trails(q, budget, |row| {
        assert_eq!(batch.check_bounded_trail(q, row), Ok(()));
        let span = batch.authored_span(row.end).unwrap();
        sites.push((row.edges.to_vec(), row.end, span));
        ControlFlow::Continue(())
    })?;
    Ok((status, sites))
}

#[test]
fn matched_paths_resolve_at_the_current_source_after_prefix_edits_and_template_repair() {
    let mut db = ResidentDatabase::default();
    let source = "<script>/*雪*/</script><template><p/><b/><i/><s/><u/></template>";
    let file = db.open("Bounded.vue", source);
    let full = read(&db, file, usize::MAX).unwrap();
    assert_eq!(full.0, TrailVisit::Complete);
    assert_eq!(
        full.1
            .iter()
            .map(|(edges, _, span)| (edges.len(), span.slice(source)))
            .collect::<Vec<_>>(),
        [
            (0, "<p/>"),
            (1, "<b/>"),
            (2, "<i/>"),
            (3, "<s/>"),
            (4, "<u/>")
        ]
    );
    let first = read(&db, file, 1).unwrap();
    assert_eq!(first, (TrailVisit::BudgetExhausted, full.1[..2].to_vec()));
    assert_eq!(executions(&db), (1, 0));
    let shifted = source.replace(
        "雪", "雪🌸
",
    );
    db.edit(file, &shifted);
    let second = read(&db, file, usize::MAX).unwrap();
    assert_eq!(executions(&db), (0, 0));
    let delta = (shifted.len() - source.len()) as u32;
    for ((edges, op, span), (new_edges, new_op, new_span)) in full.1.iter().zip(&second.1) {
        assert_eq!((edges, op), (new_edges, new_op));
        assert_eq!(*new_span, Span::new(span.start + delta, span.end + delta));
        assert_eq!(new_span.slice(&shifted), span.slice(source));
    }
    let broken = shifted.replace("<b/>", "<b>{{(}}</b>");
    db.edit(file, &broken);
    assert!(matches!(
        read(&db, file, usize::MAX),
        Err(TemplateGraphError::Incomplete { .. })
    ));
    let repaired = shifted.replace("<b/>", "<b>fixed</b>");
    db.edit(file, &repaired);
    let fresh = ResidentDatabase::default();
    assert_eq!(
        read(&db, file, usize::MAX),
        read(&fresh, fresh.open("Bounded.vue", &repaired), usize::MAX)
    );
    db.edit(file, source);
    assert_eq!(read(&db, file, usize::MAX).unwrap(), full);
}
