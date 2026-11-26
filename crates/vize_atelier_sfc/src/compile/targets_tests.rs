//! A multi-target compile gives every target the result of compiling that
//! target on its own, in every target order.

use std::path::{Path, PathBuf};

use super::{
    SfcCompileTarget, SfcScriptOutputMode, SfcTargetsCompileResult,
    compile_sfc_for_adapter_targets_with_project, compile_sfc_for_adapter_with_project,
};
use crate::types::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError,
    SfcParseOptions,
};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};

const DOM: SfcCompileTarget = SfcCompileTarget {
    ssr: false,
    vapor: false,
    script_output: SfcScriptOutputMode::SeparateTemplate,
};
pub(crate) const DOM_INLINE: SfcCompileTarget = SfcCompileTarget {
    script_output: SfcScriptOutputMode::InlineTemplate,
    ..DOM
};
pub(crate) const SSR: SfcCompileTarget = SfcCompileTarget { ssr: true, ..DOM };
const VAPOR: SfcCompileTarget = SfcCompileTarget {
    vapor: true,
    ..DOM_INLINE
};

/// Every target list the battery runs: client and server in both orders,
/// with a separate and an inline client render, a Vapor client beside a VDOM
/// server, and three targets at once.
pub(crate) const TARGET_LISTS: [&[SfcCompileTarget]; 7] = [
    &[DOM, SSR],
    &[SSR, DOM],
    &[DOM_INLINE, SSR],
    &[SSR, DOM_INLINE],
    &[VAPOR, SSR],
    &[SSR, VAPOR],
    &[DOM, SSR, DOM_INLINE],
];

/// The call-level options of one build.
#[derive(Clone, Copy)]
pub(crate) struct Build {
    pub(crate) is_prod: bool,
    pub(crate) source_map: bool,
    pub(crate) nuxt_page_meta: bool,
}

pub(crate) const BUILDS: [Build; 3] = [
    Build {
        is_prod: false,
        source_map: true,
        nuxt_page_meta: false,
    },
    Build {
        is_prod: true,
        source_map: false,
        nuxt_page_meta: false,
    },
    Build {
        is_prod: true,
        source_map: true,
        nuxt_page_meta: true,
    },
];

fn options(path: &str, build: Build) -> SfcCompileOptions {
    let mut options = SfcCompileOptions::default();
    options.script.id = Some(path.into());
    options.style.id = path.into();
    options.template.is_prod = build.is_prod;
    options
}

fn codegen(build: Build) -> CodegenOptions {
    CodegenOptions {
        source_map: build.source_map,
        ..Default::default()
    }
}

/// `descriptor` at `path` compiled for `targets` in one call.
pub(crate) fn compile_together(
    descriptor: &SfcDescriptor<'_>,
    path: &str,
    build: Build,
    targets: &[SfcCompileTarget],
) -> SfcTargetsCompileResult {
    compile_sfc_for_adapter_targets_with_project(
        descriptor,
        &options(path, build),
        targets,
        TemplateSyntaxMode::Standard,
        &CustomElementMatcher::default(),
        &codegen(build),
        &SfcCompileExperimentalOptions::default(),
        build.nuxt_page_meta,
        None,
    )
}

/// `descriptor` at `path` compiled for `target` on its own.
pub(crate) fn compile_alone(
    descriptor: &SfcDescriptor<'_>,
    path: &str,
    build: Build,
    target: &SfcCompileTarget,
) -> Result<SfcCompileResult, SfcError> {
    compile_sfc_for_adapter_with_project(
        descriptor,
        super::targets::target_options(&options(path, build), target),
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        codegen(build),
        target.script_output,
        SfcCompileExperimentalOptions::default(),
        build.nuxt_page_meta,
        None,
    )
}

