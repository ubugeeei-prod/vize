use super::{
    analyze::analyze_sfc_json, compiler::pipeline::compile_internal,
    options::resolve_template_syntax_compat, utf8_byte_to_utf16_offset,
};
use crate::CompilerOptions;
use vize_atelier_core::CodegenOptions;
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, compile_sfc_with_template_syntax_and_codegen_options,
    parse_sfc,
};

#[test]
fn test_utf8_byte_to_utf16_offset_handles_multibyte_and_astral_chars() {
    let source = "aあ😀b";

    let hiragana_start = source.find('あ').expect("hiragana should exist") as u32;
    let emoji_start = source.find('😀').expect("emoji should exist") as u32;
    let latin_b_start = source.find('b').expect("latin b should exist") as u32;

    assert_eq!(utf8_byte_to_utf16_offset(source, 0), 0);
    assert_eq!(utf8_byte_to_utf16_offset(source, hiragana_start), 1);
    assert_eq!(utf8_byte_to_utf16_offset(source, emoji_start), 2);
    assert_eq!(utf8_byte_to_utf16_offset(source, latin_b_start), 4);
    assert_eq!(utf8_byte_to_utf16_offset(source, source.len() as u32), 5);
}

#[test]
fn legacy_vue_parser_quirks_only_falls_back_without_an_explicit_mode() {
    assert_eq!(
        resolve_template_syntax_compat(None, Some(true)).as_deref(),
        Some("quirks")
    );
    assert_eq!(resolve_template_syntax_compat(None, Some(false)), None);
    assert_eq!(
        resolve_template_syntax_compat(Some("strict".to_string()), Some(true)).as_deref(),
        Some("strict")
    );
}

#[test]
fn compiler_options_change_runtime_names_syntax_and_source_maps() {
    let module = compile_internal(
        "<div>{{ message }}</div>",
        &CompilerOptions {
            mode: Some("module".to_string()),
            runtime_module_name: Some("@acme/vue-runtime".to_string()),
            ..Default::default()
        },
        false,
        None,
    )
    .expect("custom runtime module should compile");
    assert!(module.preamble.contains("@acme/vue-runtime"));

    let function = compile_internal(
        "<div />",
        &CompilerOptions {
            runtime_global_name: Some("AcmeVue".to_string()),
            ..Default::default()
        },
        false,
        None,
    )
    .expect("custom runtime global should compile");
    assert!(function.preamble.contains("AcmeVue") || function.code.contains("AcmeVue"));

    let strict = CompilerOptions {
        template_syntax: Some("strict".to_string()),
        ..Default::default()
    };
    assert!(compile_internal("<div /><span></span>", &strict, false, None).is_err());

    let with_map = compile_internal(
        "<div>{{ message }}</div>",
        &CompilerOptions {
            source_map: Some(true),
            filename: Some("src/Component.vue".to_string()),
            ..Default::default()
        },
        false,
        None,
    )
    .expect("source-map compile should succeed");
    assert_eq!(
        with_map
            .map
            .as_ref()
            .and_then(|map| map["version"].as_u64()),
        Some(3)
    );
    assert_eq!(
        with_map
            .map
            .as_ref()
            .and_then(|map| map["sources"][0].as_str()),
        Some("src/Component.vue")
    );

    let ssr_with_map = compile_internal(
        "<div>{{ message }}</div>",
        &CompilerOptions {
            ssr: Some(true),
            source_map: Some(true),
            filename: Some("src/Ssr.vue".to_string()),
            ..Default::default()
        },
        false,
        None,
    )
    .expect("SSR source-map compile should succeed");
    assert_eq!(
        ssr_with_map
            .map
            .as_ref()
            .and_then(|map| map["sources"][0].as_str()),
        Some("src/Ssr.vue")
    );

    let vapor_with_map = compile_internal(
        "<div>{{ message }}</div>",
        &CompilerOptions {
            source_map: Some(true),
            filename: Some("src/Vapor.vue".to_string()),
            ..Default::default()
        },
        true,
        None,
    )
    .expect("Vapor source-map compile should succeed");
    assert_eq!(
        vapor_with_map
            .map
            .as_ref()
            .and_then(|map| map["sources"][0].as_str()),
        Some("src/Vapor.vue")
    );
}

#[test]
fn custom_element_patterns_compile_matched_tags_as_elements() {
    let matched = compile_internal(
        "<TresMesh><TresSpotLight /></TresMesh>",
        &CompilerOptions {
            custom_elements: Some(vec!["Tres*".to_string()]),
            ..Default::default()
        },
        false,
        None,
    )
    .expect("custom-element patterns should compile");
    assert!(
        matched.code.contains("\"TresMesh\"") && matched.code.contains("\"TresSpotLight\""),
        "matched tags should be emitted as element tags:\n{}",
        matched.code
    );
    assert!(
        !matched.code.contains("_resolveComponent"),
        "matched tags must not use runtime component resolution:\n{}",
        matched.code
    );

    let unmatched = compile_internal("<TresMesh />", &CompilerOptions::default(), false, None)
        .expect("control compile should succeed");
    assert!(
        unmatched.code.contains("_resolveComponent(\"TresMesh\")"),
        "without patterns the tag should stay a component:\n{}",
        unmatched.code
    );
}

