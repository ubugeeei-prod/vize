//! A dotted `v-slot` argument names the slot with every segment, as Vue's
//! parser spells it: `<template #item.alpha>` is the slot `item.alpha`, not
//! `item`. The L4 plan refuses slots with modifiers, so the legacy walker owns
//! these on both the static slot object and the `createSlots` route.

#![cfg(feature = "legacy-differential")]
#![expect(
    clippy::disallowed_macros,
    reason = "fixture paths are built with format!"
)]

use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};
use vize_atelier_ssr::differential::compare_ssr_lanes;
use vize_atelier_ssr::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

/// `(fixture, template)`; each fixture holds the whole `@vue/compiler-ssr`
/// 3.5-equivalent module body the legacy lane emits.
const CASES: &[(&str, &str)] = &[
    (
        "static-slots",
        r#"<Comp><template #item.alpha="{ x }"><b>{{ x }}</b></template><template #a.b.c>abc</template><template #plain>p</template></Comp>"#,
    ),
    (
        "create-slots",
        r#"<Comp><template v-if="ok" #item.alpha>ifslot</template><template v-for="n in names" #[n]>{{ n }}</template><template #a.b.c>abc</template></Comp>"#,
    ),
    (
        "component-slot",
        r#"<Comp v-slot:row.top="{ r }">{{ r }}</Comp>"#,
    ),
    (
        "vnode-fallback",
        r#"<Outer><Comp><template #item.alpha>x</template><template v-if="ok" #a.b>y</template></Comp></Outer>"#,
    ),
];

fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/dotted-slot-names")
        .join(format!("{name}.ssr.txt"))
}

#[test]
fn dotted_slot_names_keep_every_segment_on_both_lanes() {
    let bless = std::env::var_os("VIZE_BLESS_DOTTED_SLOT_NAMES").is_some();
    for (name, source) in CASES {
        let pair = compare_ssr_lanes(
            source,
            &SsrCompilerOptions::default(),
            &SsrCompilerExperimentalOptions::default(),
        );
        assert_eq!(pair.lane, "legacy.operation", "{name}");
        assert_eq!(pair.selected.errors.len(), 0, "{name}");
        assert_eq!(pair.legacy.errors.len(), 0, "{name}");
        assert_eq!(pair.selected.result.code, pair.legacy.result.code, "{name}");
        let path = fixture_path(name);
        if bless {
            std::fs::write(&path, pair.selected.result.code.as_str()).expect("bless fixture");
            continue;
        }
        let expected = std::fs::read_to_string(&path).expect("pinned fixture");
        assert_eq!(pair.selected.result.code.as_str(), expected, "{name}");
    }
}

/// The pinned `compiler/sfc/ssr-dotted-slot-names` differential case: the
/// whole SSR module the CLI emits for the authored SFC.
#[test]
fn dotted_slot_names_through_the_public_sfc_api() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/compiler/ssr-dotted-slot-names/Layout.vue.txt"
    );
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("SFC");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse: SfcParseOptions {
                filename: "Layout.vue".into(),
                ..Default::default()
            },
            template: TemplateCompileOptions {
                ssr: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("compile");
    assert_eq!(result.errors.len(), 0, "{:?}", result.errors);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../tests/_fixtures/differential/compiler/ssr-dotted-slot-names/module.expected.txt",
    );
    if std::env::var_os("VIZE_BLESS_DOTTED_SLOT_NAMES").is_some() {
        std::fs::write(&path, result.code.as_str()).expect("bless module");
        return;
    }
    let expected = std::fs::read_to_string(&path).expect("pinned module");
    assert_eq!(result.code.as_str(), expected);
}
