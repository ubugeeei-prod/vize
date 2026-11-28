//! An SFC whose `<script lang="tsx">` or `lang="jsx"` mixes TypeScript with
//! JSX compiles in two steps, as Vize's bundler plugins run it: the SFC
//! compiler erases the TypeScript and keeps the JSX, as @vue/compiler-sfc
//! does, and the JSX lowering compiles that module. Each module here is the
//! whole lowered output.

use vize_atelier_jsx::{JsxCompileConfig, JsxLang, compile_jsx};
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_l0::Allocator;

fn lower(source: &str, lang: JsxLang) -> vize_l0::String {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("SFC");
    let sfc = compile_sfc(&descriptor, SfcCompileOptions::default()).expect("compile");
    assert!(sfc.errors.is_empty(), "{:?}", sfc.errors);
    let allocator = Allocator::new();
    let output = compile_jsx(&allocator, &sfc.code, lang, &JsxCompileConfig::default());
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    output.module_code()
}

#[test]
fn a_tsx_setup_script_with_type_annotations_lowers_its_jsx() {
    assert_eq!(
        lower(
            "<script setup lang=\"tsx\">\nconst label: string = 'hi'\nconst node = () => <span>{label}</span>\n</script>\n",
            JsxLang::Tsx,
        ),
        r#"import { createElementVNode as _createElementVNode } from "vue"



const label = "hi";

export default {
  __name: 'anonymous',
  setup(__props, { expose: __expose }) {
  __expose();

const node = () => (() => {
return ((_ctx, _cache) => {
  return (_createElementVNode("span", null, [
    label
  ]))
})(undefined, [])
})();

return { label, node }
}

}"#
    );
}

#[test]
fn a_tsx_normal_script_with_type_annotations_lowers_its_jsx() {
    assert_eq!(
        lower(
            "<script lang=\"tsx\">\nconst identity = <T,>(value: T): T => value\nconst label: string = 'hi'\nexport default { render: () => <span>{identity(label) as string}</span> }\n</script>\n",
            JsxLang::Tsx,
        ),
        r#"import { createElementVNode as _createElementVNode } from "vue"

const identity = (value) => value;
const label = "hi";
export default { render: () => (() => {
return ((_ctx, _cache) => {
  return (_createElementVNode("span", null, [
    identity(label)
  ]))
})(undefined, [])
})() };
"#
    );
}

#[test]
fn a_jsx_normal_script_lowers_its_jsx() {
    assert_eq!(
        lower(
            "<script lang=\"jsx\">\nconst label = 'hi'\nexport default { render: () => <span>{label}</span> }\n</script>\n",
            JsxLang::Jsx,
        ),
        r#"import { createElementVNode as _createElementVNode } from "vue"


const label = 'hi'
export default { render: () => (() => {
return ((_ctx, _cache) => {
  return (_createElementVNode("span", null, [
    label
  ]))
})(undefined, [])
})() }
"#
    );
}
