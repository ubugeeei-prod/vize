//! A normal `<script lang="tsx">` or `lang="jsx"` block is read in its own
//! grammar. Read as TypeScript or plain JavaScript, its JSX failed to parse, so
//! the default export was never rewritten (the module ended with two default
//! exports) and, beside `<script setup>`, its TypeScript reached the output.
//! The JSX stays in the module, as @vue/compiler-sfc keeps it, for the JSX
//! lowering to compile. The pinned `compiler/sfc/tsx-normal-script` and
//! `compiler/sfc/tsx-normal-script-beside-setup` differential cases hold the
//! SSR modules; `VIZE_BLESS_TSX_SCRIPT_BLOCKS=1` rewrites them.

use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};

fn fixture(case: &str, file: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/compiler")
        .join(case)
        .join(file)
}

fn compile(source: &str, ssr: bool) -> String {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("SFC");
    let options = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: "Layout.vue".into(),
            ..Default::default()
        },
        template: TemplateCompileOptions {
            ssr,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = compile_sfc(&descriptor, options).expect("compile");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    result.code.as_str().to_owned()
}

fn compile_case(case: &str, ssr: bool) -> String {
    compile(
        &std::fs::read_to_string(fixture(case, "Layout.vue.txt")).expect("input"),
        ssr,
    )
}

/// Compare the SSR module of `case` with its pinned reference, or rewrite it
/// under `VIZE_BLESS_TSX_SCRIPT_BLOCKS`.
fn assert_pinned(case: &str) {
    let code = compile_case(case, true);
    let path = fixture(case, "module.expected.txt");
    if std::env::var_os("VIZE_BLESS_TSX_SCRIPT_BLOCKS").is_some() {
        std::fs::write(&path, code).expect("bless module");
        return;
    }
    let expected = std::fs::read_to_string(&path).expect("pinned module");
    assert_eq!(code, expected, "{}", path.display());
}

#[test]
fn the_normal_script_ssr_module_matches_its_pinned_reference() {
    assert_pinned("tsx-normal-script");
}

#[test]
fn the_normal_script_beside_setup_ssr_module_matches_its_pinned_reference() {
    assert_pinned("tsx-normal-script-beside-setup");
}

#[test]
fn a_tsx_normal_script_default_export_becomes_the_component_binding() {
    assert_eq!(
        compile_case("tsx-normal-script", false),
        r#"import { defineComponent } from "vue";
const label = "hi";
const identity = (value) => value;
const badge = (text) => <b class="badge">{identity(text)}</b>;
const _sfc_main = defineComponent({
  name: "Layout",
  setup() {
    return {
      label,
      badge: badge(label)
    };
  }
});

import { toDisplayString as _toDisplayString, openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"

function _sfc_render(_ctx, _cache) {
  return (_openBlock(), _createElementBlock("p", null, _toDisplayString(_ctx.label), 1 /* TEXT */))
}
_sfc_main.render = _sfc_render
export default _sfc_main"#
    );
}

#[test]
fn a_tsx_normal_script_beside_setup_loses_its_typescript() {
    assert_eq!(
        compile_case("tsx-normal-script-beside-setup", false),
        r#"import { resolveDynamicComponent as _resolveDynamicComponent, openBlock as _openBlock, createBlock as _createBlock } from "vue"


export const Badge = (props) => <b class="badge">{props.text}</b>;
const __default__ = { inheritAttrs: false };

export default /*@__PURE__*/Object.assign(__default__, {
  __name: 'Layout',
  setup(__props) {

const label = "hi";
const node = () => <span><Badge text={label} /></span>;

return (_ctx, _cache) => {
  return (_openBlock(), _createBlock(_resolveDynamicComponent(node)))
}
}

})"#
    );
}

#[test]
fn a_script_only_tsx_component_keeps_its_one_default_export() {
    assert_eq!(
        compile(
            "<script lang=\"tsx\">\nconst label: string = 'hi'\nexport default { render: () => <span>{label}</span> }\n</script>\n",
            false,
        ),
        "const label = \"hi\";\nexport default { render: () => <span>{label}</span> };\n"
    );
}

#[test]
fn a_script_only_jsx_component_keeps_its_one_default_export() {
    assert_eq!(
        compile(
            "<script lang=\"jsx\">\nconst label = 'hi'\nexport default { render: () => <span>{label}</span> }\n</script>\n",
            false,
        ),
        "\nconst label = 'hi'\nexport default { render: () => <span>{label}</span> }\n"
    );
}

/// `<b>{1}</b>/g` also parses as TypeScript, as `<b>{1}` compared with the
/// regular expression `/b>/g`. The TSX reading divides the element by `g`.
#[test]
fn jsx_that_also_reads_as_typescript_keeps_its_tsx_meaning() {
    assert_eq!(
        compile(
            "<script lang=\"tsx\">\nconst g = 2\nexport const half = <b>{1}</b>/g\nexport default {}\n</script>\n",
            false,
        ),
        "const g = 2;\nexport const half = <b>{1}</b> / g;\nexport default {};\n"
    );
}

/// TSX has no `<T>value` assertion. A `lang="tsx"` block without JSX that
/// uses one still compiles, read as TypeScript.
#[test]
fn a_tsx_block_with_an_angle_bracket_assertion_reads_as_typescript() {
    assert_eq!(
        compile(
            "<script lang=\"tsx\">\nconst value = <string>JSON.parse('\"a\"')\nexport default { value }\n</script>\n",
            false,
        ),
        "const value = JSON.parse(\"\\\"a\\\"\");\nexport default { value };\n"
    );
}

#[test]
fn a_tsx_setup_script_erases_annotations_and_generic_arrows_beside_jsx() {
    assert_eq!(
        compile(
            "<script setup lang=\"tsx\">\nconst label: string = 'hi'\nconst identity = <T,>(value: T): T => value\nconst node = () => <span>{identity(label) as string}</span>\n</script>\n",
            false,
        ),
        r#"

const label = "hi";

export default {
  __name: 'Layout',
  setup(__props, { expose: __expose }) {
  __expose();

const identity = (value) => value;
const node = () => <span>{identity(label)}</span>;

return { label, identity, node }
}

}"#
    );
}
