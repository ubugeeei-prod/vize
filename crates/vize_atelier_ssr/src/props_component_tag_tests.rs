//! A `<script setup>` prop used as a component tag reads the props object, as
//! Vue's `resolveSetupReference` does: `_unref(__props["name"])` inline and
//! `_unref($props["name"])` otherwise. Setup bindings win over a prop
//! spelling, and a renamed props destructure (`{ icon: Icon }`) is not a setup
//! reference, so its tag resolves as an asset. Expected component arguments
//! are `@vue/compiler-sfc` 3.6.0-rc.17 `compileScript` bytes with `ssr: true`.

use vize_atelier_core::options::{
    BindingMetadata, BindingType, CustomElementMatcher, TemplateSyntaxMode,
};
use vize_l0::{Allocator, String};

use crate::compile::{SsrLane, compile_ssr_on_lane};
use crate::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

fn options(inline: bool, entries: &[(&str, BindingType)]) -> SsrCompilerOptions {
    let mut metadata = BindingMetadata {
        is_script_setup: true,
        ..Default::default()
    };
    for (name, kind) in entries {
        metadata.bindings.insert((*name).into(), *kind);
    }
    SsrCompilerOptions {
        inline,
        binding_metadata: Some(metadata),
        ..Default::default()
    }
}

/// The component push and the preamble both lanes emit, which must agree.
fn compile(source: &str, options: &SsrCompilerOptions) -> (String, String) {
    let compile = |lane| {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr_on_lane(
            &allocator,
            source,
            options.clone(),
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            SsrCompilerExperimentalOptions::default(),
            true,
            lane,
        );
        assert_eq!(errors.len(), 0, "{source}: {errors:?}");
        (result.code, result.preamble)
    };
    let selected = compile(SsrLane::Selected);
    assert_eq!(selected, compile(SsrLane::LegacyOnly), "{source}");
    let push = selected
        .0
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("_push(_ssrRenderComponent("))
        .unwrap_or("<no component push>")
        .into();
    (push, selected.1)
}

#[test]
fn a_prop_tag_reads_the_props_object_through_unref() {
    let props = [("myIcon", BindingType::Props)];
    for (source, inline, push) in [
        (
            "<my-icon />",
            true,
            r#"_push(_ssrRenderComponent(_unref(__props["myIcon"]), _attrs, null, _parent))"#,
        ),
        (
            "<my-icon />",
            false,
            r#"_push(_ssrRenderComponent(_unref($props["myIcon"]), _attrs, null, _parent))"#,
        ),
        (
            "<myIcon.Item />",
            false,
            r#"_push(_ssrRenderComponent(_unref($props["myIcon"]).Item, _attrs, null, _parent))"#,
        ),
    ] {
        let (code, preamble) = compile(source, &options(inline, &props));
        assert_eq!(code, push, "inline={inline}: {source}");
        assert_eq!(
            preamble,
            "import { ssrRenderComponent as _ssrRenderComponent } from \"@vue/server-renderer\"\n\
             import { unref as _unref } from \"vue\"\n",
            "inline={inline}: {source}"
        );
    }
}

#[test]
fn a_setup_binding_still_wins_over_a_prop_spelling() {
    let entries = [
        ("myIcon", BindingType::Props),
        ("MyIcon", BindingType::SetupConst),
    ];
    let (code, _) = compile("<my-icon />", &options(false, &entries));
    assert_eq!(
        code,
        "_push(_ssrRenderComponent($setup.MyIcon, _attrs, null, _parent))"
    );
}

#[test]
fn a_renamed_props_destructure_tag_resolves_as_an_asset() {
    let entries = [
        ("icon", BindingType::Props),
        ("Icon", BindingType::PropsAliased),
    ];
    for inline in [true, false] {
        let (code, preamble) = compile("<Icon />", &options(inline, &entries));
        assert_eq!(
            code, r#"_push(_ssrRenderComponent(_resolveComponent("Icon"), _attrs, null, _parent))"#,
            "inline={inline}"
        );
        assert_eq!(
            preamble,
            "import { ssrRenderComponent as _ssrRenderComponent } from \"@vue/server-renderer\"\n\
             import { resolveComponent as _resolveComponent } from \"vue\"\n",
            "inline={inline}"
        );
    }
}
