use std::{fs, path::Path, path::PathBuf, sync::Barrier};

use super::{VirtualProject, assert_ts_parses, assert_tsx_parses};

fn uncached_project(root: &Path, paths: &[PathBuf]) -> VirtualProject {
    let mut project = VirtualProject::new(root).unwrap();
    for path in paths {
        project.register_path(path).unwrap();
    }
    project
}

fn assert_equivalent_projects(cached: &VirtualProject, uncached: &VirtualProject) {
    let mut cached_files = cached.virtual_files_sorted();
    let mut uncached_files = uncached.virtual_files_sorted();
    // A TSX SFC also emits a .ts import shim with the same original path.
    cached_files.sort_by(|left, right| left.virtual_path.cmp(&right.virtual_path));
    uncached_files.sort_by(|left, right| left.virtual_path.cmp(&right.virtual_path));
    assert_eq!(cached_files.len(), uncached_files.len());
    for (cached, uncached) in cached_files.into_iter().zip(uncached_files) {
        assert_eq!(cached.original_path, uncached.original_path);
        assert_eq!(cached.virtual_path, uncached.virtual_path);
        assert_eq!(
            cached.content,
            uncached.content,
            "cached batch changed generated code for {}",
            cached.original_path.display()
        );
        assert_eq!(
            format!("{:?}", cached.source_map),
            format!("{:?}", uncached.source_map),
            "cached batch changed source mappings for {}",
            cached.original_path.display()
        );
    }
    assert_eq!(
        format!("{:?}", cached.diagnostics()),
        format!("{:?}", uncached.diagnostics()),
        "cached batch changed diagnostics"
    );
}

#[test]
fn cached_batch_matches_uncached_split_scripts_tsx_and_diagnostics() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("project");
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    // Shared source outside the registered project is read through the batch
    // snapshot. Its private generic default and forwarded type must retain
    // their declaring modules when different SFC roots reuse its parse.
    fs::write(
        dir.path().join("api.tsx"),
        "type Private = { label: string }; export interface Shared<T = Private> { item: T; sharedOnly: boolean }; export type { Public } from './leaf'; const node = <button />; void node;",
    )
    .unwrap();
    fs::write(
        dir.path().join("leaf.ts"),
        "export type Public = { label: string }",
    )
    .unwrap();
    let split = src.join("Split.vue");
    fs::write(
        &split,
        r#"<script lang="ts">
import type { Shared, Public } from '../../api'
export interface Props extends Omit<Shared<Public>, 'sharedOnly'> { normalOnly: number }
</script>
<script setup lang="ts">
const props = withDefaults(defineProps<Props>(), { normalOnly: 1 })
</script>
<template><div>{{ item.label }} {{ normalOnly }} {{ props.item.label }}</div></template>"#,
    )
    .unwrap();
    let tsx = src.join("Tsx.vue");
    fs::write(
        &tsx,
        r#"<script setup lang="tsx">
import type { Shared } from '../../api'
type Props = Shared<number> & { tsxOnly: boolean }
const props = defineProps<Props>()
const node = <button>{props.item}</button>
</script>
<template><div>{{ item }} {{ sharedOnly }} {{ tsxOnly }} {{ node }}</div></template>"#,
    )
    .unwrap();
    let invalid = src.join("Invalid.vue");
    fs::write(
        &invalid,
        r#"<script setup lang="ts">
import type { Shared } from '../../api'
type Props = Shared & { count?: number }
const { count = 'wrong' } = defineProps<Props>()
</script>
<template><div>{{ item.label }} {{ count }}</div></template>"#,
    )
    .unwrap();
    let paths = [split.clone(), tsx.clone(), invalid.clone()];
    let mut cached = VirtualProject::new(&root).unwrap();
    cached.register_paths(&paths).unwrap();
    assert_equivalent_projects(&cached, &uncached_project(&root, &paths));

    let split_code = &cached.find_by_original(&split).unwrap().content;
    assert_ts_parses(split_code);
    assert!(split_code.contains("const item = props[\"item\"]"));
    assert!(split_code.contains("const normalOnly = props[\"normalOnly\"]"));
    assert!(!split_code.contains("const sharedOnly = props[\"sharedOnly\"]"));
    assert!(!split_code.contains("tsxOnly"));
    let tsx_file = cached.find_by_original(&tsx).unwrap();
    assert_tsx_parses(&tsx_file.content);
    assert!(
        tsx_file
            .virtual_path
            .to_string_lossy()
            .ends_with(".vue.tsx")
    );
    assert!(
        tsx_file
            .content
            .contains("const tsxOnly = props[\"tsxOnly\"]")
    );
    assert!(
        tsx_file
            .content
            .contains("const sharedOnly = props[\"sharedOnly\"]")
    );
    assert!(!tsx_file.content.contains("normalOnly"));
    assert!(cached.diagnostics().iter().any(|diagnostic| {
        diagnostic.file == invalid
            && diagnostic
                .message
                .contains("DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE")
    }));
}

