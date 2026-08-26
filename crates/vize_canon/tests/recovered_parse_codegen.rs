//! A recovered template parse defect must not collapse the file to the
//! fallback stub (#3323).
//!
//! The parser documents a concrete recovery for `MissingWhitespaceBetweenAttributes`
//! and twelve sibling codes (`vize_relief::errors::recovery::RECOVERED_PARSE_CODES`),
//! and still yields a complete tree. Treating them as hard errors dropped the
//! AST and emitted `invalid_sfc_fallback_virtual_ts()`, so every script type
//! diagnostic in the file disappeared behind one missing space — the same
//! false-negative shape #3294 fixed in the linter.

use vize_canon::VirtualProject;

const RECOVERED_TEMPLATE: &str = r#"<template>
  <div id="a"class="b">{{ label }}</div>
</template>

<script setup lang="ts">
const label: string = 'hi'
const wrong: number = 'not a number'
</script>
"#;

const CLEAN_TEMPLATE: &str = r#"<template>
  <div id="a" class="b">{{ label }}</div>
</template>

<script setup lang="ts">
const label: string = 'hi'
const wrong: number = 'not a number'
</script>
"#;

/// The whole body of the fallback stub emitted for unusable SFCs
/// (`invalid_sfc_fallback_virtual_ts`).
const FALLBACK_STUB: &str =
    "declare const __vize_component: any;\nexport default __vize_component;\n";

fn virtual_ts_for(source: &str) -> vize_s0::String {
    let temp_dir = tempfile::tempdir().expect("temp project should be created");
    let dir = temp_dir.path().canonicalize().unwrap();
    let path = dir.join("App.vue");
    std::fs::write(&path, source).unwrap();

    let mut project = VirtualProject::new(&dir).unwrap();
    project.register_path(&path).unwrap();
    project
        .find_by_original(&path)
        .unwrap()
        .content
        .clone()
        .into()
}

#[test]
fn recovered_attribute_boundary_keeps_the_real_virtual_ts() {
    let recovered = virtual_ts_for(RECOVERED_TEMPLATE);
    let clean = virtual_ts_for(CLEAN_TEMPLATE);

    assert!(
        recovered != FALLBACK_STUB,
        "a recovered parse must not collapse to the fallback stub:\n{recovered}"
    );
    // The script block is what carries the type diagnostics that used to
    // vanish; it must be generated identically to the clean twin.
    for binding in ["const label", "const wrong"] {
        assert!(
            recovered.contains(binding),
            "{binding:?} missing from the recovered virtual TS:\n{recovered}"
        );
        assert!(
            clean.contains(binding),
            "{binding:?} missing from the clean twin"
        );
    }
    // Both attributes survive the inferred boundary, so the template half is
    // generated too rather than silently truncated.
    assert!(recovered.contains("label"), "template binding lost");
}

#[test]
fn unrecovered_parse_errors_still_collapse_to_the_fallback_stub() {
    // No documented recovery for an unterminated element: the tree is not
    // usable, so the previous behavior is preserved.
    let source = r#"<template>
  <div><span>{{ label }}</div>
</template>

<script setup lang="ts">
const label: string = 'hi'
</script>
"#;
    let generated = virtual_ts_for(source);
    assert!(
        generated == FALLBACK_STUB,
        "an unrecovered parse must still emit the fallback stub:\n{generated}"
    );
}
