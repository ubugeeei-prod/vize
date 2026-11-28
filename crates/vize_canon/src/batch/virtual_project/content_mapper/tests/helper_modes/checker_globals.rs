//! A module that embeds its helpers can still take the checker's globals from
//! the installed package, so that its emitted declaration names only helpers
//! declared in it and never augments a consumer's globals.

use std::path::Path;

use vize_l0::cstr;

use crate::batch::{
    ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options,
};

const SOURCE: &str = "<script setup lang='ts'>defineProps<{ name: string }>(); defineEmits<{ change: [value: string] }>();</script>\n<template><p>{{ name }}<slot /></p></template>";

const AUGMENTATION: &str = "// ImportMeta augmentation (reference existing framework types)
declare global {
  // Extend ImportMeta with Nuxt-specific properties not covered by vite/client
  interface ImportMeta {
    client: boolean;
    server: boolean;
    dev: boolean;
    prod: boolean;
    ssr: boolean;
  }
}

";

const MODULE_SCOPE: &str = "// ========== Module Scope (imports) ==========\n";

fn text(options: ContentMapperTransformOptions) -> vize_l0::String {
    generate_vue_content_mapper_transform_with_options(Path::new("src/App.vue"), SOURCE, options)
        .unwrap()
        .text
}

/// The preamble before the module scope, and the rest without the generated
/// offset comments that the preamble's length moves.
fn split(text: &str) -> (&str, Vec<&str>) {
    let (preamble, rest) = text.split_once(MODULE_SCOPE).unwrap();
    let rest = rest
        .lines()
        .filter(|line| line.trim_start().split_once("// @vize-map: ").is_none())
        .collect();
    (preamble, rest)
}

#[test]
fn checker_globals_module_replaces_only_the_global_augmentation() {
    let strict = ContentMapperTransformOptions::default();
    let mut loose = strict;
    loose.checks.check_unknown_props = false;
    for (options, import) in [
        (
            strict,
            "import type {} from \"vize/virtual-ts-checker-strict\";\n",
        ),
        (loose, "import type {} from \"vize/virtual-ts-checker\";\n"),
    ] {
        let embedded = text(options);
        let declaring = text(options.with_checker_globals_module(true));
        let (embedded_preamble, embedded_rest) = split(&embedded);
        let (declaring_preamble, declaring_rest) = split(&declaring);
        let header = embedded_preamble.split_once(AUGMENTATION).unwrap().0;
        assert_eq!(embedded_preamble, cstr!("{header}{AUGMENTATION}"));
        assert_eq!(declaring_preamble, cstr!("{header}{import}"));
        assert_eq!(
            declaring_rest, embedded_rest,
            "only the augmentation changes"
        );
        assert_eq!(
            declaring.matches("declare global").count(),
            0,
            "{declaring}"
        );
        for helper in ["type $PublicEmit<", "type $ComponentPublicBase<"] {
            assert_eq!(
                declaring.matches(helper).count(),
                1,
                "{helper}:\n{declaring}"
            );
        }
    }
}

#[test]
fn checker_globals_module_leaves_shared_modules_unchanged() {
    let shared = ContentMapperTransformOptions::default()
        .with_external_component_attrs_module(true)
        .with_external_component_type_helpers(true)
        .with_shared_virtual_module(true);
    assert_eq!(text(shared), text(shared.with_checker_globals_module(true)));
}
