use vize_incremental::{BlockKind, ResidentDatabase, SourceFile};
use vize_l0::{Lang, Span};
use vize_l3::{
    op::OpId,
    query::{Direction, EdgeKinds, ReachQuery, ReachStep},
};
use vize_resident_graph::{TemplateGraphError, TemplateQueries};

fn request(start: u32) -> ReachQuery {
    ReachQuery {
        start: OpId::new(start),
        direction: Direction::Forward,
        kinds: EdgeKinds::ALL,
    }
}

fn view(db: &ResidentDatabase, file: SourceFile) -> vize_incremental::block_query::BlockView<'_> {
    db.document_blocks(file)
        .unwrap()
        .iter()
        .find(|block| block.kind() == &BlockKind::Template)
        .unwrap()
}

type Read = (Vec<ReachStep>, Vec<(OpId, Span)>, Vec<(OpId, Span)>);

fn read(db: &ResidentDatabase, file: SourceFile, text: &str, lang: Lang) -> Read {
    let view = view(db, file);
    let answer = view.reachable(lang, request(0)).unwrap();
    let relative = answer.relative().spans().collect::<Vec<_>>();
    let authored = answer
        .authored_spans()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    for ((a, local), (b, absolute)) in relative.iter().zip(&authored) {
        assert_eq!(a, b);
        assert_eq!(
            view.source()
                .text
                .get(local.start as usize..local.end as usize)
                .unwrap(),
            text.get(absolute.start as usize..absolute.end as usize)
                .unwrap()
        );
    }
    (answer.relative().steps().to_vec(), relative, authored)
}

fn executions(db: &ResidentDatabase) -> (u64, u64) {
    let counts = db.take_accounting();
    let executions = |name: &str| {
        counts
            .iter()
            .filter(|(k, _)| k.contains(name))
            .map(|(_, v)| u64::from(v.executed))
            .sum()
    };
    (executions("template_graph"), executions("template_reach"))
}

#[test]
fn reuses_relative_proofs_after_prefix_edits_and_revalidates_changed_content() {
    let mut db = ResidentDatabase::default();
    let first = "<script>/*α*/</script><template><div>{{a}}</div><p>🦀</p></template>";
    let file = db.open("A.vue", first);
    let old = read(&db, file, first, Lang::Js);
    assert!(!old.0.is_empty());
    assert_eq!(executions(&db), (1, 1));
    assert_eq!(read(&db, file, first, Lang::Js), old);
    assert_eq!(executions(&db), (0, 0));
    // Another start shares the indexed table and builds only its own witness.
    view(&db, file).reachable(Lang::Js, request(1)).unwrap();
    assert_eq!(executions(&db), (0, 1));
    let prefixed = first.replace("α", "日本語");
    db.edit(file, &prefixed);
    let shifted = read(&db, file, &prefixed, Lang::Js);
    assert_eq!(old.0, shifted.0);
    assert_eq!(old.1, shifted.1);
    assert_ne!(old.2, shifted.2);
    assert_eq!(executions(&db), (0, 0));
    let same_length = prefixed.replace("{{a}}", "{{x}}");
    db.edit(file, &same_length);
    let edited = read(&db, file, &same_length, Lang::Js);
    let (graphs, _) = executions(&db);
    assert_eq!(graphs, 1);
    let fresh = ResidentDatabase::default();
    assert_eq!(
        read(
            &fresh,
            fresh.open("A.vue", &same_length),
            &same_length,
            Lang::Js
        ),
        edited
    );
    let broken = same_length.replace("{{x}}", "{{(}}");
    db.edit(file, &broken);
    let error = view(&db, file)
        .reachable(Lang::Js, request(0))
        .err()
        .unwrap();
    assert!(
        matches!(error,TemplateGraphError::Incomplete {holes,diagnostics} if !holes.is_empty() || !diagnostics.is_empty())
    );
    let clean = ResidentDatabase::default();
    assert_eq!(
        Some(error),
        view(&clean, clean.open("A.vue", &broken))
            .reachable(Lang::Js, request(0))
            .err()
    );
    db.edit(file, first);
    assert_eq!(read(&db, file, first, Lang::Js), old);
}

