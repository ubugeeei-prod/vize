#![expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]
use super::{LegacyReason, SsrL4Request, SsrL4Selection, select_ssr_lane};
use crate::compile::{SsrLane, compile_ssr_on_lane};
use crate::options::{SsrCompilerExperimentalOptions, SsrCompilerOptions};
use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_core::lane::ScriptTables;
use vize_atelier_core::options::{BindingMetadata, BindingType, CustomElementMatcher};
use vize_l0::Allocator;

fn select(source: &str, options: &SsrCompilerOptions) -> SsrL4Selection {
    let allocator = Allocator::new();
    let experimental = SsrCompilerExperimentalOptions::default();
    let (root, errors, frozen) = vize_atelier_core::parser::Parser::with_options(
        &allocator,
        source,
        crate::stage_options::parser_options(options),
    )
    .parse_with_frozen_elements();
    assert!(
        !errors.iter().any(|error| !error.is_recoverable()),
        "{errors:?}"
    );
    select_ssr_lane(
        &allocator,
        source,
        &SsrL4Request {
            options,
            experimental: &experimental,
            runtime_module_name: "vue",
            template_syntax: TemplateSyntaxMode::Standard,
            has_custom_elements: false,
            context: crate::SsrSfcContext::new(true),
            frozen: Some((&root, &frozen)),
        },
    )
}

#[test]
fn admitted_templates_are_emitted_from_the_string_plan() {
    let selection = select(
        r#"<section><h1>{{ title }}</h1><p>Ready</p></section>"#,
        &SsrCompilerOptions::default(),
    );
    let SsrL4Selection::Emitted(result) = selection else {
        panic!("expected plan emission, got {selection:?}");
    };
    assert_eq!(
        result.code.as_str(),
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(`<section${_ssrRenderAttrs(_attrs)}><h1>${_ssrInterpolate(_ctx.title)}</h1><p>Ready</p></section>`)\n}\n"
    );
    assert_eq!(
        result.preamble.as_str(),
        "import { ssrInterpolate as _ssrInterpolate, ssrRenderAttrs as _ssrRenderAttrs } from \"@vue/server-renderer\"\n"
    );
}

#[test]
fn unsupported_bridge_options_select_legacy_before_lowering() {
    let selection = select(
        r#"<div><!-- kept --></div>"#,
        &SsrCompilerOptions {
            comments: true,
            ..SsrCompilerOptions::default()
        },
    );
    assert!(matches!(
        selection,
        SsrL4Selection::Legacy(LegacyReason::Options)
    ));
}

fn metadata_with(names: &[(&str, BindingType)]) -> BindingMetadata {
    let mut metadata = BindingMetadata::default();
    for (name, kind) in names {
        metadata.bindings.insert((*name).into(), *kind);
    }
    metadata.is_script_setup = true;
    metadata
}

/// The tables of `names`: every `SetupConst` registers a component.
fn tables_with(names: &[(&str, BindingType)]) -> ScriptTables {
    let components = names
        .iter()
        .filter(|(_, kind)| *kind == BindingType::SetupConst)
        .map(|(name, _)| *name);
    ScriptTables::new([], components)
}

#[test]
fn script_tables_without_metadata_name_their_own_legacy_reason() {
    let selection = select(
        r#"<div>{{ msg }}</div>"#,
        &SsrCompilerOptions {
            script_tables: Some(ScriptTables::default()),
            ..SsrCompilerOptions::default()
        },
    );
    assert!(
        matches!(selection, SsrL4Selection::Legacy(LegacyReason::Croquis)),
        "got {selection:?}"
    );
}

#[test]
fn a_registration_the_metadata_lacks_stays_legacy() {
    let drawn = ScriptTables::new([], ["child"]);
    let selection = select(
        r#"<Child />"#,
        &SsrCompilerOptions {
            binding_metadata: Some(metadata_with(&[("Child", BindingType::SetupConst)])),
            script_tables: Some(drawn),
            ..SsrCompilerOptions::default()
        },
    );
    assert!(
        matches!(selection, SsrL4Selection::Legacy(LegacyReason::Croquis)),
        "got {selection:?}"
    );

    let hidden = tables_with(&[
        ("Child", BindingType::SetupConst),
        ("Hidden", BindingType::SetupConst),
    ]);
    let selection = select(
        r#"<Child />"#,
        &SsrCompilerOptions {
            binding_metadata: Some(metadata_with(&[("Child", BindingType::SetupConst)])),
            script_tables: Some(hidden),
            ..SsrCompilerOptions::default()
        },
    );
    assert!(
        matches!(selection, SsrL4Selection::Legacy(LegacyReason::Croquis)),
        "got {selection:?}"
    );
}

