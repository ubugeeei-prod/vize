use super::{PathAliases, materialize, prepare};
use vize_l0::{String, cstr};

#[test]
fn self_and_mutual_imports_never_alias_lint_probe_documents() {
    for mutual in [false, true] {
        let project = tempfile::tempdir().unwrap();
        let canonical_root = project.path().canonicalize().unwrap();
        let root = canonical_root.as_path();
        let session = root.join("private-session");
        let first = root.join("First.vue");
        let second = root.join("Second.vue");
        let source = format!(
            "<script setup lang=\"ts\">import Peer from './{}.vue'; defineProps<{{ label: string }}>();</script><template><span /></template>",
            if mutual { "Second" } else { "First" }
        );
        std::fs::write(&first, &source).unwrap();
        if mutual {
            std::fs::write(&second, "<script setup lang=\"ts\">import Peer from './First.vue'; defineProps<{ label: string }>();</script><template><span /></template>").unwrap();
        }
        let descriptor = vize_atelier_sfc::parse_sfc(&source, Default::default()).unwrap();
        let generated = super::project_component(&descriptor, None, &first.to_string_lossy());
        let mut cache = vize_l0::FxHashMap::default();
        let aliases = PathAliases::default();
        let prepared = prepare(
            &generated,
            &first.to_string_lossy(),
            &session,
            root,
            &aliases,
            &mut cache,
            &vize_l0::FxHashSet::default(),
        );
        assert_eq!(prepared.files.len(), if mutual { 2 } else { 1 });
        let paths: vize_l0::FxHashSet<_> = prepared
            .files
            .iter()
            .map(|(path, _)| path.clone())
            .collect();
        assert_eq!(
            paths.len(),
            prepared.files.len(),
            "one mirror per component"
        );
        for path in [&first, &second] {
            let probe = super::virtual_file_path(&session, root, &path.to_string_lossy());
            assert!(
                !paths.contains(&probe),
                "dependencies cannot own a lint document"
            );
        }
        for (_, code) in &prepared.files {
            let allocator = oxc_allocator::Allocator::default();
            let parsed =
                oxc_parser::Parser::new(&allocator, code, oxc_span::SourceType::ts()).parse();
            assert!(parsed.diagnostics.is_empty());
            let mut specifiers = super::Specifiers::default();
            oxc_ast_visit::Visit::visit_program(&mut specifiers, &parsed.program);
            assert!(!specifiers.0.is_empty(), "fixture must retain the cycle");
            let mut component_edges = 0;
            for (_, _, specifier) in specifiers.0 {
                if specifier == "vue" {
                    continue;
                }
                component_edges += 1;
                assert!(
                    paths.contains(&std::path::PathBuf::from(specifier.as_str())),
                    "a cycle must point to a component mirror: {specifier}"
                );
            }
            assert_eq!(
                component_edges, 1,
                "the authored component cycle stays intact"
            );
        }
        let retained = paths;
        let again = prepare(
            &generated,
            &first.to_string_lossy(),
            &session,
            root,
            &aliases,
            &mut cache,
            &retained,
        );
        assert!(
            again.files.is_empty(),
            "retained mirrors are not materialized again"
        );
    }
}

#[test]
fn tsx_and_external_component_mirrors_keep_disjoint_identities() {
    let project = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let project_root = project.path().canonicalize().unwrap();
    let outside_root = outside.path().canonicalize().unwrap();
    for root in [project_root.as_path(), outside_root.as_path()] {
        let child = root.join("Child'😀.vue");
        std::fs::write(&child, "<script setup lang=\"tsx\">const render = () => <span />; defineProps<{ label: string }>();</script><template />").unwrap();
        let source = format!(
            "import Child from {};\nconst component = Child;\n",
            serde_json::to_string(child.to_str().unwrap()).unwrap()
        );
        let session = project_root.join("private-session");
        let mut cache = vize_l0::FxHashMap::default();
        let prepared = prepare(
            &source,
            &project_root.join("Parent.vue").to_string_lossy(),
            &session,
            &project_root,
            &PathAliases::default(),
            &mut cache,
            &vize_l0::FxHashSet::default(),
        );
        assert_eq!(prepared.files.len(), 1);
        let (target, code) = prepared.files.first().unwrap();
        assert!(target.to_string_lossy().ends_with(".patina.component.tsx"));
        assert_ne!(
            target,
            &super::virtual_file_path(&session, &project_root, &child.to_string_lossy())
        );
        assert!(target.starts_with(&session));
        assert_eq!(
            target.starts_with(session.join("external-components")),
            root == outside_root
        );
        let allocator = oxc_allocator::Allocator::default();
        let parsed = oxc_parser::Parser::new(&allocator, code, oxc_span::SourceType::tsx()).parse();
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    }
}

