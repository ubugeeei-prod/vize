use super::{
    Arc, DocumentStore, ModuleOperandRefusal, ModuleSourceErrorKind, NativeNavigationProject,
    NavigationRefusal, NavigationWorker, Position, SourceQueryProject, SourceSnapshotCache,
    block_on, collect, range, uri, with_original, worker,
};
use crate::source_project::navigation::profile::{Profile, VueConfiguration};
use std::sync::atomic::AtomicUsize;
use vize_l0::config::VueVersion;

#[test]
fn module_link_empty_program_keeps_genuine_typed_read_refusal_and_old_queries() {
    for source in [
        "",
        "/* retained */",
        "\"use strict\";",
        "#!/usr/bin/env node\n",
    ] {
        let documents = DocumentStore::new();
        documents.open(uri(), source.into(), 1, "javascript".into());
        let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
        let worker = worker(&project);
        for _ in 0..2 {
            assert!(matches!(block_on(worker.module_operands()),
                Err(ModuleOperandRefusal::Sources(error))
                if error.kind == ModuleSourceErrorKind::EmptyProgram
                && error.unit.index() == 0
                && error.span == vize_l0::Span::new(0, source.len() as u32)));
        }
        assert_eq!(block_on(worker.definition(Position::new(0, 0))), Ok(None));
        assert_eq!(
            block_on(worker.references(Position::new(0, 0), true)),
            Ok(Vec::new())
        );
        assert_eq!(worker.counts(), (1, 4));
    }
}

#[test]
fn module_link_dynamic_and_empty_reexport_refuse_the_whole_original_prefix() {
    for (source, expected) in [
        (
            "import './before.ts'; import('./dynamic.ts');",
            ModuleSourceErrorKind::DynamicImport,
        ),
        (
            "import './before.ts'; function f(){return import('./nested.ts');}",
            ModuleSourceErrorKind::DynamicImport,
        ),
        (
            "import './before.ts'; export {} from './missing.ts';",
            ModuleSourceErrorKind::EmptySourceExport,
        ),
    ] {
        let documents = DocumentStore::new();
        documents.open(uri(), source.into(), 1, "javascript".into());
        let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
        let worker = worker(&project);
        assert!(matches!(block_on(worker.module_operands()),
            Err(ModuleOperandRefusal::Sources(error)) if error.kind == expected
            && error.unit.index() == 0 && error.span == vize_l0::Span::new(0, source.len() as u32)));
        assert_eq!(block_on(worker.definition(Position::new(0, 0))), Ok(None));
        assert_eq!(worker.counts(), (1, 2));
    }
}

#[test]
fn module_link_late_lone_surrogate_refuses_after_a_genuine_valid_prefix() {
    let source = "import './before.ts'; import '\\uD800';";
    with_original(source, |query, admission| {
        let view = admission.as_ref().unwrap();
        let mut visited = 0;
        let original = view.view.for_each(|_| visited += 1).unwrap_err();
        assert_eq!(visited, 1);
        assert_eq!(original.kind, ModuleSourceErrorKind::LoneSurrogateRequest);
        assert_eq!(query.file.imports().len(), 2);
        assert_eq!(
            collect(query, view, || false),
            Err(ModuleOperandRefusal::Sources(original))
        );
        // A genuine original read failure still discards a cancelled prefix.
        assert_eq!(
            collect(query, view, || true),
            Err(ModuleOperandRefusal::Sources(original))
        );
    });
}

#[test]
fn module_link_real_syntax_and_incomplete_file_refuse_without_new_error_relabeling() {
    for (source, language) in [
        ("import './before.ts'; const value=/x/uv;", "javascript"),
        (
            "import './before.ts'; const value:number|string=1;value;",
            "typescript",
        ),
        ("import './before.ts'; missing;", "javascript"),
    ] {
        let documents = DocumentStore::new();
        documents.open(uri(), source.into(), 1, language.into());
        let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
        let worker = worker(&project);
        let original = block_on(worker.definition(Position::new(0, 0))).unwrap_err();
        assert!(matches!(
            original,
            NavigationRefusal::Syntax | NavigationRefusal::Producer(_)
        ));
        assert_eq!(
            block_on(worker.module_operands()),
            Err(ModuleOperandRefusal::Navigation(original))
        );
    }
}

