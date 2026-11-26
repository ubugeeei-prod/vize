//! Each target of `compileSfcBatchForTargets` returns, field for field, what a
//! separate `compileSfcBatchWithResults` call with that target's options does.

use super::{
    BatchCompileOptionsNapi, BatchCompileResultWithFilesNapi, BatchCompileTargetNapi,
    BatchFileInputNapi, compile_sfc_batch_for_targets, compile_sfc_batch_with_results,
};
use std::path::{Path, PathBuf};

mod cases;
mod dependencies;
mod fields;
mod window;
use fields::{custom_blocks, macro_artifacts, module_shape, styles, type_project};

/// The options one build of an app compiles every target with.
#[derive(Clone, Copy)]
struct Build {
    name: &'static str,
    is_prod: bool,
    source_map: bool,
    nuxt_page_meta: bool,
    /// The client target compiles with Vapor; the SSR target never does.
    vapor: bool,
}

const BUILDS: [Build; 4] = [
    Build {
        name: "dev",
        is_prod: false,
        source_map: true,
        nuxt_page_meta: false,
        vapor: false,
    },
    Build {
        name: "prod",
        is_prod: true,
        source_map: false,
        nuxt_page_meta: false,
        vapor: false,
    },
    Build {
        name: "vapor",
        is_prod: false,
        source_map: true,
        nuxt_page_meta: false,
        vapor: true,
    },
    Build {
        name: "nuxt-prod",
        is_prod: true,
        source_map: true,
        nuxt_page_meta: true,
        vapor: false,
    },
];

/// The client and SSR targets of `build`, as the Vite plugin sets them.
fn targets(build: Build) -> [BatchCompileTargetNapi; 2] {
    [
        BatchCompileTargetNapi {
            ssr: Some(false),
            vapor: Some(build.vapor),
            inline_template: Some(build.is_prod),
        },
        BatchCompileTargetNapi {
            ssr: Some(true),
            vapor: Some(false),
            inline_template: Some(false),
        },
    ]
}

fn options(build: Build, target: Option<&BatchCompileTargetNapi>) -> BatchCompileOptionsNapi {
    BatchCompileOptionsNapi {
        ssr: target.and_then(|target| target.ssr),
        vapor: target.and_then(|target| target.vapor),
        inline_template: target.and_then(|target| target.inline_template),
        is_prod: Some(build.is_prod),
        nuxt_page_meta: Some(build.nuxt_page_meta),
        template_comments: Some(!build.is_prod),
        include_styles: Some(true),
        include_custom_blocks: Some(true),
        include_macro_artifacts: Some(true),
        include_hashes: Some(true),
        include_source_map: Some(build.source_map),
        ..Default::default()
    }
}

fn inputs(files: &[(String, String)]) -> Vec<BatchFileInputNapi> {
    files
        .iter()
        .map(|(path, source)| BatchFileInputNapi {
            path: path.clone(),
            source: source.clone(),
            scope_id: None,
        })
        .collect()
}

/// Compile `files` for both targets of every build, at once and apart, in
/// both target orders, check the results are the same, and return the first
/// build's errors per file as `[client, ssr]`. The order where the server
/// target comes first catches a target that leaves the shared analysis
/// changed for the next one.
fn assert_targets_compile_apart(files: &[(String, String)]) -> Vec<[Vec<String>; 2]> {
    let mut first_build_errors = Vec::new();
    for (build_index, build) in BUILDS.into_iter().enumerate() {
        let [client_after, ssr_first] = targets(build);
        assert_order_compiles_apart(files, build, vec![ssr_first, client_after]);
        let together = assert_order_compiles_apart(files, build, Vec::from(targets(build)));
        if let (0, [client, ssr]) = (build_index, together.targets.as_slice()) {
            first_build_errors = client
                .results
                .iter()
                .zip(&ssr.results)
                .map(|(client, ssr)| [client.errors.clone(), ssr.errors.clone()])
                .collect();
        }
    }
    first_build_errors
}

/// Compile `files` for `order` at once, and for each of its targets apart,
/// check the results are the same and return the call's result.
fn assert_order_compiles_apart(
    files: &[(String, String)],
    build: Build,
    order: Vec<BatchCompileTargetNapi>,
) -> super::BatchCompileTargetsResultNapi {
    let apart: Vec<_> = order
        .iter()
        .map(|target| {
            compile_sfc_batch_with_results(inputs(files), Some(options(build, Some(target))))
                .expect("separate batch")
        })
        .collect();
    let together = compile_sfc_batch_for_targets(inputs(files), Some(options(build, None)), order)
        .expect("multi-target batch");
    assert_eq!(together.targets.len(), apart.len());
    let together = with_shared_dependencies(together);
    for (index, (together, apart)) in together.targets.iter().zip(&apart).enumerate() {
        assert_same(build, index, together, apart);
    }
    together
}

