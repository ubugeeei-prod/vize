//! The Elk NavUser nested template branch is a Fragment, not an attrs owner.
#![cfg(feature = "legacy-differential")]

use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_atelier_ssr::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

const CASES: &[(&str, &str, bool)] = &[
    (
        "nested-root",
        r#"<template v-if="outer"><button v-if="inner">a</button><button v-else>b</button></template>"#,
        true,
    ),
    (
        "nested-else",
        r#"<i v-if="outer">outer</i><template v-else><button v-if="inner">a</button><button v-else>b</button></template>"#,
        true,
    ),
    (
        "single-template-root",
        r#"<template v-if="outer"><button>b</button></template>"#,
        false,
    ),
    (
        "plain-root-chain",
        r#"<i v-if="outer">outer</i><button v-else>b</button>"#,
        false,
    ),
    (
        "loop-owns-fragment",
        r#"<template v-if="outer"><button v-for="n in 2">{{ n }}</button></template>"#,
        false,
    ),
    (
        "loop-conditional-boundary",
        r#"<template v-for="n in 2"><template v-if="outer"><button v-if="inner">a</button><button v-else>b</button></template></template>"#,
        false,
    ),
    (
        "loop-single-conditional",
        r#"<template v-for="n in 2"><button v-if="inner">a</button><button v-else>b</button></template>"#,
        false,
    ),
    (
        "loop-nested-loop",
        r#"<template v-for="n in 2"><button v-for="m in 2">{{ n }}:{{ m }}</button></template>"#,
        false,
    ),
    (
        "loop-single-element",
        r#"<button v-for="n in 2">{{ n }}</button>"#,
        false,
    ),
    (
        "loop-single-text",
        r#"<template v-for="n in 2">text</template>"#,
        false,
    ),
    (
        "loop-empty",
        r#"<template v-for="n in 2"></template>"#,
        false,
    ),
    (
        "transition-group-loop-conditional",
        r#"<TransitionGroup tag="section"><template v-for="n in 2"><template v-if="outer"><button v-if="inner">a</button><button v-else>b</button></template></template></TransitionGroup>"#,
        false,
    ),
    (
        "loop-keyed-element",
        r#"<template v-for="n in 2" :key="n"><button>{{ n }}</button></template>"#,
        false,
    ),
    (
        "loop-keyed-conditional",
        r#"<template v-for="n in 2" :key="n"><button v-if="inner">a</button><button v-else>b</button></template>"#,
        false,
    ),
    (
        "loop-keyed-component",
        r#"<template v-for="n in 2" :key="n"><RuntimeChild :n="n" /></template>"#,
        false,
    ),
    (
        "loop-keyed-slot",
        r#"<template v-for="n in 2" :key="n"><slot :n="n"><button>fallback</button></slot></template>"#,
        false,
    ),
    (
        "fallback-keyed-component",
        r#"<RuntimeSlotHost><template v-for="n in 2" :key="n"><RuntimeChild :n="n" /></template></RuntimeSlotHost>"#,
        false,
    ),
    (
        "fallback-keyed-slot",
        r#"<RuntimeSlotHost><template v-for="n in 2" :key="n"><slot :n="n"><button>fallback</button></slot></template></RuntimeSlotHost>"#,
        false,
    ),
    (
        "fallback-conditional-boundary",
        r#"<RuntimeSlotHost><template v-for="n in 2"><template v-if="outer"><button v-if="inner">a</button><button v-else>b</button></template></template></RuntimeSlotHost>"#,
        false,
    ),
    (
        "fallback-nested-loop",
        r#"<RuntimeSlotHost><template v-for="n in 2"><button v-for="m in 2">{{ n }}:{{ m }}</button></template></RuntimeSlotHost>"#,
        false,
    ),
    (
        "text-template-root",
        r#"<template v-if="outer">hi</template>"#,
        false,
    ),
    (
        "text-template-nested",
        r#"<div class="box"><template v-if="outer">hi</template></div>"#,
        false,
    ),
    (
        "interpolation-template-nested",
        r#"<div class="box"><template v-if="outer">{{ inner }}</template></div>"#,
        false,
    ),
    (
        "text-template-else",
        r#"<div class="box"><b v-if="outer">on</b><template v-else>hi</template></div>"#,
        false,
    ),
    (
        "text-template-mixed",
        r#"<div class="box"><template v-if="outer">hi<i>!</i></template></div>"#,
        false,
    ),
    (
        "single-element-template-nested",
        r#"<div class="box"><template v-if="outer"><b>hi</b></template></div>"#,
        false,
    ),
];

