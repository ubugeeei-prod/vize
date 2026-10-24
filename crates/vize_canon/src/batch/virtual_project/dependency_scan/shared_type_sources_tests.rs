//! Reached SFCs share one dependency view without sharing root declarations.

use std::{fs, path::Path, path::PathBuf};

use vize_l0::{FxHashMap, cstr};

use super::VirtualProject;

const BEFORE: &str = "export interface Shared { sharedBefore: string }\n";
const AFTER: &str = "export interface Shared { sharedAfter: number }\n";
const FIRST: &str = r#"<script setup lang="ts">
import type { Shared } from './api';
type Props = Shared & { firstOnly: number };
defineProps<Props>();
</script>
<template><div>{{ sharedBefore }} {{ firstOnly }}</div></template>
"#;
const SECOND: &str = r#"<script setup lang="ts">
import type { Shared } from './api';
type Props = Shared & { secondOnly: boolean };
defineProps<Props>();
</script>
<template><div>{{ sharedBefore }} {{ secondOnly }}</div></template>
"#;
const HOST: &str = r#"import First from './First.vue';
import 'mutate-type-snapshot';
import Second from './Second.vue';
import 'restore-type-snapshot';
export { First, Second };
"#;

#[test]
fn reachable_siblings_share_dependency_text_between_registration_markers() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let host = root.join("entry.ts");
    let first = root.join("First.vue");
    let second = root.join("Second.vue");
    let api = root.join("api.ts");
    write(&host, HOST);
    write(&first, FIRST);
    write(&second, SECOND);
    write(&api, BEFORE);

    let stable = individually_registered(&root, &host, &[&first, &second], &Default::default());
    let mut shared = project(&root);
    shared.register_path(&host).unwrap();
    // Check the actual generated import order consumed by the dependency
    // walk, rather than relying on HashMap iteration or a wall-clock race.
    let code = &shared.find_by_original(&host).unwrap().content;
    let order = crate::batch::ImportRewriter::new()
        .collect_all_specifier_occurrences(code, oxc_span::SourceType::ts())
        .into_iter()
        .filter_map(|(specifier, _)| {
            if specifier.contains("First.vue") {
                Some("first")
            } else if specifier == "mutate-type-snapshot" {
                Some("mutate")
            } else if specifier.contains("Second.vue") {
                Some("second")
            } else if specifier == "restore-type-snapshot" {
                Some("restore")
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(order, ["first", "mutate", "second", "restore"]);
    let mut mutations = Vec::new();
    shared
        .register_reachable_dependencies_with_package_resolver(
            &Default::default(),
            &[],
            &mut |importer, specifier, _| {
                match specifier {
                    "mutate-type-snapshot" => {
                        assert_eq!(importer, host.as_path());
                        mutations.push("mutate");
                        write(&api, AFTER);
                    }
                    "restore-type-snapshot" => {
                        assert_eq!(importer, host.as_path());
                        mutations.push("restore");
                        write(&api, BEFORE);
                    }
                    _ => {}
                }
                false
            },
        )
        .unwrap();
    assert_eq!(mutations, ["mutate", "restore"]);
    assert_eq!(fs::read_to_string(&api).unwrap(), BEFORE);
    assert_prop(&shared, &first, "sharedBefore", true);
    assert_prop(&shared, &second, "sharedBefore", true);
    assert_prop(&shared, &second, "sharedAfter", false);
    assert_root_identities(&shared, &first, &second);
    assert_whole_project(&shared, &stable);

    // The same mutation placed between individual registrations must expose
    // the mixed view this regression prevents. Without passing the shared
    // cache into reached-file registration, the real walk does this too.
    let mut mixed = project(&root);
    mixed.register_path(&host).unwrap();
    mixed.register_path(&first).unwrap();
    write(&api, AFTER);
    mixed.register_path(&second).unwrap();
    write(&api, BEFORE);
    mixed.register_reachable_dependencies().unwrap();
    assert_prop(&mixed, &first, "sharedBefore", true);
    assert_prop(&mixed, &second, "sharedAfter", true);
    assert_prop(&mixed, &second, "sharedBefore", false);
    assert_ne!(
        mixed.find_by_original(&second).unwrap().content,
        stable.find_by_original(&second).unwrap().content
    );
}

#[test]
fn fresh_reachability_walks_refresh_helpers_unsaved_roots_and_config_targets() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let host = root.join("entry.ts");
    let first = root.join("First.vue");
    let second = root.join("Second.vue");
    let api = root.join("api.ts");
    write(&host, HOST);
    write(&first, FIRST);
    write(&second, SECOND);

    for (source, expected) in [
        (Some(BEFORE), Some("sharedBefore")),
        (Some(AFTER), Some("sharedAfter")),
        (None, None),
        (Some(BEFORE), Some("sharedBefore")),
    ] {
        if let Some(source) = source {
            write(&api, source);
        } else {
            fs::remove_file(&api).unwrap();
        }
        let unsaved = FIRST.replace("firstOnly", "unsavedOnly");
        let borrowed = [(first.clone(), unsaved.as_str())].into_iter().collect();
        let shared = reached_project(&root, &host, &borrowed);
        let stable = individually_registered(&root, &host, &[&first, &second], &borrowed);
        assert_whole_project(&shared, &stable);
        assert_prop(&shared, &first, "unsavedOnly", true);
        assert_prop(&shared, &first, "firstOnly", false);
        assert_prop(&shared, &second, "unsavedOnly", false);
        for child in [&first, &second] {
            for prop in ["sharedBefore", "sharedAfter"] {
                assert_prop(&shared, child, prop, expected == Some(prop));
            }
        }
        assert_eq!(fs::read_to_string(&first).unwrap(), FIRST);
    }

    // Separate walks on the SAME project must not retain the earlier type
    // snapshot. The newly discovered Second SFC sees the current helper;
    // First can keep its already registered projection in this graph patch.
    write(
        &host,
        "import First from './First.vue'; export { First };\n",
    );
    let mut retained = reached_project(&root, &host, &Default::default());
    let mut retained_reference =
        individually_registered(&root, &host, &[&first], &Default::default());
    write(&api, AFTER);
    write(&host, HOST);
    retained.register_path(&host).unwrap();
    retained.register_reachable_dependencies().unwrap();
    retained_reference.register_path(&host).unwrap();
    retained_reference.register_path(&second).unwrap();
    retained_reference
        .register_reachable_dependencies()
        .unwrap();
    assert_prop(&retained, &first, "sharedBefore", true);
    assert_prop(&retained, &second, "sharedAfter", true);
    assert_prop(&retained, &second, "sharedBefore", false);
    assert_whole_project(&retained, &retained_reference);

    write(
        &host,
        "import Selected from '@selected'; export { Selected };\n",
    );
    for (config, selected, absent) in [
        (
            r#"{"compilerOptions":{"strict":true,"paths":{"@selected":["./First.vue"]}}}"#,
            &first,
            &second,
        ),
        (
            r#"{"compilerOptions":{"strict":true,"noUncheckedIndexedAccess":true,"paths":{"@selected":["./Second.vue"]}}}"#,
            &second,
            &first,
        ),
    ] {
        write(&root.join("tsconfig.json"), config);
        let shared = reached_project(&root, &host, &Default::default());
        let stable = individually_registered(&root, &host, &[selected], &Default::default());
        assert!(shared.find_by_original(selected).is_some());
        assert!(shared.find_by_original(absent).is_none());
        assert_whole_project(&shared, &stable);
    }
}

#[test]
fn reached_runtime_props_and_default_names_rebind_each_local_import() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let host = root.join("entry.ts");
    let first = root.join("First.vue");
    let second = root.join("Second.vue");
    let first_defaults = root.join("FirstDefaults.vue");
    let second_defaults = root.join("SecondDefaults.vue");
    let runtime = r#"<script setup lang="ts">
import { sharedProps as rootBinding } from './runtime';
const props = defineProps(rootBinding);
const value: string = props.label;
</script>
<template>{{ value }} {{ label.toUpperCase() }}</template>
"#;
    let defaults = r#"<script setup lang="ts">
import { sharedDefaults as rootDefaults, type SharedProps } from './defaults';
const props = withDefaults(defineProps<SharedProps>(), rootDefaults);
const value: string = props.label;
</script>
<template>{{ value }} {{ label.toUpperCase() }}</template>
"#;
    write(
        &host,
        "import First from './First.vue'; import Second from './Second.vue'; import FirstDefaults from './FirstDefaults.vue'; import SecondDefaults from './SecondDefaults.vue'; export { First, Second, FirstDefaults, SecondDefaults };\n",
    );
    write(
        &root.join("runtime.ts"),
        "export const sharedProps = { label: { type: String, required: true } } as const;\n",
    );
    write(
        &root.join("defaults.ts"),
        "export interface SharedProps { label?: string }; export const sharedDefaults = { label: 'Ready' };\n",
    );
    write(&first, &runtime.replace("rootBinding", "firstBinding"));
    write(&second, &runtime.replace("rootBinding", "secondBinding"));
    write(
        &first_defaults,
        &defaults.replace("rootDefaults", "firstDefaults"),
    );
    write(
        &second_defaults,
        &defaults.replace("rootDefaults", "secondDefaults"),
    );
    let shared = reached_project(&root, &host, &Default::default());
    let stable = individually_registered(
        &root,
        &host,
        &[&first, &second, &first_defaults, &second_defaults],
        &Default::default(),
    );
    assert_whole_project(&shared, &stable);
    for child in [&first, &second, &first_defaults, &second_defaults] {
        assert_prop(&shared, child, "label", true);
    }
    assert!(
        !shared
            .find_by_original(&first)
            .unwrap()
            .content
            .contains("secondBinding")
    );
    assert!(
        !shared
            .find_by_original(&second)
            .unwrap()
            .content
            .contains("firstBinding")
    );
    assert!(
        !shared
            .find_by_original(&first_defaults)
            .unwrap()
            .content
            .contains("secondDefaults")
    );
    assert!(
        !shared
            .find_by_original(&second_defaults)
            .unwrap()
            .content
            .contains("firstDefaults")
    );
}

