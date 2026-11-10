//! `<script setup>` binding metadata pinned to Vue's `walkDeclaration`.

use std::collections::BTreeMap;

use serde::Deserialize;
use vize_atelier_core::{
    CodegenOptions, TemplateSyntaxMode,
    options::{BindingType, CustomElementMatcher},
};
use vize_atelier_dom::DomCompilerOptions;
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    TemplateCompileOptions, compile_sfc_for_adapter, parse_sfc,
};
use vize_carton::{String, ToCompactString};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReferencePack {
    fixtures: Vec<ReferenceFixture>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReferenceFixture {
    id: String,
    #[serde(default)]
    hoist_static: Option<bool>,
    source: String,
    reference_bindings: BTreeMap<String, String>,
    reference_reads: Vec<String>,
}

fn binding_name(kind: BindingType) -> &'static str {
    match kind {
        BindingType::SetupConst => "setup-const",
        BindingType::SetupLet => "setup-let",
        BindingType::SetupRef => "setup-ref",
        BindingType::SetupMaybeRef => "setup-maybe-ref",
        BindingType::SetupReactiveConst => "setup-reactive-const",
        BindingType::LiteralConst => "literal-const",
        BindingType::Props => "props",
        BindingType::PropsAliased => "props-aliased",
        _ => "other",
    }
}

#[test]
fn compiler_bindings_and_inline_reads_match_vue_3_5_35() {
    let pack: ReferencePack =
        serde_json::from_str(include_str!("fixtures/setup_binding_kinds_vue_3_5_35.json"))
            .expect("valid Vue reference pack");

    for fixture in pack.fixtures {
        let descriptor = parse_sfc(
            &fixture.source,
            SfcParseOptions {
                filename: fixture.id.clone(),
                ..Default::default()
            },
        )
        .expect("parse reference SFC");
        let result = compile_sfc_for_adapter(
            &descriptor,
            SfcCompileOptions {
                script: ScriptCompileOptions {
                    inline_template: true,
                    ..Default::default()
                },
                template: TemplateCompileOptions {
                    compiler_options: fixture.hoist_static.map(|hoist_static| DomCompilerOptions {
                        hoist_static,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            CodegenOptions::default(),
            SfcScriptOutputMode::InlineTemplate,
        )
        .expect("compile reference SFC");
        let actual = result
            .bindings
            .expect("script setup compilation returns bindings")
            .bindings
            .into_iter()
            .map(|(name, kind)| (name, binding_name(kind).to_compact_string()))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(actual, fixture.reference_bindings, "{}", fixture.id);
        for read in fixture.reference_reads {
            assert!(
                result.code.contains(read.as_str()),
                "{} must contain Vue's inline read `{read}`",
                fixture.id
            );
        }
    }
}