#[test]
fn projected_imports_escape_both_quotes_and_preserve_unicode_query_offsets() {
    let root = std::env::temp_dir().join(&*cstr!(
        "vize-patina-quoted-import-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let project = root.join("owner'\"view😀");
    std::fs::create_dir_all(&project).expect("create quoted project directory");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unsafe-template-binding/component-values/ChildPanel.vue");
    std::fs::copy(fixture, project.join("Child.vue"))
        .expect("copy authored child into private project");
    let filename = project.join("Parent.vue");
    let session = project.join("private-session");
    for quote in ['\'', '"'] {
        let source = cstr!(
            "<script setup lang=\"ts\">\nimport Child from {quote}./Child.vue{quote};\nconst marker = Child;\n</script>"
        );
        let descriptor = vize_atelier_sfc::parse_sfc(&source, Default::default()).unwrap();
        let projected = super::project_component(&descriptor, None, &filename.to_string_lossy());
        let mut cache = vize_l0::FxHashMap::default();
        let aliases = PathAliases::default();
        let prepared = prepare(
            &projected,
            &filename.to_string_lossy(),
            &session,
            &project,
            &aliases,
            &mut cache,
            &vize_l0::FxHashSet::default(),
        );
        assert_eq!(prepared.files.len(), 1, "quoted dependency must resolve");
        let (target, _) = prepared.files.first().expect("private child mirror");
        let rewritten = prepared.code.expect("component import rewrite");
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, &rewritten, oxc_span::SourceType::ts()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{quote}: {:?}",
            parsed.diagnostics
        );
        let mut specifiers = super::Specifiers::default();
        oxc_ast_visit::Visit::visit_program(&mut specifiers, &parsed.program);
        assert!(
            specifiers.0.iter().any(|(_, _, path)| {
                path.as_str() == target.to_string_lossy().replace('\\', "/")
            })
        );
        let before = projected
            .find("const marker = Child;")
            .expect("projected marker") as u32;
        let after = rewritten
            .find("const marker = Child;")
            .expect("rewritten marker") as u32;
        assert_eq!(prepared.source_map.get_virtual_offset(before), after);
        let utf16 = super::super::probe::byte_offset_to_utf16_offset(&rewritten, after);
        assert_eq!(
            utf16,
            rewritten
                .get(..after as usize)
                .unwrap()
                .encode_utf16()
                .count() as u32
        );
        assert!(
            utf16 < after,
            "Unicode path must distinguish bytes and UTF-16"
        );
    }
    std::fs::remove_dir_all(root).expect("remove private quoted project");
}

#[test]
fn retained_component_frontier_is_materialized_once_for_a_chain() {
    let root = std::env::temp_dir().join(&*cstr!(
        "vize-patina-retained-frontier-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let project = root.join("project");
    let session = root.join("session");
    std::fs::create_dir_all(&project).expect("create component project");
    const COMPONENTS: usize = 24;
    for index in 0..COMPONENTS {
        let import = (index > 0)
            .then(|| cstr!("import Item{} from './Item{}.vue';\n", index - 1, index - 1))
            .unwrap_or_default();
        std::fs::write(
            project.join(cstr!("Item{index}.vue").as_str()),
            cstr!(
                "<script setup lang=\"ts\">\n{import}defineProps<{{ label: string }}>();\n</script>\n<template><Item{} v-if=\"false\" label=\"child\" /></template>\n",
                index.saturating_sub(1)
            ),
        )
        .expect("write chain component");
    }

    let mut cache = vize_l0::FxHashMap::default();
    let mut retained = vize_l0::FxHashSet::default();
    let aliases = PathAliases::default();
    let mut materialized = 0;
    for index in (0..COMPONENTS).rev() {
        let import = (index > 0)
            .then(|| cstr!("import Item{} from './Item{}.vue';\n", index - 1, index - 1))
            .unwrap_or_default();
        let filename = project.join(cstr!("Item{index}.vue").as_str());
        let prepared = prepare(
            &import,
            &filename.to_string_lossy(),
            &session,
            &project,
            &aliases,
            &mut cache,
            &retained,
        );
        materialized += prepared.files.len();
        retained.extend(prepared.files.into_iter().map(|(path, _)| path));
    }

    assert_eq!(materialized, COMPONENTS - 1);
    assert_eq!(retained.len(), COMPONENTS - 1);
    assert_eq!(cache.len(), COMPONENTS - 1);
    std::fs::remove_dir_all(root).expect("remove retained frontier project");
}

#[test]
fn real_component_imports_have_private_mirrors_and_keep_query_offsets() {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unsafe-template-binding/component-values");
    let child = fixtures.join("ChildPanel.vue");
    let missing = fixtures.join("DefinitelyMissing.vue");
    let source = cstr!(
        "import Child from '{}';\nimport Missing from '{}';\nconst value = Child;\n",
        child.display(),
        missing.display()
    );
    let session = fixtures.join("private-test-session");
    let mut cache = vize_l0::FxHashMap::default();
    let aliases = PathAliases::default();
    let prepared = prepare(
        &source,
        &fixtures.join("ParentPanel.vue").to_string_lossy(),
        &session,
        &fixtures,
        &aliases,
        &mut cache,
        &vize_l0::FxHashSet::default(),
    );
    assert_eq!(prepared.files.len(), 1);
    let (target, component) = &prepared.files[0];
    assert_eq!(target, &session.join("ChildPanel.vue.patina.component.ts"));
    assert_eq!(
        component
            .matches("export default __vize_component__")
            .count(),
        1
    );
    let rewritten = prepared.code.expect("component import was rewritten");
    assert_eq!(
        rewritten,
        cstr!(
            "import Child from '{}';\nimport Missing from '{}';\nconst value = Child;\n",
            target.display(),
            missing.display()
        )
    );
    let before = source.rfind("Child;").unwrap() as u32;
    let after = rewritten.rfind("Child;").unwrap() as u32;
    assert_eq!(prepared.source_map.get_virtual_offset(before), after);
}

#[test]
fn path_aliased_vue_imports_have_private_mirrors() {
    let project = tempfile::tempdir().unwrap();
    let project_root = project.path().canonicalize().unwrap();
    let child = project_root.join("src/ui/Child.vue");
    std::fs::create_dir_all(child.parent().unwrap()).unwrap();
    std::fs::write(
        &child,
        "<script setup lang=\"ts\">defineProps<{ label: string }>()</script><template />",
    )
    .unwrap();
    let source = "import Child from '#ui/Child.vue';\nconst value = Child;\n";
    let session = project_root.join("private-session");
    let config = serde_json::json!({
        "compilerOptions": {
            "paths": {"#ui/*": [project_root.join("src/ui/*")]}
        }
    });
    let aliases = PathAliases::from_tsconfig(&config.to_string(), &project_root);
    let mut cache = vize_l0::FxHashMap::default();
    let prepared = prepare(
        source,
        &project_root.join("src/Parent.vue").to_string_lossy(),
        &session,
        &project_root,
        &aliases,
        &mut cache,
        &vize_l0::FxHashSet::default(),
    );
    assert_eq!(prepared.files.len(), 1);
    let (target, component) = &prepared.files[0];
    assert_eq!(
        target,
        &session.join("src/ui/Child.vue.patina.component.ts")
    );
    assert_eq!(
        component
            .matches("export default __vize_component__")
            .count(),
        1
    );
    let rewritten = prepared
        .code
        .expect("aliased component import was rewritten");
    assert_eq!(
        rewritten,
        cstr!(
            "import Child from '{}';\nconst value = Child;\n",
            target.display()
        )
    );
}

#[test]
fn component_mirror_refreshes_report_actual_creations_changes_and_deletions() {
    let root = std::env::temp_dir().join(&*cstr!(
        "vize-patina-component-mirror-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = root.join("Child.vue.patina.ts");
    let files = vec![(path.clone(), String::from("export default 1;\n"))];
    let created = materialize(&files, &[]).expect("create component mirror");
    assert_eq!(
        (
            created.created.len(),
            created.changed.len(),
            created.deleted.len()
        ),
        (1, 0, 0)
    );
    let unchanged =
        materialize(&files, std::slice::from_ref(&path)).expect("unchanged component mirror");
    assert_eq!(
        (
            unchanged.created.len(),
            unchanged.changed.len(),
            unchanged.deleted.len()
        ),
        (0, 0, 0)
    );
    let changed_files = vec![(path.clone(), String::from("export default 2;\n"))];
    let changed =
        materialize(&changed_files, std::slice::from_ref(&path)).expect("changed component mirror");
    assert_eq!(
        (
            changed.created.len(),
            changed.changed.len(),
            changed.deleted.len()
        ),
        (0, 1, 0)
    );
    let deleted = materialize(&[], std::slice::from_ref(&path)).expect("removed component mirror");
    assert_eq!(
        (
            deleted.created.len(),
            deleted.changed.len(),
            deleted.deleted.len()
        ),
        (0, 0, 1)
    );
    assert!(!path.exists());
    std::fs::remove_dir_all(root).expect("remove private mirror test directory");
}
