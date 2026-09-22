//! The croquis alias byte-identity pin and the `analyzeSfc` Spolvero feed
//! (P2-18; the full S1 -> S2 -> S3 stage ladder since C-2/C-5).
//!
//! The alias contract (Davinci P0-10): the wasm `analyzeSfc` result carries
//! the croquis folio text under both the deprecated `vir` key and the
//! nested `folio.croquis`, byte-identically. No test pinned that before
//! this one, which is why the P2-18 acceptance demands it: the ladder work
//! must not move those bytes.

use super::analyze::analyze_sfc_json;

const SOURCE: &str =
    "<script setup>\nconst msg = 1\n</script>\n\n<template>\n  <div>{{ msg }}</div>\n</template>\n";
const TEMPLATE: &str = "\n  <div>{{ msg }}</div>\n";

/// The S2 (Disegno) page for [`TEMPLATE`]: after the lowering and, byte for
/// byte, after the one transform pass the artifact selects (`hoist-static`
/// is a fact-producing analysis, so the tree it leaves is the lowering's).
const S2_PAGE: &str = "[disegno]
ops=2

[disegno.ops]
ui.element div @3:23
  ui.interpolation js(\"msg\" @11:14) @8:17

";

const S2_PROVENANCE_PAGE: &str = r#"[s2-provenance-folio]

[s2-provenance-folio.records]
rule=condense.drop-whitespace node=- before="\n  " after="" @0:3
rule=lower.element node=0 before="<div>" after="ui.element div" @3:23
rule=lower.interpolation node=1 before=" msg " after="ui.interpolation js" @8:17
rule=condense.drop-whitespace node=- before="\n" after="" @23:24
rule=pass.hoist-static.fact node=0 before="ui.element" after="level=dynamic-text props=false nested=true native=true" @3:23

"#;

const S3_PAGE: &str = "[s3-folio]
phase=built

[s3-folio.regions]
id=0 parent=- owner=- span=3:23
id=1 parent=0 owner=0 span=8:17

[s3-folio.ops]
id=0 kind=impeto.insert-node region=0 effect=- span=3:23
id=1 kind=impeto.set-text region=1 effect=0 span=8:17

[s3-folio.effects]
id=0 owner=1 region=1 span=8:17

";

const S3_PARTITION_PAGE: &str = "[s3-partition-folio]

[s3-partition-folio.ops]
op=0 kind=static span=3:23
op=1 kind=dynamic span=8:17

";

const S3_VALUES_PAGE: &str = r#"[s3-values-folio]

[s3-values-folio.operands]
operand=[0,"tag",null,null,null,"literal","div","",3,23]
operand=[0,"namespace",null,null,null,"literal","html","",3,23]
operand=[1,"text",null,null,null,"js","msg","",11,14]

"#;

/// The croquis folio text for [`SOURCE`], the exact bytes both alias keys
/// must carry.
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

#[test]
fn the_croquis_alias_keys_stay_byte_identical() {
    let result = analyze_sfc_json(SOURCE, "src/App.vue").expect("analysis succeeds");

    // The alias LAW is the two keys' byte-identity to each other. The
    // deterministic sections ([vir] header and [bindings]) are pinned
    // exactly; the [scopes] global lists iterate a hash set whose order
    // is environment-sensitive (CI proved it), so the absolute literal
    // pins only the prefix and the per-environment stability is carried
    // by the equality law plus a same-process double-run pin.
    let vir = result["vir"].as_str().expect("vir is a string");
    let expected_prefix = &CROQUIS_FOLIO[..CROQUIS_FOLIO.find("[scopes]").expect("prefix marker")];
    assert_eq!(&vir[..expected_prefix.len()], expected_prefix);
    let again = analyze_sfc_json(SOURCE, "src/App.vue").expect("analysis succeeds");
    assert_eq!(result["vir"], again["vir"]);
    assert_eq!(result["vir"], result["folio"]["croquis"]);
    // The alias object itself gained no siblings: the stage pages live in
    // the top-level `spolvero` feed, not inside the alias.
    let folio = result["folio"].as_object().expect("folio is an object");
    assert_eq!(folio.len(), 1);
    assert_eq!(result["folio"]["croquis"], result["vir"]);
}

#[test]
fn the_analyze_result_carries_the_full_stage_ladder_feed() {
    let result = analyze_sfc_json(SOURCE, "src/App.vue").expect("analysis succeeds");

    // The S1 page's text equals the authored template bytes (the TS-19
    // fidelity law observed at this consumer), proven through the surface
    // tree rather than copied from the source. The S2/S3 pages are the real
    // lowerings' canonical folios, in pipeline order. The P3-13 remark
    // explains why the `<div>` is not whole-hoistable; its span is in the
    // template's byte frame (the pages' frame), derived from the template text.
    let start = TEMPLATE.find("<div>").expect("div opens");
    let end = TEMPLATE.find("</div>").expect("div closes") + "</div>".len();
    assert_eq!(
        result["spolvero"],
        serde_json::json!({
            "schema_version": 1,
            "command": "analyze-sfc",
            "pages": [
                { "path": "src/App.vue", "stage": "s1", "pass": "parse", "text": TEMPLATE },
                { "path": "src/App.vue", "stage": "s2", "pass": "lower", "text": S2_PAGE },
                { "path": "src/App.vue", "stage": "s2", "pass": "hoist-static", "text": S2_PAGE },
                {
                    "path": "src/App.vue",
                    "stage": "s2-provenance",
                    "pass": "transform",
                    "text": S2_PROVENANCE_PAGE,
                },
                { "path": "src/App.vue", "stage": "s3", "pass": "lower", "text": S3_PAGE },
                {
                    "path": "src/App.vue",
                    "stage": "s3-partition",
                    "pass": "lower",
                    "text": S3_PARTITION_PAGE,
                },
                {
                    "path": "src/App.vue",
                    "stage": "s3-values",
                    "pass": "lower",
                    "text": S3_VALUES_PAGE,
                },
            ],
            "remarks": [{
                "path": "src/App.vue", "stage": "s2", "pass": "hoist-static", "kind": "missed",
                "name": "static-subtree", "span": { "start": start, "end": end },
                "args": [
                    { "key": "tag", "value": "div" },
                    { "key": "blocker", "value": "child" },
                    { "key": "op", "value": "ui.interpolation" },
                ],
            }],
        })
    );
}

#[test]
fn a_template_less_sfc_feeds_zero_pages() {
    let result = analyze_sfc_json("<script setup>\nconst n = 1\n</script>\n", "src/Logic.vue")
        .expect("analysis succeeds");

    assert_eq!(
        result["spolvero"],
        serde_json::json!({
            "schema_version": 1,
            "command": "analyze-sfc",
            "pages": [],
            "remarks": [],
        })
    );
}