#[test]
fn sfc_compiler_options_change_module_and_standalone_runtimes() {
    let descriptor = parse_sfc("<template><div></div></template>", Default::default())
        .expect("fixture should parse");
    let codegen_options = CodegenOptions {
        runtime_module_name: "@acme/vue-runtime".into(),
        runtime_global_name: "AcmeVue".into(),
        ..Default::default()
    };

    for vapor in [false, true] {
        let module = compile_sfc_with_template_syntax_and_codegen_options(
            &descriptor,
            SfcCompileOptions {
                vapor,
                ..Default::default()
            },
            vize_atelier_core::TemplateSyntaxMode::Standard,
            codegen_options.clone(),
        )
        .expect("module SFC should compile");
        assert!(
            module.code.contains("@acme/vue-runtime"),
            "custom module name missing from vapor={vapor}:\n{}",
            module.code
        );

        let standalone = compile_sfc_with_template_syntax_and_codegen_options(
            &descriptor,
            SfcCompileOptions {
                script: ScriptCompileOptions {
                    inline_template: true,
                    ..Default::default()
                },
                vapor,
                ..Default::default()
            },
            vize_atelier_core::TemplateSyntaxMode::Standard,
            codegen_options.clone(),
        )
        .expect("standalone SFC should compile");
        assert!(
            standalone.code.contains("AcmeVue"),
            "custom global name missing from vapor={vapor}:\n{}",
            standalone.code
        );
        assert!(!standalone.code.contains("@acme/vue-runtime"));
    }
}

const ALIAS_SOURCE: &str =
    "<script setup>\nconst msg = 1\n</script>\n\n<template>\n  <div>{{ msg }}</div>\n</template>\n";

/// The croquis folio text for [`ALIAS_SOURCE`], the exact bytes both alias
/// keys must carry.
const CROQUIS_FOLIO: &str = r"[vir]
script_setup=true
scopes=6
bindings=1

[bindings]
lit:msg

[scopes]
~0 univ @0:0 [NaN,Array,WeakMap,decodeURIComponent,Date,Number,JSON,encodeURI,Float64Array,TypeError,RangeError,Reflect,WeakSet,Iterator,arguments,AggregateError,Boolean,Uint32Array,Uint16Array,Infinity,Int8Array,Set,isFinite,decodeURI,Function,Symbol,BigUint64Array,AsyncFunction,AsyncIterator,Object,Float32Array,RegExp,AsyncGenerator,Error,EvalError,BigInt,ArrayBuffer,BigInt64Array,this,encodeURIComponent,Uint8Array,Uint8ClampedArray,Int32Array,eval,SyntaxError,ReferenceError,Math,console,Proxy,Generator,DataView,globalThis,URIError,String,Atomics,undefined,GeneratorFunction,Intl,parseFloat,AsyncGeneratorFunction,Promise,Int16Array,isNaN,parseInt,SharedArrayBuffer,Map]
!0 client @0:0 [PerformanceObserver,print,Element,cancelIdleCallback,getSelection,localStorage,queueMicrotask,KeyboardEvent,DocumentFragment,CanvasRenderingContext2D,FocusEvent,matchMedia,MediaQueryList,MouseEvent,InputEvent,clearInterval,setInterval,setTimeout,alert,requestAnimationFrame,WebGL2RenderingContext,history,requestIdleCallback,ShadowRoot,location,Node,prompt,Image,document,MutationObserver,sessionStorage,HTMLElement,WebSocket,window,WebGLRenderingContext,ResizeObserver,confirm,screen,indexedDB,customElements,self,clearTimeout,TouchEvent,XMLHttpRequest,PointerEvent,NodeList,close,Document,navigator,open,IntersectionObserver,cancelAnimationFrame,Audio,getComputedStyle] < ~0
#0 server @0:0 [setImmediate,clearImmediate,Buffer,process] < ~0
~1 vue @0:0 [$data,$emit,$forceUpdate,$parent,$refs,$root,$watch,$options,$slots,$attrs,$props,$nextTick,$el] < ~0
~2 mod @0:15 < ~0
~3 setup @0:15 < ~2

";

/// The alias contract (Davinci P0-10): `analyzeSfc` carries the croquis folio
/// text under both the deprecated `vir` key and `folio.croquis`,
/// byte-identically.
#[test]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
fn the_croquis_alias_keys_stay_byte_identical() {
    let result = analyze_sfc_json(ALIAS_SOURCE, "src/App.vue").expect("analysis succeeds");

    // The alias LAW is the two keys' byte-identity to each other. The
    // deterministic sections ([vir] header and [bindings]) are pinned
    // exactly; the [scopes] global lists iterate a hash set whose order
    // is environment-sensitive (CI proved it), so the absolute literal
    // pins only the prefix and the per-environment stability is carried
    // by the equality law plus a same-process double-run pin.
    let vir = result["vir"].as_str().expect("vir is a string");
    let expected_prefix = &CROQUIS_FOLIO[..CROQUIS_FOLIO.find("[scopes]").expect("prefix marker")];
    assert_eq!(&vir[..expected_prefix.len()], expected_prefix);
    let again = analyze_sfc_json(ALIAS_SOURCE, "src/App.vue").expect("analysis succeeds");
    assert_eq!(result["vir"], again["vir"]);
    assert_eq!(result["vir"], result["folio"]["croquis"]);
    // The alias object itself gained no siblings.
    let folio = result["folio"].as_object().expect("folio is an object");
    assert_eq!(folio.len(), 1);
    assert_eq!(result["folio"]["croquis"], result["vir"]);
}

/// `analyzeSfc` is the semantic analysis only: stage pages come from
/// `compileSfc` `captureStages`, the run that produced the module.
#[test]
fn the_analyze_result_runs_no_stage_ladder() {
    let result = analyze_sfc_json(ALIAS_SOURCE, "src/App.vue").expect("analysis succeeds");
    let mut members: Vec<&str> = result
        .as_object()
        .expect("result is an object")
        .keys()
        .map(String::as_str)
        .collect();
    members.sort_unstable();
    assert_eq!(members, ["croquis", "diagnostics", "folio", "vir"]);
}