#[test]
fn other_buffers_headers_and_language_inputs_have_separate_ownership() {
    let db = ResidentDatabase::default();
    let text = "<template><p>hi</p></template><style>x{}</style>";
    let a = db.open("A.vue", text);
    let b = db.open("B.vue", text);
    assert_eq!(read(&db, a, text, Lang::Js), read(&db, b, text, Lang::Js));
    assert_eq!(executions(&db), (2, 2));
    let _ = read(&db, a, text, Lang::Ts);
    assert_eq!(executions(&db), (1, 1));
    let style = db
        .document_blocks(a)
        .unwrap()
        .iter()
        .find(|v| v.kind() == &BlockKind::Style)
        .unwrap();
    assert_eq!(
        style.reachable(Lang::Js, request(0)).err(),
        Some(&TemplateGraphError::UnsupportedBlock)
    );
    let foreign = db.open("P.vue", "<template lang='pug'>p hello</template>");
    assert_eq!(
        view(&db, foreign).reachable(Lang::Js, request(0)).err(),
        Some(&TemplateGraphError::UnsupportedBlock)
    );
}

#[test]
fn scanning_all_starts_keeps_only_the_graph_and_evicted_answers_recheck() {
    let mut db = ResidentDatabase::default();
    let source = vize_l0::cstr!(
        "<script>1</script><template>{}</template>",
        "<p/>".repeat(256)
    );
    let file = db.open("Batch.vue", &source);
    {
        let block = view(&db, file);
        let batch = block.reach_batch(Lang::Js).unwrap();
        let mut witnesses = 0;
        for i in 0..256 {
            let row = batch.reachable(request(i)).unwrap();
            witnesses += row.relative().steps().len();
            assert!(row.authored_spans().all(|s| s.is_ok()));
        }
        assert_eq!(witnesses, 256 * 257 / 2);
        assert_eq!(executions(&db), (1, 0));
        for i in 0..256 {
            let one = block.reachable(Lang::Js, request(i)).unwrap();
            let transient = batch.reachable(request(i)).unwrap();
            assert_eq!(one.relative(), transient.relative());
            assert_eq!(
                one.authored_spans().collect::<Vec<_>>(),
                transient.authored_spans().collect::<Vec<_>>()
            );
        }
        assert_eq!(executions(&db), (0, 256));
    }
    db.edit(file, &source.replace("<script>1", "<script>1234"));
    let block = view(&db, file);
    let batch = block.reach_batch(Lang::Js).unwrap();
    for i in 0..256 {
        let one = block.reachable(Lang::Js, request(i)).unwrap();
        let transient = batch.reachable(request(i)).unwrap();
        assert_eq!(one.relative(), transient.relative());
        assert_eq!(
            one.authored_spans().collect::<Vec<_>>(),
            transient.authored_spans().collect::<Vec<_>>()
        );
    }
    let (graphs, recomputed) = executions(&db);
    assert_eq!(graphs, 0);
    assert_eq!(recomputed, 128);
}

#[test]
#[ignore = "uses the actual documents emitted by the cross-tool editor replay"]
fn original_template_graphs_follow_the_same_toolchain_session() {
    let inputs =
        std::fs::read_to_string(std::env::var_os("VIZE_TOOLCHAIN_HISTORY").unwrap()).unwrap();
    let revisions = inputs.split('\0').collect::<Vec<_>>();
    assert!(matches!(revisions.len(), 4..=6));
    let mut db = ResidentDatabase::default();
    let file = db.open("Setup雪🌸.vue", revisions[0]);
    let first = read(&db, file, revisions[0], Lang::Ts);
    assert!(!first.0.is_empty());
    assert_eq!(executions(&db), (1, 1));
    let mut previous = first.2.clone();
    for (i, text) in revisions[1..].iter().enumerate() {
        db.edit(file, text);
        let updated = read(&db, file, text, Lang::Ts);
        assert_eq!(updated.0, first.0);
        assert_eq!(updated.1, first.1);
        if i >= 3 {
            // Restoring and scoping the style after the template keep every authored site.
            assert_eq!(updated.2, previous);
        } else {
            assert_ne!(updated.2, previous);
        }
        assert_eq!(executions(&db), (0, 0));
        let fresh = ResidentDatabase::default();
        assert_eq!(
            read(&fresh, fresh.open("Setup雪🌸.vue", text), text, Lang::Ts),
            updated
        );
        previous = updated.2;
    }
    let changed = revisions[3].replace("{{other}}", "<i>{{other}}</i>");
    assert_ne!(changed, revisions[3]);
    db.edit(file, &changed);
    let updated = read(&db, file, &changed, Lang::Ts);
    assert_eq!(executions(&db), (1, 1));
    let fresh = ResidentDatabase::default();
    assert_eq!(
        read(
            &fresh,
            fresh.open("Setup雪🌸.vue", &changed),
            &changed,
            Lang::Ts
        ),
        updated
    );
}
