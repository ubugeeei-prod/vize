use super::{NUMBER, PROVIDER, Project, fs, json, pack};

#[test]
#[ignore = "requires the repository's pinned TypeScript and Vue packages"]
fn empty_native_answers_preserve_unused_authored_expectations() {
    let p = Project::new();
    p.write(
        "tsconfig.json",
        &fs::read_to_string(pack().join("tsconfig.json.txt")).unwrap(),
    );
    let host = "src/views/Empty.vue";
    let text = "<template>\r\n<!-- @vue-expect-error -->\r\n<div/>\r\n</template>\r\n";
    p.write(host, text);
    let snapshot = p.open(host, text, "vue");
    let session = p.start();
    let expected = json!([{
        "code":2578, "message":"Unused '@vue-expect-error' directive.",
        "range":{"start":{"line":1,"character":0},"end":{"line":1,"character":26}},
        "severity":1, "source":"vize/types"
    }]);
    for _ in 0..2 {
        let rows = crate::runtime::block_on(session.diagnostics(&snapshot)).unwrap();
        assert_eq!(serde_json::to_value(rows).unwrap(), expected);
    }
    crate::runtime::block_on(session.shutdown()).unwrap();
}

#[test]
#[ignore = "requires the repository's pinned TypeScript and Vue packages"]
fn edits_reproject_checked_ranges_and_callers_own_their_responses() {
    let p = Project::new();
    p.write(
        "tsconfig.json",
        &fs::read_to_string(pack().join("tsconfig.json.txt")).unwrap(),
    );
    p.write("src/components/GlobalBadge.vue", NUMBER);
    p.write("src/components.d.ts", PROVIDER);
    let host = "src/views/Edit.vue";
    let text = "<template>\r\n<GlobalBadge :level=\"'x'\" />\r\n</template>\r\n";
    p.write(host, text);
    let first = p.open(host, text, "vue");
    let session = p.start();
    for (step, line) in [(0, 1), (1, 3), (2, 1)] {
        let revision = match step {
            0 => first.clone(),
            1 => p.change(host, &format!("<!-- 🌸 -->\r\n\r\n{text}"), 2),
            _ => p.change(host, text, 3),
        };
        let expected = json!([{
            "code":2322, "message":"Type 'string' is not assignable to type 'number'.",
            "range":{"start":{"line":line,"character":14},"end":{"line":line,"character":19}},
            "severity":1, "source":"vize/types"
        }]);
        let read = || crate::runtime::block_on(session.diagnostics(&revision)).unwrap();
        let mut rows = read();
        assert_eq!(serde_json::to_value(&rows).unwrap(), expected);
        rows[0].message.clear();
        assert_eq!(serde_json::to_value(read()).unwrap(), expected);
    }
    crate::runtime::block_on(session.shutdown()).unwrap();
}

#[test]
#[ignore = "requires the repository's pinned TypeScript and Vue packages"]
fn alternating_hosts_and_raw_reads_keep_the_current_native_answer() {
    let p = Project::new();
    p.write(
        "tsconfig.json",
        &fs::read_to_string(pack().join("tsconfig.json.txt")).unwrap(),
    );
    p.write("src/components/GlobalBadge.vue", NUMBER);
    p.write("src/components.d.ts", PROVIDER);
    let bad_source = "<template>\n<GlobalBadge :level=\"'x'\" />\n</template>\n";
    let good_source = "<template>\n<GlobalBadge :level=\"123\" />\n</template>\n";
    let bad_path = "src/views/Bad.vue";
    let good_path = "src/views/Good.vue";
    p.write(bad_path, bad_source);
    p.write(good_path, good_source);
    let bad = p.open(bad_path, bad_source, "vue");
    let good = p.open(good_path, good_source, "vue");
    let session = p.start();
    crate::runtime::block_on(async {
        for document in [&bad, &good, &bad] {
            let rows = session.diagnostics(document).await.unwrap();
            if document.uri() == bad.uri() {
                assert_eq!(rows.len(), 1);
                assert_eq!(serde_json::to_value(&rows[0]).unwrap()["code"], 2322);
            } else {
                assert!(rows.is_empty());
            }
        }

        // A raw reader can consume a cursor whose owner retained the mapped
        // response. It must receive the full generated diagnostic at every read.
        let mut cursor = session
            .transaction
            .lock()
            .await
            .take()
            .unwrap()
            .read
            .into_native_snapshot();
        let overlays = p.state.corsa_overlays();
        let (options, declarations) = p
            .state
            .ambient_declaration_inputs(&session.root, &overlays)
            .await
            .unwrap();
        let path = vize_l0::path::physical_path(&p.path(bad_path));
        let overlays = super::super::borrowed(&overlays);
        let declarations = super::super::borrowed(&declarations);
        let read = async |cursor| {
            session
                .bridge
                .read_ambient_vue_diagnostics(
                    &path,
                    bad_source,
                    p.state.canonical_vue_document_options(),
                    &overlays,
                    &options,
                    &declarations,
                    cursor,
                )
                .await
                .unwrap()
        };
        let packet = |value: &vize_canon::corsa_bridge::AmbientVueDiagnostics| {
            value
                .diagnostics
                .iter()
                .map(|row| (row.phase, serde_json::to_value(&row.diagnostic).unwrap()))
                .collect::<Vec<_>>()
        };
        let expected = packet(&read(None).await);
        assert!(expected.iter().any(|(_, row)| row["code"] == 2322));
        for _ in 0..2 {
            let native = read(cursor.take()).await;
            assert_eq!(packet(&native), expected);
            cursor = native.snapshot;
        }
        drop(cursor);
        let request = || vize_canon::corsa_bridge::AmbientVueDiagnosticRequest {
            source_path: &path,
            content: bad_source,
            options: p.state.canonical_vue_document_options(),
            overlays: &overlays,
            virtual_ts_options: &options,
            requested_sources: &declarations,
        };
        let mapped = session
            .bridge
            .read_ambient_vue_diagnostic_batch(request(), None)
            .await
            .unwrap()
            .map(|host, rows| (host.code.len(), rows.len()));
        let answer = *mapped.value();
        assert!(answer.0 > 0);
        assert_eq!(answer.1, expected.len());
        let confirmed = session
            .bridge
            .read_ambient_vue_diagnostic_batch(request(), Some(mapped))
            .await
            .unwrap()
            .map(|_, _| panic!("the native snapshot and projection both matched"));
        assert_eq!(confirmed.into_value(), answer);
        session.shutdown().await.unwrap();
    });
}
