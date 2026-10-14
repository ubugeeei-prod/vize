//! Whole authored generic contracts distinguish alias defaults from inference.
#![expect(clippy::unwrap_used, reason = "complete authored contract controls")]

use crate::batch::{ImportRewriter, generate_vue_document_virtual_ts};
use crate::virtual_ts::VirtualTsOptions;
use std::path::Path;
use vize_carton::{String, cstr};

#[cfg(unix)]
mod native;

#[cfg(unix)]
const PIN: &str = "c23e0ed8a38c66cd4f9350e7a46c6b502a8491c2";
const ROOT_PATH: &str = "packages/core/src/PinInput/PinInputRoot.vue";
#[cfg(unix)]
const STORY_PATH: &str = "packages/core/src/PinInput/story/PinInput.story.vue";
#[cfg(unix)]
const TYPE_PATH: &str = "packages/core/src/shared/types.ts";
#[cfg(unix)]
const CONFIG_PATH: &str = "packages/core/tsconfig.app.json";
const GENERIC: &str = "Type extends PinInputType";
const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-call-inference"
);

fn source(name: &str) -> String {
    std::fs::read_to_string(Path::new(CORPUS).join(name))
        .unwrap()
        .into()
}

fn root_source(generic: &str) -> String {
    source("PinInputRoot.vue.txt")
        .replace(
            cstr!("generic=\"{GENERIC}\"").as_str(),
            cstr!("generic=\"{generic}\"").as_str(),
        )
        .into()
}

#[test]
fn whole_authored_generic_call_contract_keeps_inference_and_explicit_defaults() {
    for generic in [
        GENERIC.into(),
        cstr!("{GENERIC} = any"),
        cstr!("{GENERIC} = 'text'"),
        cstr!("{GENERIC} = 'number'"),
    ] {
        let source = root_source(&generic);
        let output = generate_vue_document_virtual_ts(
            Path::new(ROOT_PATH),
            &source,
            &VirtualTsOptions::default(),
            &ImportRewriter::new(),
            false,
        )
        .unwrap();
        for field in ["__vizeCheck: ", "__vizeResolveProps?: "] {
            assert!(
                output.code.contains(cstr!("{field}<{generic}>").as_str()),
                "whole source {source}\nwhole projection {}\nwhole mappings {:#?}",
                output.code,
                output.mapping
            );
        }
        if generic == GENERIC {
            assert!(
                output
                    .code
                    .contains("type Props<Type extends PinInputType = any>")
            );
            assert!(
                !output
                    .code
                    .contains("__vizeCheck: <Type extends PinInputType = any>")
            );
        }
    }
}

#[test]
fn ordinary_and_legacy_component_generation_keeps_the_existing_contract() {
    let source = "<script setup lang=\"ts\">defineProps<{ modelValue?: string[] }>()</script>";
    let ordinary = generate_vue_document_virtual_ts(
        Path::new(ROOT_PATH),
        source,
        &VirtualTsOptions::default(),
        &ImportRewriter::new(),
        false,
    )
    .unwrap();
    assert!(!ordinary.code.contains("__vizeCheck: <Type"));
    let legacy = crate::batch::virtual_project::generate_vue_document_virtual_ts_with_options(
        Path::new(ROOT_PATH),
        &root_source(GENERIC),
        &VirtualTsOptions::default(),
        &ImportRewriter::new(),
        false,
        crate::batch::virtual_project::VueDocumentVirtualTsOptions {
            legacy_vue2: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        legacy
            .code
            .contains("new <Type extends PinInputType = any>")
    );
    assert!(
        legacy
            .code
            .contains("__vizeCheck: <Type extends PinInputType = any>")
    );
}