#[test]
fn reached_cyclic_runtime_exports_preserve_individual_root_resolution() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let host = root.join("entry.ts");
    let first = root.join("First.vue");
    let second = root.join("Second.vue");
    write(
        &host,
        "import First from './First.vue'; import Second from './Second.vue'; export { First, Second };\n",
    );
    write(
        &root.join("a.ts"),
        "export * from './b'; export const sharedProps = { a: { type: String } } as const;\n",
    );
    write(
        &root.join("b.ts"),
        "export * from './a'; export const sharedProps = { b: { type: Number } } as const;\n",
    );
    write(
        &first,
        "<script setup lang=\"ts\">import { sharedProps as firstBinding } from './a'; const props = defineProps(firstBinding);</script><template>{{ a }} {{ b }}</template>\n",
    );
    write(
        &second,
        "<script setup lang=\"ts\">import { sharedProps as secondBinding } from './b'; const props = defineProps(secondBinding);</script><template>{{ a }} {{ b }}</template>\n",
    );
    let shared = reached_project(&root, &host, &Default::default());
    let stable = individually_registered(&root, &host, &[&first, &second], &Default::default());
    for child in [&first, &second] {
        for prop in ["a", "b"] {
            assert_prop(&stable, child, prop, true);
        }
    }
    assert_whole_project(&shared, &stable);
}

