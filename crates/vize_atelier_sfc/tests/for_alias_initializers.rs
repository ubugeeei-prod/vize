//! #10746: loop alias initializers read component context, not free globals.
#![expect(clippy::disallowed_types, reason = "complete compiler output")]

use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

fn compile(source: &str, ssr: bool) -> std::string::String {
    let parse = SfcParseOptions {
        filename: "App.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("valid SFC");
    let mut options = SfcCompileOptions {
        parse,
        ..Default::default()
    };
    options.template.ssr = ssr;
    let result = compile_sfc(&descriptor, options).expect("compiled SFC");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    result.code.to_string()
}

/// Every `renderList`/`ssrRenderList` call head, from its source argument to
/// the end of the line, in output order.
fn loop_heads(code: &str) -> std::vec::Vec<&str> {
    code.lines()
        .filter_map(|line| line.split_once("enderList(").map(|(_, head)| head))
        .collect()
}

fn template(script: &str, body: &str, newline: &str) -> std::string::String {
    [
        script,
        newline,
        "<template>",
        newline,
        body,
        newline,
        "</template>",
        newline,
    ]
    .concat()
}

const OPTIONS: &str = "<script>export default {}</script>";
const SETUP: &str = "<script setup>const fallback = 'A'; const items = [{}]</script>";

#[test]
fn initializers_and_computed_keys_read_component_context() {
    for newline in ["\n", "\r\n"] {
        for (body, heads) in [
            (
                "<p v-for=\"{ label = fallback } in items\">{{ label }}</p>",
                &["_ctx.items, ({ label = _ctx.fallback }) => {"][..],
            ),
            (
                "<p v-for=\"{ \\u006cabel = fallback } in items\">{{ label }}</p>",
                &["_ctx.items, ({ \\u006cabel = _ctx.fallback }) => {"],
            ),
            (
                "<p v-for=\"[a = fallback, b = a] of items\">{{ a }}{{ b }}</p>",
                &["_ctx.items, ([a = _ctx.fallback, b = a]) => {"],
            ),
            (
                "<p v-for=\"{ [key]: { name = pick(name, other) } } in items\">{{ name }}</p>",
                &["_ctx.items, ({ [_ctx.key]: { name = _ctx.pick(name, _ctx.other) } }) => {"],
            ),
            (
                "<div v-for=\"outer in rows\"><p v-for=\"{ label = outer.name } in outer.items\">{{ label }}</p></div>",
                &[
                    "_ctx.rows, (outer) => {",
                    "outer.items, ({ label = outer.name }) => {",
                ],
            ),
            (
                "<p v-for=\"(item, index) in items\">{{ item }}</p>",
                &["_ctx.items, (item, index) => {"],
            ),
        ] {
            let source = template(OPTIONS, body, newline);
            assert_eq!(loop_heads(&compile(&source, true)), heads, "SSR {body}");
            assert_eq!(loop_heads(&compile(&source, false)), heads, "VDOM {body}");
        }
    }
}

#[test]
fn setup_bindings_keep_their_render_function_access() {
    let source = template(
        SETUP,
        "<p v-for=\"{ label = fallback } in items\">{{ label }}</p>",
        "\n",
    );
    assert_eq!(
        loop_heads(&compile(&source, true)),
        ["$setup.items, ({ label = $setup.fallback }) => {"]
    );
    // The inline render closure reads the setup constants directly.
    assert_eq!(
        loop_heads(&compile(&source, false)),
        ["items, ({ label = fallback }) => {"]
    );
}
