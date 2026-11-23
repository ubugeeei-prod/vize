use crate::*;
use vize_resident_graph::TemplateGraphObservation;

fn sites(db: &ResidentDatabase, file: SourceFile) -> Vec<(Span, Vec<u8>)> {
    let TemplateGraphObservation::Incomplete(issues) =
        TemplateGraphObservation::read(view(db, file), Lang::Js).unwrap()
    else {
        panic!("expected the unresolved expression");
    };
    assert!(!issues.holes().is_empty() && !issues.diagnostics().is_empty());
    let source = file.text(db);
    issues
        .holes()
        .iter()
        .map(|hole| hole.span)
        .chain(
            issues
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.span),
        )
        .map(|relative| {
            let (authored, spelling) = issues.source_site(relative).unwrap();
            assert_eq!(authored.slice(source.as_str()), spelling);
            (authored, spelling.as_bytes().to_vec())
        })
        .collect()
}

#[test]
fn retained_recovery_moves_with_its_block_and_expires_when_the_body_is_repaired() {
    let mut db = ResidentDatabase::default();
    let original = "<script>/*雪*/</script>\r\n<template><p>{{(}}</p></template>";
    let file = db.open("Broken.vue", original);
    let first = sites(&db, file);
    assert_eq!(executions(&db), (1, 0));
    let shifted = original.replace("雪", "雪🌸");
    db.edit(file, &shifted);
    let moved = sites(&db, file);
    assert_eq!(executions(&db), (0, 0));
    let offset = u32::try_from(shifted.len() - original.len()).unwrap();
    assert_eq!(first.len(), moved.len());
    for ((old, bytes), (new, now)) in first.iter().zip(moved) {
        assert_eq!(new, Span::new(old.start + offset, old.end + offset));
        assert_eq!(*bytes, now);
    }
    let repaired = shifted.replace("{{(}}", "{{x}}");
    assert_eq!(repaired.len(), shifted.len());
    db.edit(file, &repaired);
    let TemplateGraphObservation::Complete(graph) =
        TemplateGraphObservation::read(view(&db, file), Lang::Js).unwrap()
    else {
        panic!("the repaired body was admitted");
    };
    assert!(graph.authored_span(OpId::new(0)).is_ok());
    assert_eq!(executions(&db), (1, 0));
    db.edit(file, original);
    assert_eq!(sites(&db, file), first);
}

#[test]
fn external_templates_and_other_block_languages_keep_their_original_error() {
    let db = ResidentDatabase::default();
    for (file_name, text) in [
        ("External.vue", "<template src='./page.html'></template>"),
        ("Pug.vue", "<template lang='pug'>p hello</template>"),
        ("Script.vue", "<script>const value = 1;</script>"),
    ] {
        let file = db.open(file_name, text);
        let block = db.document_blocks(file).unwrap().iter().next().unwrap();
        let original = block.reach_batch(Lang::Js).err().unwrap();
        assert_eq!(original, &TemplateGraphError::UnsupportedBlock);
        let observed = TemplateGraphObservation::read(block, Lang::Js)
            .err()
            .unwrap();
        assert_eq!(observed, original);
    }
    assert_eq!(executions(&db), (0, 0));
}
