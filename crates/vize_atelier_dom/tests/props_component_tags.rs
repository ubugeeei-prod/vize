//! A `<script setup>` prop used as a component tag reads the props object.
//!
//! Vue's `resolveSetupReference` resolves a tag to a setup binding first and to
//! a prop last, reading the prop as `_unref(__props["name"])` in an inlined
//! render function and `_unref($props["name"])` otherwise. A renamed props
//! destructure (`const { icon: Icon } = defineProps()`) is `props-aliased`,
//! which is not a setup reference, so its tag resolves as an asset. Both lanes
//! emitted the bare binding name instead, which is not in scope at render time.
//! Expected tags are `@vue/compiler-sfc` 3.6.0-rc.17 `compileScript` bytes.

use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode};
use vize_atelier_dom::{
    DomCompilerOptions, compile_template_legacy_with_options, compile_template_with_options,
};
use vize_l0::{Allocator, FxHashMap};

fn options(inline: bool, entries: &[(&str, BindingType)]) -> DomCompilerOptions {
    let mut bindings = FxHashMap::default();
    for (name, kind) in entries {
        bindings.insert((*name).into(), *kind);
    }
    DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        inline,
        binding_metadata: Some(BindingMetadata {
            bindings,
            props_aliases: FxHashMap::default(),
            is_script_setup: true,
        }),
        ..DomCompilerOptions::default()
    }
}

fn render_line(code: &str) -> &str {
    code.lines()
        .map(str::trim)
        .find(|line| line.starts_with("return (_openBlock()"))
        .unwrap_or("<no render line>")
}

fn assert_both_lanes(source: &str, options: &DomCompilerOptions, render: &str, preamble: &str) {
    let allocator = Allocator::new();
    let (_, errors, selected) = compile_template_with_options(&allocator, source, options.clone());
    assert!(errors.is_empty(), "{source}: {errors:?}");
    let allocator = Allocator::new();
    let (_, errors, legacy) =
        compile_template_legacy_with_options(&allocator, source, options.clone());
    assert!(errors.is_empty(), "{source}: {errors:?}");
    for (lane, result) in [("selected", &selected), ("legacy", &legacy)] {
        assert_eq!(
            render_line(&result.code),
            render,
            "{lane} inline={}: {source}",
            options.inline
        );
        assert!(
            result.preamble.contains(preamble),
            "{lane} inline={}: {source}: {}",
            options.inline,
            result.preamble
        );
    }
    assert_eq!(selected.code, legacy.code, "{source}");
    assert_eq!(selected.preamble, legacy.preamble, "{source}");
}

#[test]
fn a_prop_tag_reads_the_props_object_through_unref() {
    let props = [("myIcon", BindingType::Props)];
    for (source, inline, render) in [
        (
            "<my-icon />",
            true,
            r#"return (_openBlock(), _createBlock(_unref(__props["myIcon"])))"#,
        ),
        (
            "<my-icon />",
            false,
            r#"return (_openBlock(), _createBlock(_unref($props["myIcon"])))"#,
        ),
        (
            "<myIcon.Item />",
            false,
            r#"return (_openBlock(), _createBlock(_unref($props["myIcon"]).Item))"#,
        ),
    ] {
        assert_both_lanes(source, &options(inline, &props), render, "unref as _unref");
    }
}

#[test]
fn a_setup_binding_still_wins_over_a_prop_spelling() {
    // `<my-icon>` names the prop `myIcon` and the import `MyIcon`; Vue tries
    // every spelling for a setup binding before it falls back to a prop.
    let entries = [
        ("myIcon", BindingType::Props),
        ("MyIcon", BindingType::SetupConst),
    ];
    assert_both_lanes(
        "<my-icon />",
        &options(true, &entries),
        "return (_openBlock(), _createBlock(MyIcon))",
        "createBlock as _createBlock",
    );
}

#[test]
fn a_renamed_props_destructure_tag_resolves_as_an_asset() {
    let entries = [
        ("icon", BindingType::Props),
        ("Icon", BindingType::PropsAliased),
    ];
    for inline in [true, false] {
        assert_both_lanes(
            "<Icon />",
            &options(inline, &entries),
            "return (_openBlock(), _createBlock(_component_Icon))",
            "resolveComponent as _resolveComponent",
        );
    }
}