#[test]
fn invalid_reached_sources_do_not_abort_valid_sibling_registration() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let host = root.join("entry.ts");
    let valid = root.join("Valid.vue");
    let malformed = root.join("Malformed.vue");
    let unreadable = root.join("Unreadable.vue");
    let missing = root.join("Absent.vue");
    write(
        &host,
        "import Malformed from './Malformed.vue'; import Unreadable from './Unreadable.vue'; import Absent from './Absent.vue'; import Valid from './Valid.vue'; export { Malformed, Unreadable, Absent, Valid };\n",
    );
    write(&valid, FIRST);
    write(&root.join("api.ts"), BEFORE);
    let incomplete = "<script setup lang=\"ts\">const value = 1;";
    write(&malformed, incomplete);
    fs::write(&unreadable, [0xff, 0xfe]).unwrap();
    assert!(vize_atelier_sfc::parse_sfc(incomplete, Default::default()).is_err());
    assert!(fs::read_to_string(&unreadable).is_err());
    let shared = reached_project(&root, &host, &Default::default());
    let stable = individually_registered(&root, &host, &[&valid], &Default::default());
    for declined in [&malformed, &unreadable, &missing] {
        assert!(shared.find_by_original(declined).is_none());
    }
    assert_prop(&shared, &valid, "sharedBefore", true);
    assert_prop(&shared, &valid, "firstOnly", true);
    assert_whole_project(&shared, &stable);
}

