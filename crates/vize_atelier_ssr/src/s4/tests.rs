use super::{LegacyReason, SsrS4Request, SsrS4Selection, select_ssr_lane};
use crate::compile::{SsrLane, compile_ssr_on_lane};
use crate::options::{SsrCompilerExperimentalOptions, SsrCompilerOptions};
use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_core::options::{BindingMetadata, BindingType, CustomElementMatcher};
use vize_s0::Allocator;

fn select(source: &str, options: &SsrCompilerOptions) -> SsrS4Selection {
    let allocator = Allocator::new();
    let experimental = SsrCompilerExperimentalOptions::default();
    select_ssr_lane(
        &allocator,
        source,
        &SsrS4Request {
            options,
            experimental: &experimental,
            template_syntax: TemplateSyntaxMode::Standard,
            has_custom_elements: false,
        },
    )
}

#[test]
fn admitted_templates_are_emitted_from_the_string_plan() {
    let selection = select(
        r#"<section><h1>{{ title }}</h1><p>Ready</p></section>"#,
        &SsrCompilerOptions::default(),
    );
    let SsrS4Selection::Emitted(result) = selection else {
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
        SsrS4Selection::Legacy(LegacyReason::Options)
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

fn croquis_with(names: &[(&str, BindingType)]) -> vize_croquis::Croquis {
    let mut summary = vize_croquis::Croquis::default();
    for (name, kind) in names {
        summary.bindings.add(*name, *kind);
    }
    summary
}

#[test]
fn a_croquis_summary_without_metadata_names_its_own_legacy_reason() {
    let selection = select(
        r#"<div>{{ msg }}</div>"#,
        &SsrCompilerOptions {
            croquis: Some(Box::default()),
            ..SsrCompilerOptions::default()
        },
    );
    assert!(
        matches!(selection, SsrS4Selection::Legacy(LegacyReason::Croquis)),
        "got {selection:?}"
    );
}

#[test]
fn a_croquis_registration_the_metadata_lacks_stays_legacy() {
    let mut summary = croquis_with(&[("Child", BindingType::SetupConst)]);
    summary.used_components.insert("Child".into());
    let selection = select(
        r#"<Child />"#,
        &SsrCompilerOptions {
            binding_metadata: Some(metadata_with(&[("Child", BindingType::SetupConst)])),
            croquis: Some(Box::new(summary)),
            ..SsrCompilerOptions::default()
        },
    );
    assert!(
        matches!(selection, SsrS4Selection::Legacy(LegacyReason::Croquis)),
        "got {selection:?}"
    );

    let hidden = croquis_with(&[
        ("Child", BindingType::SetupConst),
        ("Hidden", BindingType::SetupConst),
    ]);
    let selection = select(
        r#"<Child />"#,
        &SsrCompilerOptions {
            binding_metadata: Some(metadata_with(&[("Child", BindingType::SetupConst)])),
            croquis: Some(Box::new(hidden)),
            ..SsrCompilerOptions::default()
        },
    );
    assert!(
        matches!(selection, SsrS4Selection::Legacy(LegacyReason::Croquis)),
        "got {selection:?}"
    );
}

#[test]
fn a_projectable_croquis_summary_matches_the_walker() {
    let names = [
        ("count", BindingType::SetupRef),
        ("Child", BindingType::SetupConst),
    ];
    let source = "<Child>{{ count }}</Child>";
    let options = || SsrCompilerOptions {
        binding_metadata: Some(metadata_with(&names)),
        croquis: Some(Box::new(croquis_with(&names))),
        ..SsrCompilerOptions::default()
    };
    let selected = select(source, &options());
    assert!(
        matches!(selected, SsrS4Selection::Emitted(_)),
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
        SsrS4Selection::Legacy(LegacyReason::Options)
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
            r#"<div>{{ a &amp;&amp; b }}</div>"#,
            LegacyReason::ExpressionOrEncoding,
        ),
    ];
    for (source, expected) in cases {
        let selection = select(source, &SsrCompilerOptions::default());
        assert!(
            matches!(selection, SsrS4Selection::Legacy(reason) if reason == expected),
            "{source}: expected {expected:?}, got {selection:?}"
        );
    }
}