#[test]
fn each_batch_refreshes_edited_removed_and_recreated_external_dependencies() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("project");
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    let api = dir.path().join("api.ts");
    let paths: Vec<_> = ["First.vue", "Second.vue"]
        .into_iter()
        .map(|name| {
            let path = src.join(name);
            fs::write(
                &path,
                r#"<script setup lang="ts">
import type { Props } from '../../api'
const props = defineProps<Props>()
</script>
<template><div>{{ before }} {{ after }} {{ recreated }}</div></template>"#,
            )
            .unwrap();
            path
        })
        .collect();
    let mut cached = VirtualProject::new(&root).unwrap();
    for name in [Some("before"), Some("after"), None, Some("recreated")] {
        match name {
            Some(name) => {
                fs::write(&api, format!("export interface Props {{ {name}: string }}")).unwrap()
            }
            None => fs::remove_file(&api).unwrap(),
        }
        cached.register_paths(&paths).unwrap();
        assert_equivalent_projects(&cached, &uncached_project(&root, &paths));
        for path in &paths {
            let code = &cached.find_by_original(path).unwrap().content;
            for prop in ["before", "after", "recreated"] {
                assert_eq!(
                    code.contains(&format!("const {prop} = props[\"{prop}\"]")),
                    name == Some(prop),
                    "batch should expose only the current dependency's prop {prop}"
                );
            }
        }
    }
}

#[test]
fn simultaneous_batches_keep_project_and_root_type_identities_separate() {
    let dir = tempfile::tempdir().unwrap();
    let barrier = Barrier::new(4);
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
            .map(|index| {
                let barrier = &barrier;
                let root = dir.path().join(format!("project{index}"));
                scope.spawn(move || {
                    fs::create_dir_all(&root).unwrap();
                    fs::write(
                        root.join("api.ts"),
                        format!("export interface Shared {{ project{index}: string }}"),
                    )
                    .unwrap();
                    let paths: Vec<_> = (0..3)
                        .map(|file_index| {
                            let path = root.join(format!("App{file_index}.vue"));
                            fs::write(
                                &path,
                                format!(
                                    r#"<script setup lang="ts">
import type {{ Shared }} from './api'
type Props = Shared & {{ root{file_index}: number }}
const props = defineProps<Props>()
</script>
<template><div>{{{{ project{index} }}}} {{{{ root{file_index} }}}}</div></template>"#
                                ),
                            )
                            .unwrap();
                            path
                        })
                        .collect();
                    let mut cached = VirtualProject::new(&root).unwrap();
                    barrier.wait();
                    cached.register_paths(&paths).unwrap();
                    assert_equivalent_projects(&cached, &uncached_project(&root, &paths));
                    for (file_index, path) in paths.iter().enumerate() {
                        let code = &cached.find_by_original(path).unwrap().content;
                        assert!(code.contains(&format!(
                            "const project{index} = props[\"project{index}\"]"
                        )));
                        assert!(code.contains(&format!(
                            "const root{file_index} = props[\"root{file_index}\"]"
                        )));
                        for other in 0..4 {
                            if other != index {
                                assert!(!code.contains(&format!("project{other}")));
                            }
                        }
                        for other in 0..3 {
                            if other != file_index {
                                assert!(!code.contains(&format!("root{other}")));
                            }
                        }
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    });
}

#[test]
fn cached_batch_matches_uncached_above_the_world_module_limit_after_warming() {
    const DEPENDENCIES: usize = 513;
    let dir = tempfile::tempdir().unwrap();
    for index in 0..DEPENDENCIES {
        fs::write(
            dir.path().join(format!("dep{index}.ts")),
            format!("export interface Public {{ prop{index}: string }}"),
        )
        .unwrap();
    }
    let mut paths = Vec::new();
    // Each warm host admits fewer than 512 modules. Together they warm every
    // dependency before the large host asks for more than its world can admit.
    for (name, range) in [
        ("WarmFirst.vue", 0..257),
        ("WarmSecond.vue", 257..DEPENDENCIES),
        ("Large.vue", 0..DEPENDENCIES),
    ] {
        let mut imports = String::new();
        let mut references = Vec::new();
        for index in range {
            imports.push_str(&format!(
                "import type {{ Public as T{index} }} from './dep{index}';\n"
            ));
            references.push(format!("T{index}"));
        }
        let path = dir.path().join(name);
        fs::write(
            &path,
            format!(
                "<script setup lang=\"ts\">\n{imports}type Props = {};\nconst props = defineProps<Props>()\n</script>\n<template><div>{{{{ prop0 }}}} {{{{ prop512 }}}}</div></template>",
                references.join(" & ")
            ),
        )
        .unwrap();
        paths.push(path);
    }
    let mut cached = VirtualProject::new(dir.path()).unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap()
        .install(|| cached.register_paths(&paths).unwrap());
    assert_equivalent_projects(&cached, &uncached_project(dir.path(), &paths));
    let large = &cached.find_by_original(&paths[2]).unwrap().content;
    assert_ts_parses(large);
    assert!(large.contains("type Props = T0 & T1"));
    // The module admission cap must still leave some dependency props unknown
    // even though their unresolved module parses are already in the cache.
    let exposed_props = (0..DEPENDENCIES)
        .filter(|index| large.contains(&format!("const prop{index} = props[\"prop{index}\"]")))
        .count();
    assert!(exposed_props > 0, "large host should retain admitted props");
    assert!(
        exposed_props < DEPENDENCIES,
        "warm parses cannot bypass the admission cap"
    );
}