#[test]
fn shared_reachability_snapshot_keeps_sibling_package_and_root_type_identities_separate() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let host = root.join("entry.ts");
    let first = root.join("First.vue");
    let nested = root.join("nested");
    let second = nested.join("Second.vue");
    write(
        &host,
        "import First from './First.vue'; import Second from './nested/Second.vue'; export { First, Second };\n",
    );
    write(&first, &FIRST.replace("'./api'", "'@scope/shared-types'"));
    write(&second, &SECOND.replace("'./api'", "'@scope/shared-types'"));
    for (scope, props) in [
        (&root, "export interface Shared { outerPackage: string }\n"),
        (
            &nested,
            "export interface Shared { innerPackage: number }\n",
        ),
    ] {
        let package = scope.join("node_modules/@scope/shared-types");
        write(
            &package.join("package.json"),
            r#"{"name":"@scope/shared-types","types":"./index.d.ts"}"#,
        );
        write(&package.join("index.d.ts"), props);
    }
    let shared = reached_project(&root, &host, &Default::default());
    let stable = individually_registered(&root, &host, &[&first, &second], &Default::default());
    assert_whole_project(&shared, &stable);
    assert_root_identities(&shared, &first, &second);
    assert_prop(&shared, &first, "outerPackage", true);
    assert_prop(&shared, &first, "innerPackage", false);
    assert_prop(&shared, &second, "innerPackage", true);
    assert_prop(&shared, &second, "outerPackage", false);
}

#[test]
fn warmed_reachable_type_facts_do_not_expand_the_512_module_root_limit() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let host = root.join("entry.ts");
    let first = root.join("First.vue");
    let second = root.join("Second.vue");
    let wide = root.join("Wide.vue");
    write(
        &host,
        "import First from './First.vue'; import Second from './Second.vue'; import Wide from './Wide.vue'; export { First, Second, Wide };\n",
    );
    for index in 0..513 {
        write(
            &root.join(cstr!("api{index}.ts").as_str()),
            &cstr!("export type Shared{index} = {{ prop{index}: number }};\n"),
        );
    }
    let first_script = admission_script(0, 256, "firstValue");
    let second_script = admission_script(256, 513, "secondValue");
    let wide_script = admission_script(0, 513, "wideValue");
    for (path, script) in [
        (&first, &first_script),
        (&second, &second_script),
        (&wide, &wide_script),
    ] {
        write(
            path,
            &cstr!("<script setup lang=\"ts\">\n{script}</script>\n<template><div /></template>\n"),
        );
    }
    // The first two roots together warm every dependency before Wide is
    // reached. That shared store must not turn all 514 modules (root + 513
    // dependencies) into admitted facts in Wide's independent type world.
    let world = vize_atelier_sfc::script::ScriptCompileContext::new(&wide_script)
        .resolve_type_world(wide.to_str().unwrap(), None);
    assert_eq!(world.modules.len(), 514);
    assert_eq!(
        world
            .modules
            .values()
            .filter(|module| module.complete)
            .count(),
        512
    );
    assert_eq!(
        world
            .modules
            .values()
            .filter(|module| !module.complete)
            .count(),
        2
    );

    let shared = reached_project(&root, &host, &Default::default());
    let stable =
        individually_registered(&root, &host, &[&first, &second, &wide], &Default::default());
    assert_prop(&shared, &first, "firstValue", true);
    assert_prop(&shared, &second, "secondValue", true);
    assert_prop(&shared, &wide, "wideValue", true);
    assert_prop(&shared, &wide, "firstValue", false);
    assert_prop(&shared, &wide, "secondValue", false);
    assert_whole_project(&shared, &stable);
}

