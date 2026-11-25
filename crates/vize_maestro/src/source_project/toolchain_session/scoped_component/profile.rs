//! Timed requests include the real snapshot, lowering, lint passes, compiler,
//! map serialization and guarded publication. Formatting and typing share the
//! same sixth revision in the enclosing end-to-end test.
use super::*;
use crate::source_project::style_format::composition::compare_costs;
use vize_atelier_sfc::NativeSfcOutput;
use vize_patina::rules::css::CssLintResult;

fn compile_and_lint(
    docs: &DocumentStore,
    snapshot: &SourceSnapshot,
    session: &NativeScssSession<'_, '_>,
    rules: &CssLinter,
    share: bool,
) -> (NativeSfcOutput, Vec<CssLintResult>) {
    let owner = session.observation().l1ir();
    let (prepared, results): (Vec<_>, Vec<_>) = owner
        .admitted()
        .unwrap()
        .styles()
        .enumerate()
        .map(|(ordinal, _)| {
            snapshot
                .prepare_scss_with(docs, owner, ordinal, |style| {
                    rules.lint_file_style(StyleRuleInput::Observed(style), true)
                })
                .unwrap()
        })
        .unzip();
    let compiled = if share {
        session.emit(prepared.iter().map(PreparedScss::products))
    } else {
        session.emit([])
    }
    .unwrap();
    (compiled, results)
}

#[test]
#[ignore = "paired current editor requests with complete scoped compiler and lint responses"]
fn current_scoped_compiler_and_lint_publication() {
    let options = NativeSfcCompileOptions {
        filename: "Card🌸.vue",
        source_map: true,
        ..Default::default()
    };
    let rules = CssLinter::with_all_rules();
    for styles in [1, 8, 48] {
        let mut source = "<template>\r\n<p> a&#32; b </p>\r\n</template>".to_owned();
        for n in 0..styles {
            source.push_str(&format!("<style scoped lang=scss>$ink:red;.card{n}{{color:$ink;display:none;margin-left:4px}}</style>"));
        }
        let docs = DocumentStore::new();
        let uri = Url::parse("file:///Card🌸.vue").unwrap();
        docs.open(uri.clone(), source, 1, "vue".into());
        let cache = SourceSnapshotCache::default();
        let run = |shared| {
            let (query, _) = cache.begin_query(&docs, &uri).unwrap();
            let completed = if shared {
                block_on(
                    query.run_scoped_component(&docs, options, |snapshot, session| {
                        compile_and_lint(&docs, snapshot, session, &rules, true)
                    }),
                )
                .unwrap()
            } else {
                let documents = &docs;
                let registry = &rules;
                block_on(query.run(documents, |snapshot| async move {
                    let arena = Allocator::default();
                    let session = NativeScssSession::new_scoped(&arena, snapshot.source(), options);
                    compile_and_lint(documents, &snapshot, &session, registry, false)
                }))
                .unwrap()
            };
            completed.publish(&docs, core::convert::identity).unwrap()
        };
        let (before, original) = run(false);
        let (after, joined) = run(true);
        assert_eq!(before.document(), after.document());
        assert_eq!(before.css_document(), after.css_document());
        assert_eq!(before.source_map(), after.source_map());
        assert_eq!(before.css_source_map(), after.css_source_map());
        assert_eq!(joined.len(), styles);
        assert_eq!(original.len(), styles);
        for (left, right) in original.iter().zip(&joined) {
            assert!(!left.diagnostics.is_empty());
            assert_eq!(diagnostics(left), diagnostics(right));
        }
        compare_costs(&format!("scoped editor styles={styles}"), run);
    }
}
