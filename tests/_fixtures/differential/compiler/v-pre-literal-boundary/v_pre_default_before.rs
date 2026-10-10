//! Additive source-bound old/default observer; no old provider changes.
use vize_atelier_core::{
    CodegenOptions,
    options::{CustomElementMatcher, TemplateSyntaxMode},
};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileExperimentalOptions, SfcCompileOptions, SfcParseOptions,
    SfcScriptOutputMode, StyleCompileOptions, TemplateCompileOptions,
    compile_sfc_for_adapter_with_experimental_options, parse_sfc,
};
use vize_l0::{Allocator, dump::Dump};

const SLOTS: &[(&str, &str)] = &[
    ("own-empty", "<template><slot v-pre></slot></template>"),
    (
        "inherited-empty",
        "<template><div v-pre><slot></slot></div></template>",
    ),
    ("normal-outlet", "<template><slot></slot></template>"),
    (
        "original-nonempty",
        "<template><slot v-pre>{{ not }} an interpolation</slot></template>",
    ),
];
const LOWER: &[(&str, &str)] = &[
    (
        "original-own-after-modifier",
        "<div v-pre v-my:a.m=\"v\"><span></span></div>",
    ),
    (
        "neighbor-own-before-modifier",
        "<div v-my:a.m=\"v\" v-pre><span></span></div>",
    ),
    (
        "original-inherited-longhand",
        "<div v-pre><span v-bind:x=\"1\"></span></div>",
    ),
    (
        "neighbor-own-before-longhand",
        "<div v-bind:x=\"1\" v-pre><span></span></div>",
    ),
    (
        "neighbor-own-after-longhand",
        "<div v-pre v-bind:x=\"1\"><span></span></div>",
    ),
    (
        "original-component-gap",
        "<MyComp v-pre :x=\"1\">c</MyComp>",
    ),
    (
        "original-child-component",
        "<div><div v-pre><MyComp :x=\"1\"></MyComp></div></div>",
    ),
    (
        "original-slot-marker",
        "<slot v-pre><textarea>{{x}}</textarea></slot>",
    ),
    ("original-facade-component", "<ExampleComponent v-pre/>"),
];

fn public_packet(name: &str, source: &str, kind: &str) -> serde_json::Value {
    let parse = SfcParseOptions::default();
    let descriptor = parse_sfc(source, parse.clone());
    let options = SfcCompileOptions {
        parse,
        script: ScriptCompileOptions::default(),
        template: TemplateCompileOptions::default(),
        style: StyleCompileOptions::default(),
        vapor: kind == "vapor",
        scope_id: None,
    };
    let codegen = CodegenOptions {
        source_map: true,
        prefix_identifiers: true,
        ..Default::default()
    };
    let experimental = SfcCompileExperimentalOptions::default();
    let options_debug = format!("{options:?}");
    let codegen_debug = format!("{codegen:?}");
    let experimental_debug = format!("{experimental:?}");
    let descriptor_debug = format!("{descriptor:?}");
    let (result, error) = match descriptor {
        Ok(descriptor) => match compile_sfc_for_adapter_with_experimental_options(
            &descriptor,
            options,
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            codegen,
            SfcScriptOutputMode::SeparateTemplate,
            experimental,
        ) {
            Ok(result) => (Some(result), None),
            Err(error) => (None, Some(format!("{error:?}"))),
        },
        Err(error) => (None, Some(format!("{error:?}"))),
    };
    serde_json::json!({"name":name,"source":source,"kind":kind,"route":"original-ca-default",
        "descriptorDebug":descriptor_debug,"optionsDebug":options_debug,"codegenDebug":codegen_debug,
        "experimentalDebug":experimental_debug,"customElements":"default","templateSyntax":"Standard",
        "scriptOutputMode":"SeparateTemplate","result":result,"errorDebug":error})
}

#[test]
fn actual_ca_default_complete_public_and_lowering_observations() {
    let mut public = Vec::new();
    for &(name, source) in SLOTS {
        for kind in ["dom", "vapor"] {
            public.push(public_packet(name, source, kind));
        }
    }
    let mut lower = Vec::new();
    for &(name, source) in LOWER {
        let arena = Allocator::new();
        let (root, parser_errors) = vize_atelier_core::parser::Parser::new(&arena, source).parse();
        let (tree, surface_errors) = vize_l1::parse(&arena, source);
        let lowered = vize_l1_to_l2::lower(&arena, &tree, &surface_errors);
        lower.push(serde_json::json!({"name":name,"source":source,"parserOptions":"default",
            "rootDebug":format!("{root:#?}"),"parserErrorsDebug":format!("{parser_errors:#?}"),
            "surfaceTreeDebug":format!("{tree:#?}"),"surfaceErrorsDebug":format!("{surface_errors:#?}"),
            "loweredDebug":format!("{lowered:#?}"),
            "page":vize_l2::dump::Page::of(&lowered.root.ops).print_to_string(vize_l0::dump::Mode::Full)}));
    }
    println!(
        "ACTUAL_CA_DEFAULT_PACKET={}",
        serde_json::json!({"public":public,"lower":lower})
    );
    assert_eq!(public.len(), 8);
    assert_eq!(lower.len(), 9);
    assert!(
        public.iter().all(|packet| packet["errorDebug"].is_null()),
        "complete failures retained before assertions"
    );
}