fn admission_script(start: usize, end: usize, prop: &str) -> vize_l0::String {
    let mut script = vize_l0::String::default();
    for index in start..end {
        script.push_str(&cstr!(
            "import type {{ Shared{index} }} from './api{index}';\n"
        ));
    }
    script.push_str(&cstr!("type Props = {{ {prop}: ["));
    for index in start..end {
        script.push_str(&cstr!("Shared{index},"));
    }
    script.push_str("]; };\ndefineProps<Props>();\n");
    script
}

fn project(root: &Path) -> VirtualProject {
    let mut project = VirtualProject::new(root).unwrap();
    project.set_session_script_registration(true);
    project
}

fn reached_project(
    root: &Path,
    host: &Path,
    overlays: &FxHashMap<PathBuf, &str>,
) -> VirtualProject {
    let mut project = project(root);
    project.register_path(host).unwrap();
    project
        .register_reachable_dependencies_with_overlays(overlays)
        .unwrap();
    project
}

fn individually_registered(
    root: &Path,
    host: &Path,
    children: &[&Path],
    overlays: &FxHashMap<PathBuf, &str>,
) -> VirtualProject {
    let mut project = project(root);
    project.register_path(host).unwrap();
    for child in children {
        if let Some(source) = overlays.get(*child) {
            project.register_path_with_content(child, source).unwrap();
        } else {
            project.register_path(child).unwrap();
        }
    }
    project
        .register_reachable_dependencies_with_overlays(overlays)
        .unwrap();
    project
}

fn assert_root_identities(project: &VirtualProject, first: &Path, second: &Path) {
    assert_prop(project, first, "firstOnly", true);
    assert_prop(project, first, "secondOnly", false);
    assert_prop(project, second, "secondOnly", true);
    assert_prop(project, second, "firstOnly", false);
}

fn assert_prop(project: &VirtualProject, child: &Path, prop: &str, expected: bool) {
    let code = &project.find_by_original(child).unwrap().content;
    assert_eq!(
        code.contains(cstr!("const {prop} = props[\"{prop}\"]").as_str()),
        expected,
        "{}: {prop}: {code}",
        child.display()
    );
}

fn assert_whole_project(actual: &VirtualProject, expected: &VirtualProject) {
    let mut actual_files = actual.virtual_files_sorted();
    let mut expected_files = expected.virtual_files_sorted();
    actual_files.sort_by(|left, right| left.virtual_path.cmp(&right.virtual_path));
    expected_files.sort_by(|left, right| left.virtual_path.cmp(&right.virtual_path));
    assert_eq!(actual_files.len(), expected_files.len());
    for (actual, expected) in actual_files.into_iter().zip(expected_files) {
        assert_eq!(actual.original_path, expected.original_path);
        assert_eq!(actual.virtual_path, expected.virtual_path);
        assert_eq!(
            actual.content,
            expected.content,
            "{}",
            actual.original_path.display()
        );
        assert_eq!(
            cstr!("{:?}", actual.source_map),
            cstr!("{:?}", expected.source_map)
        );
    }
    assert_eq!(
        cstr!("{:?}", actual.diagnostics()),
        cstr!("{:?}", expected.diagnostics())
    );
}

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}
