mod lifecycle;
mod selected;

use super::{
    Arc, NativeNavigationProject, NavigationRefusal, Position, SourceQueryProject, block_on,
};
use crate::server::ServerState;
use tower_lsp::lsp_types::{DocumentHighlight, DocumentHighlightKind as Kind, Range, Url};

fn uri() -> Url {
    Url::parse("file:///highlight.fixture").unwrap()
}
fn project(source: &str, language: &str) -> (Arc<ServerState>, NativeNavigationProject<'static>) {
    let state = Arc::new(ServerState::new());
    state
        .documents
        .open(uri(), source.into(), 1, language.into());
    let project = NativeNavigationProject::new(SourceQueryProject::new_server(Arc::clone(&state)));
    (state, project)
}
fn position(source: &str, byte: usize) -> Position {
    let prefix = source.get(..byte).unwrap();
    Position::new(
        prefix.bytes().filter(|b| *b == b'\n').count() as u32,
        prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    )
}
fn occurrence(source: &str, name: &str, index: usize) -> (Position, Range) {
    let byte = source.match_indices(name).nth(index).unwrap().0;
    let start = position(source, byte);
    (
        start,
        Range::new(start, position(source, byte + name.len())),
    )
}
fn expected(source: &str, name: &str, sites: &[(usize, Kind)]) -> Vec<DocumentHighlight> {
    sites
        .iter()
        .map(|(index, kind)| DocumentHighlight {
            range: occurrence(source, name, *index).1,
            kind: Some(*kind),
        })
        .collect()
}
fn worker(
    project: &NativeNavigationProject<'_>,
    selected: bool,
) -> Arc<super::super::worker::NavigationWorker> {
    let cache = if selected {
        &project.selected_workers
    } else {
        &project.workers
    };
    Arc::clone(cache.lock().get(&uri()).unwrap().result.as_ref().unwrap())
}
fn both(project: &NativeNavigationProject<'_>) {
    assert_eq!(
        block_on(project.highlights(&uri(), Position::new(0, 1))),
        Ok(vec![])
    );
    assert_eq!(
        block_on(project.template_highlights(&uri(), Position::new(0, 1))),
        Ok(vec![])
    );
}

#[test]
fn original_js_and_ts_use_real_read_write_roles_and_retain_one_program_file() {
    for language in ["javascript", "typescript"] {
        let source = "let value=1;value;value=2;value+=3;value++;";
        let (_, project) = project(source, language);
        let first = occurrence(source, "value", 0).0;
        let wanted = expected(
            source,
            "value",
            &[
                (0, Kind::TEXT),
                (1, Kind::READ),
                (2, Kind::WRITE),
                (3, Kind::WRITE),
                (4, Kind::WRITE),
            ],
        );
        assert_eq!(
            block_on(project.highlights(&uri(), first)),
            Ok(wanted.clone())
        );
        let original = worker(&project, false);
        let before = block_on(original.inspect(first)).unwrap();
        assert_eq!(before.parses, 1);
        assert_eq!(
            block_on(project.highlights(&uri(), occurrence(source, "value", 4).0)),
            Ok(wanted)
        );
        assert_eq!(before, block_on(original.inspect(first)).unwrap());
        assert!(Arc::ptr_eq(&original, &worker(&project, false)));
        assert_eq!(original.counts(), (1, 2));
    }
}

#[test]
fn imported_local_and_exported_local_have_no_remote_or_public_alias_fallback() {
    let source = "import { remote as local } from 'pkg';local;export { local as publicName };";
    let (_, project) = project(source, "typescript");
    assert_eq!(
        block_on(project.highlights(&uri(), occurrence(source, "local", 1).0)),
        Ok(expected(
            source,
            "local",
            &[(0, Kind::TEXT), (1, Kind::READ), (2, Kind::READ)]
        ))
    );
    for name in ["remote", "publicName", "pkg"] {
        assert_eq!(
            block_on(project.highlights(&uri(), occurrence(source, name, 0).0)),
            Ok(vec![])
        );
    }
}