/// The result with every dependency list a later target left out restored
/// from the first target, after checking it was left out only when equal.
fn with_shared_dependencies(
    mut together: super::BatchCompileTargetsResultNapi,
) -> super::BatchCompileTargetsResultNapi {
    assert_eq!(together.dependencies_shared.len(), files_of(&together));
    let Some((first, later)) = together.targets.split_first_mut() else {
        return together;
    };
    for (index, shared) in together.dependencies_shared.iter().enumerate() {
        let first = first.results.get(index).expect("first target result");
        for target in later.iter_mut() {
            let result = target.results.get_mut(index).expect("later target result");
            if *shared {
                assert_eq!(result.dependencies, None, "a shared list is left out");
                assert!(first.dependencies.is_some(), "only a list is shared");
                result.dependencies = first.dependencies.clone();
            }
        }
    }
    together
}

fn files_of(together: &super::BatchCompileTargetsResultNapi) -> usize {
    together
        .targets
        .first()
        .map_or(0, |target| target.results.len())
}

fn assert_same(
    build: Build,
    target: usize,
    together: &BatchCompileResultWithFilesNapi,
    apart: &BatchCompileResultWithFilesNapi,
) {
    assert_eq!(together.results.len(), apart.results.len());
    assert_eq!(
        together.success_count, apart.success_count,
        "{} {target}",
        build.name
    );
    assert_eq!(
        together.failed_count, apart.failed_count,
        "{} {target}",
        build.name
    );
    for (together, apart) in together.results.iter().zip(&apart.results) {
        // One field at a time names the field that differs.
        let file = &apart.path;
        let at = |field: &str| vize_l0::cstr!("{} target {target}: {file}: {field}", build.name);
        assert_eq!(together.path, apart.path, "{}", at("path"));
        assert_eq!(together.code, apart.code, "{}", at("code"));
        assert_eq!(together.map, apart.map, "{}", at("map"));
        assert_eq!(together.css, apart.css, "{}", at("css"));
        assert_eq!(together.scope_id, apart.scope_id, "{}", at("scope_id"));
        assert_eq!(
            together.has_scoped,
            apart.has_scoped,
            "{}",
            at("has_scoped")
        );
        assert_eq!(together.errors, apart.errors, "{}", at("errors"));
        assert_eq!(together.warnings, apart.warnings, "{}", at("warnings"));
        assert_eq!(
            together.template_hash,
            apart.template_hash,
            "{}",
            at("template_hash")
        );
        assert_eq!(
            together.style_hash,
            apart.style_hash,
            "{}",
            at("style_hash")
        );
        assert_eq!(
            together.script_hash,
            apart.script_hash,
            "{}",
            at("script_hash")
        );
        assert_eq!(styles(together), styles(apart), "{}", at("styles"));
        assert_eq!(
            custom_blocks(together),
            custom_blocks(apart),
            "{}",
            at("custom_blocks")
        );
        assert_eq!(
            macro_artifacts(together),
            macro_artifacts(apart),
            "{}",
            at("macro_artifacts")
        );
        assert_eq!(
            module_shape(together),
            module_shape(apart),
            "{}",
            at("module_shape")
        );
        assert_eq!(
            together.dependencies,
            apart.dependencies,
            "{}",
            at("dependencies")
        );
        assert_eq!(
            type_project(together),
            type_project(apart),
            "{}",
            at("type_project")
        );
    }
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root")
        .to_path_buf()
}

fn collect_vue_fixtures(dir: &Path, files: &mut Vec<(String, String)>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("fixture directory")
        .map(|entry| entry.expect("fixture entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_vue_fixtures(&path, files);
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(stem) = name.strip_suffix(".vue.txt") else {
            continue;
        };
        let source = std::fs::read_to_string(&path).expect("fixture source");
        let file = path.with_file_name(vize_l0::cstr!("{stem}.vue").as_str());
        files.push((file.to_string_lossy().into_owned(), source));
    }
}

#[test]
fn every_target_of_the_compiler_fixture_corpus_compiles_as_it_does_apart() {
    let mut files = Vec::new();
    collect_vue_fixtures(
        &repository_root().join("tests/_fixtures/differential/compiler"),
        &mut files,
    );
    assert!(files.len() > 200, "{} fixtures", files.len());
    assert_targets_compile_apart(&files);
}

fn case(name: &str, source: &str) -> (String, String) {
    (
        vize_l0::cstr!("/virtual/cases/{name}.vue").into(),
        source.into(),
    )
}

#[test]
fn target_specific_cases_compile_as_they_do_apart() {
    let files = [
        case(
            "CssVars",
            "<script setup>\nconst color = 'red'\n</script>\n<template><p class=\"a\">x</p></template>\n<style scoped>\n.a { color: v-bind(color) }\n</style>\n",
        ),
        case(
            "Pug",
            "<script setup>\nconst items = [1, 2]\n</script>\n<template lang=\"pug\">\nul\n  li(v-for=\"item in items\" :key=\"item\") {{ item }}\n</template>\n",
        ),
        case(
            "VModelDemotion",
            "<script setup>\nimport { reactive } from 'vue'\nconst state = reactive({ text: '' })\n</script>\n<template><input v-model=\"state\" /></template>\n",
        ),
        case(
            "PageMeta",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/_fixtures/differential/compiler/page-meta-runtime/Page.vue.txt"
            )),
        ),
        case(
            "DuplicateProps",
            "<template><div id=\"a\" id=\"b\">x</div></template>\n",
        ),
        case("Unparsable", "<template><div></template>\n<script setup>\n"),
        case(
            "TypedProps",
            "<script setup lang=\"ts\">\ndefineProps<{ label: string; count?: number }>()\n</script>\n<template><p>{{ label }} {{ count }}</p></template>\n",
        ),
    ];
    let errors = assert_targets_compile_apart(&files);
    // The duplicate is refused before the SSR template is lowered.
    let [_, ssr] = errors.get(4).expect("duplicate props case");
    assert!(!ssr.is_empty(), "the SSR target refuses duplicate props");
}