#[test]
fn module_link_jsx_profile_refusal_preserves_other_completed_program_queries() {
    let documents = DocumentStore::new();
    documents.open(
        uri(),
        "import './a.ts'; const value=1;value;".into(),
        1,
        "javascriptreact".into(),
    );
    let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
    let worker = worker(&project);
    assert!(
        matches!(block_on(worker.module_operands()), Err(ModuleOperandRefusal::Sources(error))
        if error.kind == ModuleSourceErrorKind::Profile)
    );
    assert!(
        block_on(worker.definition(Position::new(0, 31)))
            .unwrap()
            .is_some()
    );
    assert_eq!(worker.counts(), (1, 2));
}

#[test]
fn module_link_all_actual_vue_owner_families_refuse_without_program_promotion() {
    let documents = DocumentStore::new();
    documents.open(
        uri(),
        "<template><i></i></template>".into(),
        1,
        "vue".into(),
    );
    let snapshot = SourceSnapshotCache::default()
        .capture(&documents, &uri())
        .unwrap();
    let configuration = VueConfiguration {
        version: VueVersion::V3,
        configured_dialect: None,
        legacy: false,
        patterned: false,
    };
    for profile in [
        Profile::Vue(configuration),
        Profile::SelectedVue(configuration),
        Profile::TemplateNamesVue(configuration),
    ] {
        let worker = NavigationWorker::spawn(
            Arc::clone(&snapshot),
            Arc::new(AtomicUsize::new(0)),
            profile,
        )
        .unwrap();
        assert_eq!(
            block_on(worker.module_operands()),
            Err(ModuleOperandRefusal::Navigation(
                NavigationRefusal::Language
            ))
        );
        assert_eq!(worker.sfc_productions(), 1);
        worker.retire();
        worker.wait_exit();
    }
}

#[test]
fn module_link_original_legacy_receipt_refuses_only_links_and_keeps_modern_controls() {
    for (request, literal) in [
        ("\\056/child.ts", "1"),
        ("./child.ts", "010"),
        ("./child.ts", "08"),
        ("./child.ts", "09.5"),
        ("./child.ts", "'\\1'"),
        ("./child.ts", "'\\8'"),
        ("./child.ts", "'\\9'"),
        ("./child.ts", "'\\00'"),
    ] {
        let source = vize_l0::cstr!("import '{request}';\nconst value={literal};value;");
        let documents = DocumentStore::new();
        documents.open(uri(), source.to_string(), 1, "javascript".into());
        let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
        let worker = worker(&project);
        assert_eq!(
            block_on(worker.module_operands()),
            Err(ModuleOperandRefusal::Navigation(NavigationRefusal::Syntax))
        );
        // The neutral original Program/File and existing query family stay intact.
        let reference = 13 + literal.len() as u32;
        let declaration = tower_lsp::lsp_types::Location::new(uri(), range((1, 6), (1, 11)));
        assert_eq!(
            block_on(worker.definition(Position::new(1, reference))),
            Ok(Some(declaration.clone()))
        );
        assert_eq!(
            block_on(worker.references(Position::new(1, reference), true)),
            Ok(vec![
                declaration,
                tower_lsp::lsp_types::Location::new(
                    uri(),
                    range((1, reference), (1, reference + 5))
                )
            ])
        );
        assert_eq!(worker.counts(), (1, 3));
    }
    for literal in ["0o10", "0x10", "0b10", "'\\x01'", "'\\u0001'", "'\\\\1'"] {
        let source =
            vize_l0::cstr!("/*\\056 010*/import '\\x2e/child.ts';\nconst value={literal};value;");
        let documents = DocumentStore::new();
        documents.open(uri(), source.to_string(), 1, "javascript".into());
        let project = NativeNavigationProject::new(SourceQueryProject::new(&documents));
        let worker = worker(&project);
        let operands = block_on(worker.module_operands()).unwrap();
        assert_eq!(
            operands.decoded_requests().collect::<Vec<_>>(),
            ["./child.ts"]
        );
        let target = tower_lsp::lsp_types::Url::parse("file:///observed/child.ts").unwrap();
        assert_eq!(
            operands.into_links(std::slice::from_ref(&target)),
            Ok(vec![tower_lsp::lsp_types::DocumentLink {
                range: range((0, 19), (0, 34)),
                target: Some(target),
                tooltip: None,
                data: None,
            }])
        );
        assert_eq!(worker.counts(), (1, 1));
    }
}