/// Compile `source` at `path` for `targets` at once and one by one, and
/// check every target's result is the same. Returns how many targets
/// compiled. No type-resolution window is opened: every other test of the
/// process would read through it.
pub(crate) fn assert_targets_alike(
    path: &str,
    source: &str,
    build: Build,
    targets: &[SfcCompileTarget],
) -> usize {
    let parse = SfcParseOptions {
        filename: path.into(),
        ..Default::default()
    };
    let Ok(descriptor) = crate::parse::parse_sfc(source, parse) else {
        return 0;
    };
    let together = with_dependencies_restored(compile_together(&descriptor, path, build, targets));
    assert_eq!(together.len(), targets.len(), "{path}");
    let mut compiled = 0;
    for (index, (target, together)) in targets.iter().zip(together).enumerate() {
        let apart = compile_alone(&descriptor, path, build, target);
        compiled += usize::from(apart.is_ok());
        let json = |result| serde_json::to_value(result).expect("serializable result");
        assert_eq!(
            json(&together),
            json(&apart),
            "{path}: target {index} of {targets:?}"
        );
    }
    compiled
}

/// The results with each left-out dependency list put back, after checking
/// one was left out only when every target compiled.
fn with_dependencies_restored(
    compiled: SfcTargetsCompileResult,
) -> Vec<Result<SfcCompileResult, SfcError>> {
    let mut results = compiled.results;
    if compiled.dependencies_shared {
        assert!(
            results.iter().all(Result::is_ok),
            "only compiled lists are shared"
        );
        let first = match results.first() {
            Some(Ok(first)) => first.dependencies.clone(),
            _ => Vec::new(),
        };
        for result in results.iter_mut().skip(1).flatten() {
            assert!(result.dependencies.is_empty(), "a shared list is left out");
            result.dependencies.clone_from(&first);
        }
    }
    results
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root")
        .to_path_buf()
}

fn collect_vue_fixtures(dir: &Path, files: &mut Vec<(std::string::String, std::string::String)>) {
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
        let Some(stem) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".vue.txt"))
        else {
            continue;
        };
        let source = std::fs::read_to_string(&path).expect("fixture source");
        let file = path.with_file_name(vize_l0::cstr!("{stem}.vue").as_str());
        files.push((file.to_string_lossy().into_owned(), source));
    }
}

/// A target emits from the context's analyzed bindings and macros, and
/// changes both: the next target must start from the analyzed state again.
#[test]
fn each_target_starts_from_the_analyzed_context() {
    let cases = [
        // A setup binding hides the CSS module of the same name, which only
        // the analyzed bindings tell.
        "<script setup>\nconst theme = { dark: 'local' }\n</script>\n<template><p :class=\"theme.dark\">x</p></template>\n<style module=\"theme\">\n.dark { color: black }\n</style>\n",
        // TypeScript erased from the macro arguments of a JavaScript module.
        "<script setup lang=\"ts\">\nconst { count = 1 as number } = defineProps<{ count?: number }>()\nconst emit = defineEmits(['save'] as string[])\ndefineExpose({ double: () => (count as number) * 2 })\n</script>\n<template><p @click=\"emit('save')\">{{ count }}</p></template>\n",
    ];
    for source in cases {
        for build in BUILDS {
            for targets in TARGET_LISTS {
                let compiled = assert_targets_alike("/virtual/Restore.vue", source, build, targets);
                assert_eq!(compiled, targets.len(), "{source}");
            }
        }
    }
}

#[test]
fn every_target_of_the_compiler_fixture_corpus_compiles_as_it_does_alone() {
    let mut files = Vec::new();
    collect_vue_fixtures(
        &repository_root().join("tests/_fixtures/differential/compiler"),
        &mut files,
    );
    assert!(files.len() > 200, "{} fixtures", files.len());
    let mut compiled = 0;
    for build in BUILDS {
        for targets in TARGET_LISTS {
            for (path, source) in &files {
                compiled += assert_targets_alike(path, source, build, targets);
            }
        }
    }
    // Most of the corpus compiles, so the comparison is not of errors only.
    assert!(
        compiled > files.len() * BUILDS.len() * 10,
        "{compiled} compiled"
    );
}