#[test]
fn original_react_profiles_distinguish_component_reads_containers_and_static_properties() {
    for language in ["javascriptreact", "typescriptreact"] {
        let source = "const Widget=1;let value=1;const view=<Widget value={value}/>;value++;";
        let (_, project) = project(source, language);
        assert_eq!(
            block_on(project.highlights(&uri(), occurrence(source, "Widget", 1).0)),
            Ok(expected(
                source,
                "Widget",
                &[(0, Kind::TEXT), (1, Kind::READ)]
            ))
        );
        assert_eq!(
            block_on(project.highlights(&uri(), occurrence(source, "value", 2).0)),
            Ok(expected(
                source,
                "value",
                &[(0, Kind::TEXT), (2, Kind::READ), (3, Kind::WRITE)]
            ))
        );
        assert_eq!(
            block_on(project.highlights(&uri(), occurrence(source, "value", 1).0)),
            Ok(vec![])
        );
        assert_eq!(worker(&project, false).counts().0, 1);
    }
}

#[test]
fn original_vue_script_and_entity_embeds_keep_usage_authored_utf16_and_same_owner() {
    let source = "<script setup>/*kept\r\n😀*/ let café=1;café=2;café++;</script><template>{{caf&#233;}}</template>";
    let (_, project) = project(source, "vue");
    let query = occurrence(source, "café", 0).0;
    let mut wanted = expected(
        source,
        "café",
        &[(0, Kind::TEXT), (1, Kind::WRITE), (2, Kind::WRITE)],
    );
    wanted.extend(expected(source, "caf&#233;", &[(0, Kind::READ)]));
    assert_eq!(
        block_on(project.highlights(&uri(), query)),
        Ok(wanted.clone())
    );
    let original = worker(&project, false);
    let before = block_on(original.inspect(query)).unwrap();
    assert_eq!(
        block_on(project.highlights(&uri(), occurrence(source, "caf&#233;", 0).0)),
        Ok(wanted)
    );
    assert_eq!(before, block_on(original.inspect(query)).unwrap());
    assert_eq!(original.sfc_productions(), 1);
    assert_eq!(
        block_on(project.highlights(&uri(), Position::new(1, 1))),
        Err(NavigationRefusal::Position)
    );
    assert_eq!(
        block_on(project.highlights(&uri(), occurrence(source, "caf&#233;", 0).1.end)),
        Ok(vec![])
    );
}

#[test]
fn original_shadow_scopes_source_holes_and_foreign_family_refusals_are_not_name_scans() {
    let source = "let value=1;function f(value){value++;}value;/*value*/'value';";
    let (_, project) = project(source, "javascript");
    assert_eq!(
        block_on(project.highlights(&uri(), occurrence(source, "value", 0).0)),
        Ok(expected(
            source,
            "value",
            &[(0, Kind::TEXT), (3, Kind::READ)]
        ))
    );
    assert_eq!(
        block_on(project.highlights(&uri(), occurrence(source, "value", 2).0)),
        Ok(expected(
            source,
            "value",
            &[(1, Kind::TEXT), (2, Kind::WRITE)]
        ))
    );
    for index in [4, 5] {
        assert_eq!(
            block_on(project.highlights(&uri(), occurrence(source, "value", index).0)),
            Ok(vec![])
        );
    }
    assert_eq!(
        block_on(project.template_highlights(&uri(), occurrence(source, "value", 0).0)),
        Err(NavigationRefusal::Language)
    );
    assert!(project.selected_workers.lock().is_empty());
}

#[test]
fn genuine_original_file_refusal_stays_sticky_for_highlight_requests() {
    use vize_l2::file::FileIssueKind::{UnresolvedReference, UnsupportedSyntax};
    let (_, project) = project("const {value}=other;value;", "javascript");
    let first = block_on(project.highlights(&uri(), Position::new(0, 7)));
    let Err(NavigationRefusal::Producer(issues)) = &first else {
        panic!("original incomplete File must refuse highlights")
    };
    assert_eq!(
        issues
            .iter()
            .map(|issue| (issue.unit.index(), issue.span, issue.kind))
            .collect::<Vec<_>>(),
        vec![
            (0, vize_l0::Span::new(6, 19), UnsupportedSyntax),
            (0, vize_l0::Span::new(20, 25), UnresolvedReference),
        ]
    );
    for _ in 0..2 {
        assert_eq!(
            block_on(project.highlights(&uri(), Position::new(0, 7))),
            first
        );
    }
    assert_eq!(worker(&project, false).counts().0, 1);
}
