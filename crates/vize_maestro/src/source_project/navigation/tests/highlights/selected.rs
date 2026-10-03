use super::{
    Kind, NavigationRefusal, Position, block_on, expected, occurrence, position, project, uri,
    worker,
};

#[test]
fn original_handler_var_sites_and_read_write_uses_exclude_other_handlers() {
    let source = "<template><button @click='var value=$event;{var value=2;}value=3;value+=1;return value' @blur='let value=$event;value'/></template>";
    let (_, project) = project(source, "vue");
    let target = occurrence(source, "value", 4).0;
    assert_eq!(
        block_on(project.template_highlights(&uri(), target)),
        Ok(expected(
            source,
            "value",
            &[
                (0, Kind::TEXT),
                (1, Kind::TEXT),
                (2, Kind::WRITE),
                (3, Kind::WRITE),
                (4, Kind::READ)
            ]
        ))
    );
    assert_eq!(
        block_on(project.template_highlights(&uri(), occurrence(source, "value", 6).0)),
        Ok(expected(
            source,
            "value",
            &[(5, Kind::TEXT), (6, Kind::READ)]
        ))
    );
    assert_eq!(worker(&project, true).sfc_productions(), 1);
    assert!(project.workers.lock().is_empty());
}

#[test]
fn genuine_handler_shadowing_and_implicit_event_have_exact_site_roles() {
    let source = "<template><button @click='let value=$event;{let value=2;value++;}return value;$event' @blur='$event'/></template>";
    let (_, project) = project(source, "vue");
    assert_eq!(
        block_on(project.template_highlights(&uri(), occurrence(source, "value", 3).0)),
        Ok(expected(
            source,
            "value",
            &[(0, Kind::TEXT), (3, Kind::READ)]
        ))
    );
    assert_eq!(
        block_on(project.template_highlights(&uri(), occurrence(source, "value", 2).0)),
        Ok(expected(
            source,
            "value",
            &[(1, Kind::TEXT), (2, Kind::WRITE)]
        ))
    );
    assert_eq!(
        block_on(project.template_highlights(&uri(), occurrence(source, "$event", 0).0)),
        Ok(expected(
            source,
            "$event",
            &[(0, Kind::READ), (1, Kind::READ)]
        ))
    );
    assert_eq!(
        block_on(project.template_highlights(&uri(), occurrence(source, "$event", 2).0)),
        Ok(expected(source, "$event", &[(2, Kind::READ)]))
    );
}

#[test]
fn original_entity_crlf_non_bmp_write_sites_keep_one_handler_body_and_utf16() {
    let source = "<template><button @click='/*kept\r\n😀*/ let café=$event;caf&#233;=1;café++;return café'/></template>";
    let (_, project) = project(source, "vue");
    let query = occurrence(source, "café", 0).0;
    let mut wanted = expected(source, "café", &[(0, Kind::TEXT)]);
    wanted.extend(expected(source, "caf&#233;", &[(0, Kind::WRITE)]));
    wanted.extend(expected(
        source,
        "café",
        &[(1, Kind::WRITE), (2, Kind::READ)],
    ));
    assert_eq!(
        block_on(project.template_highlights(&uri(), query)),
        Ok(wanted.clone())
    );
    let original = worker(&project, true);
    let before = block_on(original.selected_inspect(query)).unwrap();
    let entity_tail = position(source, source.find("&#233;").unwrap() + 2);
    assert_eq!(
        block_on(project.template_highlights(&uri(), entity_tail)),
        Ok(wanted)
    );
    assert_eq!(before, block_on(original.selected_inspect(query)).unwrap());
    assert_eq!(before.productions, 1);
    assert!(before.handler_body.is_some());
    assert_eq!(
        block_on(project.template_highlights(&uri(), Position::new(1, 1))),
        Err(NavigationRefusal::Position)
    );
    assert_eq!(
        block_on(project.template_highlights(&uri(), occurrence(source, "café", 2).1.end)),
        Ok(vec![])
    );
    assert_eq!(
        block_on(project.template_highlights(&uri(), Position::new(0, 12))),
        Ok(vec![])
    );
}

#[test]
fn whole_selected_script_style_and_encoded_for_refusals_never_fallback_for_highlights() {
    for source in [
        "<script setup>let value=1;</script><template>{{value}}</template>",
        "<template><button @click='$event'/></template><style>button{color:red}</style>",
        "<template><div v-for='caf&#233; in it&#101;ms'/></template>",
    ] {
        let (_, project) = project(source, "vue");
        for _ in 0..2 {
            assert!(matches!(
                block_on(project.template_highlights(&uri(), Position::new(0, 1))),
                Err(NavigationRefusal::SelectedSfcProducer(_))
            ));
        }
        assert_eq!(worker(&project, true).sfc_productions(), 1);
        assert!(project.workers.lock().is_empty());
    }
}