// #11920: execute the actual public client compiler alongside both SSR routes.
fn client_module(source: &str) -> serde_json::Value {
    let mut sfc_source = vize_l0::String::from("<template>");
    sfc_source.push_str(source);
    sfc_source.push_str("</template>");
    let descriptor =
        parse_sfc(&sfc_source, SfcParseOptions::default()).expect("parse complete client SFC");
    let result = compile_sfc(&descriptor, SfcCompileOptions::default())
        .expect("compile complete client SFC");
    assert!(result.errors.is_empty(), "{source}: {:?}", result.errors);
    assert!(
        result.warnings.is_empty(),
        "{source}: {:?}",
        result.warnings
    );
    serde_json::to_value(result).expect("retain every public client compiler field")
}

// These complete modules also run against official Vue in the scoped SSR action.
// Pin both fragment anchors and attribute ownership without partial code checks.
const NESTED_ROOT_CODE: &str = r#"function ssrRender(_ctx, _push, _parent, _attrs) {
  if (_ctx.outer) {
    _push(`<!--[-->`)
    if (_ctx.inner) {
      _push(`<button>a</button>`)
    } else {
      _push(`<button>b</button>`)
    }
    _push(`<!--]-->`)
  } else {
    _push(`<!---->`)
  }
}
"#;
const NESTED_ELSE_CODE: &str = r#"function ssrRender(_ctx, _push, _parent, _attrs) {
  if (_ctx.outer) {
    _push(`<i${_ssrRenderAttrs(_attrs)}>outer</i>`)
  } else {
    _push(`<!--[-->`)
    if (_ctx.inner) {
      _push(`<button>a</button>`)
    } else {
      _push(`<button>b</button>`)
    }
    _push(`<!--]-->`)
  }
}
"#;

#[test]
fn nested_template_conditionals_keep_fragments_and_both_emission_routes() {
    let mut captures = Vec::new();
    for vapor in [false, true] {
        for &(name, source, nested) in CASES {
            let compiler_options = SsrCompilerOptions {
                vapor,
                ..Default::default()
            };
            let experimental_options = SsrCompilerExperimentalOptions {
                source_map: true,
                source_map_filename: Some("NestedBranch.vue".into()),
                ..Default::default()
            };
            let pair = vize_atelier_ssr::differential::compare_ssr_lanes(
                source,
                &compiler_options,
                &experimental_options,
            );
            assert_eq!(pair.lane, "s4", "native plan must own {name}");
            assert!(pair.selected.errors.is_empty(), "{name}");
            assert!(pair.legacy.errors.is_empty(), "{name}");
            assert_eq!(pair.selected.result.code, pair.legacy.result.code, "{name}");
            assert_eq!(
                pair.selected.result.preamble, pair.legacy.result.preamble,
                "{name}"
            );
            assert_eq!(pair.selected.result.map, pair.legacy.result.map, "{name}");
            if nested {
                let (code, preamble) = match name {
                    "nested-root" => (NESTED_ROOT_CODE, ""),
                    "nested-else" => (
                        NESTED_ELSE_CODE,
                        "import { ssrRenderAttrs as _ssrRenderAttrs } from \"@vue/server-renderer\"\n",
                    ),
                    _ => panic!("missing complete nested branch reference: {name}"),
                };
                assert_eq!(pair.selected.result.code, code, "{name}");
                assert_eq!(pair.selected.result.preamble, preamble, "{name}");
            }
            captures.push(serde_json::json!({
                "name": name,
                "source": source,
                "vapor": vapor,
                "lane": pair.lane,
                "compilerOptions": compiler_options,
                "experimentalOptions": {
                    "componentName": experimental_options.component_name,
                    "selfComponent": experimental_options.self_component,
                    "sourceMap": experimental_options.source_map,
                    "sourceMapFilename": experimental_options.source_map_filename,
                },
                "selected": {
                    "code": pair.selected.result.code,
                    "preamble": pair.selected.result.preamble,
                    "map": pair.selected.result.map,
                },
                "legacy": {
                    "code": pair.legacy.result.code,
                    "preamble": pair.legacy.result.preamble,
                    "map": pair.legacy.result.map,
                },
                "client": if !vapor && matches!(name,
                    "text-template-root"
                    | "text-template-nested"
                    | "interpolation-template-nested"
                    | "text-template-else"
                    | "text-template-mixed"
                    | "single-element-template-nested") {
                    client_module(source)
                } else {
                    serde_json::Value::Null
                },
            }));
        }
    }
    if let Some(destination) = std::env::var_os("VIZE_NESTED_BRANCH_CAPTURE") {
        std::fs::write(
            destination,
            serde_json::to_vec_pretty(&captures).expect("serialize complete native modules"),
        )
        .expect("retain native modules for actual Vue execution");
    }
}
