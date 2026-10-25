use super::{IntrinsicTagTarget, NativeNavigationProject, NavigationRefusal, coordinates};
use crate::{document::DocumentStore, runtime::block_on, source_project::SourceQueryProject};
use std::sync::Arc;
use tower_lsp::lsp_types::{Position, Url};
use vize_l0::{Span, line_index::LineBreaks};

mod lifecycle;

fn uri() -> Url {
    Url::parse("file:///native.ts").unwrap()
}
fn cached(project: &NativeNavigationProject<'_>) -> Arc<super::worker::NavigationWorker> {
    Arc::clone(
        project
            .workers
            .lock()
            .get(&uri())
            .unwrap()
            .result
            .as_ref()
            .unwrap(),
    )
}

#[test]
fn original_intrinsic_targets_reuse_program_file_and_walk_for_both_headers() {
    for language in ["javascriptreact", "typescriptreact"] {
        for newline in ["\n", "\r\n"] {
            let source = "import Widget from 'widget';\nconst text = '<button>'; const comparison = 1 < 2;\nconst view = <div title='🧭'><button disabled>Save</button><Widget /></div>;\n".replace('\n', newline);
            let documents = DocumentStore::new();
            documents.open(uri(), source.clone(), 1, language.into());
            let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
            let lines = LineBreaks::Lsp.line_starts(&source).collect::<Vec<_>>();
            let position = |at| LineBreaks::Lsp.offset_to_position(&source, at);
            let target = |name: &str, at: usize| IntrinsicTagTarget {
                name: name.into(),
                range: coordinates::range(
                    &source,
                    &lines,
                    Span::new(at as u32, (at + name.len()) as u32),
                )
                .unwrap(),
            };
            for (name, at) in [
                ("div", source.rfind("<div").unwrap() + 1),
                ("button", source.rfind("<button").unwrap() + 1),
                ("button", source.find("</button").unwrap() + 2),
                ("div", source.find("</div").unwrap() + 2),
            ] {
                let expected = target(name, at);
                for at in [at, at + name.len() - 1] {
                    let (line, character) = position(at);
                    assert_eq!(
                        block_on(project.intrinsic_tag(&uri(), Position::new(line, character))),
                        Ok(Some(expected.clone()))
                    );
                }
            }
            let first = cached(&project);
            for at in [
                source.find("<button").unwrap() + 1,
                source.find("1 < 2").unwrap() + 2,
                source.find("Widget />").unwrap(),
                source.find("disabled").unwrap(),
                source.rfind("<button").unwrap() + "<button".len(),
            ] {
                let (line, character) = position(at);
                assert_eq!(
                    block_on(project.intrinsic_tag(&uri(), Position::new(line, character))),
                    Ok(None)
                );
            }
            assert!(Arc::ptr_eq(&first, &cached(&project)));
            assert_eq!(
                first.counts(),
                (1, 13),
                "one original parse across every target and refusal"
            );
            assert_eq!(
                first.program_walks(),
                1,
                "one original File walk across every query"
            );
            assert_eq!(
                project.cached_program_worker_for_test(&uri()).unwrap(),
                (Arc::as_ptr(&first) as usize, (1, 13))
            );
            let inspect_at = Position::new(2, 30);
            let original = block_on(first.inspect(inspect_at)).unwrap();
            assert_eq!(original, block_on(first.inspect(inspect_at)).unwrap());
            assert_eq!(original.statements, 4);
            assert_ne!(original.program, 0);
            assert_ne!(original.file, 0);
            assert_eq!(original.parses, 1);
        }
    }
}

#[test]
fn intrinsic_targets_refuse_wrong_profiles_and_recovery_without_fallback() {
    for (language, source, expected) in [
        (
            "typescript",
            "const view = <button />;",
            NavigationRefusal::Language,
        ),
        (
            "javascriptreact",
            "const view = <button",
            NavigationRefusal::Syntax,
        ),
        (
            "typescriptreact",
            "const view = <button>bad</div>;",
            NavigationRefusal::Syntax,
        ),
    ] {
        let documents = DocumentStore::new();
        documents.open(uri(), source.into(), 1, language.into());
        let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
        assert_eq!(
            block_on(project.intrinsic_tag(&uri(), Position::new(0, 15))),
            Err(expected)
        );
    }
}

#[test]
fn late_unsupported_namespace_keeps_the_complete_original_producer_refusal() {
    let documents = DocumentStore::new();
    documents.open(
        uri(),
        "const view = <><button/><ns:tag/></>;".into(),
        1,
        "typescriptreact".into(),
    );
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let error = block_on(project.intrinsic_tag(&uri(), Position::new(0, 15))).unwrap_err();
    let NavigationRefusal::Producer(issues) = error else {
        panic!("original producer refusal: {error:?}")
    };
    assert_eq!(
        issues
            .iter()
            .map(|issue| (issue.unit.index(), issue.span, issue.kind))
            .collect::<Vec<_>>(),
        [(
            0,
            Span::new(25, 31),
            vize_l2::file::FileIssueKind::UnsupportedSyntax
        )]
    );
}