/// A project whose components type their props through imported modules: a
/// relative import, a `paths` alias, a star barrel, and a pug include.
fn typed_project(root: &Path) -> Vec<(String, String)> {
    let write = |name: &str, text: &str| {
        let path = root.join(name);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("directory");
        std::fs::write(path, text).expect("write");
    };
    write(
        "tsconfig.json",
        "{ \"compilerOptions\": { \"baseUrl\": \".\", \"paths\": { \"@models/*\": [\"src/models/*\"] } } }\n",
    );
    for index in 0..8 {
        write(
            &vize_l0::cstr!("src/models/Model{index}.ts"),
            &vize_l0::cstr!("export interface Model{index} {{\n  field{index}: string;\n}}\n"),
        );
    }
    write(
        "src/models/index.ts",
        &(0..8)
            .map(|index| vize_l0::cstr!("export * from \"./Model{index}\";\n"))
            .collect::<Vec<_>>()
            .concat(),
    );
    write(
        "src/types.ts",
        "import type { Model0 } from \"./models\";\n\nexport interface ItemProps extends Model0 {\n  id: string;\n  label: string;\n}\n",
    );
    write("src/components/partial.pug", "span included\n");
    let mut files = Vec::new();
    for index in 0..12 {
        let script = match index % 3 {
            0 => {
                "import type { ItemProps } from \"../types\";\n\nconst props = defineProps<ItemProps>();"
            }
            1 => {
                "import type { Model1 } from \"@models/Model1\";\n\nconst props = defineProps<Model1 & { label: string }>();"
            }
            _ => {
                "import type { Model2 } from \"../models\";\n\nconst props = defineProps<Model2 & { label: string }>();"
            }
        };
        let template = if index % 4 == 0 {
            "<template lang=\"pug\">\ndiv {{ props.label }}\n  include partial.pug\n</template>"
        } else {
            "<template><div>{{ props.label }}</div></template>"
        };
        let path = root.join(vize_l0::cstr!("src/components/Item{index}.vue").as_str());
        let source =
            vize_l0::cstr!("<script setup lang=\"ts\">\n{script}\n</script>\n\n{template}\n");
        std::fs::write(&path, source.as_str()).expect("write component");
        files.push((path.to_string_lossy().into_owned(), source.into()));
    }
    files
}

#[test]
fn imported_prop_types_resolve_alike_for_every_target() {
    let project = tempfile::tempdir().expect("temporary project");
    let files = typed_project(project.path());
    let errors = assert_targets_compile_apart(&files);
    for [client, ssr] in errors {
        assert!(client.is_empty(), "{client:?}");
        assert!(ssr.is_empty(), "{ssr:?}");
    }
    // Every component depends on its type modules, alike for both targets:
    // the SSR target leaves the list out.
    let build = BUILDS[0];
    let together = compile_sfc_batch_for_targets(
        inputs(&files),
        Some(options(build, None)),
        Vec::from(targets(build)),
    )
    .expect("multi-target batch");
    assert_eq!(together.dependencies_shared, vec![true; files.len()]);
}

#[test]
fn an_omitted_target_field_keeps_the_call_option() {
    let files = [case("Plain", "<template><p>plain</p></template>\n")];
    let build = BUILDS[0];
    let mut call = options(build, None);
    call.ssr = Some(true);
    let together = compile_sfc_batch_for_targets(
        inputs(&files),
        Some(call),
        vec![BatchCompileTargetNapi::default()],
    )
    .expect("multi-target batch");
    let ssr = BatchCompileTargetNapi {
        ssr: Some(true),
        ..Default::default()
    };
    let apart = compile_sfc_batch_with_results(inputs(&files), Some(options(build, Some(&ssr))))
        .expect("separate batch");
    let together = together.targets.first().expect("one target");
    assert_same(build, 0, together, &apart);
}

#[test]
fn a_call_without_targets_is_refused() {
    let refused = compile_sfc_batch_for_targets(vec![], None, vec![]);
    assert!(refused.is_err());
}