#[test]
fn projectable_script_tables_match_the_walker() {
    let names = [
        ("count", BindingType::SetupRef),
        ("Child", BindingType::SetupConst),
    ];
    let source = "<Child>{{ count }}</Child>";
    let options = || SsrCompilerOptions {
        binding_metadata: Some(metadata_with(&names)),
        script_tables: Some(tables_with(&names)),
        ..SsrCompilerOptions::default()
    };
    let selected = select(source, &options());
    assert!(
        matches!(selected, SsrL4Selection::Emitted(_)),
        "got {selected:?}"
    );
    let compile = |lane| {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr_on_lane(
            &allocator,
            source,
            options(),
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            SsrCompilerExperimentalOptions::default(),
            true,
            lane,
        );
        (format!("{errors:?}"), result.code)
    };
    let (legacy_errors, legacy) = compile(SsrLane::LegacyOnly);
    let (selected_errors, selected_code) = compile(SsrLane::Selected);
    assert_eq!(selected_code, legacy, "code diverged");
    assert_eq!(selected_errors, legacy_errors, "diagnostics diverged");
}

#[test]
fn unsupported_emission_options_select_legacy_after_the_witness() {
    let selection = select(
        r#"<div>{{ msg }}</div>"#,
        &SsrCompilerOptions {
            inline: true,
            ..SsrCompilerOptions::default()
        },
    );
    assert!(matches!(
        selection,
        SsrL4Selection::Legacy(LegacyReason::Options)
    ));
}

#[test]
fn unowned_shapes_name_their_legacy_reason() {
    let cases = [
        (
            r#"<Foo><div v-if="a"><template #a>x</template></div></Foo>"#,
            LegacyReason::Operation,
        ),
        (r#"<script>x</script>"#, LegacyReason::Element),
        (
            "<p v-if=\"a\">1</p><!-- @vize:todo x --><p v-else>2</p>",
            LegacyReason::SurfaceSemantics,
        ),
        (
            r#"<div :title="a +"></div>"#,
            LegacyReason::ExpressionOrEncoding,
        ),
    ];
    for (source, expected) in cases {
        let selection = select(source, &SsrCompilerOptions::default());
        assert!(
            matches!(selection, SsrL4Selection::Legacy(reason) if reason == expected),
            "{source}: expected {expected:?}, got {selection:?}"
        );
    }
}

#[test]
fn type_annotated_v_for_aliases_stay_in_scope() {
    let source = r#"<li v-for="(row: Row, index: number) in rows">{{ index }} {{ row.id }}</li>"#;
    for lane in [SsrLane::Selected, SsrLane::LegacyOnly] {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr_on_lane(
            &allocator,
            source,
            SsrCompilerOptions {
                is_ts: true,
                ..SsrCompilerOptions::default()
            },
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            SsrCompilerExperimentalOptions::default(),
            false,
            lane,
        );
        assert!(errors.is_empty(), "{errors:?}\n{}", result.code);
        // Type annotations change neither the renderer nor its helper imports.
        assert_eq!(
            result.code.as_str(),
            concat!(
                "function ssrRender(_ctx, _push, _parent, _attrs) {\n",
                "  _push(`<!--[-->`)\n",
                "  _ssrRenderList(_ctx.rows, (row, index) => {\n",
                "    _push(`<li>${_ssrInterpolate(index)} ${_ssrInterpolate(row.id)}</li>`)\n",
                "  })\n",
                "  _push(`<!--]-->`)\n",
                "}\n",
            ),
            "{lane:?}",
        );
        assert_eq!(
            result.preamble.as_str(),
            "import { ssrInterpolate as _ssrInterpolate, ssrRenderList as _ssrRenderList } from \"@vue/server-renderer\"\n",
            "{lane:?}",
        );
    }
}
